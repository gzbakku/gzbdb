use crate::v2::client::ipc::{ThreadMessage,ResponseAdd};
use tokio::net::tcp::OwnedReadHalf;
use tokio::sync::mpsc::Sender;
// use tokio::io::AsyncWriteExt;
use crate::messenger::read_tcp_stream;

pub async fn start(read_half:OwnedReadHalf,response_sender:Sender<ThreadMessage>){

    let mut local_read_half = read_half;
    let mut overflow = Vec::new();

    loop {
        match read_tcp_stream(&mut local_read_half,&mut overflow).await{
            Ok(responses)=>{
                // println!("read complete");
                match response_sender.send(ThreadMessage::ResponseAdd(ResponseAdd{
                    response:responses[0].clone()
                })).await {
                    Ok(_)=>{},
                    Err(_)=>{}
                }
            },
            Err(e)=>{
                if e.is("closed".to_string()){
                    break;
                }
            }
        }
    }

}