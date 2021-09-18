use std::sync::{Arc};
use crate::controller::start as controller_start;
use std::sync::mpsc::{channel,Sender};
use crate::ipc::{ThreadMessage,AddToMap,GetFromMap,OnlyAddToMap};
use utils::Error;
use tokio::sync::Notify;
use tokio::time::sleep;
use std::time::Duration;
use tokio::runtime::Handle;

#[derive(Debug,Clone)]
pub struct IoBox<T>
where
    T:Send + 'static
{
    sender:Sender<ThreadMessage<T>>
}

use std::fmt::Debug;

impl<T> IoBox<T>
where
    T:Send + 'static + Debug
{
    pub async fn new() -> IoBox<T>{
        let (sender, receiver) = channel::<ThreadMessage<T>>();
        let handle = Handle::current();
        handle.spawn(async {
            controller_start(receiver).await;
        });
        sleep(Duration::from_millis(200)).await;
        IoBox{
            sender:sender
        }
    }
    pub async fn push(&mut self,k:String,v:T) -> Result<(),Error>{
        // println!("pushing");
        let (sender, receiver) = channel::<ThreadMessage<T>>();
        match self.sender.send(ThreadMessage::AddToMap(AddToMap{
            k:k,
            v:v,
            s:sender
        })){
            Ok(_)=>{
                // println!("push message sent");
            },
            Err(_)=>{
                return Err(Error!("failed-send_to_main_thread"));
            }
        }
        match receiver.recv(){
            Ok(msg)=>{
                match msg{
                    ThreadMessage::Ok=>{
                        // println!("push notitication received");
                        return Ok(());
                    },
                    _=>{
                        return Err(Error!("invalid-confirmation_message-received")); 
                    }
                }
            },
            Err(_)=>{
                return Err(Error!("failed-receive_confirmation_message")); 
            }
        }
    }
    pub async fn push_unchecked(&mut self,k:String,v:T) -> Result<(),Error>{
        match self.sender.send(ThreadMessage::OnlyAddToMap(OnlyAddToMap{
            k:k,
            v:v
        })){
            Ok(_)=>{
                // println!("push message sent");
                return Ok(());
            },
            Err(_)=>{
                return Err(Error!("failed-send_to_main_thread"));
            }
        }
    }
    pub async fn pop(&mut self,k:String) -> Result<T,Error>{

        // println!("pop called");

        let (sender, receiver) = channel::<ThreadMessage<T>>();

        // println!("sending get message");
        match self.sender.send(ThreadMessage::GetFromMap(GetFromMap{
            k:k,s:sender
        })){
            Ok(_)=>{},
            Err(_)=>{
                return Err(Error!("failed-send_to_main_thread"));
            }
        }
        // println!("get message sent");
        // sleeper.notified().await;

        match receiver.recv(){
            Ok(msg)=>{
                // println!("message received");
                match msg{
                    ThreadMessage::SendToPoP(data)=>{
                        return Ok(data.into_inner());
                    },
                    _=>{
                        return Err(Error!("not_found"));
                    }
                }
            },
            Err(_)=>{
                return Err(Error!("failed-receive_data_from_controller"));
            }
        }

        // return Err(Error!("no_error"));

    }
}

