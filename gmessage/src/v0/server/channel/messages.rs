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

pub async fn process_messages<F,T>(
    data:Arc<Mutex<SharedData>>,
    func:F,
    id:&str,
    password:&Vec<u8>,
    requests:&mut Vec<gObjectValue>
) -> Result<Vec<Vec<u8>>,Error>
where
    F:Fn(Request) -> T + Unpin + Send + 'static + Copy + Sync,
    T:Future<Output = Response> + Send + 'static
{

    let mut collect_requests = Vec::new();

    loop {

        match requests.pop(){
            Some(request)=>{
                collect_requests.push(
                    process_request(
                        data.clone(),
                        &id,
                        request, 
                        func, 
                        &password
                    )
                );
            },
            None=>{break;}
        }

    }

    // let len = collect_requests.len();

    let mut collect:Vec<Vec<u8>> = Vec::new();
    for result in join_all(collect_requests).await{
        match result{
            Ok(data)=>{
                collect.push(data);
            },
            Err(_)=>{}
        }
    }
    Ok(collect)

}

use std::collections::HashMap;

async fn process_request<F,T>(
    data:Arc<Mutex<SharedData>>,
    channel_id:&str,
    r:gObjectValue,
    func:F,
    password:&Vec<u8>
) -> Result<Vec<u8>,Error>
where
    F:Fn(Request) -> T + Unpin + Send + 'static + Copy + Sync,
    T:Future<Output = Response> + Send + 'static
{

    match parse_request(&channel_id,r, &password){
        Ok(parsed)=>{
            match parse_request_to_struct(&channel_id,&parsed){
                Ok(request)=>{
                    match func(request).await.parse(&password){
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