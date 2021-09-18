use crate::Error;
use crate::common::Error;
use tokio::sync::mpsc::Sender;
use crate::v1::client::ipc::{ThreadMessage,BookAdd,WriterSend};
use gobject::{gObject,gObjectValue};
use crate::crypt::aes::encrypt;
use rand::{distributions::Alphanumeric, Rng};
use std::sync::Arc;
use tokio::sync::{RwLock,Mutex};
use tokio::sync::mpsc;
use tokio::time::timeout;
use core::time::Duration;
use crate::query::{Response};

pub struct RequestBuilder{
    pub id:Arc<RwLock<String>>,
    pub password:Arc<RwLock<Vec<u8>>>,
    pub book_sender:Arc<Mutex<Sender<ThreadMessage>>>,
    pub writer_sender:Arc<Mutex<Sender<ThreadMessage>>>
}

impl RequestBuilder{
    pub fn new(
        id:String,
        password:Vec<u8>,
        book_sender:Sender<ThreadMessage>,
        writer_sender:Sender<ThreadMessage>
    )->RequestBuilder{
        RequestBuilder{
            id:Arc::new(RwLock::new(id)),
            password:Arc::new(RwLock::new(password)),
            book_sender:Arc::new(Mutex::new(book_sender)),
            writer_sender:Arc::new(Mutex::new(writer_sender))
        }
    }
    pub async fn send(&self,body:gObject,timeout_in_millis:u64,encrypted:bool) -> Result<Response,Error>{

        let body_val:gObjectValue;
        if encrypted{
            let password = self.password.read().await;
            match encrypt(&password,&body.build()){
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

        let (request_sender,mut request_receiver) = mpsc::channel::<ThreadMessage>(2);

        let book_request_sender = Arc::new(Mutex::new(request_sender.clone()));
        // let writer_request_sender = Arc::new(Mutex::new(request_sender));

        let book_lock = self.book_sender.lock().await;
        match book_lock.send(ThreadMessage::BookAdd(BookAdd{
            id:id.clone(),
            sender:book_request_sender
        })).await{
            Ok(_)=>{},
            Err(_)=>{
                return Err(Error!("failed-send_to_book"));
            }
        }

        let writer_lock = self.writer_sender.lock().await;
        match writer_lock.send(ThreadMessage::WriterSend(WriterSend{
            data:built
        })).await{
            Ok(_)=>{},
            Err(_)=>{
                return Err(Error!("failed-send_to_writer"));
            }
        }
        
        match timeout(Duration::from_millis(timeout_in_millis),request_receiver.recv()).await{
            Ok(result)=>{
                match result{
                    Some(msg)=>{
                        match msg{
                            ThreadMessage::RequestFailed=>{
                                return Err(Error!("failed-RequestFailed"));
                            },
                            ThreadMessage::RequestSuccessfull(data)=>{
                                return Ok(data.response);
                            },
                            _=>{}
                        }
                    },
                    None=>{
                        return Err(Error!("failed-get_result-message"));
                    }
                }
            },
            Err(_)=>{
                return Err(Error!("failed-get_message"));
            }
        }

        return Err(Error!("unknow-error"));

    }
}