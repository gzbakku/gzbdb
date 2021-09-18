
use gobject::gObjectValue;
use tokio::sync::mpsc::Sender;

#[derive(Debug)]
pub enum ThreadMessage{
    Request(RequestMessage),
    RequestError(Error),
    ResponseObject(gObjectValue),
    RequestTimeout
}

// impl ThreadMessage{
//     pub fn request_error(e:Error)->ThreadMessage{
//         ThreadMessage::RequestError(RequestError{
//             error:e
//         })
//     }
// }

use std::time::Instant;

#[derive(Debug)]
pub struct RequestMessage{
    pub request_id:String,
    pub body:Vec<u8>,
    pub writer:Sender<ThreadMessage>,
    pub timer:Instant,
    pub timeout:u64
}

use crate::common::Error;

#[derive(Debug)]
pub struct RequestError{
    pub error:Error
}


