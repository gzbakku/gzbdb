
use crate::mapify::{MapStructure,MapStructureStatus};
use crate::mapify::clean::handle_unmarked_data;

pub fn process_flag(pool:&mut Vec<u8>,map_book:&mut MapStructure)->bool{
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