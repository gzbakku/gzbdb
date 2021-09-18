

use crate::client::ipc::{ThreadMessage,RequestMessage};
use std::sync::mpsc::Sender;
use crate::query::Response;
use crate::Error;
use crate::common::Error;
use gobject::gObject;
use mio::{Events, Token, Poll, Waker};
use std::sync::mpsc::{channel};
use rand::{distributions::Alphanumeric, Rng};
use std::time::Duration;
use std::sync::{Arc,Mutex};

#[allow(dead_code)]
#[derive(Debug,Clone)]
pub struct Client{
    id:String,
    sender:Arc<Mutex<Sender<ThreadMessage>>>
}

impl Client{
    pub fn new(id:String,m:Sender<ThreadMessage>)->Client{
        Client{
            id:id,
            sender:Arc::new(Mutex::new(m))
        }
    }
    pub async fn send(&self,body:gObject,timeout:u64,encrypted:bool)->Result<Response,Error>{

        let mut poll;
        let mut events = Events::with_capacity(2);
        let token_id: Token = Token(10);//this id number cannot be used ever again
        match Poll::new(){
            Ok(v)=>{poll = v;},
            Err(_)=>{
                return Err(Error!("build-poll"));
            }
        }

        let waker:Waker;
        match Waker::new(poll.registry(), token_id){
            Ok(v)=>{waker = v;},
            Err(_)=>{
                return Err(Error!("build-waker"));
            }
        }

        let (write, read) = channel::<ThreadMessage>();

        let request_id: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();

        let mut send_failed = false;
        {
            // let sender =;
            match  self.sender.clone().lock(){
                Ok(lock)=>{
                    match lock.send(ThreadMessage::Request(RequestMessage{
                        encrypted:encrypted,
                        request_id:request_id,
                        body:body,
                        waker:waker,
                        writer:write
                    })){
                        Ok(_)=>{},
                        Err(_)=>{send_failed = true;}
                    }
                },
                Err(_)=>{send_failed = true;}
            }
        }
        if send_failed{
            return Err(Error!("connection-io-thread-blocked"));
        }
        

        match poll.poll(&mut events, Some(Duration::from_millis(timeout))){
            Ok(_)=>{},
            Err(_)=>{
                println!("poll failed");
            }
        }

        match read.try_recv(){
            Ok(m)=>{
                match m{
                    ThreadMessage::Response(v)=>{
                        return Ok(v);
                    },
                    ThreadMessage::RequestError(v)=>{
                        return Err(v.error);
                    },
                    _=>{return Err(Error!("unknown_failure"));}
                }
            },
            Err(_)=>{
                return Err(Error!("failed_fetch_response/timeout"));
            }
        }

    }
}