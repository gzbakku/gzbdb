

use crate::mapify::{MapStructure,MapStructureStatus};
use crate::mapify::flag::process_flag;
use crate::parse::read_u64_num;

pub fn process_key(pool:&mut Vec<u8>,map_book:&mut MapStructure)->bool{

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