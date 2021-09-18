


mod myio;
mod map;
mod parse;
mod mapify;

pub use map::{Map,Value};
pub use mapify::{
    DataFrame,
    DataFrameType,
    MapStructure,
    MapStructureStatus,
    Frame,
    Pointer
};

fn main(){

    // map::seek_test("D://workstation/expo/rust/gzbdb/test/keyVal/testSeek".to_string());

    remake();

    rewrite();

    delete();

    rewrite();

    // read_map_for_test();

}

fn remake() {

    let dir = "D://workstation/expo/rust/gzbdb/test/keyVal".to_string();
    let file_name = format!("test1");

    if true{
        match myio::delete_dir(&dir){
            Ok(_)=>{},
            Err(_)=>{
                println!("delete test folder failed");
            }
        }
    }

    let mut map1:Map;
    match Map::ensure(dir,file_name){
        Ok(v)=>{map1 = v;},
        Err(e)=>{
            println!("map ensure failed => {:?}",e);
            return;
        }
    }

    if true{
        match map1.add("name",Value::String("Akku".to_string()),false){
            Ok(_)=>{
                // println!("name write complete");
            },
            Err(e)=>{
                println!("name write failed :{:?}",e);
            }
        }
    }

}

use std::fs::File;
use std::io::Read;

pub fn read_map_for_test(){

    let map_path = "D://workstation/expo/rust/gzbdb/test/keyVal/test1".to_string();

    match File::open(map_path){
        Ok(mut file)=>{

            let mut collect = Vec::new();

            match file.read_to_end(&mut collect){
                Ok(_)=>{},
                Err(_)=>{}
            }

            let another = collect.clone();

            println!("\n--------------------------------------\n");

            if true {
                println!("{:?}",collect);
            }

            if false{
                println!("\n++++++++++++++++++++\n");
                let mut index = 0; 
                loop{
                    if index == 100{break;}
                    if collect.len() == 0{
                        break;
                    }
                    if collect.len() > 10{
                        let hold = collect.split_off(10);
                        println!("{:?} : {:?} : {:?}",index,collect,collect.len());
                        collect = hold;
                    } else {
                        println!("{:?} : {:?} : {:?}",index,collect,collect.len());
                        break;
                    }
                    index += 1;
                }
                println!("\n++++++++++++++++++++\n");
            }

            if false{
                println!("\n++++++++++++++++++++\n");
                let mut index = 0; 
                for i in another {
                    print!("{:?}-{:?} \t",index,i);
                    index += 1;
                }
                println!("\n\n++++++++++++++++++++\n");
            }
            

            println!("\n--------------------------------------\n");

        },
        Err(_)=>{
            println!("failed-open_file-read_map_for_test");
        }
    }

}

fn delete(){

    let dir = "D://workstation/expo/rust/gzbdb/test/keyVal".to_string();
    let file_name = format!("test1");

    let mut map1:Map;
    match Map::ensure(dir,file_name){
        Ok(v)=>{map1 = v;},
        Err(e)=>{
            println!("map ensure failed => {:?}",e);
            return;
        }
    }

    if true{
        match map1.delete("name"){
            Ok(_)=>{
                println!("success delete");
            },
            Err(e)=>{
                println!("failed-delete => {:?}",e);
                return;
            }
        }
    }
    if false {
        read(&mut map1);
    }

    fn read(_map1:&mut Map){
        if false{
            println!("\nname : {:?}",_map1.read("name"));
        }
    }

    println!("read after : {:?}",map1.read("name"));

}

#[allow(dead_code)]
fn rewrite(){

    let dir = "D://workstation/expo/rust/gzbdb/test/keyVal".to_string();
    let file_name = format!("test1");

    let mut map1:Map;
    match Map::ensure(dir,file_name){
        Ok(v)=>{map1 = v;},
        Err(e)=>{
            println!("map ensure failed => {:?}",e);
            return;
        }
    }

    // read(&mut map1);
    if true{
        map1.add("name",Value::String("KING Akku".to_string()),false).unwrap();
        map1.add("binary",Value::Binary(vec![0,1,2,3]),false).unwrap();
        map1.add("i64",Value::I64(18),false).unwrap();
        map1.add("u64",Value::U64(20),false).unwrap();
        map1.add("f64",Value::F64(21.1),false).unwrap();
        read(&mut map1);
    }
    if false{
        map1.add("name",Value::String("Emperor Akku".to_string()),false).unwrap();
        map1.add("binary",Value::Binary(vec![77,11,22,33]),false).unwrap();
        map1.add("i64",Value::I64(180),false).unwrap();
        map1.add("u64",Value::U64(200),false).unwrap();
        map1.add("f64",Value::F64(210.10),false).unwrap();
        read(&mut map1);
    }
    

    fn read(map1:&mut Map){
        if true{
            println!("\nname : {:?}",map1.read("name"));
            println!("binary : {:?}",map1.read("binary"));
            println!("i64 : {:?}",map1.read("i64"));
            println!("u64 : {:?}",map1.read("u64"));
            println!("f64 : {:?}\n",map1.read("f64"));
        }
    }

}
