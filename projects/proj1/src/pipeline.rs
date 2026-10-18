use std::env;
use std::fs::File;
use std::os::unix::fs::FileExt;

use std::sync::mpsc;
use std::time::Instant;

mod hash;
use hash::sum_block_hash;
use hash::stretch_md5;

fn main() {
    let benchmark_iters = 100000;
    let mut min_duration = std::time::Duration::MAX;

    for _ in 0..benchmark_iters {
        let start = Instant::now();

        let args: Vec<String> = env::args().collect();
        if args.len() != 4 {
            println!("Usage: cargo run --release --bin pipeline filepath M K");
            return;
        }

        let file_res = File::open(args[1].clone());
        if file_res.is_err() {
            eprintln!("File not found: {}", args[1]);
            return;
        }
        let file = file_res.unwrap();

        let metadata = file.metadata();
        if metadata.is_err() {
            eprintln!("Error opening file: {}", args[1]);
            return;
        }

        let block_size : u64 = args[2].parse().expect("Error: block_size must be a number");
        assert!(block_size > 0, "Error: Block size block_size must be greater than 0");
        let iters : u64 = args[3].parse().expect("Error: iters must be a number");
        assert!(iters > 0, "Error: iters must be greater than 0");

        let total_size = metadata.unwrap().len();
        let num_blocks = (total_size + block_size - 1) / block_size;

        let (tx, rx) = mpsc::sync_channel::<Vec<u8>>(num_blocks as usize);

        let handle = std::thread::spawn(move || {
            let mut buffer = vec![0u8; block_size as usize];

            for block_idx in 0..num_blocks
            {
                let offset = block_idx * block_size;
                let bytes_to_read = std::cmp::min(block_size, total_size - offset) as usize;

                let target_buf = &mut buffer[..bytes_to_read];
                let err = file.read_exact_at(target_buf, offset);
                if err.is_err() {
                    eprintln!("Error reading file: {}", err.unwrap_err());
                    return;
                }

                tx.send(target_buf.to_vec()).unwrap();
            }
        });

        std::thread::spawn(move || {
            let mut result = vec![0u8; 16];

            for _ in 0..num_blocks
            {
                let target_buf = rx.recv().unwrap();

                let block_hash = stretch_md5(&target_buf, iters);
                sum_block_hash(&mut result, &block_hash);
            }

            // print!("{} 0x", args[1].clone());
            // for byte in &result {
            //     print!("{:02x}", byte);
            // }
            // println!("");
        }).join().unwrap();

        handle.join().unwrap(); // idk czy zostawić

        let duration = start.elapsed();
        if duration < min_duration {
            min_duration = duration;
        }
    }

    println!("Najkrótszy czas (milisekundy): {} ms", min_duration.as_micros());
}