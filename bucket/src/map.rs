use utils::Error;
use crate::myio;
use std::collections::HashMap;
use std::fs::{File,OpenOptions};
use byteorder::{BigEndian, WriteBytesExt,ReadBytesExt};
use std::io::{Seek,SeekFrom,Cursor};
// use std::io::{Read,Write};
use std::io::prelude::*;
use std::io::Read;

#[derive(Debug)]
pub enum Value{
    String(String),
    Binary(Vec<u8>),
    F64(f64),
    U64(u64),
    I64(i64)
}

#[derive(Debug)]
pub struct Map{
    pub corrupt:bool,
    pub pointers:HashMap<String,Pointer>,
    pub frames:Vec<DataFrame>,
    pub file:File,
    pub store:HashMap<String,Value>,
    pub path:String,
    pub size:usize
}

impl Map{
    pub fn add(&mut self,key_b:&str,value:Value,store:bool)->Result<(),Error>{
        
        let key = key_b.to_string();
        let parsed = parse_keyval(&key,&value);

        if self.pointers.contains_key(&key) == false{

            let new_frame_start:u64;
            match self.file.seek(SeekFrom::End(0)){
                Ok(v)=>{
                    new_frame_start = v;
                },
                Err(_e)=>{
                    return Err(Error!("failed-seek-file_end"));
                }
            }

            let new_frame_length:u64;
            match self.file.write(&parsed){
                Ok(v)=>{
                    new_frame_length = v as u64;
                },
                Err(_e)=>{
                    return Err(Error!("failed-write-data"));
                }
            }

            self.size = self.size + parsed.len();
            let new_frame_end = new_frame_start + new_frame_length - 1;

            if true {
                match parse_frame_to_pointer(&parsed,new_frame_start,new_frame_end){
                    Ok(pointer)=>{
                        match self.pointers.insert(key.to_string(),pointer){
                            Some(_)=>{},
                            None=>{}
                        }
                    },
                    Err(_)=>{
                        return Err(Error!("failed-make-pointer"));
                    }
                }
            }

            self.frames.push(DataFrame{
                key:key.to_string(),
                frame_type:DataFrameType::Frame,
                start:new_frame_start,
                end:new_frame_end,
                len:new_frame_length
            });
            
        } else {

            // println!("\n{:#?}\n",self.frames);

            //--------------------------------
            //find old frame
            //--------------------------------
            if true{
                let mut current_frame_index = 0;
                let mut current_frame_found = false;
                for item in &self.frames{
                    if item.key == key{
                        current_frame_found = true;
                        break;
                    } else {
                        current_frame_index += 1;
                    }
                }
                if current_frame_found == false{
                    return Err(Error!("failed-fetch-pointer-frame"));
                }
                self.frames[current_frame_index].frame_type = DataFrameType::Clear;
            }

            //--------------------------------
            //write data to a empty frame or a new frame
            //--------------------------------

            //find next first frame with enough capacity
            let current_data_len = parsed.len() as u64;
            let mut empty_frame_index = 0;
            let mut empty_frame_found = false;
            for frame in &self.frames{
                if frame.len >= current_data_len{
                    match frame.frame_type{
                        DataFrameType::Empty=>{
                            empty_frame_found = true;
                            break;
                        },
                        _=>{}
                    }
                }
                empty_frame_index += 1;
            }

            //--------------------------------
            //clean current frame if its the first empty frame
            //--------------------------------
            if empty_frame_found == true && true{

                let new_frame_start:u64;
                match self.file.seek(SeekFrom::Start(self.frames[empty_frame_index].start)){
                    Ok(v)=>{
                        new_frame_start = v;
                    },
                    Err(_e)=>{
                        return Err(Error!("failed-seek-file_end-empty_frame"));
                    }
                }

                let new_frame_length:u64;
                match self.file.write(&parsed){
                    Ok(v)=>{
                        new_frame_length = v as u64;
                    },
                    Err(_e)=>{
                        return Err(Error!("failed-write-data-empty_frame"));
                    }
                }

                let new_frame_end = new_frame_start + new_frame_length - 1;
                match parse_frame_to_pointer(&parsed,new_frame_start,new_frame_end){
                    Ok(pointer)=>{
                        match self.pointers.insert(key.to_string(),pointer){
                            Some(_)=>{},
                            None=>{}
                        }
                    },
                    Err(_)=>{
                        return Err(Error!("failed-make-pointer"));
                    }
                }

                //split empty frame
                if self.frames[empty_frame_index].len > new_frame_length{
                    let empty_frame_start = new_frame_end + 1;
                    let empty_frame_end = self.frames[empty_frame_index].end;
                    let empty_frame_length = empty_frame_end - empty_frame_start + 1;
                    self.frames.insert(empty_frame_index+1,DataFrame{
                        key:String::new(),
                        frame_type:DataFrameType::Empty,
                        start:empty_frame_start,
                        end:empty_frame_end,
                        len:empty_frame_length
                    });
                }

                self.frames[empty_frame_index].end = new_frame_end;
                self.frames[empty_frame_index].len = new_frame_length;
                self.frames[empty_frame_index].frame_type = DataFrameType::Frame;
                self.frames[empty_frame_index].key = key.to_string();

            }//clean current frame if its the first empty frame


            //--------------------------------
            //write to file end
            //--------------------------------
            if empty_frame_found == false && true{

                let new_frame_start:u64;
                match self.file.seek(SeekFrom::End(0)){
                    Ok(v)=>{
                        new_frame_start = v;
                    },
                    Err(_e)=>{
                        return Err(Error!("failed-seek-file_end-no_empty_frame"));
                    }
                }

                let new_frame_length:u64;
                match self.file.write(&parsed){
                    Ok(v)=>{
                        new_frame_length = v as u64;
                    },
                    Err(_e)=>{
                        return Err(Error!("failed-write-data-no_empty_frame"));
                    }
                }
    
                self.size += parsed.len();
                let new_frame_end = new_frame_start + new_frame_length - 1;

                match parse_frame_to_pointer(&parsed,new_frame_start,new_frame_end){
                    Ok(pointer)=>{
                        match self.pointers.insert(key.to_string(),pointer){
                            Some(_)=>{},
                            None=>{}
                        }
                    },
                    Err(_)=>{
                        return Err(Error!("failed-make-pointer"));
                    }
                }

                self.frames.push(DataFrame{
                    key:key.to_string(),
                    frame_type:DataFrameType::Frame,
                    start:new_frame_start,
                    end:new_frame_end,
                    len:new_frame_length
                });


            }//write to file end

            //--------------------------------
            //clear old frame
            //--------------------------------

            //--------------------------------
            //get clear frame
            //--------------------------------

            let mut current_frame_index = 0;
            let mut current_frame_found = false;
            for item in &self.frames{
                if empty_frame_found{
                    match item.frame_type{
                        DataFrameType::Clear=>{
                            if item.key == key{
                                current_frame_found = true;
                                break;
                            }
                        },
                        _=>{}
                    }
                } else {
                    if item.key == key{
                        current_frame_found = true;
                        break;
                    }
                }
                current_frame_index += 1;
            }
            if current_frame_found == false{
                return Err(Error!("failed-fetch-clear-frame"));
            }

            //--------------------------------
            //expand old frame and mark as Empty
            //--------------------------------
            if true{
                
                //expand previous frame
                if current_frame_index > 0{
                    match self.frames[current_frame_index-1].frame_type{
                        DataFrameType::Empty=>{
                            self.frames[current_frame_index].start = self.frames[current_frame_index-1].start;
                            self.frames[current_frame_index].len = 
                                self.frames[current_frame_index].end - 
                                self.frames[current_frame_index].start + 
                                1;
                            self.frames.remove(current_frame_index-1);
                            current_frame_index -= 1;
                        },
                        _=>{}
                    }
                }

                //expand next frame
                if self.frames.len() > current_frame_index+1{
                    match self.frames[current_frame_index+1].frame_type{
                        DataFrameType::Empty=>{
                            self.frames[current_frame_index].end = self.frames[current_frame_index+1].end;
                            self.frames[current_frame_index].len = 
                                self.frames[current_frame_index].end - 
                                self.frames[current_frame_index].start + 
                                1;
                            self.frames.remove(current_frame_index+1);
                        },
                        _=>{}
                    }
                }

            }

            //--------------------------------
            //clean old frame data
            //--------------------------------
            if true {

                //flag the frame clear
                self.frames[current_frame_index].frame_type = DataFrameType::Empty;
                self.frames[current_frame_index].key = String::new();

                match self.file.seek(SeekFrom::Start(self.frames[current_frame_index].start)){
                    Ok(_v)=>{
                        
                    },
                    Err(_e)=>{
                        return Err(Error!("failed-seek-current_frame_start"));
                    }
                }
    
                let mut empty_array = vec![];
                let empty_array_size = self.frames[current_frame_index].len as usize;
                loop{
                    if empty_array.len() == empty_array_size{
                        break;
                    }
                    empty_array.push(0);
                }
    
                match self.file.write(&empty_array){
                    Ok(_v)=>{
                        
                    },
                    Err(_)=>{
                        return Err(Error!("failed-write-clean_frame_empty_array"));
                    }
                }

                // println!("\n{:#?}\n",self.frames);

            }

            return Ok(());

        }

        if store{
            match self.store.insert(key,value){
                Some(_)=>{},
                None=>{}
            }
        }

        return Ok(());

    }
    pub fn delete(&mut self,key:&str) -> Result<(),Error>{

        if true{
            match self.store.remove(key){
                Some(_)=>{},
                None=>{}
            }
        }

        if true{
            match self.pointers.remove(key){
                Some(_)=>{},
                None=>{}
            }
        }

        //--------------------------------
        //find old frame
        //--------------------------------
        let mut current_frame_index = 0;
        let mut current_frame_found = false;
        for item in &self.frames{
            if item.key == key{
                current_frame_found = true;
                break;
            } else {
                current_frame_index += 1;
            }
        }
        if current_frame_found == false{
            return Err(Error!("failed-fetch-pointer-frame"));
        }
        self.frames[current_frame_index].frame_type = DataFrameType::Clear;

        //--------------------------------
        //expand old frame and mark as Empty
        //--------------------------------
        if true{
            
            //expand previous frame
            if current_frame_index > 0{
                match self.frames[current_frame_index-1].frame_type{
                    DataFrameType::Empty=>{
                        self.frames[current_frame_index].start = self.frames[current_frame_index-1].start;
                        self.frames[current_frame_index].len = 
                            self.frames[current_frame_index].end - 
                            self.frames[current_frame_index].start + 
                            1;
                        self.frames.remove(current_frame_index-1);
                        current_frame_index -= 1;
                    },
                    _=>{}
                }
            }

            //expand next frame
            if self.frames.len() > current_frame_index+1{
                match self.frames[current_frame_index+1].frame_type{
                    DataFrameType::Empty=>{
                        self.frames[current_frame_index].end = self.frames[current_frame_index+1].end;
                        self.frames[current_frame_index].len = 
                            self.frames[current_frame_index].end - 
                            self.frames[current_frame_index].start + 
                            1;
                        self.frames.remove(current_frame_index+1);
                    },
                    _=>{}
                }
            }

        }

        //--------------------------------
        //clean old frame data
        //--------------------------------
        if true {

            //flag the frame clear
            self.frames[current_frame_index].frame_type = DataFrameType::Empty;
            self.frames[current_frame_index].key = String::new();

            match self.file.seek(SeekFrom::Start(self.frames[current_frame_index].start)){
                Ok(_v)=>{
                    
                },
                Err(_e)=>{
                    return Err(Error!("failed-seek-current_frame_start"));
                }
            }

            let mut empty_array = vec![];
            let empty_array_size = self.frames[current_frame_index].len as usize;
            loop{
                if empty_array.len() == empty_array_size{
                    break;
                }
                empty_array.push(0);
            }

            match self.file.write(&empty_array){
                Ok(_v)=>{
                    
                },
                Err(_)=>{
                    return Err(Error!("failed-write-clean_frame_empty_array"));
                }
            }

        }

        return Ok(());

    }
    pub fn path(&mut self,p:String){
        self.path = p;
    }
    pub fn ensure(dir:String,file_name:String)->Result<Map,Error>{
        match myio::ensure(&dir, &file_name){
            Ok(_)=>{},
            Err(_)=>{
                return Err(Error!("failed-ensure-map"));
            }
        }
        let mut file:File;
        match OpenOptions::new()
        .read(true)
        .write(true)
        // .append(true)
        .open(format!("{}/{}",dir,file_name)){
            Ok(v)=>{file = v;},
            Err(_)=>{
                return Err(Error!("failed-open_file"));
            }
        }
        let len:usize;
        match file.metadata(){
            Ok(v)=>{len = v.len() as usize;},
            Err(_)=>{
                return Err(Error!("failed-fetch-metadata-file"));
            }
        }
        let map:MapStructure;
        match parse_map(&mut file){
            Ok(v)=>{map = v;},
            Err(e)=>{
                return Err(e);
            }
        }
        return Ok(Map{
            corrupt:map.corrupt,
            pointers:map.pointers,
            frames:map.frames,
            file:file,
            store:HashMap::new(),
            path:String::new(),
            size:len
        });
    }
    pub fn read(&mut self,key:&str) -> Result<Value,ReadPointerError>{

        // let key = key_b.to_string();

        if self.pointers.contains_key(key) == false{
            return Err(ReadPointerError::NotFound);
        }

        match self.pointers.get(key){
            Some(pointer)=>{

                match self.file.seek(SeekFrom::Start(pointer.value_start)){
                    Ok(_)=>{},
                    Err(_)=>{
                        return Err(ReadPointerError::FailedSeek);
                    }
                }

                let mut buffer:Vec<u8> = Vec::with_capacity(pointer.value_size as usize);
                match Read::by_ref(&mut self.file).take(pointer.value_size).read_to_end(&mut buffer){
                    Ok(v)=>{
                        if (v as u64) != pointer.value_size{
                            return Err(ReadPointerError::InvalidRead);
                        }
                    },
                    Err(_)=>{
                        return Err(ReadPointerError::ReadFailed);
                    }
                }

                let value:Value;
                if pointer.value_type == 3{//string
                    match String::from_utf8(buffer){
                        Ok(v)=>{
                            value = Value::String(v);
                        },
                        Err(_)=>{
                            return Err(ReadPointerError::InvalidValue);
                        }
                    }
                } else if pointer.value_type == 4{//string
                    value = Value::Binary(buffer);
                } else if pointer.value_type == 5{//i64
                    let mut rdr = Cursor::new(buffer);
                    match rdr.read_i64::<BigEndian>(){
                        Ok(v)=>{
                            return Ok(Value::I64(v));
                        },
                        Err(_)=>{
                            return Err(ReadPointerError::InvalidValue);
                        }
                    }
                } else if pointer.value_type == 6{//u64
                    let mut rdr = Cursor::new(buffer);
                    match rdr.read_u64::<BigEndian>(){
                        Ok(v)=>{
                            return Ok(Value::U64(v));
                        },
                        Err(_)=>{
                            return Err(ReadPointerError::InvalidValue);
                        }
                    }
                } else if pointer.value_type == 7{//f64
                    let mut rdr = Cursor::new(buffer);
                    match rdr.read_f64::<BigEndian>(){
                        Ok(v)=>{
                            return Ok(Value::F64(v));
                        },
                        Err(_)=>{
                            return Err(ReadPointerError::InvalidValue);
                        }
                    }
                } else {
                    return Err(ReadPointerError::InvalidValue);
                }

                return Ok(value);

            },
            None=>{
                return Err(ReadPointerError::FailedGetPointer);
            }
        }//get pointer 

    }//read func Map struct
}

