// use std::sync::{Arc};
// use tokio::sync::Notify;
use std::cell::RefCell;
use std::sync::mpsc::Sender;

pub enum ThreadMessage<T>{
    Null,
    Ok,
    AddToMap(AddToMap<T>),
    OnlyAddToMap(OnlyAddToMap<T>),
    GetFromMap(GetFromMap<T>),
    SendToPoP(RefCell<T>)
}

pub struct AddToMap<T>{
    pub k:String,
    pub v:T,
    pub s:Sender<ThreadMessage<T>>
}

pub struct OnlyAddToMap<T>{
    pub k:String,
    pub v:T
}

pub struct GetFromMap<T>{
    pub k:String,
    pub s:Sender<ThreadMessage<T>>
}

