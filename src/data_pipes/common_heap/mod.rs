use std::sync::{Arc, Mutex};
use syncpool::SyncPool;
use crate::data_pipes::channel::in_channel::InChannel;
use crate::data_pipes::channel::out_channel::OutChannel;


#[derive(Clone)]
pub struct HeapChannelKernel<T>{
    resource_output:    OutChannel<Box<T>>,
    pool:               Arc<Mutex<SyncPool<T>>>,
    waste_input:        InChannel<Box<T>>
}

#[derive(Clone)]
pub struct HeapChannelDataProducer<T>{
    resource_input:     InChannel<Box<T>>,
    production_output:  OutChannel<Box<T>>,
}

#[derive(Clone)]
pub struct HeapChannelDataConsumer<T>{
    production_input:   InChannel<Box<T>>,
    waste_output:       OutChannel<Box<T>>,
}

pub fn make_heap_channel(){
    todo!()
}



