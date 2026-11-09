#![allow(dead_code)]

mod batch;
mod file;
mod serializer;
mod deserializer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Usage: cargo run --release --bin calculate_metrics <filepath>",
        )
    })?;

    let reader = file::FileReader::new(&path)?;
    let mut batch_number = 0;

    let mut row_count = 0;
    let mut int_sum: i128 = 0;
    let mut true_count: f64 = 0.0;
    let mut ascii_count = 0;

    for batch_result in reader {
        let batch = batch_result?;
        batch_number += 1;
        row_count += batch.row_count;
        
        println!(
            "=== Batch {batch_number} ({} rows, {} columns) ===",
            batch.row_count,
            batch.columns.len()
        );

        for (column_number, column) in batch.columns.iter().enumerate() {
            match column {
                batch::Column::IntColumn(values) => {
                    println!("  [Column {column_number} - INT] Data: {values:?}");
                    for &val in values {
                        int_sum += val as i128;
                    }
                }
                batch::Column::BoolColumn(values) => {
                    println!("  [Column {column_number} - BOOL] Data: {values:?}");
                    for &b in values {
                        if b {
                            true_count += 1.0;
                        }
                    }
                }
                batch::Column::VarcharColumn { offsets, data } => {
                    let mut strings = Vec::new();
                    for i in 0..offsets.len() - 1 {
                        let start = offsets[i] as usize;
                        let end = offsets[i + 1] as usize;
                        let text = std::str::from_utf8(&data[start..end])?;
                        strings.push(text);
                    }
                    println!("  [Kolumna {column_number} - VARCHAR] Wartości: {:?}", strings);
                    
                    for &byte in data {
                        if byte.is_ascii() {
                            ascii_count += 1;
                        }
                    }
                }
            }
        }
    }

    let avg = int_sum as f64 / row_count as f64;
    println!("\nAverage: {:.2}", avg);

    let p = true_count / row_count as f64;
    let entropy = 
        if p == 0.0 || p == 1.0 { 0.0 }
        else { - (p * p.log2() + (1.0 - p) * (1.0 - p).log2()) };
    println!("Binary entropy: {entropy:.4}");

    println!("ASCII signs number: {}", ascii_count);

    Ok(())
}