#[derive(Debug)]
pub enum ReadPointerError{
    NotFound,FailedRead,FailedGetPointer,FailedSeek,ReadFailed,InvalidRead,InvalidValue
}

fn parse_frame_to_pointer(pool:&Vec<u8>,start:u64,end:u64)->Result<Pointer,()>{

    let len = pool.len();

    //----------------------------------
    //check starting and ending flags
    //----------------------------------
    if 
        pool[0] != 0 ||
        pool[1] != 1 ||
        pool[2] != 0 ||
        pool[3] != 0 ||
        pool[len-1] != 0 ||
        pool[len-2] != 2 ||
        pool[len-3] != 0
    {
        println!("invalid pool parse_frame_to_pointer");
        return Err(());
    }

    //----------------------------------
    //get key cursor
    //----------------------------------
    let key_size:u64;
    let key_cursor:u64;
    match read_size_int(&pool,4){
        Ok((n,c))=>{
            key_size = n;
            key_cursor = c;
        },
        Err(_)=>{
            println!("failed read_size_int key parse_frame_to_pointer");
            return Err(());
        }
    }

    //----------------------------------
    //get value type
    //----------------------------------
    let value_type:u8 = pool[(key_cursor+key_size+3) as usize] as u8;
    let value_type_cursor = key_cursor+key_size+3;

    //----------------------------------
    //get value cursor
    //----------------------------------
    let value_size:u64;
    let value_cursor:u64;
    let value_size_int_cursor = value_type_cursor+2;
    match read_size_int(&pool,value_size_int_cursor as usize){
        Ok((n,c))=>{
            value_size = n;
            value_cursor = c;
        },
        Err(_)=>{
            println!("!!! failed read_size_int value parse_frame_to_pointer");
            return Err(());
        }
    }

    let value_start = value_cursor+2;
    let value_end = value_cursor+value_size+1;
    let value_len = value_end - value_start + 1;

    //----------------------------------
    //make pointer
    //----------------------------------

    return Ok(Pointer{
        start:start,
        end:end,
        value_start:start+value_start,
        value_end:start+value_end,
        value_size:value_len,
        value_type:value_type
    });

}

