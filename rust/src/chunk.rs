pub use config;

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
    config: config::Config,
    mask_s: u64, // Small mask for normalization
    mask_l: u64, // Large mask for normalization
}

impl<'a> FastCDC<'a> {
    pub fn new(data: &'a [u8], config: Config) -> Self {
        // Typical masks for 16KB avg chunk size
        // mask = (1 << bits) - 1
        //consecutive bits
        // << n shifts the bits n times to left from nth point we start 1000...index0
        // -1 brings 11 remove one from 11 carry towrads right hence 1111...1
        // 0111 1111 1111 1111
        // let mask_s = (1 << 15) - 1; 
        // 0111 1111 1111
        // let mask_l = (1 << 11) - 1;
        // 0001 1111 1111 1111
        // let mask_a = (1 << 13) - 1;
        // FastCDC masks from the paper (scattered 64-bit patterns)
        // These create the 48-byte sliding window effect
        let mask_s: u64 = 0x0003590703530000LL;  // 15 '1' bits (SCATTERED)
        let mask_l: u64 = 0x0000d90003530000LL;  // 11 '1' bits (SCATTERED)
        let mask_a: u64 = 0x0000d90303530000LL;  // 13 '1' bits (SCATTERED)

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
