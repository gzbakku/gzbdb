use crate::v4::client::ipc::{ThreadMessage,ResponseAdd};
use tokio::net::tcp::OwnedReadHalf;
use tokio::sync::mpsc::Sender;
// use tokio::io::AsyncWriteExt;
// use crate::messenger::read_tcp_stream;
use gobject::{gObjectReader,gObjectReaderStatus};
use tokio::io::{AsyncReadExt};
use std::time::Duration;
use tokio::time::sleep;
use std::io::ErrorKind;
use std::sync::Arc;
use tokio::sync::RwLock;

pub async fn start(read_half:OwnedReadHalf,response_sender:Sender<ThreadMessage>,password_open:Vec<u8>){

    let mut local_read_half = read_half;
    let password:Arc<RwLock<Vec<u8>>> = Arc::new(RwLock::new(password_open));

    let mut reader = gObjectReader::new();
    let mut wait_time = 0;
    let wait_limit = 100;

    loop{

        let local_password = password.clone();

        let mut buffer = vec![0;1000];
        match local_read_half.read(&mut buffer).await{
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
                                let another_local_password = local_password.clone();
                                match reader.pop(){
                                    Some(doc)=>{
                                        // println!("doc popped");
                                        match response_sender.send(ThreadMessage::ResponseAdd(ResponseAdd{
                                            password:another_local_password,
                                            response:doc["body"].clone()
                                        })).await {
                                            Ok(_)=>{},
                                            Err(_)=>{}
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

}