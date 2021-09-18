use crate::v4::client::ipc::{ThreadMessage};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::mpsc::Receiver;
use tokio::io::AsyncWriteExt;
use std::collections::HashMap;

pub async fn start(reader:Receiver<ThreadMessage>,writers:HashMap<u32,OwnedWriteHalf>){

    // let mut local_write_half = write_half;
    let mut local_reader = reader;
    let mut local_writers = writers;

    loop {
        match local_reader.recv().await{
            Some(msg)=>{
                // println!("writer message received");
                match msg{
                    ThreadMessage::WriterSend(message)=>{
                        // println!("data to send received");

                        match local_writers.get_mut(&message.id){
                            Some(writer)=>{
                                match writer.write(&message.data).await{
                                    Ok(_)=>{
                                        // println!("socket data sent");
                                    },
                                    Err(_)=>{}
                                }
                            },
                            None=>{}
                        }

                        
                    },
                    _=>{}
                }
            },
            None=>{
                break;
            }
        }
    }
    

    // loop {
    //     match local_reader.recv().await{
    //         Some(msg)=>{
    //             // println!("writer message received");
    //             match msg{
    //                 ThreadMessage::WriterSend(message)=>{
    //                     // println!("data to send received");
    //                     match local_write_half.write(&message.data).await{
    //                         Ok(_)=>{
    //                             // println!("socket data sent");
    //                         },
    //                         Err(_)=>{}
    //                     }
    //                 },
    //                 _=>{}
    //             }
    //         },
    //         None=>{
    //             break;
    //         }
    //     }
    // }

}