

use crate::mapify::flag::process_flag;
use crate::mapify::key::process_key;
use crate::mapify::{MapStructureStatus,MapStructure,};
use crate::parse::read_u64_num;

pub fn process_value(pool:&mut Vec<u8>,map_book:&mut MapStructure)->bool{

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