use std::io::{SeekFrom,Write,Seek};
use std::fs::{File,OpenOptions};
use std::collections::HashMap;
use crate::Error;

pub struct Pointer{

}

pub struct Map{
    pub biggest:u64,
    pub file:File,
    pub pointers:HashMap<String,Pointer>
}

impl Map{
    pub fn ensure(dir:&str,file_name:&str)->Result<Map,Error>{

        match crate::myio::ensure(&dir, &file_name){
            Ok(_)=>{},
            Err(_)=>{
                return Err(Error!("failed-ensure_dir"));
            }
        }

        let file:File;
        match OpenOptions::new()
        .read(true)
        .write(true)
        .open(format!("{}/{}",dir,file_name))
        {
            Ok(f)=>{
                file = f;
            },
            Err(_)=>{
                return Err(Error!("failed-ensure_dir"));
            }
        }

        //parse

        let builder = Map{
            biggest:0,
            file:file,
            pointers:HashMap::new()
        };

        return Ok(builder);

    }
    pub fn add(&mut self,message:&str) -> Result<(),Error>{

        let parsed = crate::parse::parse_message(self,message);

        match self.file.seek(SeekFrom::End(0)){
            Ok(_)=>{},
            Err(_)=>{
                return Err(Error!("failed-seek-file_end"));
            }
        }

        match self.file.write(&parsed){
            Ok(_)=>{},
            Err(_)=>{
                return Err(Error!("failed-seek-file_end"));
            }
        }

        return Ok(());

    }
}