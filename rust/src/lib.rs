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

use std::io::Read;

/// StreamingChunker handles CDC on an arbitrary stream.
pub struct StreamingChunker<R: Read> {
    reader: R,
    buffer: Vec<u8>,
    pos: usize,
    config: Config,
    // ... additional fields for state management
}

impl<R: Read> StreamingChunker<R> {
    pub fn new(reader: R, config: Config) -> Self {
        Self {
            reader,
            buffer: vec![0u8; config.max_size * 2],
            pos: 0,
            config,
        }
    }
}

/// FastCDC implementation.
pub struct FastCDC<'a> {
    data: &'a [u8],
    pos: usize,
    config: Config,
    mask_s: u64, // Small mask for normalization
    mask_l: u64, // Large mask for normalization
}

impl<'a> FastCDC<'a> {
    pub fn new(data: &'a [u8], config: Config) -> Self {
        // Typical masks for 16KB avg chunk size
        // mask = (1 << bits) - 1
        let mask_s = (1 << 15) - 1; 
        let mask_l = (1 << 11) - 1;
        let mask_a = (1 << 13) - 1;

        Self {
            data,
            pos: 0,
            config,
            mask_s,
            mask_l,
        }
    }
}

impl<'a> Chunker for FastCDC<'a> {
    fn next_chunk(&mut self) -> Option<Chunk> {
        if self.pos >= self.data.len() {
            return None;
        }

        let remaining = self.data.len() - self.pos;
        if remaining <= self.config.min_size {
            let chunk = Chunk {
                offset: self.pos,//byte offset in th file or steam
                length: remaining,//remaining bytes from offset
                hash: 0, 
            };
            //this gets the byte position
            self.pos = self.data.len();
            return Some(chunk);
        }
        //we start with zero, the hash changes as we slide over the window
        let mut hash = 0u64;
        let start = self.pos;
        let mut end = start + self.config.min_size;
        let max = (start + self.config.max_size).min(self.data.len());
        let avg = start + self.config.avg_size;

        // Phase 1: Normalized Chunking with small mask
        let limit_s = avg.min(max);
        while end < limit_s {
            hash = gear::update_hash(hash, self.data[end]);
            if (hash & self.mask_s) == 0 {
                let length = (end + 1) - start;
                self.pos = end + 1;
                return Some(Chunk { offset: start, length, hash });
            }
            end += 1;
        }

        // Phase 2: Normalized Chunking with large mask
        while end < max {
            hash = gear::update_hash(hash, self.data[end]);
            if (hash & self.mask_l) == 0 {
                let length = (end + 1) - start;
                self.pos = end + 1;
                return Some(Chunk { offset: start, length, hash });
            }
            end += 1;
        }

        // Phase 3: Max size reached
        let length = max - start;
        self.pos = max;
        Some(Chunk {
            offset: start,
            length,
            hash,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fastcdc_basic() {
        let data = vec![0u8; 100 * 1024]; // 100KB
        let config = Config::default();
        let mut cdc = FastCDC::new(&data, config);
        
        let mut count = 0;
        let mut total_len = 0;
        while let Some(chunk) = cdc.next_chunk() {
            count += 1;
            total_len += chunk.length;
            assert!(chunk.length >= config.min_size || total_len == data.len());
            assert!(chunk.length <= config.max_size);
        }
        
        assert!(count > 0);
        assert_eq!(total_len, data.len());
    }
}
