
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
    .worker_threads(1)
    .thread_name("connection listener")
    .enable_all()
    // .thread_stack_size(3 * 1024 * 1024)
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

use mio::Waker;
use std::sync::mpsc::Sender;

#[allow(dead_code)]
struct RequestWorker{
    request_id:String,
    waker:Waker,
    writer:Sender<ThreadMessage>
}

use gobject::{gObject,gObjectValue,gSchema};
use crate::crypt::aes::{encrypt,Encrypted};

async fn process_channels(connection:&mut MessageBox,reader:Receiver<ThreadMessage>){

    let mut requests:HashMap<String,RequestWorker> = HashMap::new();

    loop {

        {
            match reader.try_recv(){
                Ok(message_type)=>{
                    match message_type{
                        ThreadMessage::Request(request)=>{
                            // println!("request received");
                            match parse_request(&request,&connection.password){
                                Ok(parsed_request)=>{
                                    // println!("request parsed");
                                    match connection.send(parsed_request).await{
                                        Ok(_)=>{
                                            // println!("request sent");
                                            match requests.insert(request.request_id.clone(),RequestWorker{
                                                request_id:request.request_id,
                                                waker:request.waker,
                                                writer:request.writer
                                            }){Some(_)=>{
                                                // println!("request replaced");
                                            },None=>{
                                                // println!("request inserted");
                                            }}
                                        },
                                        Err(e)=>{
                                            match request.writer.send(
                                                ThreadMessage::request_error(Error!("failed-send_request"=>e))
                                            ){Ok(_)=>{},Err(_)=>{}}
                                        }
                                    }//send data on message box
                                },
                                Err(e)=>{
                                    match request.writer.send(
                                        ThreadMessage::request_error(Error!("failed-parse_request"=>e))
                                    ){Ok(_)=>{},Err(_)=>{}}
                                }
                            }//parse request
                        }//request type message
                        _=>{}//all other messages are ignored   
                    }//check mpsc message type
                },
                Err(_)=>{}//error on mpsc channel message read dont do anything
            }//try read message on mpsc channle
        }//mpsc block

        {
            match &connection.try_read(){
                Ok(responses)=>{
                    if responses.len() > 0{
                        // println!("stream read successfull");
                        match parse_responses(&responses,&connection.password){
                            Ok(mut parsed)=>{
                                
                                loop{
                                    match parsed.pop(){
                                        Some(response)=>{
                                            let id = response.request_id.clone();
                                            match requests.get_mut(&id){
                                                Some(worker)=>{
                                                    match worker.writer.send(ThreadMessage::Response(response)){
                                                        Ok(_)=>{},
                                                        Err(_)=>{}
                                                    }
                                                    match worker.waker.wake(){
                                                        Ok(_)=>{},
                                                        Err(_)=>{}
                                                    }
                                                },
                                                None=>{}
                                            }
                                            match requests.remove(&id){
                                                Some(_)=>{},
                                                None=>{}
                                            }
                                        },
                                        None=>{break;}
                                    }//pop parsed response from vector
                                }//loop responses

                            },
                            Err(_)=>{}
                        }//parse responses
                    } else {//response received
                        sleep(Duration::from_millis(1)).await
                    }
                },
                Err(_)=>{}
            }//try read
        }//connection stream read block

    }//master loop

}

use crate::client::ipc::RequestMessage;

fn parse_request(request:&RequestMessage,password:&Vec<u8>) -> Result<Vec<u8>,Error>{

    let body:gObjectValue;
    if request.encrypted{//build encrypted body
        match encrypt(password,&request.body.clone().build()){
            Ok(encrypted)=>{
                body = gObjectValue::object(encrypted.g_object());
            },
            Err(e)=>{
                return Err(Error!("failed-encrypt-request"=>e));
            }
        }
    } else {//build normal body
        body = gObjectValue::object(request.body.clone());
    }
    
    Ok(gObject!{
        "id"=>gObjectValue::string(request.request_id.clone()),
        "body"=>body,
        "encrypted"=>gObjectValue::bool(request.encrypted)
    }.build())

}

pub fn parse_responses(v:&Vec<gObjectValue>,password:&Vec<u8>) -> Result<Vec<Response>,Error>{

    let schema = gSchema!{
        "id"=>gSchemaValue::string,
        "response_type"=>gSchemaValue::i32,
        "body"=>gSchemaValue::object
    };

    let mut collect = Vec::new();

    for val in v.iter(){

        match val{
            gObjectValue::object(obj)=>{
                match schema.validate(&obj){
                    Ok(_)=>{
                        match parse_response(obj,&password){
                            Ok(v)=>{
                                // println!("{:?}",v);
                                collect.push(v);
                            },
                            Err(e)=>{
                                println!("parse response failed : {:?}",e);
                            }
                        }
                    },
                    Err(e)=>{
                        println!("schema validation failed : {:?}",e);
                    }
                }
            },
            _=>{
                println!("response not gobject");
            }
        }

    }

    Ok(collect)

}

use crate::query::{Response,ResponseType};

fn parse_response(obj:&gObject,password:&Vec<u8>) -> Result<Response,Error>{

    let response_type:i32;
    match &obj["response_type"]{
        gObjectValue::i32(v)=>{response_type = v.clone();},
        _=>{return Err(Error!("failed-extract-response_type"));}
    }

    let id:String;
    match &obj["id"]{
        gObjectValue::string(v)=>{id = v.clone();},
        _=>{return Err(Error!("failed-extract-id"));}
    }

    let data:gObject;
    match &obj["body"]{
        gObjectValue::object(v)=>{data = v.clone();},
        _=>{return Err(Error!("failed-extract-data"));}
    }

    let make:Response;
    if response_type == 0{
        make = Response{
            channel_id:String::new(),
            request_id:id,
            data:ResponseType::Ok
        };
    } 
    else if response_type == 1{
        let body:gObject;
        match &data["data"]{
            gObjectValue::object(v)=>{body = v.clone();},
            _=>{return Err(Error!("invalid-body-data"));}
        }
        make = Response{
            channel_id:String::new(),
            request_id:id,
            data:ResponseType::Data(body)
        };
    } 
    else if response_type == 2{
        let error:String;
        match &data["error"]{
            gObjectValue::string(v)=>{error = v.clone();},
            _=>{return Err(Error!("invalid-body-data"));}
        }
        make = Response{
            channel_id:String::new(),
            request_id:id,
            data:ResponseType::Error(error)
        };
    } 
    else if response_type == 3{
        make = Response{
            channel_id:String::new(),
            request_id:id,
            data:ResponseType::Down
        };
    } 
    else if response_type == 4{
        let encrypted:gObject;
        match &data["encrypted"]{
            gObjectValue::object(v)=>{encrypted = v.clone();},
            _=>{return Err(Error!("invalid-body-encrypted-data"));}
        }
        let unlocked:gObject;
        match Encrypted::load_from_g_object(&encrypted){
            Ok(e)=>{
                match e.unlock_first_g_object(&password){
                    Ok(doc)=>{
                        match doc{
                            gObjectValue::object(v)=>{unlocked = v;}
                            _=>{return Err(Error!("invalid-unlocked-g_object"));}
                        }
                    },
                    Err(e)=>{return Err(Error!("failed-unlock_first_g_object"=>e));}
                }
            },
            Err(e)=>{
                return Err(Error!("failed-load_from_g_object"=>e));
            }
        }
        make = Response{
            channel_id:String::new(),
            request_id:id,
            data:ResponseType::Encrypted(unlocked)
        };
    } 
    else {return Err(Error!("failed-invalid-response_type"));}

    Ok(make)

}

