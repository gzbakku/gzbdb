use tokio::runtime::{Builder,Runtime};
use std::sync::{Arc};
use crate::v1::server::ipc::{ThreadMessage,ReaderMessage,ExecuterMessage};
use crate::common::Error;
use crate::Error;
use std::sync::mpsc::{Receiver};
use crate::messenger::read_tcp_stream;
// use std::sync::RwLock;
use tokio::sync::RwLock;
use tokio::sync::mpsc::Sender as TokioSender;
use tokio::sync::Mutex;

pub fn start(
    reader_read_half:Receiver<ThreadMessage>,
    executer_writer_locked:Arc<Mutex<TokioSender<ThreadMessage>>>
) -> Result<(),Error>{

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
        let local_executer_writer_locked = executer_writer_locked.clone();
        match reader_read_half.recv(){
            Ok(msg)=>{
                match msg{
                    ThreadMessage::ReaderMessage(data)=>{
                        runtime.spawn(async move {
                            let mut hold = data;
                            read_on_connection(&mut hold,local_executer_writer_locked).await
                        });
                    },
                    _=>{}
                }
            },
            Err(_)=>{
                break;
            }
        }
    }

    Ok(())

}

async fn read_on_connection(
    data:&mut ReaderMessage,
    executer_writer_locked:Arc<Mutex<TokioSender<ThreadMessage>>>
){

    let mut overflow = Vec::new();
    let password:Arc<RwLock<Vec<u8>>> = Arc::new(RwLock::new(data.password.clone()));

    loop{
        match read_tcp_stream(&mut data.read_half,&mut overflow).await{
            Ok(requests)=>{
                // println!("requests received");
                // println!("{:?}",requests);
                let locked = executer_writer_locked.lock().await;
                match locked.send(ThreadMessage::ExecuterMessage(ExecuterMessage{
                    sender:data.sender.clone(),
                    password:password.clone(),
                    connection_id:data.connection_id.clone(),
                    request:requests[0].clone()
                })).await{
                    Ok(_)=>{
                        // println!("executer message sent");
                    },
                    Err(_)=>{
                        // println!("executer message failed");
                    }
                }
            },
            Err(e)=>{
                if e.is("closed".to_string()){
                    break;
                }
            }
        }
    }

    println!("!!! connection closed : {:?}",data.connection_id);

}