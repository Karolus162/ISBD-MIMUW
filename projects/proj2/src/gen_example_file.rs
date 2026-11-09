#![allow(dead_code)]

mod batch;
mod file;
mod serializer;
mod deserializer;

use rand::Rng;
use rand::rngs::StdRng;
use rand::SeedableRng;

fn build_varchar(strings: &[String]) -> batch::Column {
    let mut offsets = vec![0u32];
    let mut data = Vec::new();
    for s in strings {
        data.extend_from_slice(s.as_bytes());
        offsets.push(data.len() as u32);
    }
    batch::Column::VarcharColumn { offsets, data }
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Usage: cargo run --release --bin gen_example <filepath>",
        )
    })?;

    let mut writer = file::FileWriter::new(
        &path,
        vec![
            batch::ColumnType::Int,
            batch::ColumnType::Bool,
            batch::ColumnType::Varchar,
            batch::ColumnType::Varchar,
        ],
    )?;

    const BATCH_SIZE: usize = 4;
    const TOTAL_ROWS: usize = 15;

    let mut rng = StdRng::seed_from_u64(448449);

    let en_words = vec![
        "one", "two", "three", "four", "five", 
        "six", "seven", "eight", "nine", "ten", 
        "eleven", "twelve", "thirteen", "fourteen", "fifteen"
    ];
    let pl_words = vec![
        "jeden", "dwa", "trzy", "cztery", "pięć", 
        "sześć", "siedem", "osiem", "dziewięć", "dziesięć", 
        "jedenastu", "dwanaście", "trzynaście", "czternaście", "piętnaście"
    ];

    let mut remaining_rows = TOTAL_ROWS;
    let mut row_index = 0;

    while remaining_rows > 0 {
        let current_batch_rows = std::cmp::min(remaining_rows, BATCH_SIZE);
        
        let mut int_data = Vec::with_capacity(current_batch_rows);
        let mut bool_data = Vec::with_capacity(current_batch_rows);
        let mut en_batch_strings = Vec::with_capacity(current_batch_rows);
        let mut pl_batch_strings = Vec::with_capacity(current_batch_rows);

        for _ in 0..current_batch_rows {
            let val_int: i64 = rng.gen_range(-500..=500);
            let val_bool: bool = rng.r#gen();
            
            let val_en = en_words[row_index % en_words.len()].to_string();
            let val_pl = pl_words[row_index % pl_words.len()].to_string();
            
            row_index += 1;

            int_data.push(val_int);
            bool_data.push(val_bool);
            en_batch_strings.push(val_en);
            pl_batch_strings.push(val_pl);
        }

        let batch_to_write = batch::Batch::new(
            current_batch_rows as u16,
            vec![
                batch::Column::IntColumn(int_data),
                batch::Column::BoolColumn(bool_data),
                build_varchar(&en_batch_strings),
                build_varchar(&pl_batch_strings),
            ],
        )?;

        writer.write_batch(&batch_to_write)?;
        remaining_rows -= current_batch_rows;
    }

    writer.flush()?;
    println!("Saved {TOTAL_ROWS} rows.\n");
    Ok(())
}