fn read_size_int(pool:&Vec<u8>,key_val_size_int_pos:usize) -> Result<(u64,u64),()>{

    let key_val_size_int = pool[key_val_size_int_pos] as usize;

    let mut collect:Vec<u8> = Vec::new();

    for i in key_val_size_int_pos+2..=key_val_size_int_pos+key_val_size_int+1{
        collect.push(pool[i as usize]);
    }

    match read_u64_num(collect){
        Ok(v)=>{
            return Ok((v,(key_val_size_int_pos+key_val_size_int+1) as u64));
        },
        Err(e)=>{
            println!("!!! failed-read_u64_num-read_size_int => {:?}",e);
            return Err(());
        }
    }

}

fn parse_map(file:&mut File) -> Result<MapStructure,Error>{

    let mut last_read = 0;
    let mut map_book = MapStructure::new();
    const READ_LEN:usize = 10;
    let mut collect:Vec<u8> = vec![];
    let mut last_read_map_chunk;

    loop {
        match file.seek(SeekFrom::Start(last_read)){
            Ok(_)=>{},Err(_)=>{
                return Err(Error!("failed-seek-file"));
            }
        }
        let mut buffer = [0;READ_LEN];
        match file.read(&mut buffer){
            Ok(v)=>{
                if v < READ_LEN{
                    let mut hold = buffer.to_vec();
                    hold.truncate(v);
                    &collect.append(&mut hold);
                } else {
                    &collect.append(&mut buffer.to_vec());
                }
                last_read_map_chunk = read_map_chunk(&mut collect,&mut map_book);
                if v < READ_LEN{
                    break;
                } else {
                    last_read = last_read + v as u64;
                }
            },
            Err(_)=>{
                return Err(Error!("failed-read-chunk"));
            }
        }
    }

    if collect.len() > 0{
        let end = collect.len() - 1;
        handle_unmarked_data(&mut collect, 0, end, &mut map_book);
    }

    if last_read_map_chunk == false && map_book.corrupt == false{
        map_book.corrupt = true;
    }

    return Ok(map_book);

}

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
struct Frame{
    start:u64,
    end:u64,
    key:String,
    key_size_int:u64,
    key_size:u64,
    value_size_int:u64,
    value_type:u8,
    value_size:u64,
    value_start:u64,
    value_end:u64
}

