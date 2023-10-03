
#[derive(Clone, PartialEq, Eq)]
pub enum PipeStatus {
    Okay,
    Disconnected,
}

impl PipeStatus {
    pub const fn is_okay(&self) -> bool {
        match self{
            PipeStatus::Okay => true,
            _                   => false
        }
    }
}

pub trait Pipe: Clone {
    fn verify_connection(&mut self) -> PipeStatus;
    fn disconnect(&mut self);
    fn try_flow_data(&mut self) -> bool;
}


pub trait InPipe<Val>: Pipe {
    fn has_data_ready(&self) -> bool;
    fn get_data_unchecked(&mut self) -> Val;
    fn put_data_back_unchecked(&mut self, val: Val);
}

pub trait OutPipe<Val>: Pipe {
    fn has_space_ready(&self) -> bool;
    fn take_data_back_unchecked(&mut self) -> Val;
    fn put_data_unchecked(&mut self, val: Val);
}
