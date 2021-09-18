
mod messenger;
mod common;
// mod server;
// mod client;
mod query;
pub mod v1;
mod v2;
mod v3;
mod v4;

use v3::server;
use v3::client;
use v4::client as client_v4;

mod io;
mod crypt;

use tokio::sync::Mutex as TokioMutex;
use std::sync::Arc;

#[tokio::main]
async fn main() {

    if true{
        if true{
            start_client_v4().await
        } else {
            start_client().await
        }
    } else {
        start_server().await
    }

}

// use tokio::time::sleep;
// use std::time::Duration;
use gobject::{gObject};
use gobject;
use futures::future::join_all;
// use std::sync::Arc;
// use std::sync::Mutex;
use std::time::Instant;
// use rand::{distributions::{Alphanumeric,Uniform}, Rng};

async fn start_client_v4() {

    let addr = "127.0.0.1:1200".to_string();
    let keys_dir = "D:/workstation/expo/rust/gzbdb/keys";
    let private_key_path = format!("{}/private.pem",keys_dir);
    let public_key_path = format!("{}/public.pem",keys_dir);

    let auth_token:client_v4::AuthToken;
    match client_v4::AuthToken::key(public_key_path,private_key_path).await{
        Ok(v)=>{auth_token = v;},
        Err(_)=>{return;}
    }

    let tries = 10000;
    let config = client_v4::ClientConfig::new(auth_token,addr,128);
    
    let conn:client_v4::RequestBuilder;
    match client_v4::start(config).await{
        Ok(v)=>{
            conn = v;
        },
        Err(_)=>{return}
    }

    println!("client connected");

    if false{
        return;
    }

    let started_at = Instant::now();

    // let mut collect = Vec::new(); 

    let handle = tokio::runtime::Handle::current();

    

    for id in 0..tries{
        // collect.push(runner(&conn));
        let hold_conn_worker = conn.clone();
        if id == tries-1{
            match handle.spawn(async move {
                let local_request_worker = hold_conn_worker;
                let timeout:u64 = 20000;
                match local_request_worker.send(gObject!{
                    "non"=>gObjectValue::null
                },timeout,false).await{
                    Ok(_r)=>{
                        println!("completed : {:?}",id);
                        // println!("c : {:?}",_r.request_id);
                    },
                    Err(_e)=>{
                        println!("e : {:?}",_e);
                    }
                }
            }).await{
                Ok(_)=>{},
                Err(_)=>{}
            }
        } else {
            handle.spawn(async move {
                let local_request_worker = hold_conn_worker;
                let timeout:u64 = 60000;
                match local_request_worker.send(gObject!{
                    "non"=>gObjectValue::null
                },timeout,false).await{
                    Ok(_r)=>{
                        // println!("completed : {:?}",id);
                        // println!("c : {:?}",_r.request_id);
                    },
                    Err(_e)=>{
                        println!("e : {:?}",_e);
                    }
                }
            });
        }
    }

    /*
    
        //simple
        1       = 115 128
        10      = 431
        100     = 4833
        1000    = 55069

        //async
        1       = 112
        10      = 168
        100     = 859
        1000    = 3419
        10000   = 39874

        //v2 mutex hashmap client request book
        1       = 35
        10      = 37
        100     = 197
        1000    = 1716 1505 1816
        10000   = 16755 17521 15316

        //v3 dashmap client request book
        1       = 41
        10      = 42
        100     = 200
        1000    = 1649
        10000   = 15588 17152 16632 

    */

    async fn runner(b:&client_v4::RequestBuilder){
        // let id: String = rand::thread_rng()
        // .sample_iter(&Alphanumeric)
        // .take(7)
        // .map(char::from)
        // .collect();
        let timeout:u64 = 20000;
        match b.send(gObject!{
            "non"=>gObjectValue::null
        },timeout,true).await{
            Ok(_r)=>{
                // println!("completed : {:?}",id);
                // println!("c : {:?}",_r.request_id);
            },
            Err(_e)=>{
                println!("e : {:?}",_e);
            }
        }
    }

    // println!("{:?}",join_all(collect).await.len());
    println!("time : {:?}",started_at.elapsed().as_millis());

    // sleep(Duration::from_secs(100)).await

}

