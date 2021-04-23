
mod messenger;
mod common;
mod server;
mod client;
mod query;

mod io;
mod crypt;

#[tokio::main]
async fn main() {

    if true{
        start_client().await
    } else {
        start_server().await
    }

}

// use tokio::time::sleep;
// use std::time::Duration;
use gobject::{gObject};
use gobject;
use futures::future::join_all;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

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
    
    let mut conn:client::Client;
    match client::start(config).await{
        Ok(v)=>{conn = v;},
        Err(_)=>{return}
    }

    let started_at = Instant::now();

    let mut collect = Vec::new(); 

    let conn_arc = Arc::new(Mutex::new(conn));

    for _ in 0..100{
        collect.push(runner(conn_arc.clone()));
    }

    async fn runner(a:Arc<Mutex<client::Client>>){
        match a.lock(){
            Ok(mut lock)=>{
                match lock.send(gObject!{
                    "non"=>gObjectValue::null
                },3000,false){
                    Ok(r)=>{
                        // println!("{:?}",r);
                    },
                    Err(e)=>{
                        // println!("e : {:?}",e);
                    }
                }   
            },
            Err(_)=>{}
        }   
    }

    println!("{:?}",join_all(collect).await);
    println!("time : {:?}",started_at.elapsed().as_millis());

    // sleep(Duration::from_secs(100)).await

}

async fn start_server() {

    println!("starting server");

    let keys_dir = "D:/workstation/expo/rust/gzbdb/keys";
    let private_key_path = format!("{}/private.pem",keys_dir);
    let public_key_path = format!("{}/public.pem",keys_dir);

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

    match server::start(server_config,process_request){
        Ok(_)=>{},
        Err(_)=>{}
    }

}

async fn process_request(request:query::Request) -> query::Response{

    // println!("request : {:?}",request);

    query::Response::encrypted(&request,gObject!{
        "one"=>gObjectValue::bool(true)
    })

}
