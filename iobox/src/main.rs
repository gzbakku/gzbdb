
use tokio;
mod builder;
mod controller;
mod ipc;

pub use builder::IoBox;

use tokio::runtime::Handle;
use rand::{distributions::{Alphanumeric}, Rng};
use tokio::sync::mpsc::{channel,Receiver,Sender};
// use tokio::time::sleep;
// use std::time::Duration;
use std::time::Instant;

const no_of_tasks:u32 = 100;
const no_of_items_per_threads:u32 = 100;

#[tokio::main]
async fn main() {

    let mut make:IoBox<u32> = IoBox::new().await;

    let (sen, rec) = channel::<String>(100000);

    let handle = Handle::current();

    let make_clone = make.clone();

    handle.spawn(async move {
        let local_make_clone = make_clone.clone();
        sender(local_make_clone,sen).await;
    });

    receiver(make,rec).await;

}

async fn receiver(iobox:IoBox<u32>,receiver:Receiver<String>){

    let mut local_iobox = iobox;
    let mut local_receiver = receiver;

    let now = Instant::now();

    let mut count = 0;
    let limit = (no_of_tasks * no_of_items_per_threads) - 1;

    // println!("limit : {:?}",limit);

    loop {

        match local_receiver.recv().await{
            Some(msg)=>{
                match local_iobox.pop(msg).await{
                    Ok(d)=>{
                        // println!("d : {:?} c : {:?}",d,count);
                        if d == limit {
                            println!("popped : {:?} {:?}",d,now.elapsed().as_millis());
                        }
                        if count == limit{
                            break;
                        }
                        count += 1;
                    },
                    Err(_)=>{
                        println!("pop failed");
                    }
                }
            },
            None=>{}
        }

    }
    
    println!("finished : {:?}",now.elapsed().as_millis());

}

async fn sender(iobox:IoBox<u32>,sender:Sender<String>){

    // println!("sending");

    // sleep(Duration::from_millis(2000)).await;

    let handle = Handle::current();

    for thread_no in 0..no_of_tasks{

        // println!("thread_no : {:?}",thread_no);

        let hold_iobox = iobox.clone();
        let hold_sender = sender.clone();

        // println!("no_of_items_per_threads : {:?}",no_of_items_per_threads);

        handle.spawn(async move {

            let mut local_io_box = hold_iobox.clone();

            let local_thread_no = thread_no.clone();

            let mut lock_hold_sender = hold_sender.clone();

            for send_no in 0..no_of_items_per_threads{

                // println!("send_no : {:?}",send_no);

                // let id: String = rand::thread_rng()
                // .sample_iter(&Alphanumeric)
                // .take(7)
                // .map(char::from)
                // .collect();

                let send_no = (local_thread_no * no_of_items_per_threads) + send_no;

                // println!("send_no : {:?} {:?} {:?}",send_no,thread_no,send_no);

                if true{
                    match local_io_box.push_unchecked(send_no.clone().to_string(),send_no).await{
                        Ok(_)=>{
                            // println!("pushed");
                        },
                        Err(_)=>{
                            println!("push failed");
                        }
                    }
                } else {
                    match local_io_box.push(send_no.clone().to_string(),send_no).await{
                        Ok(_)=>{
                            // println!("pushed");
                        },
                        Err(_)=>{
                            println!("push failed");
                        }
                    }
                }

                match lock_hold_sender.send(send_no.to_string()).await{
                    Ok(_)=>{},
                    Err(_)=>{}
                }

            }

        });

    }

}
