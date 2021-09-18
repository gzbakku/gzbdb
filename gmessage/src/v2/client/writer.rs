use crate::v2::client::ipc::{ThreadMessage};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::mpsc::Receiver;
use tokio::io::AsyncWriteExt;

pub async fn start(reader:Receiver<ThreadMessage>,write_half:OwnedWriteHalf){

    let mut local_write_half = write_half;
    let mut local_reader = reader;

    loop {
        match local_reader.recv().await{
            Some(msg)=>{
                // println!("writer message received");
                match msg{
                    ThreadMessage::WriterSend(message)=>{
                        // println!("data to send received");
                        match local_write_half.write(&message.data).await{
                            Ok(_)=>{
                                // println!("socket data sent");
                            },
                            Err(_)=>{}
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

}