impl Frame{
    fn reset(&mut self){
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
    status:MapStructureStatus,
    file_cursor:u64,
    array_cursor:u64,
    frame:Frame,
    pub frames:Vec<DataFrame>,
    pub pointers:HashMap<String,Pointer>,
    pub corrupt:bool
}

impl MapStructure{
    fn new()->MapStructure{MapStructure{
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
    fn add(&mut self,k:String,t:DataFrameType,s:u64,e:u64,l:u64){
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
    fn pointer(&mut self){
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

fn read_map_chunk(pool:&mut Vec<u8>,map_book:&mut MapStructure) -> bool{

    let mut index = 0;

    loop{

        let run_process_flag = process_flag(pool,map_book);
        // println!("1 : {:?} {:?}",run_process_flag,map_book.status);
        if run_process_flag == false{
            // println!("failed run_process_flag");
            return false;
        }

        let run_process_key = process_key(pool,map_book);
        // println!("2 : {:?} {:?}",run_process_key,map_book.status);
        if run_process_key == false{
            // println!("failed run_process_key");
            return false;
        }

        let run_process_value = process_value(pool,map_book);
        // println!("3 : {:?} {:?}",run_process_value,map_book.status);
        if run_process_value == false{
            // println!("failed run_process_value");
            return false;
        }

        let (failed,end) = process_end(pool,map_book);
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

fn process_end(pool:&mut Vec<u8>,map_book:&mut MapStructure)->(bool,bool){
    
    match map_book.status{
        MapStructureStatus::FrameEnd=>{
            
            let array_size_required:usize = map_book.array_cursor as usize + 3;
            if pool.len() > array_size_required{

                //check end flag
                if
                    pool[(map_book.array_cursor+1) as usize] != 0 || 
                    pool[(map_book.array_cursor+2) as usize] != 2 ||
                    pool[(map_book.array_cursor+3) as usize] != 0
                {
                    map_book.status = MapStructureStatus::Flag;
                    map_book.array_cursor += 3;
                    process_flag(pool,map_book);
                    return (true,false);
                }

                //mark end in map_book frame
                map_book.frame.end = map_book.file_cursor + map_book.array_cursor + 3;

                //add frame
                let frame_size = map_book.frame.end - map_book.frame.start + 1;
                map_book.add(
                    map_book.frame.key.clone(),
                    DataFrameType::Frame,
                    map_book.frame.start,
                    map_book.frame.end,
                    frame_size
                );

                //remove data from read array
                map_book.array_cursor = 0;
                map_book.file_cursor += frame_size;
                *pool = pool.split_off(frame_size as usize);

                //add pointer
                map_book.pointer();
                map_book.status = MapStructureStatus::Flag;
                if pool.len() == 0{
                    return (false,true);
                } else {
                    return (false,false);
                }

            } else {
                return (true,false);
            }
        },
        _=>{return (false,false);}
    }

}

fn process_value(pool:&mut Vec<u8>,map_book:&mut MapStructure)->bool{

    //----------------------------
    //find value type
    //----------------------------
    match map_book.status{
        MapStructureStatus::ValueType=>{
         let array_size_required:usize = map_book.array_cursor as usize + 2;
         if pool.len() > array_size_required{
            if
                pool[(map_book.array_cursor+1) as usize] != 0 || 
                pool[(map_book.array_cursor+2) as usize] == 0 ||
                pool[(map_book.array_cursor+2) as usize] >= 100
            {
                map_book.status = MapStructureStatus::Flag;
                map_book.array_cursor = map_book.array_cursor+1;
                return process_flag(pool,map_book);
            }
            map_book.frame.value_type = pool[(map_book.array_cursor+2) as usize] as u8;
            map_book.array_cursor += 2;
            map_book.status = MapStructureStatus::ValueSizeInt;
         } else {
             return false;
         }
        },
        _=>{
            // return true;
        }
    }

    //----------------------------
    //find value size int
    //----------------------------
    match map_book.status{
        MapStructureStatus::ValueSizeInt=>{
         let array_size_required:usize = map_book.array_cursor as usize + 2;
         if pool.len() > array_size_required{
            if
                pool[(map_book.array_cursor+1) as usize] != 0 || 
                pool[(map_book.array_cursor+2) as usize] == 0
            {
                map_book.status = MapStructureStatus::Flag;
                map_book.array_cursor = map_book.array_cursor+1;
                return process_flag(pool,map_book);
            }
            map_book.frame.value_size_int = pool[(map_book.array_cursor+2) as usize] as u64;
            map_book.array_cursor += 2;
            map_book.status = MapStructureStatus::ValueSize;
         } else {
             return false;
         }
        },
        _=>{
            // return true;
        }
    }

    //----------------------------
    //find value size
    //----------------------------
    match map_book.status{
        MapStructureStatus::ValueSize=>{
            let array_size_required = map_book.array_cursor + map_book.frame.value_size_int + 1;
            if pool.len() > array_size_required as usize{
                if
                    pool[(map_book.array_cursor+1) as usize] != 0
                {
                    map_book.status = MapStructureStatus::Flag;
                    map_book.array_cursor = map_book.array_cursor+map_book.frame.value_size_int;
                    return process_flag(pool,map_book);
                }
                let mut collect:Vec<u8> = Vec::new();
                for i in map_book.array_cursor+2..=map_book.array_cursor+map_book.frame.value_size_int+1{
                    collect.push(pool[i as usize]);
                }
                map_book.array_cursor += map_book.frame.value_size_int + 1;
                match read_u64_num(collect){
                    Ok(v)=>{
                        map_book.status = MapStructureStatus::Value;
                        map_book.frame.value_size = v;
                    },
                    Err(_)=>{
                        map_book.status = MapStructureStatus::Flag;
                        if process_flag(pool,map_book) == false{return false;}
                        return process_key(pool,map_book);
                    }
                }
             } else {
                 return false;
             }
        },
        _=>{
            // return true;
        }
    }

    //----------------------------
    //find value bounderies
    //----------------------------
    match map_book.status{
        MapStructureStatus::Value=>{
            let array_size_required = map_book.array_cursor + map_book.frame.value_size + 1;
            if pool.len() > array_size_required as usize{
                if
                    pool[(map_book.array_cursor+1) as usize] != 0
                {
                    map_book.status = MapStructureStatus::Flag;
                    map_book.array_cursor = map_book.array_cursor+map_book.frame.value_size;
                    return process_flag(pool,map_book);
                }
                map_book.frame.value_start = map_book.file_cursor + map_book.array_cursor + 2;
                map_book.frame.value_end = map_book.file_cursor + map_book.array_cursor + map_book.frame.value_size + 1;
                map_book.array_cursor += map_book.frame.value_size + 1;
                map_book.status = MapStructureStatus::FrameEnd;
            } else {
                return false;
            }
        },
        _=>{
            // return true;
        }
    }

    return true;

}//process value

fn process_key(pool:&mut Vec<u8>,map_book:&mut MapStructure)->bool{

    //----------------------------
    //find key size int
    //----------------------------
    match map_book.status{
        MapStructureStatus::KeySizeInt=>{
         let array_size_required:usize = map_book.array_cursor as usize + 2;
         if pool.len() > array_size_required{
            if
                pool[(map_book.array_cursor+1) as usize] != 0 || 
                pool[(map_book.array_cursor+2) as usize] == 0
            {
                map_book.status = MapStructureStatus::Flag;
                map_book.array_cursor = map_book.array_cursor+1;
                if process_flag(pool,map_book) == false{return false;}
                return process_key(pool,map_book);
            }
            map_book.frame.key_size_int = pool[(map_book.array_cursor+2) as usize] as u64;
            map_book.array_cursor += 2;
            map_book.status = MapStructureStatus::KeySize;
         } else {
            return false;
         }
        },
        _=>{}
    }

    //----------------------------
    //find key size
    //----------------------------
    match map_book.status{
        MapStructureStatus::KeySize=>{
            let array_size_required = map_book.array_cursor + map_book.frame.key_size_int + 1;
            if pool.len() > array_size_required as usize{
                if
                    pool[(map_book.array_cursor+1) as usize] != 0
                {
                    map_book.status = MapStructureStatus::Flag;
                    map_book.array_cursor = map_book.array_cursor+map_book.frame.key_size_int;
                    if process_flag(pool,map_book) == false{return false;}
                    return process_key(pool,map_book);
                }
                let mut collect:Vec<u8> = Vec::new();
                for i in map_book.array_cursor+2..=map_book.array_cursor+map_book.frame.key_size_int+1{
                    collect.push(pool[i as usize]);
                }
                map_book.array_cursor += map_book.frame.key_size_int + 1;
                match read_u64_num(collect){
                    Ok(v)=>{
                        map_book.status = MapStructureStatus::Key;
                        map_book.frame.key_size = v;
                    },
                    Err(_)=>{
                        map_book.status = MapStructureStatus::Flag;
                        if process_flag(pool,map_book) == false{return false;}
                        return process_key(pool,map_book);
                    }
                }
             } else {
                return false;
             }
        },
        _=>{}
    }

    //----------------------------
    //find key
    //----------------------------
    match map_book.status{
        MapStructureStatus::Key=>{
            let array_size_required = map_book.array_cursor + map_book.frame.key_size + 1;
            if pool.len() > array_size_required as usize{
                if
                    pool[(map_book.array_cursor+1) as usize] != 0
                {
                    map_book.status = MapStructureStatus::Flag;
                    map_book.array_cursor = map_book.array_cursor+map_book.frame.key_size;
                    if process_flag(pool,map_book) == false{return false;}
                    return process_key(pool,map_book);
                }
                let mut collect:Vec<u8> = Vec::new();
                for i in map_book.array_cursor+2..=map_book.array_cursor+map_book.frame.key_size+1{
                    collect.push(pool[i as usize]);
                }
                map_book.array_cursor += map_book.frame.key_size + 1;
                match String::from_utf8(collect){
                    Ok(v)=>{
                        map_book.status = MapStructureStatus::ValueType;
                        map_book.frame.key = v;
                        return true;
                    },
                    Err(_)=>{
                        map_book.status = MapStructureStatus::Flag;
                        if process_flag(pool,map_book) == false{return false;}
                        return process_key(pool,map_book);
                    }
                }
            } else {
                return false;
            }
        },
        _=>{}
    }

    return true;

}//process key

fn process_flag(pool:&mut Vec<u8>,map_book:&mut MapStructure)->bool{
    match map_book.status{
        MapStructureStatus::Flag=>{
            let (flag_found,flag_cursor) = find_flag(pool,map_book.array_cursor);
            if flag_found{
                map_book.frame.reset();
                /*
                    only remove the data if flag is found and flag is at index more then 2 of data pool               
                */
                if flag_cursor >= 3{
                    let data_before:usize = flag_cursor as usize - 3;
                    handle_unmarked_data(pool,0,data_before,map_book);
                    map_book.array_cursor = 2;
                    map_book.frame.start = map_book.file_cursor + 0;
                } else {
                    map_book.array_cursor = flag_cursor;
                    map_book.frame.start = map_book.file_cursor + 0;
                }
                map_book.status = MapStructureStatus::KeySizeInt;
                return true;
            } else {
                return false;
            }
        },
        _=>{
            return true;
        }
    }
}

fn handle_unmarked_data(pool:&mut Vec<u8>,start:usize,end:usize,map_book:&mut MapStructure){

    let mut zero_count:u64 = 0;
    let mut data_count:u64 = 0;
    let mut zero_start:u64 = 0;
    let mut data_start:u64 = 0;

    let mut count = 0;

    for n in start..=end{

        count += 1;

        let i = pool[n];

        if i == 0{
            if data_count > 0{
                map_book.add(
                    String::new(),
                    DataFrameType::Corrupt,
                    map_book.file_cursor + data_start,
                    map_book.file_cursor + n as u64-1,
                    map_book.file_cursor + n as u64-data_start
                );
                data_count = 0;
                data_start = 0;
            }
            if zero_start == 0 && zero_count == 0{
                zero_start = n as u64;
            }
            zero_count += 1;
        } else {
            if zero_count > 0{
                map_book.add(
                    String::new(),
                    DataFrameType::Empty,
                    map_book.file_cursor + zero_start,
                    map_book.file_cursor + n as u64-1,
                    map_book.file_cursor + n as u64-zero_start
                );
                zero_start = 0;
                zero_count = 0;
            }
            if data_start == 0 && data_count == 0{
                data_start = n as u64;
            }
            data_count += 1;
        }

    }

    if zero_count > 0{
        map_book.add(
            String::new(),
            DataFrameType::Empty,
            map_book.file_cursor + zero_start,
            map_book.file_cursor + end as u64,
            zero_count
        );
    }
    if data_count > 0{
        map_book.add(
            String::new(),
            DataFrameType::Corrupt,
            map_book.file_cursor + data_start,
            map_book.file_cursor + end as u64,
            data_count
        );
    }

    for _ in start..=end{
        pool.remove(0);
    }

    map_book.file_cursor += count as u64;
    map_book.array_cursor = 0;

}

fn find_flag(pool:&mut Vec<u8>,cursor:u64) -> (bool,u64){

    let mut zero_count = 0;
    let mut one_found = false;
    let mut index:u64 = 0;

    for i in pool{

        if index >= cursor{

            if i == &mut 0 {
                zero_count = zero_count + 1;
                if one_found{
                    return (true,index);
                }
            } else if i == &mut 1 {
                if one_found {
                    one_found = false;
                } else if zero_count > 0 {
                    one_found = true;
                }
            } else {
                one_found = false;
                zero_count = 0;
            }
            
        }

        index = index + 1;

    }

    return (false,index);

}

fn parse_keyval(key:&String,value:&Value)->Vec<u8>{
    let mut collect:Vec<u8> = vec![0,1,0];
    let mut key_len_as_bytes = Vec::new();
    match key_len_as_bytes.write_u64::<BigEndian>(key.len() as u64){
        Ok(_)=>{},
        Err(_)=>{}
    }
    collect.push(0);
    collect.push(key_len_as_bytes.len() as u8);
    collect.push(0);
    collect.append(&mut key_len_as_bytes);
    collect.push(0);
    collect.append(&mut key.as_bytes().to_vec());
    match value{
        Value::String(v)=>{
            collect.append(&mut vec![0,3]);
            let mut value_len_as_bytes = Vec::new();
            match value_len_as_bytes.write_u64::<BigEndian>(v.len() as u64){
                Ok(_)=>{},
                Err(_)=>{}
            }
            collect.push(0);
            collect.push(value_len_as_bytes.len() as u8);
            collect.push(0);
            collect.append(&mut value_len_as_bytes);
            collect.push(0);
            collect.append(&mut v.as_bytes().to_vec());
        },
        Value::Binary(v)=>{
            collect.append(&mut vec![0,4]);
            let mut value_len_as_bytes = Vec::new();
            match value_len_as_bytes.write_u64::<BigEndian>(v.len() as u64){
                Ok(_)=>{},
                Err(_)=>{}
            }
            collect.push(0);
            collect.push(value_len_as_bytes.len() as u8);
            collect.push(0);
            collect.append(&mut value_len_as_bytes);
            collect.push(0);
            collect.append(&mut v.clone());
        },
        Value::I64(v)=>{
            collect.append(&mut vec![0,5]);
            let mut value_as_bytes = Vec::new();
            match value_as_bytes.write_i64::<BigEndian>(*v){
                Ok(_)=>{},
                Err(_)=>{
                    value_as_bytes = vec![0];
                }
            }
            let mut value_len_as_bytes = Vec::new();
            match value_len_as_bytes.write_u64::<BigEndian>(value_as_bytes.len() as u64){
                Ok(_)=>{},
                Err(_)=>{}
            }
            collect.push(0);
            collect.push(value_len_as_bytes.len() as u8);
            collect.push(0);
            collect.append(&mut value_len_as_bytes);
            collect.push(0);
            collect.append(&mut value_as_bytes);
        },
        Value::U64(v)=>{
            collect.append(&mut vec![0,6]);
            let mut value_as_bytes = Vec::new();
            match value_as_bytes.write_u64::<BigEndian>(*v){
                Ok(_)=>{},
                Err(_)=>{
                    value_as_bytes = vec![0];
                }
            }
            let mut value_len_as_bytes = Vec::new();
            match value_len_as_bytes.write_u64::<BigEndian>(value_as_bytes.len() as u64){
                Ok(_)=>{},
                Err(_)=>{}
            }
            collect.push(0);
            collect.push(value_len_as_bytes.len() as u8);
            collect.push(0);
            collect.append(&mut value_len_as_bytes);
            collect.push(0);
            collect.append(&mut value_as_bytes);
        },
        Value::F64(v)=>{
            collect.append(&mut vec![0,7]);
            let mut value_as_bytes = Vec::new();
            match value_as_bytes.write_f64::<BigEndian>(*v){
                Ok(_)=>{},
                Err(_)=>{
                    // value_as_bytes = vec![0];
                }
            }
            let mut value_len_as_bytes = Vec::new();
            match value_len_as_bytes.write_u64::<BigEndian>(value_as_bytes.len() as u64){
                Ok(_)=>{},
                Err(_)=>{}
            }
            collect.push(0);
            collect.push(value_len_as_bytes.len() as u8);
            collect.push(0);
            collect.append(&mut value_len_as_bytes);
            collect.push(0);
            collect.append(&mut value_as_bytes);
        }
    }

    collect.append(&mut vec![0,2,0]);

    return collect;
}

fn read_u64_num(pool:Vec<u8>)->Result<u64,()>{
    let mut rdr = Cursor::new(pool);
    match rdr.read_u64::<BigEndian>(){
        Ok(v)=>{return Ok(v)},
        Err(_e)=>{
            return Err(());
        }
    }
}