use std::fs::File;
use std::io::{self, Read};
use crate::chunk::{Chunker, FastCDC};
use crate::config::Config;

/// Read a file into a byte vector.
pub fn read_file(path: &str) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

/// Compare two datasets and return the deduplication ratio.
pub fn compare_dedupe(data1: &[u8], data2: &[u8], config: Config) {
    let mut cdc1 = FastCDC::new(data1, config);
    let mut chunks1 = Vec::new();
    while let Some(chunk) = cdc1.next_chunk() {
        chunks1.push(chunk);
    }

    let mut cdc2 = FastCDC::new(data2, config);
    let mut chunks2 = Vec::new();
    while let Some(chunk) = cdc2.next_chunk() {
        chunks2.push(chunk);
    }

    println!("--- Chunk Breakdown ---");
    print!("File 1 Chunks: ");
    for c in &chunks1 { print!("[{}KB] ", c.length / 1024); }
    println!("\nFile 2 Chunks: ");
    for c in &chunks2 { print!("[{}KB] ", c.length / 1024); }
    println!("\n");

    let total_chunks = chunks2.len();
    let mut duplicates = 0;
    
    let hashes1: Vec<_> = chunks1.iter().map(|c| c.content_hash).collect();

    for chunk in &chunks2 {
        if hashes1.contains(&chunk.content_hash) {
            duplicates += 1;
        }
    }

    println!("--- Deduplication Report ---");
    println!("Duplicate Chunks: {} / {}", duplicates, total_chunks);
    println!("Dedupe Ratio: {:.2}%", (duplicates as f64 / total_chunks as f64) * 100.0);
}

/// Simulate an insertion test.
pub fn test_insertion(original: &str, insertion: &str, config: Config) {
    let data1 = original.as_bytes();
    let mut data2 = original.as_bytes().to_vec();
    
    // Insert at 1/4th of the way through
    let pos = data1.len() / 4;
    for (i, byte) in insertion.as_bytes().iter().enumerate() {
        data2.insert(pos + i, *byte);
    }

    println!("Testing insertion of '{}' at position {}", insertion, pos);
    compare_dedupe(data1, &data2, config);
}
