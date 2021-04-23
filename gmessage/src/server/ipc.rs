
// use std::sync::mpsc::{Receiver,Sender,channel};
// use std::sync::Arc;
// use std::sync::Mutex;


#[allow(dead_code)]
#[derive(Debug,Clone)]
pub struct SharedData{
    
}

impl SharedData{
    pub fn new()->SharedData{SharedData{}}
}

use crate::messenger::MessageBox;
use gobject::gObjectValue;

pub enum ThreadMessage{
    MessageBox(MessageBox),
    Messages(ChannelMessages),
    Responses(Responses)
}

impl ThreadMessage{
    pub fn messages(id:String,p:Vec<gObjectValue>)->ThreadMessage{
        ThreadMessage::Messages(ChannelMessages{
            id:id,
            pool:p
        })
    }
    pub fn responses(p:Vec<Response>)->ThreadMessage{
        ThreadMessage::Responses(Responses{
            pool:p
        })
    }
}

pub struct ChannelMessages{
    pub id:String,
    pub pool:Vec<gObjectValue>
}

use crate::query::Response;

pub struct Responses{
    pub pool:Vec<Response>
}


