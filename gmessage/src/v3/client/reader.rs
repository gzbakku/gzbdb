use crate::v3::client::ipc::{ThreadMessage,ResponseAdd};
use tokio::net::tcp::OwnedReadHalf;
use tokio::sync::mpsc::Sender;
// use tokio::io::AsyncWriteExt;
// use crate::messenger::read_tcp_stream;
use gobject::{gObjectReader,gObjectReaderStatus};
use tokio::io::{AsyncReadExt};
use std::time::Duration;
use tokio::time::sleep;
use std::io::ErrorKind;

pub async fn start(read_half:OwnedReadHalf,response_sender:Sender<ThreadMessage>){

    let mut local_read_half = read_half;
    // let mut overflow = Vec::new();

    // loop {
    //     match read_tcp_stream(&mut local_read_half,&mut overflow).await{
    //         Ok(responses)=>{
    //             // println!("read complete");
                // match response_sender.send(ThreadMessage::ResponseAdd(ResponseAdd{
                //     response:responses[0].clone()
                // })).await {
                //     Ok(_)=>{},
                //     Err(_)=>{}
                // }
    //         },
    //         Err(e)=>{
    //             if e.is("closed".to_string()){
    //                 break;
    //             }
    //         }
    //     }
    // }

    let mut reader = gObjectReader::new();
    let mut wait_time = 0;
    let wait_limit = 100;

    loop{

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
                                match reader.pop(){
                                    Some(doc)=>{
                                        // println!("doc popped");
                                        match response_sender.send(ThreadMessage::ResponseAdd(ResponseAdd{
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