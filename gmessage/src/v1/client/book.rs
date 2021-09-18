// use tokio::runtime::{Builder,Runtime};
// use std::sync::{Mutex,Arc};
use crate::v1::client::ipc::{ThreadMessage,RequestSuccessfull};
// use crate::common::Error;
// use crate::Error;
// use std::sync::mpsc::Receiver;
use tokio::sync::mpsc::{Receiver,Sender};
// use crate::messenger::read_tcp_stream;
use std::collections::HashMap;

use std::sync::Arc;
use tokio::sync::{Mutex};

pub async fn start(book_receiver:Receiver<ThreadMessage>){

    let mut book:HashMap<String,Arc<Mutex<Sender<ThreadMessage>>>> = HashMap::new();

    let mut local_receiver = book_receiver;

    loop{

        match local_receiver.recv().await{
            Some(msg)=>{
                match msg{
                    ThreadMessage::BookAdd(message)=>{
                        match book.insert(message.id,message.sender){
                            Some(_)=>{},
                            None=>{}
                        }
                    },
                    ThreadMessage::BookDelete(message)=>{
                        match book.remove(&message.id){
                            Some(_)=>{},
                            None=>{}
                        }
                    },
                    ThreadMessage::BookFinish(message)=>{
                        match book.get_mut(&message.response.request_id){
                            Some(channel)=>{
                                let locked = channel.lock().await;
                                match locked.send(ThreadMessage::RequestSuccessfull(RequestSuccessfull{
                                    response:message.response
                                })).await{
                                    Ok(_)=>{},
                                    Err(_)=>{}
                                }
                            },
                            None=>{}
                        }
                    },
                    _=>{}
                }
            },
            None=>{}
        }

    }

}