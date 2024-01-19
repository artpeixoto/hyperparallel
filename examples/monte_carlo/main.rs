
use itertools::{Itertools};
use std::{iter::Iterator, sync::{RwLock, Mutex, Arc, TryLockError}, ops::{DerefMut, Deref}, thread, array};
use std::cmp::max;
use std::f64::consts::PI;
use std::io::{Read, stdin};
use std::io::ErrorKind::InvalidInput;
use std::iter::repeat_with;
use std::marker::PhantomData;
use std::task::Waker;
use std::time::Instant;
use crossbeam_channel::{bounded, TrySendError, TryRecvError,  Sender, Receiver};
use fastrand::u8;
use syncpool::SyncPool;
use hyperparallel::data_pipes::base::OutPipe;
use hyperparallel::data_pipes::base::InPipe;
use hyperparallel::data_pipes::channel::make_channels;
use hyperparallel::processes::base::ResLockingFn;
use hyperparallel::processes::data_flow::DataFlowProcess;
use hyperparallel::processes::data_sink::ProcSinkNode;
use hyperparallel::processes::data_source::DataSourceProcess;

mod bll {
    pub mod contagem {
        #[derive(Default, Clone)]
        pub struct Contagem {
            pub total: u64,
            pub dentro: u64,
        }

        impl Contagem {
            pub fn estime_pi(&self) -> f64 {
                (self.dentro as f64 / self.total as f64) * 4.0
            }
        }

        impl core::ops::Add<&Contagem> for Contagem {
            type Output = Contagem;
            fn add(self, rhs: &Contagem) -> Self::Output {
                Contagem {
                    total: self.total + rhs.total,
                    dentro: self.dentro + rhs.dentro,
                }
            }
        }

        impl core::ops::Add<Contagem> for Contagem {
            type Output = Contagem;
            fn add(self, rhs: Contagem) -> Self::Output {
                Contagem {
                    total: self.total + rhs.total,
                    dentro: self.dentro + rhs.dentro,
                }
            }
        }
    }

    pub use contagem::*;

    pub mod ponto {
        #[derive(Clone, Debug, Default)]
        pub struct Ponto {
            pub x: f64,
            pub y: f64,
        }
        impl Ponto {
            pub fn abs(&self) -> f64 {
                (self.x.powi(2) + self.y.powi(2)).powf(0.5)
            }
        }
    }

    pub use ponto::*;
}
use bll::*;

