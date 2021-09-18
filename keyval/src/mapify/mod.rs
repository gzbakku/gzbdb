

use std::collections::HashMap;

#[derive(Debug)]
pub enum DataFrameType{
    Corrupt,Empty,Frame,Clear
}

#[derive(Debug)]
pub enum MapStructureStatus{
    Flag,KeySizeInt,KeySize,Key,ValueType,ValueSizeInt,ValueSize,Value,FrameEnd
}

#[derive(Debug)]
pub struct DataFrame{
    pub key:String,
    pub frame_type:DataFrameType,
    pub start:u64,
    pub end:u64,
    pub len:u64
}

#[derive(Debug)]
pub struct Frame{
    pub start:u64,
    pub end:u64,
    pub key:String,
    pub key_size_int:u64,
    pub key_size:u64,
    pub value_size_int:u64,
    pub value_type:u8,
    pub value_size:u64,
    pub value_start:u64,
    pub value_end:u64
}

impl Frame{
    pub fn reset(&mut self){
        self.start = 0;
        self.end = 0;
        self.key = String::new();
        self.key_size_int = 0;
        self.key_size = 0;
        self.value_type = 0;
        self.value_size_int = 0;
        self.value_size = 0;
        self.value_start = 0;
        self.value_end = 0;
    }
}

#[derive(Debug)]
pub struct Pointer{
    pub start:u64,
    pub end:u64,
    pub value_start:u64,
    pub value_end:u64,
    pub value_size:u64,
    pub value_type:u8
}

#[derive(Debug)]
pub struct MapStructure{
    pub status:MapStructureStatus,
    pub file_cursor:u64,
    pub array_cursor:u64,
    pub frame:Frame,
    pub frames:Vec<DataFrame>,
    pub pointers:HashMap<String,Pointer>,
    pub corrupt:bool
}

impl MapStructure{
    pub fn new()->MapStructure{MapStructure{
        corrupt:false,
        status:MapStructureStatus::Flag,
        frames:Vec::new(),
        file_cursor:0,
        array_cursor:0,
        frame:Frame{
            start:0,
            end:0,
            key:String::new(),
            key_size_int:0,
            key_size:0,
            value_type:0,
            value_size_int:0,
            value_size:0,
            value_start:0,
            value_end:0
        },
        pointers:HashMap::new()
    }}
    pub fn add(&mut self,k:String,t:DataFrameType,s:u64,e:u64,l:u64){
        if self.corrupt == false{
            match t{
                DataFrameType::Corrupt=>{
                    self.corrupt = true;
                },
                _=>{}
            }
        }
        self.frames.push(DataFrame{
            key:k,frame_type:t,start:s,end:e,len:l
        });
    }
    pub fn pointer(&mut self){
        if
            self.frame.end == 0 ||
            self.frame.key.len() == 0 ||
            self.frame.value_start == 0 ||
            self.frame.value_end == 0
        {
            return;
        }
        self.pointers.insert(self.frame.key.clone(), Pointer{
            start:self.frame.start,
            end:self.frame.end,
            value_start:self.frame.value_start,
            value_end:self.frame.value_end,
            value_size:self.frame.value_size,
            value_type:self.frame.value_type
        });
        // self.frame.reset();
    }
}

pub mod clean;
pub mod end;
pub mod flag;
pub mod key;
pub mod value;

pub fn read_map_chunk(pool:&mut Vec<u8>,map_book:&mut MapStructure) -> bool{

    let mut index = 0;

    loop{

        let run_process_flag = flag::process_flag(pool,map_book);
        // println!("1 : {:?} {:?}",run_process_flag,map_book.status);
        if run_process_flag == false{
            // println!("failed run_process_flag");
            return false;
        }

        let run_process_key = key::process_key(pool,map_book);
        // println!("2 : {:?} {:?}",run_process_key,map_book.status);
        if run_process_key == false{
            // println!("failed run_process_key");
            return false;
        }

        let run_process_value = value::process_value(pool,map_book);
        // println!("3 : {:?} {:?}",run_process_value,map_book.status);
        if run_process_value == false{
            // println!("failed run_process_value");
            return false;
        }

        let (failed,end) = end::process_end(pool,map_book);
        // println!("4 : failed : {:?} end : {:?}",failed,end);
        if failed {return false;}
        if end {return true;}

        if index == 3{
            break;
        } else {
            index += 1;
        }
        
    }

    return false;

}