use tokio::runtime::{Builder,Runtime};
use crate::messenger::MessageBox;
use crate::common::Error;
use crate::Error;
use std::sync::mpsc::{Receiver,Sender};

pub fn start(channel_read:Receiver<ThreadMessage>,message_write:&Sender<ThreadMessage>) -> Result<(),Error>{

    let builder = Builder::new_multi_thread()
    .worker_threads(10)
    .thread_name("channel listener")
    .enable_all()
    .thread_stack_size(32 * 1024 * 1024)
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
use crate::server::ipc::{ThreadMessage,Requests};
use std::collections::HashMap;
use std::sync::{Arc,RwLock};

async fn process_channels(reader:Receiver<ThreadMessage>,message_write:&Sender<ThreadMessage>){

    let mut map:HashMap<String,MessageBox> = HashMap::new();
    let mut keys:Vec<String> = Vec::new();
    let mut passwords:Arc<RwLock<HashMap<String,Vec<u8>>>> = Arc::new(RwLock::new(HashMap::new()));
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
                                                match message_write.send(
                                                    ThreadMessage::Requests(Requests{
                                                        id:channel.id.clone(),
                                                        requests:pool,
                                                        passwords:passwords.clone()
                                                    })
                                                ){Ok(_)=>{},Err(_)=>{}}
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
                        if !no_data{no_data = false;}
                        match message{
                            ThreadMessage::MessageBox(mb)=>{
                                keys.push(mb.id.clone());
                                match passwords.write(){
                                    Ok(mut lock)=>{
                                        match lock.insert(mb.id.clone(),mb.password.clone()){Some(_)=>{},None=>{}}
                                    },Err(_)=>{}
                                }
                                match map.insert(mb.id.clone(),mb){Some(_)=>{},None=>{}}  
                                len += 1; 
                            },
                            ThreadMessage::Response(response)=>{
                                match map.get_mut(&response.id){
                                    Some(channel)=>{
                                        match channel.try_write(response.body){
                                            Ok(_)=>{},
                                            Err(_)=>{}
                                        }
                                    },
                                    None=>{}
                                }
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
