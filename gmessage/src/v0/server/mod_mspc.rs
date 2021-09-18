use std::thread;
use std::sync::{Mutex,Arc};
use crate::Error;
use crate::common::Error;
use crate::query::{Request,Response};
use std::future::Future;

mod config;
mod entry;
mod message;
mod channel;
mod ipc;

// use crate::io;

pub use config::{ServerConfig,RsaKey};
use std::sync::mpsc;

pub fn start<F,T>(config:ServerConfig,func:F,) -> Result<(),Error>
where
    F:Fn(Request) -> T + Unpin + Send + 'static + Copy,
    T:Future<Output = Response> + Send + 'static
{

    // println!("started threads");

    let shared:Arc<Mutex<ipc::SharedData>> = Arc::new(Mutex::new(ipc::SharedData::new()));

    let (channel_write,channel_read) = mpsc::channel::<ipc::ThreadMessage>();
    let (message_write,message_read) = mpsc::channel::<ipc::ThreadMessage>();
    let channel_writer = Arc::new(Mutex::new(channel_write));

    //connection entry
    let enrty_channel_writer = channel_writer.clone();
    let entry_arc = Arc::clone(&shared);
    let entry_thread = thread::spawn(move || {
        match entry::start(entry_arc,config,enrty_channel_writer){
            Ok(_)=>{return Ok(());},
            Err(_)=>{
                return Err(Error!("failed-start-entry_thread"));
            }
        }
    });

    // println!("entry thread started");

    //connection read
    let channel_thread = thread::spawn(move || {
        match channel::start(channel_read,&message_write){
            Ok(_)=>{return Ok(());},
            Err(_)=>{
                return Err(Error!("failed-start-channel_thread"));
            }
        }
    });

    // println!("channel thread started");

    //process message
    let message_channel_writer = channel_writer.clone();
    let message_arc = Arc::clone(&shared);
    let message_thread = thread::spawn(move || {
        match message::start(message_arc,func,message_channel_writer,message_read){
            Ok(_)=>{return Ok(());},
            Err(_)=>{
                return Err(Error!("failed-start-message_thread"));
            }
        }
    });

    // println!("message thread started");

    match entry_thread.join(){
        Ok(_)=>{},
        Err(_)=>{panic!("entry thread failed");}
    }
    match channel_thread.join(){
        Ok(_)=>{},
        Err(_)=>{panic!("read thread failed");}
    }
    match message_thread.join(){
        Ok(_)=>{},
        Err(_)=>{panic!("message thread failed");}
    }

    return Ok(());

}//validate function