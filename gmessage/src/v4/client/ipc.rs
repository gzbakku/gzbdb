use gobject::gObjectValue;
use tokio::sync::mpsc::Sender;
use std::sync::Arc;
use tokio::sync::{Mutex,RwLock};
use crate::query::{Response};
use std::sync::Mutex as StdMutex;
use tokio::sync::Notify;

//---------------------------------
//waker book

#[derive(Debug,Clone)]
pub enum ResponseHolderValue{
    Null,
    Response(Response)
}

#[derive(Debug,Clone)]
pub struct ResponseHolder{
    pub hold:ResponseHolderValue
}

#[derive(Debug,Clone)]
pub struct WakerBook{
    pub response_holder:Arc<StdMutex<ResponseHolder>>,
    pub notify:Arc<Notify>
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum ThreadMessage{
    BookAdd(BookAdd),
    BookDelete(BookDelete),
    BookFinish(BookFinish),
    WriterSend(WriterSend),
    RequestFailed,
    RequestSuccessfull(RequestSuccessfull),
    ResponseAdd(ResponseAdd)
}

//---------------------------------
//response

#[derive(Debug)]
pub struct ResponseAdd{
    pub password:Arc<RwLock<Vec<u8>>>,
    pub response:gObjectValue
}

//---------------------------------
//request

#[derive(Debug)]
pub struct RequestSuccessfull{
    pub response:Response
}

//---------------------------------
//book

#[derive(Debug)]
pub struct BookAdd{
    pub id:String,
    pub sender:Arc<Mutex<Sender<ThreadMessage>>>
}

#[derive(Debug)]
pub struct BookDelete{
    pub id:String
}

#[derive(Debug)]
pub struct BookFinish{
    pub response:Response
}

//---------------------------------
//writer

#[derive(Debug)]
pub struct WriterSend{
    pub id:u32,
    pub data:Vec<u8>
}



