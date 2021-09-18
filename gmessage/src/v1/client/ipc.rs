
use gobject::gObjectValue;
use tokio::sync::mpsc::Sender;
use std::sync::Arc;
use tokio::sync::{Mutex};
use crate::query::{Response};

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
    pub data:Vec<u8>
}



