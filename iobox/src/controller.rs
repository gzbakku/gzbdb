use std::cell::RefCell;
use crate::ipc::{ThreadMessage};
use std::sync::mpsc::{Receiver};
use std::collections::HashMap;

pub async fn start<T>(receiver:Receiver<ThreadMessage<T>>){

    let mut map:HashMap<String,RefCell<T>> = HashMap::new();

    // println!("controller thred started");

    loop {

        match receiver.recv(){
            Ok(msg)=>{
                match msg{
                    ThreadMessage::AddToMap(data)=>{
                        // println!("push message received");
                        match map.insert(data.k,RefCell::new(data.v)){
                            Some(_)=>{},
                            None=>{}
                        }
                        match data.s.send(ThreadMessage::Ok){
                            Ok(_)=>{
                                // println!("push confirmation message sent");
                            },
                            Err(_)=>{}
                        }
                    },
                    ThreadMessage::OnlyAddToMap(data)=>{
                        // println!("push message received");
                        match map.insert(data.k,RefCell::new(data.v)){
                            Some(_)=>{},
                            None=>{}
                        }
                    },
                    ThreadMessage::GetFromMap(data)=>{
                        // println!("get message received");
                        match map.remove(&data.k){
                            Some(posted)=>{
                                match data.s.send(ThreadMessage::SendToPoP(posted)){
                                    Ok(_)=>{},
                                    Err(_)=>{
                                        println!("failed-SendToPoP");
                                        match data.s.send(ThreadMessage::Null){
                                            Ok(_)=>{},
                                            Err(_)=>{}
                                        }
                                    }
                                }
                            },
                            None=>{
                                println!("data-not_found");
                                match data.s.send(ThreadMessage::Null){
                                    Ok(_)=>{},
                                    Err(_)=>{}
                                }
                            }
                        }
                    },
                    _=>{}
                }
            },
            Err(_)=>{
                println!("controller thread crashed");
                break;
            }
        }

    }

}