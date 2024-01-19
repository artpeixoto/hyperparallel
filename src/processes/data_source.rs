use std::marker::PhantomData;
use crate::data_pipes::base::OutPipe;
use crate::processes::base::{ProcStatus, ResLockingFn};

pub trait FnDataSource<TVal>
    : FnMut() -> ResLockingFn<(),Option<TVal>> + Clone  + Send
    {}

impl<TVal, TSelf>
    FnDataSource<TVal>
    for TSelf
    where TSelf : FnMut() -> ResLockingFn<(),Option<TVal>> + Clone + Send
{}

#[derive(Clone)]
pub struct DataSourceProcess<TFnData, TOutlet,  TVal>
    where
    TFnData:    FnDataSource<TVal>,
    TOutlet:    OutPipe<TVal>,
{
    fn_data:    TFnData,
    outlet:     TOutlet,
    _phantom:   PhantomData<TVal>
}



impl<TFnData, TOutPipe,  TVal>
    DataSourceProcess<TFnData, TOutPipe,  TVal>
    where
        TFnData:     FnDataSource<TVal>,
        TOutPipe:    OutPipe<TVal>
{
    pub fn new(fn_data: TFnData, outlet: TOutPipe) -> Self{
        Self {
            fn_data,
            outlet,
            _phantom: PhantomData{},
        }
    }
    pub fn run(&mut self) -> ProcStatus {
        if self.outlet.verify_connection().is_okay(){
            self.outlet.try_flow_data();
            loop {
                if !self.outlet.has_space_ready() {
                    break ProcStatus::Running;
                }

                let data_res = (self.fn_data)();

                match data_res {
                    ResLockingFn::LockBusy(()) => {
                        break ProcStatus::Running;
                    }
                    ResLockingFn::Data(Some(val))  => {
                        self.outlet.put_data_unchecked(val);
                        self.outlet.try_flow_data();
                    }
                    ResLockingFn::Data(None) => {
                        self.outlet.disconnect();
                        break ProcStatus::Finished;
                    }
                }
            }
        } else {
            ProcStatus::Finished
        }
    }
}



impl<TFnData, TOutlet,  TVal>
    Iterator for DataSourceProcess<TFnData,  TOutlet,  TVal>
    where
        TFnData:    FnDataSource<TVal>,
        TOutlet:    OutPipe<TVal>
{
    type Item = ();
    fn next(&mut self) -> Option<Self::Item> {
        match self.run(){
            ProcStatus::Running  => {Some(())}
            ProcStatus::Finished => {None}
        }
    }
}