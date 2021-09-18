
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
    Response(ResponseResult),
    Requests(Requests)
}

pub struct ResponseResult{
    pub id:String,
    pub body:Vec<u8>
}

use std::sync::{Arc,RwLock};
use std::collections::HashMap;

pub struct Requests{
    pub id:String,
    pub requests:Vec<gObjectValue>,
    pub passwords:Arc<RwLock<HashMap<String,Vec<u8>>>>
}

pub struct ChannelMessages{
    pub id:String,
    pub pool:Vec<gObjectValue>
}

use crate::query::Response;

pub struct Responses{
    pub pool:Vec<Response>
}


