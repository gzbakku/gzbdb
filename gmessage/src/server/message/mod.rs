use tokio::runtime::{Builder,Runtime};
use std::sync::{Mutex,Arc};
use crate::server::ipc::{SharedData,ChannelMessages};
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
    F:Fn(Request) -> T + Copy,
    T:Future<Output = Response> + Send + 'static
{

    let builder = Builder::new_multi_thread()
    .worker_threads(4)
    .thread_name("connection listener")
    .enable_all()
    // .thread_stack_size(3 * 1024 * 1024)
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

    let mut pending:Vec<ChannelMessages> = Vec::new();
    let request_limit:usize = 100;

    loop {

        {

            let mut collect_async_requests = Vec::new();

            loop{
                if pending.len() == 0{break;}
                if collect_async_requests.len() >= request_limit{break;}
                match pending.pop(){
                    Some(channel_messages)=>{
                        for request in channel_messages.pool.iter(){
                            collect_async_requests.push(process_request(channel_messages.id.clone(),request.clone(),func));
                        }
                    },
                    None=>{}
                }
            }

            let mut process_all = join_all(collect_async_requests).await;
            
            let mut collect_valid_responses = Vec::new();
            loop{
                match process_all.pop(){
                    Some(response_result)=>{
                        match response_result{
                            Ok(response)=>{
                                collect_valid_responses.push(response);
                            },
                            Err(_)=>{}
                        }
                    },
                    None=>{break;}
                }
            }

            if collect_valid_responses.len() > 0{
                match channel_writer.lock(){
                    Ok(writer)=>{
                        match writer.send(ThreadMessage::responses(collect_valid_responses)){
                            Ok(_)=>{},
                            Err(_)=>{}
                        }
                    },
                    Err(_)=>{}
                }
            }

        }

        {

            loop{
                match message_reader.try_recv(){
                    Ok(thread_request)=>{
                        match thread_request{
                            ThreadMessage::Messages(channel_messages)=>{
                                pending.insert(0,channel_messages);
                            },
                            _=>{}
                        }
                    },
                    Err(_)=>{break;}
                }
            }

        }//listen for requests

        sleep(Duration::from_millis(1)).await;

    }

}

async fn process_request<F,T>(
    channel_id:String,
    r:gObjectValue,
    func:F,
) -> Result<Response,Error>
where
    F:Fn(Request) -> T + Copy,
    T:Future<Output = Response> + Send + 'static 
{

    let request:Request;
    match parse_request_to_struct(channel_id,&r){
        Ok(v)=>{request = v;},
        Err(e)=>{
            return Err(Error!("failed-parse_request"=>e));
        }
    }
    let hold = func(request).await;
    return Ok(hold);
                    
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