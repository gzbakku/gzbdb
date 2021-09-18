
use utils::Error;
use std::fs::{create_dir_all,File,remove_dir_all,remove_file};
use std::path::Path;

#[allow(dead_code)]
pub fn cwd()->String{
    match std::env::current_dir(){
        Ok(r)=>{
            match r.as_path().to_str(){
                Some(v)=>{String::from(v)},
                None=>{String::new()}
            }
        },
        Err(_)=>{String::new()}
    }
}

pub fn ensure(dir:&String,file_name:&String) -> Result<(),Error>{

    match create_dir_all(&dir){
        Ok(_)=>{},
        Err(_)=>{
            return Err(Error!("failed-ensure_dir"));
        }
    }

    let file_path = format!("{}/{}",dir,file_name);
     if Path::new(&file_path).exists() == false{
        match File::create(file_path){
            Ok(_)=>{
                return Ok(());
            },
            Err(_)=>{
                return Err(Error!("failed-ensure_file")); 
            }
        }
     } else {
        return Ok(());
     }

}

#[allow(dead_code)]
pub fn delete_dir(path:&String) -> Result<(),Error>{
    match remove_dir_all(&path){
        Ok(_)=>{Ok(())},
        Err(_)=>{
            return Err(Error!("failed-delete_dir"));
        }
    }
}

#[allow(dead_code)]
pub fn delete_file(path:&String) -> Result<(),Error>{
    match remove_file(&path){
        Ok(_)=>{Ok(())},
        Err(_)=>{
            return Err(Error!("failed-delete_file"));
        }
    }
}