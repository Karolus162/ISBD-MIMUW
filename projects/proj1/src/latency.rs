use std::env;
use std::fs::File;
use std::os::unix::fs::FileExt;
use rand::{rng, Rng, RngExt, SeedableRng};
use rand::rngs::StdRng;

use std::sync::{Arc, Mutex};
use std::sync::mpsc;

mod hash;
use hash::sum_block_hash;

pub struct RandomOrderIterator {
    elements: Vec<u64>,
    rng: Box<dyn Rng>
}

impl RandomOrderIterator {
    pub fn with_seed(size: u64, seed: u64) -> Self {
        Self { elements: (0 as u64..size).collect() , rng: Box::new(StdRng::seed_from_u64(seed)), }
    }

    pub fn new(size: u64) -> Self {
        Self {
            elements: (0..size).collect(),
            rng: Box::new(rng()),
        }
    }
}

impl Iterator for RandomOrderIterator {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.elements.is_empty() {
            None
        } else {
            let rand_idx = self.rng.random_range(0..self.elements.len());
            Some(self.elements.swap_remove(rand_idx))
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.elements.len(), Some(self.elements.len()))
    }
}

const RAND_SEED: u64 = 1234;

fn main() {

    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        println!("Usage: cargo run --release --bin latency filepath block_size threads");
        return;
    }

    let file_res = File::open(args[1].clone());
    if file_res.is_err() {
        eprintln!("File not found: {}", args[1]);
        return;
    }
    let file = Arc::new(file_res.unwrap());

    let metadata = file.metadata();
    if metadata.is_err() {
        eprintln!("Error opening file: {}", args[1]);
        return;
    }

    let block_size : u64 = args[2].parse().expect("Error: block_size must be a number");
    assert!(block_size > 0, "Error: Block size block_size must be greater than 0");
    let num_threads : usize = args[3].parse().expect("Error: number of threads must be a number");
    assert!(num_threads > 0, concat!("Error: Number of threads must be greater than 0",
        "(at least one thread for reading and one thread for computing)"));

    let total_size = metadata.unwrap().len();
    let num_blocks = (total_size + block_size - 1) / block_size;
    let random_iter = RandomOrderIterator::with_seed(num_blocks, RAND_SEED);

    let mut offsets = Vec::with_capacity(num_blocks as usize);
    for block_idx in random_iter {
        let offset = block_idx * block_size;
        offsets.push(offset);
    }
    let shared_offsets = Arc::new(Mutex::new(offsets));
    let (tx, rx) = mpsc::sync_channel::<Vec<u8>>(num_blocks as usize);

    for _ in 0..num_threads {
        let file_clone = Arc::clone(&file);
        let offsets_clone = Arc::clone(&shared_offsets);
        let tx_clone = tx.clone();
        
        std::thread::spawn(move || {
            let mut buffer = vec![0u8; block_size as usize];

            loop {
                let offset = {
                    let mut offsets = offsets_clone.lock().unwrap();
                    if offsets.is_empty() {
                        break;
                    }
                    offsets.pop().unwrap()
                };

                let bytes_to_read = std::cmp::min(block_size, total_size - offset) as usize;

                let target_buf = &mut buffer[..bytes_to_read];
                let err = file_clone.read_exact_at(target_buf, offset);
                if err.is_err() {
                    eprintln!("Error reading file: {}", err.unwrap_err());
                    return;
                }

                tx_clone.send(target_buf.to_vec()).unwrap();
            }
        });
    }

    let mut result = vec![0u8; 16];
    drop(tx);
    
    loop {
        match rx.recv() {
            Ok(block) => {
                let block_hash = md5::compute(block);
                sum_block_hash(&mut result, &block_hash);
            }
            Err(_) => {
                break; 
            }
        }
    }
    print!("{} 0x", args[1].clone());
    for byte in &result {
        print!("{:02x}", byte);
    }
    println!("");
}