/// Esse teste é um exemplo de uso para executar uma simulação de monte carlo para estimar o valor de PI
/// usando varios núcleos do computador.
///
/// Idealmente, você deve executar este teste usando otimizações `(--release)`
pub fn main() {
    fn execute_paralelo() {
        const N_PONTOS: u64 = 1_000_000_000;
        /// por alguma razao, a velocidade de execução é bem sensivel a essas constantes abaixo...
        const BUFFER_CAP: usize = 512;
        const N_PONTOS_BATCH: usize = 1024;

        let n_threads: usize =
            max(<usize>::from(std::thread::available_parallelism().unwrap()), 1);

        type RngPipeDataType = Box<[Ponto; N_PONTOS_BATCH]>;
        let (rng_sender, rng_receiver)
            = make_channels(bounded::<RngPipeDataType>(BUFFER_CAP));

        type AbsPipeDataType = Box<[f64; N_PONTOS_BATCH]>;
        let (abs_sender, abs_receiver)
            = make_channels(bounded::<AbsPipeDataType>(BUFFER_CAP));

        type ContagemPipeDataType = Contagem;
        let (contagem_sender, contagem_parcial_receiver)
            = make_channels(bounded::<ContagemPipeDataType>(BUFFER_CAP));

        let proc_rng = {
            let make_data_generator = {
                let ng = Arc::new(Mutex::new({
                    let mut gen = fastrand::Rng::new();

                    repeat_with(move || (gen.f64() * 2.0) - 1.0 )
                    .tuples::<(f64, f64)>()
                    .map(|tup| Ponto { x: tup.0, y: tup.1 })
                }));

                move || {
                    let ng = ng.clone();
                    move || {
                        match ng.try_lock().ok(){
                            None => {
                                ResLockingFn::LockBusy(())
                            }
                            Some(mut ng) => {
                                let mut res_data_space: Box<[Ponto; N_PONTOS_BATCH]> =
                                    Box::new(array::from_fn(|_i| Ponto::default()));

                                for elem in res_data_space.iter_mut(){
                                    let generated = unsafe{ng.next().unwrap_unchecked()} ;
                                    elem.x = generated.x;
                                    elem.y = generated.y;
                                }
                                ResLockingFn::Data(Some(res_data_space))
                            }
                        }
                    }
                }
            };


            DataSourceProcess::new(
                make_data_generator(),
                rng_sender,
            )
        };

        let proc_calcular_abs = {
            let proc_fn = |batch: RngPipeDataType| {
                let mut res: AbsPipeDataType = Box::new([0_f64; N_PONTOS_BATCH]);
                let res_iter = res.deref_mut().iter_mut();

                for (el_batch, el_res) in batch.into_iter().map(|vetor| vetor.abs()).zip( res_iter){
                    *el_res  = el_batch
                }
                
                res
            };

            DataFlowProcess::new(
                proc_fn,
                rng_receiver,
                abs_sender,
            )
        };
        let proc_contagem =
            DataFlowProcess::new(
                |vetores| {
                    Contagem {
                        total: vetores.len() as u64,
                        dentro: vetores.into_iter().filter(|vet| vet <= &1.0).count() as u64,
                    }
                },
                abs_receiver,
                contagem_sender,
            );
        let proc_contagem_final = {
            let contagem_mutex = Arc::new(Mutex::new(Contagem::default()));

            fn informe_status(contagem: &Contagem) {
                let contagem_total = contagem.total;
                let estimativa_pi = contagem.estime_pi();
                let erro_relativo = (estimativa_pi as f64 - PI) / PI;

                println!(
                    "{} pontos foram contados.\tEstimativa para pi é: {}.\tErro relativo é de {}%.",
                    contagem_total, estimativa_pi, erro_relativo * 100.0
                );
            }

            let sink_fun = move |contagem: Contagem| -> ResLockingFn<Contagem, Option<()>>{
                match contagem_mutex.try_lock() {
                    Ok(mut contagem_lock) => {
                        contagem_lock.total += contagem.total;
                        contagem_lock.dentro += contagem.dentro;

                        let finalizado = contagem_lock.total >= N_PONTOS;

                        if contagem_lock.total % 100_000_000 == 0 || finalizado {
                            informe_status(contagem_lock.deref());
                        }

                        if finalizado {
                            ResLockingFn::Data(None)
                        } else {
                            ResLockingFn::Data(Some(()))
                        }
                    }
                    Err(_) => {
                        ResLockingFn::LockBusy(contagem)
                    }
                }
            };

            ProcSinkNode::new(
                sink_fun,
                contagem_parcial_receiver
            )
        };

        let make_fn_final = {
            || {
                let mut proc_rng = proc_rng.clone();
                let mut proc_montar_vetores = proc_calcular_abs.clone();
                let mut proc_contagem = proc_contagem.clone();
                let mut proc_contagem_final = proc_contagem_final.clone();

                move || {
                    let mut interleaver_ref_arr: [(&mut dyn Iterator<Item = ()>, bool); 4] = [
                        (&mut proc_rng, true),
                        (&mut proc_montar_vetores, true),
                        (&mut proc_contagem, true),
                        (&mut proc_contagem_final, true),
                    ];

                    loop {
                        let mut all_finished = true;
                        for (proc, is_running) in interleaver_ref_arr.iter_mut() {
                            if *is_running {
                                if proc.next().is_some() {
                                    all_finished = false;
                                } else {
                                    *is_running = false;
                                }
                            }
                        }
                        // if proc_rng.run()               == ProcStatus::Running { all_finished = false; }
                        // if proc_montar_vetores.run()    == ProcStatus::Running { all_finished = false; }
                        // if proc_contagem.run()          == ProcStatus::Running { all_finished = false; }
                        // if proc_contagem_final.run()    == ProcStatus::Running { all_finished = false; }

                        if all_finished { break; }
                    }
                }
            }
        };

        let start = Instant::now();

        println!("Usando {n_threads} threads para executar processamento");
        let threads =
        (1..n_threads as u64)
        .map(|_i| thread::spawn(make_fn_final()))
        .collect_vec();


        for thread in threads {
            thread.join().unwrap();
        }

        let finish = Instant::now();
        let duration = finish.duration_since(start);

        let duration_in_clocks = duration.as_secs_f64() * 1_000_000_000.0 * 4.0 * n_threads as f64;
        let clocks_per_item = duration_in_clocks / N_PONTOS as f64;

        println!("Duração: {duration:?}. Ciclos de cock por item: {clocks_per_item} ");
        {
            println!("Pressione enter para sair.");
            let mut line = [0_u8;1];

            stdin().read_exact(&mut line);
        }
    }
    execute_paralelo();

    fn execute_normal(){


    }
}