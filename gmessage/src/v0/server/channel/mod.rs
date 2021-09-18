use tokio::runtime::{Builder,Runtime};
use crate::messenger::MessageBox;
use crate::common::Error;
// use crate::Error;
use std::sync::mpsc::Receiver;
use std::future::Future;
use std::sync::{Mutex,Arc};
use crate::server::ipc::{SharedData,ThreadMessage};
use crate::query::{Request,Response};
use tokio::time::sleep;
use std::time::Duration;
use tokio::task;

mod messages;

pub fn start<F,T>(
    channel_read:Receiver<ThreadMessage>,
    data:Arc<Mutex<SharedData>>,
    func:F
)
where
    F:Fn(Request) -> T + Unpin + Send + 'static + Copy + Sync + Clone,
    T:Future<Output = Response> + Send + 'static
{

    let builder = Builder::new_multi_thread()
    .worker_threads(100)
    .thread_name("channel listener")
    .enable_all()
    .thread_stack_size(100 * 1024 * 1024)
    .build();

    let runtime:Runtime;
    match builder{
        Ok(v)=>{runtime = v;},
        Err(_)=>{
            // return Err(Error!("failed-start-tokio-runtime"));
            return;
        }
    }

    // println!("here");

    loop{
        // println!("reading channel message");
        // let local_func = func.clone();
        let local_data = data.clone();
        match channel_read.recv(){
            Ok(message)=>{
                // println!("recv working");
                match message{
                    ThreadMessage::MessageBox(mb)=>{
                        // println!("got message box");
                        runtime.spawn(async move {
                            process_channel(local_data,func,mb).await
                        });
                    },
                    _=>{
                        // println!("other");
                    }
                }
            },
            Err(_)=>{
                // println!("recv failed");
            }
        }
    }

}

// impl Send for process_channel{}

async fn process_channel<F,T>(
    data:Arc<Mutex<SharedData>>,
    func:F,
    message_box:MessageBox,
) -> Result<(),Error>
where
    F:Fn(Request) -> T + Unpin + Send + 'static + Copy + Sync,
    T:Future<Output = Response> + Send + 'static
{

    let mut channel = message_box;

    loop{

        match channel.read(10).await{
            Ok(mut pool)=>{
                if pool.len() > 0{
                    match messages::process_messages(
                        data.clone(),func,
                        &channel.id,&channel.password,
                        &mut pool
                    ).await{
                        Ok(v)=>{
                            for response in v{
                                match channel.send_b(&response).await{
                                    Ok(_)=>{},
                                    Err(_)=>{}
                                }
                            }
                        },
                        Err(_)=>{}
                    }
                }
            },
            Err(e)=>{
                if e.val == "closed"{
                    break;
                }
            }
        }

        // println!("processing messagebox");
        // sleep(Duration::from_secs(1)).await;
    }

    println!("connection closed");

    Ok(())

}


// use crate::server::ipc::{ThreadMessage,Requests};
// use std::collections::HashMap;
// use std::sync::{Arc,RwLock};
