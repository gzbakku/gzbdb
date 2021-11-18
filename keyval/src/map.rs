use crate::Error;
use crate::myio;
use std::collections::HashMap;
use std::fs::{File,OpenOptions};
use byteorder::{BigEndian,ReadBytesExt};
use std::io::{Seek,SeekFrom,Cursor};
// use std::io::{Read,Write};
use std::io::prelude::*;
use std::io::Read;

use crate::parse::{parse_frame_to_pointer,parse_keyval};
use crate::mapify::{Pointer,read_map_chunk,MapStructure,DataFrame,DataFrameType};
use crate::mapify::clean::handle_unmarked_data;

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

///Open a Map File
/// map file will be created if it doesnt exists or else it will open the map file
/// ```rust
/// let dir = "c://test_keyval/".to_string();
/// let file_name = format!("test1");
/// let mut map1:Map;
/// match Map::ensure(dir,file_name){
///     Ok(v)=>{map1 = v;},
///     Err(e)=>{
///         println!("map ensure failed => {:?}",e);
///         return;
///     }
/// }
/// 
/// //add keyval
/// map1.add("name",Value::String("KING Akku".to_string()),false).unwrap();
/// //update keyval
/// map1.add("name",Value::String("Emperor Akku".to_string()),false).unwrap();
/// //read keyval
/// println!("\nname : {:?}",map1.read("name"));
/// //delete keyval
/// map1.delete("name").unwrap();
/// ```
impl Map{
    /// Add Keyval
    /// ```rust
    /// let dir = "c://test_keyval/".to_string();
    /// let file_name = format!("test1");
    /// let mut map1:Map;
    /// match Map::ensure(dir,file_name){
    ///     Ok(v)=>{map1 = v;},
    ///     Err(e)=>{
    ///         println!("map ensure failed => {:?}",e);
    ///         return;
    ///     }
    /// }
    /// 
    /// //add keyval
    /// map1.add("name",Value::String("KING Akku".to_string()),false).unwrap();
    /// map1.add("binary",Value::Binary(vec![0,1,2,3]),false).unwrap();
    /// map1.add("i64",Value::I64(18),false).unwrap();
    /// map1.add("u64",Value::U64(20),false).unwrap();
    /// map1.add("f64",Value::F64(21.1),false).unwrap();
    /// ```
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
    /// Delete Keyval
    /// ```rust
    /// let dir = "c://test_keyval/".to_string();
    /// let file_name = format!("test1");
    /// let mut map1:Map;
    /// match Map::ensure(dir,file_name){
    ///     Ok(v)=>{map1 = v;},
    ///     Err(e)=>{
    ///         println!("map ensure failed => {:?}",e);
    ///         return;
    ///     }
    /// }
    /// 
    /// //delete keyval
    /// map1.delete("name").unwrap();
    /// ```
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
    /// Open Map
    /// ```rust
    /// let dir = "c://test_keyval/".to_string();
    /// let file_name = format!("test1");
    /// let mut map1:Map;
    /// match Map::ensure(dir,file_name){
    ///     Ok(v)=>{map1 = v;},
    ///     Err(e)=>{
    ///         println!("map ensure failed => {:?}",e);
    ///         return;
    ///     }
    /// }
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
    /// Read Keyval
    /// ```rust
    /// let dir = "c://test_keyval/".to_string();
    /// let file_name = format!("test1");
    /// let mut map1:Map;
    /// match Map::ensure(dir,file_name){
    ///     Ok(v)=>{map1 = v;},
    ///     Err(e)=>{
    ///         println!("map ensure failed => {:?}",e);
    ///         return;
    ///     }
    /// }
    /// 
    /// //add keyval
    /// map1.add("name",Value::String("Emperor Akku".to_string()),false).unwrap();
    /// //read keyval
    /// println!("\nname : {:?}",map1.read("name"));
    /// ```
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