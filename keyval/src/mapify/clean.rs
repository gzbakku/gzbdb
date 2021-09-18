

use crate::mapify::{MapStructure,DataFrameType};

pub fn handle_unmarked_data(pool:&mut Vec<u8>,start:usize,end:usize,map_book:&mut MapStructure){

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