use std::ops::Deref;
use std::sync::{Arc, RwLock, TryLockError};
use crossbeam_channel::{Receiver, Sender, TryRecvError};
use crate::data_pipes::base::{Pipe, PipeStatus, InPipe};
use crate::data_pipes::channel::in_channel::InChannel;
use crate::data_pipes::channel::out_channel::OutChannel;

pub mod channel_status {
    use std::sync::{Arc, RwLock};
    use crate::data_pipes::base::PipeStatus;

    #[derive(Clone, )]
    pub struct ChannelStatusRef(pub Arc<RwLock<PipeStatus>>);

    impl ChannelStatusRef {
        pub fn new() -> Self {
            ChannelStatusRef(Arc::new(RwLock::new(PipeStatus::Okay)))
        }
    }
}
pub use channel_status::*;
pub mod in_channel;
pub mod out_channel;

pub fn make_channels<Val>(channel_pair: (Sender<Val>, Receiver<Val>)) -> (OutChannel<Val>, InChannel<Val>){
    let (sender, receiver) = channel_pair;

    let status = ChannelStatusRef::new();

    let in_channel = InChannel::new(receiver, status.clone());
    let out_channel = OutChannel::new(sender, status.clone());

    (out_channel, in_channel)
}