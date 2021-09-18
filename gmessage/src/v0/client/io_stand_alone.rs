
use tokio::runtime::{Builder,Runtime};
use crate::messenger::MessageBox;
use std::sync::mpsc::Receiver;
use crate::client::ipc::{ThreadMessage};
use crate::common::Error;
use crate::Error;
use tokio::time::sleep;
use std::time::Duration;
use std::collections::HashMap;

pub fn start(connection:&mut MessageBox,reader:Receiver<ThreadMessage>) -> Result<(),Error>{

    let builder = Builder::new_multi_thread()
    .worker_threads(4)
    .thread_name("connection listener")
    .enable_all()
    .thread_stack_size(3 * 1024 * 1024)
    .build();

    let runtime:Runtime;
    match builder{
        Ok(v)=>{runtime = v;},
        Err(_)=>{
            println!("builder failed");
            return Err(Error!("failed-start-tokio-runtime"));
        }
    }

    runtime.block_on(async {
        process_channels(connection,reader).await
    });

    Ok(())

}

use tokio::sync::mpsc::Sender;
use gobject::gObjectValue;
use std::time::Instant;

struct ChannelWorker{
    writer:Sender<ThreadMessage>,
    timeout:u128,
    timer:Instant
}

async fn process_channels(connection:&mut MessageBox,reader:Receiver<ThreadMessage>){

    let mut requests:HashMap<String,ChannelWorker> = HashMap::new();

    let mut last_timeout_check = Instant::now();

    loop {

        {
            match reader.try_recv(){
                Ok(message_type)=>{
                    match message_type{
                        ThreadMessage::Request(request)=>{
                            match connection.try_write(request.body){
                                Ok(_)=>{
                                    match requests.insert(
                                        request.request_id.clone(),
                                        ChannelWorker{
                                            writer:request.writer,
                                            timer:request.timer,
                                            timeout:request.timeout
                                        }
                                    ){Some(_)=>{},None=>{}}
                                    // println!("request sent");
                                },
                                Err(e)=>{
                                    // println!("request send failed");
                                    match request.writer.send(
                                        ThreadMessage::request_error(Error!("failed-send_request"=>e))
                                    ).await{Ok(_)=>{},Err(_)=>{}}
                                }
                            }//send data on message box
                        }//request type message
                        _=>{}//all other messages are ignored   
                    }//check mpsc message type
                },
                Err(_)=>{}//error on mpsc channel message read dont do anything
            }//try read message on mpsc channle
        }//mpsc block

        // println!("here 0");

        {
            if last_timeout_check.elapsed().as_secs() > 5{
                let mut to_remove = Vec::new();
                for (key,val) in requests.iter_mut(){
                    if val.timer.elapsed().as_millis() > val.timeout{
                        // println!("timeout");
                        match val.writer.send(ThreadMessage::RequestTimeout).await{
                            Ok(_)=>{},Err(_)=>{}
                        }
                        to_remove.push(key.clone());
                    }
                }
                loop {
                    match to_remove.pop(){
                        Some(id)=>{
                            // println!("id : {:?}",id);
                            match requests.remove(&id){
                                Some(_)=>{},None=>{}
                            }
                        },
                        None=>{break;}
                    }
                }
                last_timeout_check = Instant::now();
            }
        }

        // println!("here 1");

        loop{
            match &connection.try_read(){
                Ok(responses)=>{
                    if responses.len() > 0{
                        for response in responses.iter(){
                            match &response["id"]{
                                gObjectValue::string(id)=>{
                                    // println!("got response");
                                    match requests.get_mut(id){
                                        Some(channel)=>{
                                            // print!("got request worker");  
                                            match channel.writer.send(
                                                ThreadMessage::ResponseObject(response.clone())
                                            ).await{Ok(_)=>{},Err(_)=>{}}
                                            match requests.remove(id){Some(_)=>{},None=>{}}             
                                        },
                                        None=>{}
                                    }
                                },
                                _=>{}
                            }
                        }
                        
                    } else {//response received
                        // sleep(Duration::from_millis(1)).await
                        // println!("no more to read");
                        break;
                    }
                },
                Err(_)=>{
                    // println!("read failed");
                    break;
                }
            }//try read
        }//connection stream read block

        // println!("here 2");

        // sleep(Duration::from_secs(1)).await;

        // sleep(Duration::from_millis(1)).await;

    }//master loop

}

