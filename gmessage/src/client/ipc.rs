
// use crate::messenger::MessageBox;
use gobject::gObject;
// use std::sync::mpsc::{Receiver,Sender,channel};
// use std::sync::Arc;
// use std::sync::Mutex;

use crate::query::Response;
use std::sync::mpsc::Sender;
use mio::Waker;

#[derive(Debug)]
pub enum ThreadMessage{
    Request(RequestMessage),
    RequestError(RequestError),
    Response(Response)
}

impl ThreadMessage{
    pub fn request_error(e:Error)->ThreadMessage{
        ThreadMessage::RequestError(RequestError{
            error:e
        })
    }
}

#[derive(Debug)]
pub struct RequestMessage{
    pub encrypted:bool,
    pub request_id:String,
    pub body:gObject,
    pub waker:Waker,
    pub writer:Sender<ThreadMessage>
}

use crate::common::Error;

#[derive(Debug)]
pub struct RequestError{
    pub error:Error
}


