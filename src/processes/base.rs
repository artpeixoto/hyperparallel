pub enum ResLockingFn<TArgs, TRet>{
    LockBusy(TArgs),
    Data(TRet)
}
#[derive(Clone, PartialEq, Eq)]
pub enum ProcStatus{
    Running,
    Finished,
}
