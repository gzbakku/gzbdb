
use std::sync::{Arc,Mutex};
use std::sync::mpsc::Sender;
use tokio::net::TcpStream;
use crate::common::Error;
use crate::Error;
use tokio::sync::RwLock;
use tokio::sync::mpsc as TokioMPSC;
// use tokio::sync::Mutex as TokioMutex;

#[allow(dead_code)]
#[derive(Debug,Clone)]
pub struct SharedData<T>{
    pub shared:T
}

pub enum ThreadMessage{
    ReaderMessage(ReaderMessage),
    WriterMessage(WriterMessage),
    ExecuterMessage(ExecuterMessage),
    WriterResponseMessage(WriterResponseMessage)
}

//------------------------------------------------
//executer

use gobject::gObjectValue;

pub struct ExecuterMessage{
    // pub sender:Arc<TokioMutex<TokioMPSC::Sender<ThreadMessage>>>,
    pub sender:TokioMPSC::Sender<ThreadMessage>,
    pub password:Arc<RwLock<Vec<u8>>>,
    pub connection_id:String,
    pub request:gObjectValue
}

pub struct WriterResponseMessage{
    pub data:Vec<u8>
}

//------------------------------------------------
//reader writer

use tokio::net::tcp::{OwnedReadHalf,OwnedWriteHalf};
// use std::sync::mpsc;

pub struct ReaderMessage{
    pub password:Vec<u8>,
    pub connection_id:String,
    pub read_half:OwnedReadHalf,
    // pub sender:Arc<TokioMutex<TokioMPSC::Sender<ThreadMessage>>>,
    pub sender:TokioMPSC::Sender<ThreadMessage>
}

pub struct WriterMessage{
    pub write_half:OwnedWriteHalf,
    pub receiver:TokioMPSC::Receiver<ThreadMessage>
}

pub struct ReaderWriter{
    reader:Sender<ThreadMessage>,
    writer:Sender<ThreadMessage>
}
 impl ReaderWriter{
     pub fn new(reader:Sender<ThreadMessage>,writer:Sender<ThreadMessage>)->Arc<Mutex<ReaderWriter>>{
        Arc::new(Mutex::new(ReaderWriter{
            reader:reader,
            writer:writer
        }))
     }
     pub fn send(&mut self,stream:TcpStream,id:String,password:Vec<u8>) -> Result<(),Error>{
        let (read_half, write_half) = stream.into_split();
        let (sender,receiver) = TokioMPSC::channel::<ThreadMessage>(500);
        match self.reader.send(ThreadMessage::ReaderMessage(ReaderMessage{
            password:password,
            connection_id:id,
            read_half:read_half,
            sender:sender
        })){
            Ok(_)=>{},
            Err(_)=>{
                return Err(Error!("failed-send-ReaderMessage"));
            }
        }
        match self.writer.send(ThreadMessage::WriterMessage(WriterMessage{
            write_half:write_half,
            receiver:receiver
        })){
            Ok(_)=>{},
            Err(_)=>{
                return Err(Error!("failed-send-WriterMessage"));
            }
        }
        return Ok(());
     }
 }



