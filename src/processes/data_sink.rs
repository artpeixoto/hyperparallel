use std::marker::PhantomData;
use crate::data_pipes::base::InPipe;
use crate::processes::base::{ProcStatus, ResLockingFn};

pub trait FnDataSink<TVal>: FnMut(TVal) -> ResLockingFn<TVal, Option<()>> + Clone + Send + Sync{}
impl<TVal, TSelf> FnDataSink<TVal> for TSelf where TSelf: FnMut(TVal) -> ResLockingFn<TVal, Option<()>> + Clone + Send + Sync{}

#[derive(Clone)]
pub struct ProcSinkNode<TFnData, TInPipe,  TVal>
    where
    TFnData: FnDataSink<TVal>,
    TInPipe: InPipe<TVal>
{
    in_pipe: TInPipe,
    fn_data: TFnData,
    _phantom: PhantomData<TVal>
}

impl<TFnData, TInPipe,  TVal>
    ProcSinkNode<TFnData, TInPipe,  TVal>
    where
        TFnData:    FnMut(TVal) -> ResLockingFn<TVal, Option<()>> + Clone + Send + Sync,
        TInPipe:    InPipe<TVal>
{
    pub fn new(fn_data: TFnData, in_pipe: TInPipe) -> Self{
        Self{
            in_pipe,
            fn_data,
            _phantom: PhantomData {}
        }
    }

    pub fn run(&mut self) -> ProcStatus{
        if self.in_pipe.verify_connection().is_okay(){
            self.in_pipe.try_flow_data();
            loop {
                if !self.in_pipe.has_data_ready() {
                    break ProcStatus::Running;
                }
                let val = self.in_pipe.get_data_unchecked();
                match (self.fn_data)(val){
                    ResLockingFn::Data(Some(())) => {
                        self.in_pipe.try_flow_data();
                    }
                    ResLockingFn::Data(None) => {
                        self.in_pipe.disconnect();
                        break ProcStatus::Finished;
                    }
                    ResLockingFn::LockBusy(val) => {
                        self.in_pipe.put_data_back_unchecked(val);
                        break ProcStatus::Running;
                    }
                }

            }
        } else {
            ProcStatus::Finished
        }
    }
}

impl<TFnData, TInPipe,  TVal>
    Iterator for ProcSinkNode<TFnData, TInPipe,  TVal>
    where
        TFnData:     FnMut(TVal) -> ResLockingFn<TVal, Option<()>> + Clone + Send + Sync,
        TInPipe:     InPipe<TVal>
{
    type Item = ();
    fn next(&mut self) -> Option<Self::Item> {
        match self.run(){
            ProcStatus::Running =>  {Option::Some(())}
            ProcStatus::Finished => {None}
        }
    }
}