async fn start_client() {

    let addr = "127.0.0.1:1200".to_string();
    let keys_dir = "D:/workstation/expo/rust/gzbdb/keys";
    let private_key_path = format!("{}/private.pem",keys_dir);
    let public_key_path = format!("{}/public.pem",keys_dir);

    let auth_token:client::AuthToken;
    match client::AuthToken::key(public_key_path,private_key_path).await{
        Ok(v)=>{auth_token = v;},
        Err(_)=>{return;}
    }

    let config = client::ClientConfig::new(auth_token,addr);
    
    let conn:client::RequestBuilder;
    match client::start(config).await{
        Ok(v)=>{
            conn = v;
        },
        Err(_)=>{return}
    }

    println!("client connected");

    let started_at = Instant::now();

    // let mut collect = Vec::new(); 

    let handle = tokio::runtime::Handle::current();

    let tries = 1;

    for id in 0..tries{
        // collect.push(runner(&conn));
        let hold_conn_worker = conn.clone();
        if id == tries-1{
            match handle.spawn(async move {
                let local_request_worker = hold_conn_worker;
                let timeout:u64 = 20000;
                match local_request_worker.send(gObject!{
                    "non"=>gObjectValue::null
                },timeout,false).await{
                    Ok(_r)=>{
                        println!("completed : {:?}",id);
                        // println!("c : {:?}",_r.request_id);
                    },
                    Err(_e)=>{
                        println!("e : {:?}",_e);
                    }
                }
            }).await{
                Ok(_)=>{},
                Err(_)=>{}
            }
        } else {
            handle.spawn(async move {
                let local_request_worker = hold_conn_worker;
                let timeout:u64 = 60000;
                match local_request_worker.send(gObject!{
                    "non"=>gObjectValue::null
                },timeout,false).await{
                    Ok(_r)=>{
                        // println!("completed : {:?}",id);
                        // println!("c : {:?}",_r.request_id);
                    },
                    Err(_e)=>{
                        println!("e : {:?}",_e);
                    }
                }
            });
        }
    }

    /*
    
        //simple
        1       = 115 128
        10      = 431
        100     = 4833
        1000    = 55069

        //async
        1       = 112
        10      = 168
        100     = 859
        1000    = 3419
        10000   = 39874

        //v2 mutex hashmap client request book
        1       = 35
        10      = 37
        100     = 197
        1000    = 1716 1505 1816
        10000   = 16755 17521 15316

        //v3 dashmap client request book
        1       = 41
        10      = 42
        100     = 200
        1000    = 1649
        10000   = 15588 17152 16632 

    */

    async fn runner(b:&client::RequestBuilder){
        // let id: String = rand::thread_rng()
        // .sample_iter(&Alphanumeric)
        // .take(7)
        // .map(char::from)
        // .collect();
        let timeout:u64 = 20000;
        match b.send(gObject!{
            "non"=>gObjectValue::null
        },timeout,true).await{
            Ok(_r)=>{
                // println!("completed : {:?}",id);
                // println!("c : {:?}",_r.request_id);
            },
            Err(_e)=>{
                println!("e : {:?}",_e);
            }
        }
    }

    // println!("{:?}",join_all(collect).await.len());
    println!("time : {:?}",started_at.elapsed().as_millis());

    // sleep(Duration::from_secs(100)).await

}

async fn start_server() {

    println!("starting server !!!");

    let keys_dir = "D:/workstation/expo/rust/gzbdb/keys";
    let private_key_path = format!("{}/private.pem",keys_dir);
    let public_key_path = format!("{}/public.pem",keys_dir);

    // println!("config server started");

    let server_config;
    match server::ServerConfig::new(
        1200, private_key_path, public_key_path
    ).await{
        Ok(v)=>{server_config = v;},
        Err(e)=>{
            println!("{:?}",e);
            return;
        }
    }

    // println!("server configed");

    match server::start(server_config,process_request,Vec::new()){
        Ok(_)=>{},
        Err(_)=>{}
    }

    println!("server failed");

}

async fn process_request(_shared:Arc<TokioMutex<server::SharedData<Vec<u8>>>>,request:query::Request) -> query::Response{

    // println!("request : {:?}",request);

    // query::Response::encrypted(&request,gObject!{
    //     "one"=>gObjectValue::bool(true)
    // })

    query::Response::data(&request,gObject!{
        "one"=>gObjectValue::bool(true)
    })

}
