
use tokio::runtime::{Builder,Runtime};
use crate::messenger::{MessageBox,read_tcp_stream};
use tokio::sync::mpsc::Receiver;
use crate::client::ipc::{ThreadMessage};
use crate::common::Error;
use crate::Error;
use tokio::time::{interval,sleep};
use std::time::Duration;
use std::collections::HashMap;
use tokio::task;
use tokio::io::AsyncWriteExt;

pub fn start(connection:MessageBox,reader:Receiver<ThreadMessage>) -> Result<(),Error>{

    let builder = Builder::new_multi_thread()
    .worker_threads(50)
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

    runtime.block_on(async move {
        process_channels(connection,reader).await
    });

    Ok(())

}

use tokio::sync::mpsc::Sender;
use gobject::gObjectValue;
use std::time::Instant;

struct ChannelWorker{
    writer:Sender<ThreadMessage>,
}

use std::sync::Arc;
use tokio::sync::Mutex;

async fn process_channels(connection:MessageBox,reader:Receiver<ThreadMessage>){

    let mut global_workers:Arc<Mutex<HashMap<String,ChannelWorker>>> = Arc::new(Mutex::new(HashMap::new()));

    let (mut OwnedReadHalf,mut OwnedWriteHalf) = connection.stream.into_inner().into_split();

    let writer_workers = global_workers.clone();
    let writer_mprc_reader:Arc<Mutex<Receiver<ThreadMessage>>> =Arc::new(Mutex::new(reader));
    task::spawn(async move {
        let mut writer = OwnedWriteHalf;
        let local_reader = writer_mprc_reader.clone();
        let global_write_task_workers = writer_workers.clone();
        loop{
            let workers_0 = global_write_task_workers.clone();
            let workers_1 = global_write_task_workers.clone();
            let mut locked_reader = local_reader.lock().await;
            match locked_reader.recv().await{
                Some(message)=>{
                    match message{
                        ThreadMessage::Request(request)=>{
                            match writer.write(&request.body).await{
                                Ok(s)=>{
                                    // writer.write(&vec![0,0,0]).await;
                                    // println!("written : {:?}",s);
                                    // println!("data : {:?}",request.body);
                                    let id = request.request_id.clone();
                                    let timeout = request.timeout.clone();
                                    {
                                        let mut lock = workers_0.lock().await;
                                        match lock.insert(id.clone(),ChannelWorker{
                                            writer:request.writer
                                        }){
                                            Some(_)=>{},
                                            None=>{}
                                        }
                                    }
                                    task::spawn(async move {
                                        let task_workers = workers_1.clone();
                                        let local_id = id.clone();
                                        let local_timeout = timeout.clone();
                                        sleep(Duration::from_millis(local_timeout)).await;
                                        {
                                            let mut lock = task_workers.lock().await;
                                            match lock.get_mut(&local_id){
                                                Some(worker)=>{
                                                    match worker.writer.send(
                                                        ThreadMessage::RequestTimeout
                                                    ).await{Ok(_)=>{},Err(_)=>{}} 
                                                },None=>{}
                                            }
                                            match lock.remove(&local_id){Some(_)=>{},None=>{}}
                                        }
                                    });
                                },
                                Err(e)=>{
                                    match request.writer.send(
                                        ThreadMessage::RequestError(
                                            Error!(
                                                format!("failed-send-request => {:?}",e)
                                            )
                                        )
                                    ).await{
                                        Ok(_)=>{},Err(_)=>{}
                                    }
                                }
                            }
                        },
                        _=>{}
                    }
                },
                None=>{}
            }
        }
    });//writer task

    let reader_workers = global_workers.clone();
    let mut read_half = OwnedReadHalf;
    let mut overflow:Vec<u8> = Vec::new();
    loop{
        match read_tcp_stream(&mut read_half,&mut overflow).await{
            Ok(mut blocks)=>{
                let mut locked = reader_workers.lock().await;
                loop {
                    match blocks.pop(){
                        Some(response)=>{
                            match &response["id"]{
                                gObjectValue::string(id)=>{
                                    match locked.get_mut(id){
                                        Some(channel)=>{
                                            match channel.writer.send(
                                                ThreadMessage::ResponseObject(response.clone())
                                            ).await{Ok(_)=>{},Err(_)=>{}}
                                        },
                                        None=>{}
                                    }
                                    match locked.remove(id){
                                        Some(_)=>{},None=>{}
                                    }
                                },
                                _=>{}
                            }
                        },
                        None=>{break;}
                    }
                }
                
            },
            Err(_)=>{}
        }
    } 

}

