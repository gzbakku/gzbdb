use crate::Error;
use crate::common::Error;
use tokio::sync::mpsc::Sender;
use crate::v3::client::ipc::{ThreadMessage,WriterSend,WakerBook,ResponseHolder,ResponseHolderValue};
use gobject::{gObject,gObjectValue};
use crate::crypt::aes::encrypt;
use rand::{distributions::Alphanumeric, Rng};
use std::sync::Arc;
use tokio::sync::{RwLock};
use tokio::time::timeout;
use core::time::Duration;
use crate::query::{Response};
use std::sync::Mutex as StdMutex;
use tokio::sync::Notify;
use dashmap::DashMap;

#[derive(Debug,Clone)]
pub struct RequestBuilder{
    pub id:Arc<RwLock<String>>,
    pub password:Arc<RwLock<Vec<u8>>>,
    // pub book_sender:Arc<Mutex<Sender<ThreadMessage>>>,
    pub writer_sender:Sender<ThreadMessage>,
    pub waker_book:Arc<DashMap<String,WakerBook>>
}

#[allow(dead_code)]
impl RequestBuilder{
    pub fn new(
        id:String,
        password:Vec<u8>,
        // book_sender:Sender<ThreadMessage>,
        writer_sender:Sender<ThreadMessage>,
        waker_book:Arc<DashMap<String,WakerBook>>
    )->RequestBuilder{
        RequestBuilder{
            id:Arc::new(RwLock::new(id)),
            password:Arc::new(RwLock::new(password)),
            // book_sender:Arc::new(Mutex::new(book_sender)),
            writer_sender:writer_sender,
            waker_book:waker_book
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

        let notify = Arc::new(Notify::new());
        let waker = notify.clone();

        let response_holder:Arc<StdMutex<ResponseHolder>> = Arc::new(StdMutex::new(ResponseHolder {
            hold:ResponseHolderValue::Null
        }));

        match self.waker_book.insert(id.clone(),WakerBook{
            response_holder:response_holder.clone(),
            notify:waker
        }){
            Some(_)=>{
                // println!("request replaced in dashmap");
            },
            None=>{
                // println!("request inserted in dashmap");
            }
        }

        match self.writer_sender.send(ThreadMessage::WriterSend(WriterSend{
            data:built
        })).await{
            Ok(_)=>{
                // println!("writer sent");
            },
            Err(e)=>{
                return Err(Error!(format!("failed-send_to_writer => {:?}",e)));
            }
        }

        match timeout(Duration::from_millis(timeout_in_millis),notify.notified()).await{
            Ok(_)=>{
                match response_holder.lock(){
                    Ok(locked)=>{
                        match &locked.hold{
                            ResponseHolderValue::Null=>{
                                return Err(Error!("timeout"));
                            },
                            ResponseHolderValue::Response(v)=>{
                                return Ok(v.clone());
                            }
                        }
                    },
                    Err(_)=>{
                        return Err(Error!("failed-lock-response_holder"));
                    }
                }
            },
            Err(_)=>{
                return Err(Error!("failed-get_message"));
            }
        }

    }
}