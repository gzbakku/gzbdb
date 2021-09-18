use std::thread;
use std::sync::{Arc};
use crate::Error;
use crate::common::Error;
use crate::query::{Request,Response};
use std::future::Future;
use tokio::sync::mpsc as TokioMPSC;
use tokio::sync::Mutex as TokioMutex;

mod config;
mod ipc;

mod entry;
mod reader;
mod executer;
mod writer;

pub use config::{ServerConfig,RsaKey};
pub use ipc::SharedData;
use std::sync::mpsc;

#[allow(dead_code)]
pub fn start<F,T,U>(config:ServerConfig,func:F,shared_data:U) -> Result<(),Error>
where
    F:Fn(Arc<TokioMutex<SharedData<U>>>,Request) -> T + Unpin + Send + 'static + Copy + Sync,
    T:Future<Output = Response> + Send + 'static,
    U:Unpin + Send + 'static + Sync
{

    // println!("started threads");

    let shared:Arc<TokioMutex<ipc::SharedData<U>>> = Arc::new(TokioMutex::new(ipc::SharedData{
        shared:shared_data
    }));

    let (reader_write,reader_read) = mpsc::channel::<ipc::ThreadMessage>();
    let (writer_write,writer_read) = mpsc::channel::<ipc::ThreadMessage>();
    let (executer_write,executer_read) = TokioMPSC::channel::<ipc::ThreadMessage>(100);

    let reader_writer_controller = ipc::ReaderWriter::new(reader_write,writer_write);
    let executer_writer_locked = Arc::new(TokioMutex::new(executer_write));

    //connection entry
    // let entry_arc = Arc::clone(&shared);
    let entry_thread = thread::spawn(move || {
        match entry::start(config,reader_writer_controller){
            Ok(_)=>{return Ok(());},
            Err(_)=>{
                return Err(Error!("failed-start-entry_thread"));
            }
        }
    });

    thread::spawn(move || {
        match reader::start(reader_read,executer_writer_locked){
            Ok(_)=>{return Ok(());},
            Err(_)=>{
                return Err(Error!("failed-start-reader_thread"));
            }
        }
    });

    thread::spawn(move || {
        match writer::start(writer_read){
            Ok(_)=>{return Ok(());},
            Err(_)=>{
                return Err(Error!("failed-start-writer_thread"));
            }
        }
    });

    thread::spawn(move || {
        match executer::start(shared,func,executer_read){
            Ok(_)=>{return Ok(());},
            Err(_)=>{
                return Err(Error!("failed-start-writer_thread"));
            }
        }
    });

    match entry_thread.join(){
        Ok(_)=>{},
        Err(_)=>{}
    }

    return Ok(());

}//validate function