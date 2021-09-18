use tokio;
use gmessage::v1;
use gobject::{gObject};

use std::time::Instant;
// use tokio::time::sleep;
use futures::future::join_all;
// use std::time::Duration;

// #[tokio::main]

use tokio::runtime::{Builder,Runtime};

pub fn main(){

    let builder = Builder::new_multi_thread()
    .worker_threads(256)
    .thread_name("client listener")
    // .enable_all()
    .enable_io()
    .enable_time()
    .thread_stack_size(32 * 1024 * 1024)
    .build();

    let runtime:Runtime;
    match builder{
        Ok(v)=>{runtime = v;},
        Err(_)=>{return}
    }

    runtime.block_on(async {
        start().await
    });

}


async fn start() {

    let addr = "127.0.0.1:1200".to_string();
    let keys_dir = "D:/workstation/expo/rust/gzbdb/keys";
    let private_key_path = format!("{}/private.pem",keys_dir);
    let public_key_path = format!("{}/public.pem",keys_dir);

    let auth_token:v1::client::AuthToken;
    match v1::client::AuthToken::key(public_key_path,private_key_path).await{
        Ok(v)=>{auth_token = v;},
        Err(_)=>{return;}
    }

    let config = v1::client::ClientConfig::new(auth_token,addr);
    
    let conn:v1::client::RequestBuilder;
    match v1::client::start(config).await{
        Ok(v)=>{
            conn = v;
        },
        Err(_)=>{return}
    }

    println!("client connected");

    let started_at = Instant::now();

    let mut collect = Vec::new(); 

    for _ in 0..2000{
        collect.push(runner(&conn));
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

    */

    async fn runner(b:&v1::client::RequestBuilder){
        // let id: String = rand::thread_rng()
        // .sample_iter(&Alphanumeric)
        // .take(7)
        // .map(char::from)
        // .collect();
        let timeout:u64 = 10000;
        match b.send(gObject!{
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
    }

    println!("{:?}",join_all(collect).await.len());
    println!("time : {:?}",started_at.elapsed().as_millis());

    // sleep(Duration::from_secs(100)).await

}

#[allow(dead_code)]
async fn start_old(){

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
    
    let conn:client::Client;
    match client::start(config).await{
        Ok(v)=>{conn = v;},
        Err(_)=>{return}
    }

    // sleep(Duration::from_millis(1000)).await;

    // println!("sleep complete");

    let mut collect = Vec::new(); 

    for _ in 0..1{
        collect.push(runner(&conn));
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

        //async unchain
        1       = 101
        10      = 137
        100     = 377
        1000    = 3334
        10000   = 38812

        //async optimized
        1       = 111
        10      = 129
        100     = 380
        1000    = 2940
        10000   = 34355

        //async client io timeout 

    */

    async fn runner(b:&client::Client) -> Result<(),String>{
        // let id: String = rand::thread_rng()
        // .sample_iter(&Alphanumeric)
        // .take(7)
        // .map(char::from)
        // .collect();
        match b.send(gObject!{
            "non"=>gObjectValue::null
        },30000,false).await{
            Ok(_r)=>{
                // println!("completed : {:?}",id);
                // println!("c : {:?}",_r);
                Ok(())
            },
            Err(_e)=>{
                // println!("e : {:?}",_e.val);
                Err(_e.val)
                // Ok(())
            }
        }
    }

    let started_at = Instant::now();

    println!("{:?}",join_all(collect).await.len());

    // let collect 

    println!("time : {:?}",started_at.elapsed().as_millis());
    
}
