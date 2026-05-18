use std::fs::File;
use std::io::{self, Read};
use crate::chunk::{Chunk, Chunker, FastCDC};
use crate::config::Config;
use memmap2::Mmap;

/// Read a file into a byte vector.
pub fn read_file(path: &str) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

/// Memory-map a file for zero-copy access.
///
/// This avoids a userspace read buffer and gives the kernel readahead a clean,
/// sequential access pattern.
pub fn mmap_file(path: &str) -> io::Result<Mmap> {
    let file = File::open(path)?;
    // Safety: the file handle outlives the mapping and we treat the mapping as read-only.
    unsafe { Mmap::map(&file) }
}

/// Chunk a file by first reading it into a `Vec<u8>`.
///
/// Returns the number of chunks produced.
pub fn chunk_file_read_to_vec(path: &str, config: Config) -> io::Result<u64> {
    let data = read_file(path)?;
    Ok(chunk_bytes(&data, config))
}

/// Chunk a file using a zero-copy memory map.
///
/// Returns the number of chunks produced.
pub fn chunk_file_mmap(path: &str, config: Config) -> io::Result<u64> {
    let mmap = mmap_file(path)?;
    Ok(chunk_bytes(&mmap, config))
}

fn chunk_bytes(data: &[u8], config: Config) -> u64 {
    let mut cdc = FastCDC::new(data, config);
    let mut count = 0u64;
    while let Some(chunk) = cdc.next_chunk() {
        std::hint::black_box(chunk);
        count += 1;
    }
    count
}

/// Compare two datasets and return the deduplication ratio.
pub fn compare_dedupe(data1: &[u8], data2: &[u8], config: Config) {
    let mut cdc1 = FastCDC::new(data1, config);
    let mut chunks1 = Vec::new();
    while let Some(chunk) = cdc1.next_chunk() { chunks1.push(chunk); }

    let mut cdc2 = FastCDC::new(data2, config);
    let mut chunks2 = Vec::new();
    while let Some(chunk) = cdc2.next_chunk() { chunks2.push(chunk); }

    let dups = get_duplicates(&chunks1, &chunks2);
    let total = chunks2.len();

    println!("--- Chunk Breakdown ---");
    print!("File 1: ");
    for c in &chunks1 { print!("[{}KB] ", c.length / 1024); }
    println!("\nFile 2: ");
    for c in &chunks2 { print!("[{}KB] ", c.length / 1024); }
    println!("\n");

    println!("--- Deduplication Report ---");
    println!("Duplicate Chunks: {} / {}", dups, total);
    println!("Dedupe Ratio:     {:.2}%", (dups as f64 / total as f64) * 100.0);
}

/// Detailed statistical report of chunk distribution.
pub fn print_stats(chunks: &[Chunk]) {
    if chunks.is_empty() { return; }
    
    let mut min = usize::MAX;
    let mut max = 0;
    let mut sum = 0;
    
    for c in chunks {
        if c.length < min { min = c.length; }
        if c.length > max { max = c.length; }
        sum += c.length;
    }
    
    let avg = sum / chunks.len();
    
    println!("--- Distribution Statistics ---");
    println!("Total Chunks: {}", chunks.len());
    println!("Min Size:     {:.2} KB", min as f64 / 1024.0);
    println!("Max Size:     {:.2} KB", max as f64 / 1024.0);
    println!("Avg Size:     {:.2} KB", avg as f64 / 1024.0);
    println!("Total Data:   {:.2} KB", sum as f64 / 1024.0);
}

/// Compare two sets of chunks and return duplicate count.
pub fn get_duplicates(chunks1: &[Chunk], chunks2: &[Chunk]) -> usize {
    let hashes1: Vec<_> = chunks1.iter().map(|c| c.content_hash).collect();
    let mut duplicates = 0;
    for chunk in chunks2 {
        if hashes1.contains(&chunk.content_hash) {
            duplicates += 1;
        }
    }
    duplicates
}

/// Experiment 4: Single-Byte Corruption.
pub fn test_corruption(data: &[u8], config: Config) {
    let mut corrupted = data.to_vec();
    if corrupted.len() > 5000 {
        corrupted[5000] ^= 0xFF; // Flip bits of the 5000th byte
    }

    let mut cdc1 = FastCDC::new(data, config);
    let mut chunks1 = Vec::new();
    while let Some(c) = cdc1.next_chunk() { chunks1.push(c); }

    let mut cdc2 = FastCDC::new(&corrupted, config);
    let mut chunks2 = Vec::new();
    while let Some(c) = cdc2.next_chunk() { chunks2.push(c); }

    let dups = get_duplicates(&chunks1, &chunks2);
    println!("Original vs Corrupted (1-byte change at index 5000)");
    println!("Duplicate Chunks: {} / {}", dups, chunks2.len());
    println!("Dedupe Ratio:     {:.2}%", (dups as f64 / chunks2.len() as f64) * 100.0);
}

/// Experiment 5: Block Reordering.
pub fn test_reordering(data: &[u8], config: Config) {
    let mut reordered = Vec::new();
    let mid = data.len() / 2;
    
    // Swap first half and second half
    reordered.extend_from_slice(&data[mid..]);
    reordered.extend_from_slice(&data[..mid]);

    let mut cdc1 = FastCDC::new(data, config);
    let mut chunks1 = Vec::new();
    while let Some(c) = cdc1.next_chunk() { chunks1.push(c); }

    let mut cdc2 = FastCDC::new(&reordered, config);
    let mut chunks2 = Vec::new();
    while let Some(c) = cdc2.next_chunk() { chunks2.push(c); }

    println!("Trace: Midpoint index is {}", mid);
    println!("File 1 (Original) boundaries:");
    for c in &chunks1 { println!("  - Offset: {}, Len: {}, Hash: {:02x?}", c.offset, c.length, &c.content_hash[..4]); }
    
    println!("File 2 (Reordered) boundaries:");
    for c in &chunks2 { println!("  - Offset: {}, Len: {}, Hash: {:02x?}", c.offset, c.length, &c.content_hash[..4]); }

    let dups = get_duplicates(&chunks1, &chunks2);
    println!("Original vs Reordered (Swapped halves)");
    println!("Duplicate Chunks: {} / {}", dups, chunks2.len());
    println!("Dedupe Ratio:     {:.2}%", (dups as f64 / chunks2.len() as f64) * 100.0);
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
