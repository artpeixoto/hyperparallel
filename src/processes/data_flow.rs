use std::marker::PhantomData;
use crate::data_pipes::base::{InPipe, OutPipe, PipeStatus};
use crate::processes::base::ProcStatus;

#[derive(Clone)]
pub struct DataFlowProcess<TFn, TIn, TOut, TInlet, TOutlet>
    where
    TFn:        (Fn(TIn) -> TOut) + Send + Sync + Clone,
    TInlet:     InPipe<TIn>,
    TOutlet:    OutPipe<TOut>,
{
    inlet:     TInlet,
    outlet:    TOutlet,
    proc_fun:  TFn,

    _phantom: PhantomData<(TIn, TOut)>
}


impl<TFn, TIn, TOut, TInlet, TOutlet> DataFlowProcess<TFn, TIn, TOut, TInlet, TOutlet>
    where
    TFn: (Fn(TIn) -> TOut) + Send + Sync + Clone,
    TInlet:  InPipe<TIn>,
    TOutlet: OutPipe<TOut>,
{
    pub fn new(proc_fun: TFn, inlet: TInlet, outlet: TOutlet) -> Self{
        Self{
            inlet,
            outlet,
            proc_fun,
            _phantom: PhantomData::default()
        }
    }
    pub fn verify_channels_connections(&mut self) -> PipeStatus{
        use PipeStatus::*;
        match (self.inlet.verify_connection(), self.outlet.verify_connection()){
            (Disconnected   , Disconnected) => { Disconnected },
            (Disconnected   , Okay)         => { self.outlet.disconnect(); Disconnected },
            (Okay           , Disconnected) => { self.inlet.disconnect(); Disconnected },
            (Okay           , Okay)         => { Okay },
        }
    }
    pub fn run(&mut self) -> ProcStatus{
        if self.verify_channels_connections() == PipeStatus::Okay{
            self.inlet.try_flow_data();
            self.outlet.try_flow_data();

            while self.inlet.has_data_ready() && self.outlet.has_space_ready(){
                let in_data = self.inlet.get_data_unchecked();
                let res = (self.proc_fun)(in_data);
                self.outlet.put_data_unchecked(res);

                self.outlet.try_flow_data();
                self.inlet.try_flow_data();
            }

            ProcStatus::Running
        } else {
            ProcStatus::Finished
        }
    }
}

impl<TFn, TIn, TOut, TInlet, TOutlet>
Iterator for DataFlowProcess<TFn, TIn, TOut, TInlet, TOutlet>
    where
    TFn: (Fn(TIn) -> TOut) + Send + Sync + Clone,
    TInlet:  InPipe<TIn>,
    TOutlet: OutPipe<TOut>,
{
    type Item = ();
    fn next(&mut self) -> Option<Self::Item> {
        match self.run(){
            ProcStatus::Running  => {Some(())}
            ProcStatus::Finished => {None}
        }
    }
}