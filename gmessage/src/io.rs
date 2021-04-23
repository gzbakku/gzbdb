
use tokio::io::{AsyncReadExt};
use tokio::fs::File;
use crate::common::Error;
use crate::Error;
// use tokio::io::prelude;

#[allow(dead_code)]
pub async fn read_file(p:&str) -> Result<Vec<u8>,Error>{

    let mut file;
    match File::open(p).await{
        Ok(f)=>{file = f;},
        Err(e)=>{
            return Err(Error!(format!("open_file_failed=>{:?}",e)));
        }
    }

    let mut buffer = Vec::new();
    match file.read_to_end(&mut buffer).await{
        Ok(_)=>{},
        Err(e)=>{
            return Err(Error!(format!("read_file_failed=>{:?}",e)));
        }
    }

    Ok(buffer)

}

#[allow(dead_code)]
pub fn read_g_object(_:&str){



}

#[allow(dead_code)]
pub async fn read_string(p:&str) -> Result<String,Error>{

    match read_file(&p).await{
        Ok(v)=>{
            match String::from_utf8(v){
                Ok(v)=>{return Ok(v);},
                Err(e)=>{
                    return Err(Error!(format!("failed-parse_file_to_string=>{:?}",e)));
                }
            }
        },
        Err(e)=>{return Err(Error!("failed-read_file"=>e));}
    }

}