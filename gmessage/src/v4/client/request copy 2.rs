use crate::Error;
use crate::common::Error;
use tokio::sync::mpsc::Sender;
use crate::v4::client::ipc::{ThreadMessage,WriterSend,WakerBook,ResponseHolder,ResponseHolderValue};
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
use std::collections::HashMap;

#[derive(Debug,Clone)]
pub struct RequestBuilder{
    pub channels:u32,
    pub last_channel:Arc<StdMutex<u32>>,
    // pub id:Arc<RwLock<String>>,
    // pub password:Arc<RwLock<Vec<u8>>>,
    pub passwords:Arc<RwLock<HashMap<u32,Vec<u8>>>>,
    pub writer_sender:Sender<ThreadMessage>,
    pub waker_book:Arc<DashMap<String,WakerBook>>
}

#[allow(dead_code)]
impl RequestBuilder{
    pub fn new(
        channels:u32,
        // id:String,
        // password:Vec<u8>,
        passwords:HashMap<u32,Vec<u8>>,
        writer_sender:Sender<ThreadMessage>,
        waker_book:Arc<DashMap<String,WakerBook>>
    )->RequestBuilder{
        RequestBuilder{
            channels:channels,
            // id:Arc::new(RwLock::new(id)),
            // password:Arc::new(RwLock::new(password)),
            passwords:Arc::new(RwLock::new(passwords)),
            last_channel:Arc::new(StdMutex::new(0)),
            writer_sender:writer_sender,
            waker_book:waker_book
        }
    }
    pub async fn send(&self,body:gObject,timeout_in_millis:u64,encrypted:bool) -> Result<Response,Error>{

        let mut get_channel:u32 = 0;
        let mut channel_found = true;
        {
            match self.last_channel.lock(){
                Ok(mut locked)=>{
                    get_channel = locked.clone();
                    if locked.clone() == self.channels.clone(){
                        *locked = 1;
                    } else {
                        *locked += 1;
                    }
                },
                Err(_)=>{
                    channel_found = false;
                }
            }
        }
        if !channel_found{
            return Err(Error!("failed-get_channel"));
        }

        let body_val:gObjectValue;
        if encrypted{
            let passwords = self.passwords.read().await;
            match passwords.get(&get_channel){
                Some(password)=>{
                    match encrypt(&password,&body.build()){
                        Ok(b)=>{body_val = gObjectValue::object(b.g_object());},
                        Err(e)=>{
                            return Err(Error!("failed-ecnrypt-request"=>e));
                        }
                    }
                },
                None=>{
                    return Err(Error!("failed-get_password"));
                }
            }
        } else {
            body_val = gObjectValue::object(body);
        }

        // let body_val:gObjectValue;
        // if encrypted{
        //     let password = self.password.read().await;
        //     match encrypt(&password,&body.build()){
        //         Ok(b)=>{body_val = gObjectValue::object(b.g_object());},
        //         Err(e)=>{
        //             return Err(Error!("failed-ecnrypt-request"=>e));
        //         }
        //     }
        // } else {
        //     body_val = gObjectValue::object(body);
        // }

        let id:String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();

        // let built = gObject!{
        //     "id"=>gObjectValue::string(id.clone()),
        //     "encrypted"=>gObjectValue::bool(encrypted),
        //     "body"=>body_val
        // }.build();

        // let notify = Arc::new(Notify::new());
        // let waker = notify.clone();

        // let response_holder:Arc<StdMutex<ResponseHolder>> = Arc::new(StdMutex::new(ResponseHolder {
        //     hold:ResponseHolderValue::Null
        // }));

        // match self.waker_book.insert(id.clone(),WakerBook{
        //     response_holder:response_holder.clone(),
        //     notify:waker
        // }){
        //     Some(_)=>{
        //         // println!("request replaced in dashmap");
        //     },
        //     None=>{
        //         // println!("request inserted in dashmap");
        //     }
        // }

        // match self.writer_sender.send(ThreadMessage::WriterSend(WriterSend{
        //     data:built
        // })).await{
        //     Ok(_)=>{
        //         // println!("writer sent");
        //     },
        //     Err(e)=>{
        //         return Err(Error!(format!("failed-send_to_writer => {:?}",e)));
        //     }
        // }

        // match timeout(Duration::from_millis(timeout_in_millis),notify.notified()).await{
        //     Ok(_)=>{
        //         match response_holder.lock(){
        //             Ok(locked)=>{
        //                 match &locked.hold{
        //                     ResponseHolderValue::Null=>{
        //                         return Err(Error!("timeout"));
        //                     },
        //                     ResponseHolderValue::Response(v)=>{
        //                         return Ok(v.clone());
        //                     }
        //                 }
        //             },
        //             Err(_)=>{
        //                 return Err(Error!("failed-lock-response_holder"));
        //             }
        //         }
        //     },
        //     Err(_)=>{
        //         return Err(Error!("failed-get_message"));
        //     }
        // }

        return Err(Error!("no_error"));

    }
}