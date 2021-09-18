use tokio::runtime::{Builder,Runtime};
// use std::sync::{Mutex,Arc};
use crate::v3::server::ipc::{ThreadMessage,WriterMessage};
use crate::common::Error;
use crate::Error;
use std::sync::mpsc::Receiver;
// use crate::messenger::read_tcp_stream;

pub fn start(reader_read_half:Receiver<ThreadMessage>) -> Result<(),Error>{

    let builder = Builder::new_multi_thread()
    .worker_threads(10)
    .thread_name("connection listener")
    .enable_all()
    .thread_stack_size(32 * 1024 * 1024)
    .build();

    let runtime:Runtime;
    match builder{
        Ok(v)=>{runtime = v;},
        Err(_)=>{return Err(Error!("failed-start-tokio-runtime"));}
    }

    loop{
        match reader_read_half.recv(){
            Ok(msg)=>{
                match msg{
                    ThreadMessage::WriterMessage(data)=>{
                        runtime.spawn(async move {
                            let mut hold = data;
                            write_on_connection(&mut hold).await
                        });
                    },
                    _=>{}
                }
            },
            Err(_)=>{
                println!("writer main receiver dropped");
                break;
            }
        }
    }

    Ok(())

}

use gobject::gObject;
use tokio::io::AsyncWriteExt;

async fn write_on_connection(data:&mut WriterMessage){

    match data.write_half.write(&gObject!{
        "ready"=>gObjectValue::bool(true)
    }.build()).await{
        Ok(_)=>{},
        Err(_)=>{return;}
    }

    // println!("write conn message receive");

    loop {
        // println!("reading writer conn message");
        match data.receiver.recv().await{
            Some(msg)=>{
                // println!("writer message received");
                match msg{
                    ThreadMessage::WriterResponseMessage(message)=>{
                        // println!("{:?}",&message.data);
                        match data.write_half.write(&message.data).await{
                            Ok(_)=>{
                                // println!("writer response sent");
                            },
                            Err(_)=>{}
                        }
                    },
                    _=>{}
                }
            },
            None=>{
                println!("writer conn receiver dropped");
                break;
            }
        }
    }

}