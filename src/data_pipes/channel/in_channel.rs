use std::ops::Deref;
use std::sync::{Arc, RwLock, TryLockError};
use crossbeam_channel::{Receiver,  TryRecvError};
use crate::data_pipes::base::{Pipe, PipeStatus, InPipe};
use crate::data_pipes::channel::ChannelStatusRef;

pub struct InChannel<Val> {
    channel:        Option<Receiver<Val>>,
    data_buffer:    Option<Val>,
    status:         ChannelStatusRef,
}

impl<Val> InChannel<Val>{
    pub fn new(receiver: Receiver<Val>, status_ref: ChannelStatusRef) -> Self{
        Self{
            channel:     Some(receiver),
            data_buffer: None,
            status:      status_ref
        }
    }
}


impl<Val> Clone for InChannel<Val> {
    fn clone(&self) -> Self {
        Self {
            channel: self.channel.clone(),
            data_buffer: None,
            status: self.status.clone()
        }
    }
}

impl<Val> Pipe for InChannel<Val> {
    fn verify_connection(&mut self) -> PipeStatus {
        if self.channel.is_some() {
            match self.status.0.try_read() {
                Ok(status_lock) => {
                    match status_lock.deref() {
                        PipeStatus::Okay => PipeStatus::Okay,
                        PipeStatus::Disconnected => {
                            self.channel = None;
                            PipeStatus::Disconnected
                        }
                    }
                },
                Err(TryLockError::WouldBlock) => {
                    PipeStatus::Okay
                }
                Err(TryLockError::Poisoned(_)) => {
                    panic!()
                }
            }
        } else {
            PipeStatus::Disconnected
        }
    }

    fn disconnect(&mut self) {
        *self.status.0.write().unwrap() = PipeStatus::Disconnected;
        self.channel = None;
    }

    fn try_flow_data(&mut self) -> bool {
        if self.data_buffer.is_none() {
            match self.channel.as_ref().unwrap().try_recv() {
                Ok(data) => {
                    self.data_buffer = Some(data);
                    true
                }
                Err(TryRecvError::Empty) => { false }
                Err(TryRecvError::Disconnected) => {
                    self.verify_connection();
                    false
                }
            }
        } else {
            false
        }
    }
}


impl<Val> InPipe<Val> for InChannel<Val> {
    fn put_data_back_unchecked(&mut self, val: Val) {
        self.data_buffer = Some(val);
    }
    fn get_data_unchecked(&mut self) -> Val {
        self.data_buffer.take().unwrap()
    }
    fn has_data_ready(&self) -> bool {
        self.data_buffer.is_some()
    }
}
