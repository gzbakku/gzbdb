use tokio::runtime::{Builder,Runtime};
use std::sync::{Mutex,Arc,RwLock};
use crate::server::ipc::{SharedData,ChannelMessages,ResponseResult};
use crate::common::Error;
use crate::Error;
use std::future::Future;
use crate::query::{Request,Response};
use std::sync::mpsc::{Sender,Receiver};
use crate::server::ipc::ThreadMessage;
use tokio::time::sleep;
use std::time::Duration;
use gobject::{gObjectValue,gObject,gSchema};
use futures::future::join_all;

pub fn start<F,T>(
    data:Arc<Mutex<SharedData>>,
    func:F,
    channel_writer:Arc<Mutex<Sender<ThreadMessage>>>,
    message_reader:Receiver<ThreadMessage>
) -> Result<(),Error>
where
    F:Fn(Request) -> T + Copy + Send,
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
        process_messages(data,func,channel_writer,message_reader).await
    });

    Ok(())

}

async fn process_messages<F,T>(
    _data:Arc<Mutex<SharedData>>,
    func:F,
    channel_writer:Arc<Mutex<Sender<ThreadMessage>>>,
    message_reader:Receiver<ThreadMessage>
)
where
    F:Fn(Request) -> T + Copy,
    T:Future<Output = Response> + Send + 'static
{

    let request_limit:usize = 5000;

    loop {

        let mut collect_requests = Vec::new();
        loop {

            if collect_requests.len() > request_limit{break;}

            match message_reader.try_recv(){
                Ok(thread_request)=>{
                    match thread_request{
                        ThreadMessage::Requests(mut requests)=>{
                            loop {
                                match requests.requests.pop(){
                                    Some(request)=>{
                                        collect_requests.push(
                                            process_request(
                                                requests.id.clone(), 
                                                request, 
                                                func, 
                                                requests.passwords.clone(),
                                                channel_writer.clone()
                                            )
                                        );
                                    },
                                    None=>{break;}
                                }
                            }
                        },
                        _=>{}
                    }
                },
                Err(_)=>{break;}
            }

        }

        let len = collect_requests.len();

        join_all(collect_requests).await;

        if len == 0{
            sleep(Duration::from_millis(1)).await;
        }

    }

}

use std::collections::HashMap;

async fn process_request<F,T>(
    channel_id:String,
    r:gObjectValue,
    func:F,
    passwords:Arc<RwLock<HashMap<String,Vec<u8>>>>,
    channel_writer:Arc<Mutex<Sender<ThreadMessage>>>,
)
where
    F:Fn(Request) -> T + Copy,
    T:Future<Output = Response> + Send + 'static 
{

    let response:Response;
    match parse_request(channel_id.clone(),r, passwords.clone()){
        Ok(parsed)=>{
            match parse_request_to_struct(channel_id.clone(),&parsed){
                Ok(request)=>{
                    response = func(request).await;
                },
                Err(_)=>{return;}
            }
        },
        Err(_)=>{return;}
    }

    let response_parsed:Vec<u8>;
    match passwords.read(){
        Ok(lock)=>{
            match lock.get(&channel_id){
                Some(password)=>{
                    match response.parse(&password){
                        Ok(parsed)=>{
                            response_parsed = parsed.build();
                        },
                        Err(_)=>{return;}
                    }
                },
                None=>{return;}
            }
        },
        Err(_)=>{return;}
    }

    match channel_writer.lock(){
        Ok(lock)=>{
            match lock.send(ThreadMessage::Response(ResponseResult{
                id:channel_id,
                body:response_parsed
            })){
                Ok(_)=>{},
                Err(_)=>{return;}
            }
        },
        Err(_)=>{}
    }
                    
}

fn parse_request_to_struct(channel_id:String,o:&gObjectValue) -> Result<Request,Error>{

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
        channel_id:channel_id,
        request_id:id,
        body:body
    });

}

use crate::crypt::aes::Encrypted;

fn parse_request(id:String,request:gObjectValue,passwords:Arc<RwLock<HashMap<String,Vec<u8>>>>) -> Result<gObjectValue,Error>{

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

    match passwords.read(){
        Ok(lock)=>{
            match lock.get(&id){
                Some(password)=>{
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
                },
                None=>{
                    return Err(Error!("not_found-password"));
                }
            }
        },
        Err(_)=>{return Err(Error!("failed-lock-passwords"));}
    }

}