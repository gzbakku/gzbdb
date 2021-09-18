use tokio::runtime::{Builder,Runtime};
use std::sync::{Arc};
use crate::v3::server::ipc::{ThreadMessage,ReaderMessage,ExecuterMessage};
use crate::common::Error;
use crate::Error;
use std::sync::mpsc::{Receiver};
// use crate::messenger::read_tcp_stream;
use tokio::sync::RwLock;
use tokio::sync::mpsc::Sender as TokioSender;
use std::io::ErrorKind;
use tokio::io::{AsyncReadExt};

pub fn start(
    reader_read_half:Receiver<ThreadMessage>,
    executer_writer_locked:TokioSender<ThreadMessage>
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

use std::time::Duration;
use tokio::time::sleep;
use gobject::{gObjectReader,gObjectReaderStatus};

async fn read_on_connection(
    data:&mut ReaderMessage,
    // executer_writer_locked:Arc<Mutex<TokioSender<ThreadMessage>>>,
    executer_writer_locked:TokioSender<ThreadMessage>
){

    // let mut overflow = Vec::new();
    let password:Arc<RwLock<Vec<u8>>> = Arc::new(RwLock::new(data.password.clone()));

    // let mut num = 0;

    let mut reader = gObjectReader::new();
    let mut wait_time = 0;
    let wait_limit = 100;

    loop{

        // println!("reading");
        
        // sleep(Duration::from_millis(1000)).await;

        // let mut buffer = [0;100];
        let mut buffer = vec![0;1000];
        
        

        match data.read_half.read(&mut buffer).await{
            Ok(size)=>{
                
                if size == 0{
                    if wait_time < wait_limit{
                        wait_time += 10;
                    }
                    sleep(Duration::from_millis(wait_time)).await;
                } else {
                    // let (first,_) = buffer.split_at(size);
                    buffer.truncate(size);
                    wait_time = 0;
                    match reader.push(&buffer){
                        gObjectReaderStatus::Doc=>{
                            // println!("doc found");
                            loop{
                                match reader.pop(){
                                    Some(doc)=>{
                                        // println!("doc popped");
                                        // println!("{:?}",doc["body"]);
                                        match executer_writer_locked.send(ThreadMessage::ExecuterMessage(ExecuterMessage{
                                            sender:data.sender.clone(),
                                            password:password.clone(),
                                            connection_id:data.connection_id.clone(),
                                            request:doc["body"].clone()
                                        })).await{
                                            Ok(_)=>{
                                                // println!("executer message sent");
                                            },
                                            Err(_)=>{
                                                // println!("executer message failed");
                                            }
                                        }
                                    },
                                    None=>{
                                        break;
                                    }
                                }
                            }
                        },
                        _=>{}
                    }
                }
            },
            Err(e)=>{
                match e.kind() {
                    ErrorKind::WouldBlock=>{break;},
                    ErrorKind::ConnectionAborted=>{break;},
                    ErrorKind::Interrupted=>{break;},
                    ErrorKind::PermissionDenied=>{break;},
                    ErrorKind::BrokenPipe=>{break;},
                    ErrorKind::ConnectionReset=>{break;},
                    ErrorKind::NotConnected=>{break;},
                    _=>{}
                }
            }
        }

    }

    println!("!!! connection closed : {:?}",data.connection_id);

}