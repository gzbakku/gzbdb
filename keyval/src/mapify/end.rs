

use crate::mapify::{MapStructure,MapStructureStatus,DataFrameType};
use crate::mapify::flag::process_flag;

pub fn process_end(pool:&mut Vec<u8>,map_book:&mut MapStructure)->(bool,bool){
    
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