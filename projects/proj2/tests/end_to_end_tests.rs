use proj2::{batch, file};
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

fn run_test_case(path: &str, total_rows: usize, batch_size: usize, seed: u64) -> Result<(), Box<dyn std::error::Error>> {
    let mut writer = file::FileWriter::new(
        path,
        vec![
            batch::ColumnType::Int,
            batch::ColumnType::Bool,
            batch::ColumnType::Varchar,
            batch::ColumnType::Varchar,
        ],
    )?;

    let mut rng = StdRng::seed_from_u64(seed);
    let charset: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 _-!@#$%^&*()";

    let mut remaining_rows = total_rows;
    let mut expected_batches = Vec::new();

    while remaining_rows > 0 {
        let current_batch_rows = std::cmp::min(remaining_rows, batch_size);
        
        let mut int_data = Vec::with_capacity(current_batch_rows);
        let mut bool_data = Vec::with_capacity(current_batch_rows);
        let mut str1_data = Vec::with_capacity(current_batch_rows);
        let mut str2_data = Vec::with_capacity(current_batch_rows);

        for _ in 0..current_batch_rows {
            let val_int: i64 = rng.gen_range(-100_000..=100_000);
            let val_bool: bool = rng.r#gen();
            
            let len1 = rng.gen_range(0..100);
            let val_str1: String = (0..len1)
                .map(|_| charset[rng.gen_range(0..charset.len())] as char)
                .collect();

            let len2 = rng.gen_range(0..100);
            let val_str2: String = (0..len2)
                .map(|_| charset[rng.gen_range(0..charset.len())] as char)
                .collect();

            int_data.push(val_int);
            bool_data.push(val_bool);
            str1_data.push(val_str1);
            str2_data.push(val_str2);
        }

        let batch_to_write = batch::Batch::new(
            current_batch_rows as u16,
            vec![
                batch::Column::IntColumn(int_data),
                batch::Column::BoolColumn(bool_data),
                build_varchar(&str1_data),
                build_varchar(&str2_data),
            ],
        )?;

        writer.write_batch(&batch_to_write)?;
        expected_batches.push(batch_to_write);
        
        remaining_rows -= current_batch_rows;
    }

    writer.flush()?;

    let mut reader = file::FileReader::new(path)?;
    let mut batch_index = 0;

    while let Some(actual_batch) = reader.read_next_batch()? {
        assert_eq!(actual_batch, expected_batches[batch_index]);
        batch_index += 1;
    }

    assert_eq!(batch_index, expected_batches.len());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let test_cases = vec![
            ("test_1.bin", 10_000, 256, 448449),
            ("test_2.bin", 15_000, 512, 448450),
            ("test_3.bin", 20_000, 1024, 448451),
            ("test_4.bin", 5_000, 32, 448452),
        ];

        for (path, total_rows, batch_size, seed) in test_cases {
            let _ = std::fs::remove_file(path);
            
            run_test_case(path, total_rows, batch_size, seed)
                .unwrap_or_else(|e| panic!("Test failed for path {}: {}", path, e));
            
            let _ = std::fs::remove_file(path);
        }
    }
}