use std::ops::Deref;
use std::sync::{Arc, RwLock, TryLockError};
use crossbeam_channel::{Sender, TrySendError};
use crate::data_pipes::base::{PipeStatus, OutPipe, Pipe};
use crate::data_pipes::channel::ChannelStatusRef;

pub struct OutChannel<Val>{
    channel:        Option<Sender<Val>>,
    data_buffer:    Option<Val>,
    status:         ChannelStatusRef
}

impl<Val> OutChannel<Val> {
    pub fn new(sender: Sender<Val>, status: ChannelStatusRef) -> Self{
        Self{
            channel: Some(sender),
            data_buffer: None,
            status: status,
        }
    }
}

impl<Val> Clone for OutChannel<Val>{
    fn clone(&self) -> Self{
        Self{
            channel:     self.channel.clone(),
            data_buffer: None,
            status:      self.status.clone(),
        }
    }
}

impl<Val> Pipe for OutChannel<Val>{
    fn disconnect(&mut self) {
        *self.status.0.write().unwrap() = PipeStatus::Disconnected;
        self.channel = None;
    }
    fn try_flow_data(&mut self) -> bool {
        if self.data_buffer.is_some() {
            let data = self.data_buffer.take().unwrap();
            match self.channel.as_ref().unwrap().try_send(data){
                Ok(_)  => {
                    true
                },
                Err(TrySendError::Disconnected(val)) => {
                    self.data_buffer = Some(val);
                    self.verify_connection();
                    false
                },
                Err(TrySendError::Full(val)) => {
                    self.data_buffer = Some(val);
                    false
                }
            }
        } else {
            false
        }
    }
    fn verify_connection(&mut self) -> PipeStatus {
        if self.channel.is_some() {
            match self.status.0.try_read(){
                Ok(status_lock) => {
                    match status_lock.deref() {
                        PipeStatus::Okay         => PipeStatus::Okay,
                        PipeStatus::Disconnected => {
                            self.channel = None;
                            PipeStatus::Disconnected
                        }
                    }
                }
                Err(TryLockError::WouldBlock) =>  { PipeStatus::Okay}
                Err(TryLockError::Poisoned(_)) => {panic!("fuck");}
            }

        } else {
            PipeStatus::Disconnected
        }
    }
}

impl<Val> OutPipe<Val> for OutChannel<Val>{
    fn has_space_ready(&self) -> bool {
                                            self.data_buffer.is_none()
                                                                      }
    fn put_data_unchecked(&mut self, val: Val) {
                                                     self.data_buffer = Some(val);
                                                                                  }
    fn take_data_back_unchecked(&mut self) -> Val {
                                                        self.data_buffer.take().unwrap()
                                                                                        }
}
