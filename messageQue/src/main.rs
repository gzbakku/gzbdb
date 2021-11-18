
use std::fs::File;
use std::io::Read;

mod myio;
mod map;
use utils::Error;
use map::Map;
mod parse;

fn main() {

    let dir = "D://workstation/expo/rust/gzbdb/test/messageQue".to_string();
    let file_name = format!("test1");

    if true{
        match myio::delete_dir(&dir){
            Ok(_)=>{},
            Err(_)=>{
                println!("delete test folder failed");
            }
        }
    }

    let mut que:Map;
    match Map::ensure(&dir,&file_name){
        Ok(q)=>{
            que = q;
        },
        Err(_)=>{
            println!("map ensure failed");
            return;
        }
    }

    match que.add(&"message one"){
        Ok(_)=>{
            println!("message added successfully");
        },
        Err(e)=>{
            println!("message add failed : {:?}",e);
        }
    }

    que.add(&"message two").unwrap();
    que.add(&"message three").unwrap();
    que.add(&"message four").unwrap();

    read_map_for_test();

}


pub fn read_map_for_test(){

    let map_path = "D://workstation/expo/rust/gzbdb/test/messageQue/test1".to_string();

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