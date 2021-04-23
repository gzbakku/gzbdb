
use gobject::gObject;

#[derive(Debug,Clone)]
pub struct Request{
    pub channel_id:String,
    pub request_id:String,
    pub body:gObject
}