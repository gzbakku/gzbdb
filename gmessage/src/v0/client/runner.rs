

use crate::client::ipc::{ThreadMessage,RequestMessage};
// use std::sync::mpsc::Sender;
use crate::query::{Response,ResponseType};
use crate::Error;
use crate::common::Error;
use gobject::{gObject,gObjectValue};
// use mio::{Events, Token, Poll, Waker};
// use std::sync::mpsc::{channel};
use rand::{distributions::Alphanumeric, Rng};
// use std::time::Duration;
use std::sync::Arc;
use tokio::sync::Mutex;
// use std::sync::{Arc,Mutex};
use tokio::sync::mpsc::Sender;
use crate::crypt::aes::encrypt;
use tokio::sync::mpsc::channel;
use std::time::Instant;

#[allow(dead_code)]
#[derive(Debug,Clone)]
pub struct Client{
    id:String,
    password:Vec<u8>,
    sender:Arc<Mutex<Sender<ThreadMessage>>>
}

impl Client{
    pub fn new(id:String,m:Sender<ThreadMessage>,p:Vec<u8>)->Client{
        Client{
            id:id,
            password:p,
            sender:Arc::new(Mutex::new(m))
        }
    }
    pub async fn send(&self,body:gObject,timeout:u64,encrypted:bool)->Result<Response,Error>{

        let body_val:gObjectValue;
        if encrypted{
            match encrypt(&self.password,&body.build()){
                Ok(b)=>{body_val = gObjectValue::object(b.g_object());},
                Err(e)=>{
                    return Err(Error!("failed-ecnrypt-request"=>e));
                }
            }
        } else {
            body_val = gObjectValue::object(body);
        }

        let id:String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();

        let built = gObject!{
            "id"=>gObjectValue::string(id.clone()),
            "encrypted"=>gObjectValue::bool(encrypted),
            "body"=>body_val
        }.build();

        // println!("request len : {:?}",built.len());

        let (writer,mut reader) = channel::<ThreadMessage>(100);
        let timer = Instant::now();

        match self.sender.lock().await.send(ThreadMessage::Request(RequestMessage{
            request_id:id,
            body:built,
            writer:writer,
            timer:timer,
            timeout:timeout
        })).await{
            Ok(_)=>{},
            Err(_)=>{
                return Err(Error!("failed-io-request"));
            }
        }

        match reader.recv().await{
            Some(message)=>{
                // println!("message : {:?}",message);
                match message{
                    ThreadMessage::ResponseObject(v)=>{
                        match parse_response(&v,&self.password){
                            Ok(r)=>{
                                return Ok(r);
                            },
                            Err(e)=>{
                                return Err(Error!("failed-parse-request"=>e));
                            }
                        }
                    },
                    ThreadMessage::RequestTimeout=>{
                        return Err(Error!("timeout"));
                    },
                    _=>{
                        return Err(Error!(format!("invalid-message : {:?}",message)));
                    }
                }
            },
            None=>{
                return Err(Error!("failed-read-request"));
            }
        }

    }
}

use crate::crypt::aes::Encrypted;

fn parse_response(obj:&gObjectValue,password:&Vec<u8>) -> Result<Response,Error>{

    let response_type:i32;
    match &obj["response_type"]{
        gObjectValue::i32(v)=>{response_type = v.clone();},
        _=>{return Err(Error!("failed-extract-response_type"));}
    }

    let id:String;
    match &obj["id"]{
        gObjectValue::string(v)=>{id = v.clone();},
        _=>{return Err(Error!("failed-extract-id"));}
    }

    let data:gObject;
    match &obj["body"]{
        gObjectValue::object(v)=>{data = v.clone();},
        _=>{return Err(Error!("failed-extract-data"));}
    }

    let make:Response;
    if response_type == 0{
        make = Response{
            channel_id:String::new(),
            request_id:id,
            data:ResponseType::Ok
        };
    } 
    else if response_type == 1{
        let body:gObject;
        match &data["data"]{
            gObjectValue::object(v)=>{body = v.clone();},
            _=>{return Err(Error!("invalid-body-data"));}
        }
        make = Response{
            channel_id:String::new(),
            request_id:id,
            data:ResponseType::Data(body)
        };
    } 
    else if response_type == 2{
        let error:String;
        match &data["error"]{
            gObjectValue::string(v)=>{error = v.clone();},
            _=>{return Err(Error!("invalid-body-data"));}
        }
        make = Response{
            channel_id:String::new(),
            request_id:id,
            data:ResponseType::Error(error)
        };
    } 
    else if response_type == 3{
        make = Response{
            channel_id:String::new(),
            request_id:id,
            data:ResponseType::Down
        };
    } 
    else if response_type == 4{
        let encrypted:gObject;
        match &data["encrypted"]{
            gObjectValue::object(v)=>{encrypted = v.clone();},
            _=>{return Err(Error!("invalid-body-encrypted-data"));}
        }
        let unlocked:gObject;
        match Encrypted::load_from_g_object(&encrypted){
            Ok(e)=>{
                match e.unlock_first_g_object(&password){
                    Ok(doc)=>{
                        match doc{
                            gObjectValue::object(v)=>{unlocked = v;}
                            _=>{return Err(Error!("invalid-unlocked-g_object"));}
                        }
                    },
                    Err(e)=>{return Err(Error!("failed-unlock_first_g_object"=>e));}
                }
            },
            Err(e)=>{
                return Err(Error!("failed-load_from_g_object"=>e));
            }
        }
        make = Response{
            channel_id:String::new(),
            request_id:id,
            data:ResponseType::Encrypted(unlocked)
        };
    } 
    else {return Err(Error!("failed-invalid-response_type"));}

    Ok(make)

}