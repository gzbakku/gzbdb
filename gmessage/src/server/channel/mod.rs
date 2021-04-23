use tokio::runtime::{Builder,Runtime};
use crate::messenger::MessageBox;
use crate::common::Error;
use crate::Error;
use std::sync::mpsc::{Receiver,Sender};

pub fn start(channel_read:Receiver<ThreadMessage>,message_write:&Sender<ThreadMessage>) -> Result<(),Error>{

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
        process_channels(channel_read,&message_write).await
    });

    Ok(())

}

use tokio::time::sleep;
use std::time::Duration;
use crate::server::ipc::ThreadMessage;
use std::collections::HashMap;

async fn process_channels(reader:Receiver<ThreadMessage>,message_write:&Sender<ThreadMessage>){

    let mut map:HashMap<String,MessageBox> = HashMap::new();
    let mut keys:Vec<String> = Vec::new();
    let mut len:usize = 0;

    loop{

        let mut no_data = true;

        for i in 0..len+1{

            {//channel loop

                if len > 0 {

                    let mut h = 0;
                    if i > 1{h = i - 1;}

                    match keys.get(h){
                        Some(string_ref)=>{

                            let key = string_ref.clone();

                            let mut do_remove = false;
    
                            match map.get_mut(&key){
                                Some(channel)=>{
                                    match channel.try_read(){
                                        Ok(pool)=>{
                                            if pool.len() > 0{
                                                no_data = false;
                                                match parse_requests(pool,&channel.password){
                                                    Ok(unlocked)=>{
                                                        match message_write.send(
                                                            ThreadMessage::messages(
                                                                channel.id.clone(),
                                                                unlocked
                                                            )
                                                        ){Ok(_)=>{},Err(_)=>{}}
                                                    },
                                                    Err(_)=>{println!("parse_request-failed");}
                                                }
                                            }
                                        },
                                        Err(e)=>{
                                            if e.val == "closed"{do_remove = true;}
                                        }
                                    }//read from channel
                                },
                                None=>{}
                            }//get channel from channel map

                            if do_remove{
                                match map.remove(&key){
                                    Some(_)=>{},
                                    None=>{}
                                }
                                match keys.iter().position(|r| r == &key){
                                    Some(pos)=>{
                                        keys.remove(pos);
                                    },
                                    None=>{}
                                }
                                len -= 1;
                                println!("connection closed");
                            }//remove if connection closed

                        },
                        None=>{}
                    }//get string from vector
    
                }

            }//channel loop

            loop{//message loop

                match reader.try_recv(){
                    Ok(message)=>{
                        match message{
                            ThreadMessage::MessageBox(mb)=>{
                                keys.push(mb.id.clone());
                                match map.insert(mb.id.clone(),mb){
                                    Some(_)=>{},
                                    None=>{}
                                }
                                len += 1;
                            },
                            ThreadMessage::Responses(mut responses)=>{
                                loop {
                                    match responses.pool.pop(){
                                        Some(response)=>{
                                            match map.get_mut(&response.channel_id){
                                                Some(message_box)=>{
                                                    match response.parse(&message_box.password){
                                                        Ok(v)=>{
                                                            match message_box.send(v.build()).await{
                                                                Ok(_)=>{},
                                                                Err(_)=>{}
                                                            }//response set on stream
                                                        },//response parsed to send via stream
                                                        Err(_)=>{}//response paring failed
                                                    }//parse response
                                                },//got channel
                                                None=>{}//chanle not found
                                            }//query channel to which response have to be sent
                                        },//got some response
                                        None=>{break}
                                    }//pop items from response pool
                                }//loop response pool and 
                            },//response type message
                            _=>{}//non operative mpsc messaage type
                        }//check message enum type
                    },//got message
                    Err(_)=>{break;}//no message
                }//read a message
    
            }//message loop

        }//loop through channels

        if no_data{
            sleep(Duration::from_millis(1)).await;
        }

    }//base loop

}

use gobject::{gObjectValue,gSchema,gObject};
use crate::crypt::aes::Encrypted;

fn parse_requests(pool_h:Vec<gObjectValue>,password:&Vec<u8>) -> Result<Vec<gObjectValue>,Error>{

    let mut pool = pool_h;

    let schema = gSchema!{
        "id"=>gSchemaValue::string,
        "encrypted"=>gSchemaValue::bool,
        "body"=>gSchemaValue::object
    };

    let mut collect = Vec::new();

    loop {

        match pool.pop(){
            Some(request_gv)=>{
                match request_gv{
                    gObjectValue::object(request)=>{
                        match schema.validate(&request){
                            Ok(_)=>{
                                match &request["encrypted"]{
                                    gObjectValue::bool(v)=>{
                                        if v == &true{
                                            match &request["body"]{
                                                gObjectValue::object(encrypted)=>{
                                                    match Encrypted::load_from_g_object(encrypted){
                                                        Ok(e_control)=>{
                                                            match e_control.unlock_first_g_object(&password){
                                                                Ok(v)=>{
                                                                    collect.push(gObjectValue::object(gObject!{
                                                                        "id"=>request["id"].clone(),
                                                                        "encrypted"=>request["encrypted"].clone(),
                                                                        "body"=>v
                                                                    }));//collect request
                                                                },//success decrypt to first gObject
                                                                Err(_)=>{}//failed decrypt to first gObject
                                                            }//decrypt to first gObject
                                                        },
                                                        Err(_)=>{}//failed load encrypted to Encrypted
                                                    }//load encrypted body into Encrypted struct
                                                },//object type body
                                                _=>{}//non object body type
                                            }//extract body
                                        } else {
                                            collect.push(gObjectValue::object(request));
                                        }//normal request
                                    }//bool encrypted
                                    _=>{}//non bool encrypted
                                }//get encrypted
                            },//validation successfull
                            Err(_)=>{}//validation error
                        }//validate request gObject
                    },//object gObjectValue type
                    _=>{}//all other gObjectValue types
                }//match request to gObjectValue
            },//val from request vector
            None=>{break;}
        }//pop request from pool

    }//master loop

    return Ok(collect);

}