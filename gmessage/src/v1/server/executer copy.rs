use tokio::runtime::{Builder,Runtime};
use std::sync::{Mutex,Arc};
use crate::v1::server::ipc::{SharedData,ThreadMessage,ExecuterMessage,WriterResponseMessage};
use crate::common::Error;
use crate::Error;
// use std::sync::mpsc::Receiver;
use crate::query::{Request,Response};
use std::future::Future;
// use std::sync::RwLock;
use tokio::sync::RwLock;
use tokio::sync::mpsc::Receiver;
use tokio::sync::Mutex as TokioMutex;

pub fn start<F,T>(
    data:Arc<TokioMutex<SharedData>>,
    func:F,
    reader:Receiver<ThreadMessage>
) -> Result<(),Error>
where
    F:Fn(Arc<TokioMutex<SharedData>>,Request) -> T + Unpin + Send + 'static + Copy + Sync,
    T:Future<Output = Response> + Send + 'static
{

    let builder = Builder::new_multi_thread()
    .worker_threads(250)
    .thread_name("message listener")
    .enable_all()
    .thread_stack_size(100 * 1024 * 1024)
    .build();

    let runtime:Runtime;
    match builder{
        Ok(v)=>{runtime = v;},
        Err(_)=>{return Err(Error!("failed-start-tokio-runtime"));}
    }

    runtime.block_on(async {
        let mut local_receiver = reader;
        loop{
            let local_data = data.clone();
            match local_receiver.recv().await{
                Some(msg)=>{
                    // println!("executer message received");
                    match msg{
                        ThreadMessage::ExecuterMessage(data)=>{
                            runtime.spawn(async move {
                                let hold = data;
                                execute_request(local_data,func,hold).await
                            });
                        },
                        _=>{}
                    }
                },
                None=>{
                    // println!("executer receive failed");
                }
            }
        }
    });

    Ok(())

}

use gobject::{gObjectValue,gObject,gSchema};
// use tokio::io::AsyncWriteExt;

async fn execute_request<F,T>(
    data:Arc<TokioMutex<SharedData>>,
    func:F,
    message:ExecuterMessage
)
where
    F:Fn(Arc<TokioMutex<SharedData>>,Request) -> T + Unpin + Send + 'static + Copy + Sync,
    T:Future<Output = Response> + Send + 'static
{

    // println!("executing request");

    // match func().await

    match process_request(
        data, 
        &message.connection_id, 
        message.request, 
        func, 
        message.password
    ).await{
        Ok(response)=>{
            let locked = message.sender.lock().await;
            // println!("lock created");
            match locked.send(ThreadMessage::WriterResponseMessage(WriterResponseMessage{
                data:response
            })).await{
                Ok(_)=>{
                    // println!("response sent to writer");
                },
                Err(_)=>{
                    // println!("response send failed");
                }
            }
        },
        Err(_)=>{
            // println!("request failed");
        }
    }

}

async fn process_request<F,T>(
    data:Arc<TokioMutex<SharedData>>,
    channel_id:&str,
    r:gObjectValue,
    func:F,
    password_lock:Arc<RwLock<Vec<u8>>>
) -> Result<Vec<u8>,Error>
where
    F:Fn(Arc<TokioMutex<SharedData>>,Request) -> T + Unpin + Send + 'static + Copy + Sync,
    T:Future<Output = Response> + Send + 'static
{

    let password = password_lock.read().await;

    match parse_request(&channel_id,r, &password){
        Ok(parsed)=>{
            match parse_request_to_struct(&channel_id,&parsed){
                Ok(request)=>{
                    match func(data,request).await.parse(&password){
                        Ok(parsed)=>{
                            return Ok(parsed.build());
                        },
                        Err(_)=>{
                            return Err(Error!("failed-parse_response"));
                        }
                    }
                },
                Err(_)=>{
                    return Err(Error!("failed-parse_request_to_struct"));
                }
            }
        },
        Err(_)=>{
            return Err(Error!("failed-parse_request"));
        }
    }
                    
}

fn parse_request_to_struct(channel_id:&str,o:&gObjectValue) -> Result<Request,Error>{

    let hold:gObject;
    match o{
        gObjectValue::object(v)=>{hold = v.clone();},
        _=>{return Err(Error!("invalid_request"));}
    }

    match gSchema!{
        "id"=>gSchemaValue::string,
        "body"=>gSchemaValue::object
    }.validate(&hold){
        Ok(_)=>{},
        Err(_)=>{
            return Err(Error!("invalid_request-schema"));
        }
    }

    let id:String;
    match &hold["id"]{
        gObjectValue::string(v)=>{id = v.clone();},
        _=>{return Err(Error!("invalid_request-id-data_type"));}
    }

    let body:gObject;
    match &hold["body"]{
        gObjectValue::object(v)=>{body = v.clone();},
        _=>{return Err(Error!("invalid_request-body-data_type"));}
    }

    return Ok(Request{
        channel_id:channel_id.to_string(),
        request_id:id,
        body:body
    });

}

use crate::crypt::aes::Encrypted;

fn parse_request(id:&str,request:gObjectValue,password:&Vec<u8>) -> Result<gObjectValue,Error>{

    let schema = gSchema!{
        "id"=>gSchemaValue::string,
        "encrypted"=>gSchemaValue::bool,
        "body"=>gSchemaValue::object
    };

    match schema.validate_value(&request){
        Ok(_)=>{},
        Err(_)=>{return Err(Error!("invalid_schema"));}
    }

    match &request["encrypted"]{
        gObjectValue::bool(v)=>{
            if v == &false {return Ok(request);}
        },
        _=>{return Err(Error!("invalid_schema-item-id"));}
    }

    let body:gObject;
    match &request["body"]{
        gObjectValue::object(b)=>{body = b.clone();},
        _=>{return Err(Error!("failed-extract-body"));}
    }

    let crypter:Encrypted;
    match Encrypted::load_from_g_object(&body){
        Ok(v)=>{crypter = v;},
        Err(_)=>{return Err(Error!("invalid_schema-item-body"));}
    }

    match crypter.unlock_first_g_object(&password){
        Ok(v)=>{
            return Ok(gObjectValue::object(gObject!{
                "id"=>request["id"].clone(),
                "encrypted"=>request["encrypted"].clone(),
                "body"=>v
            }));                                                            
        },
        Err(_)=>{
            return Err(Error!("not_found-unlock-encrypted"));
        }
    }

}