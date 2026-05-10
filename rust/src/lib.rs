//! # Resplix CDC (Content-Defined Chunking)
//!
//! Implementation of the FastCDC (2016) algorithm with Gear Hash.
//! Engineered for the Resplix industrial data transit platform.

pub mod gear;

/// Configuration for FastCDC chunking.
#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub min_size: usize,
    pub avg_size: usize,
    pub max_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            min_size: 8 * 1024,      // 8KB
            avg_size: 16 * 1024,     // 16KB
            max_size: 32 * 1024,     // 32KB
        }
    }
}

/// A chunk of data identified by CDC.
pub struct Chunk {
    pub offset: usize,
    pub length: usize,
    pub hash: u64, // Rolling hash or final checksum
}

pub trait Chunker {
    fn next_chunk(&mut self) -> Option<Chunk>;
}
