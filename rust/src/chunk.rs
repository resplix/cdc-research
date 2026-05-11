use crate::gear;
use crate::config::Config;
use std::io::Read;

/// A chunk of data identified by CDC.
#[derive(Debug, Clone, Copy)]
pub struct Chunk {
    pub offset: usize,
    pub length: usize,
    pub hash: u64, // Rolling hash or final checksum
}

pub trait Chunker {
    fn next_chunk(&mut self) -> Option<Chunk>;
}

/// StreamingChunker handles CDC on an arbitrary stream.
pub struct StreamingChunker<R: Read> {
    reader: R,
    buffer: Vec<u8>,
    pos: usize,
    len: usize,
    config: Config,
    mask_s: u64,
    mask_l: u64,
    eof: bool,
}

impl<R: Read> StreamingChunker<R> {
    pub fn new(reader: R, config: Config) -> Self {
        // FastCDC masks from the paper (scattered 64-bit patterns)
        let mask_s: u64 = 0x0003590703530000;
        let mask_l: u64 = 0x0000d90003530000;

        Self {
            reader,
            buffer: vec![0u8; config.max_size * 2],
            pos: 0,
            len: 0,
            config,
            mask_s,
            mask_l,
            eof: false,
        }
    }

    fn fill_buffer(&mut self) -> std::io::Result<()> {
        if self.pos > 0 {
            self.buffer.copy_within(self.pos..self.len, 0);
            self.len -= self.pos;
            self.pos = 0;
        }

        let n = self.reader.read(&mut self.buffer[self.len..])?;
        if n == 0 {
            self.eof = true;
        }
        self.len += n;
        Ok(())
    }
}

impl<R: Read> Chunker for StreamingChunker<R> {
    fn next_chunk(&mut self) -> Option<Chunk> {
        if self.eof && self.pos >= self.len {
            return None;
        }

        // Ensure we have enough data for at least one max-sized chunk if possible
        if !self.eof && (self.len - self.pos) < self.config.max_size {
            let _ = self.fill_buffer();
        }

        let remaining = self.len - self.pos;
        if remaining == 0 {
            return None;
        }

        if remaining <= self.config.min_size {
            let chunk = Chunk {
                offset: self.pos,
                length: remaining,
                hash: 0,
            };
            self.pos = self.len;
            return Some(chunk);
        }

        let mut hash = 0u64;
        let start = self.pos;
        let mut end = start + self.config.min_size;
        let max = (start + self.config.max_size).min(self.len);
        let avg = start + self.config.avg_size;

        let limit_s = avg.min(max);
        while end < limit_s {
            hash = gear::update_hash(hash, self.buffer[end]);
            if (hash & self.mask_s) == 0 {
                let length = (end + 1) - start;
                self.pos = end + 1;
                return Some(Chunk { offset: start, length, hash });
            }
            end += 1;
        }

        while end < max {
            hash = gear::update_hash(hash, self.buffer[end]);
            if (hash & self.mask_l) == 0 {
                let length = (end + 1) - start;
                self.pos = end + 1;
                return Some(Chunk { offset: start, length, hash });
            }
            end += 1;
        }

        let length = max - start;
        self.pos = max;
        Some(Chunk {
            offset: start,
            length,
            hash,
        })
    }
}

/// FastCDC implementation for in-memory data.
pub struct FastCDC<'a> {
    data: &'a [u8],
    pos: usize,
    config: Config,
    mask_s: u64,
    mask_l: u64,
}

impl<'a> FastCDC<'a> {
    pub fn new(data: &'a [u8], config: Config) -> Self {
        let mask_s: u64 = 0x0003590703530000;
        let mask_l: u64 = 0x0000d90003530000;

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
                offset: self.pos,
                length: remaining,
                hash: 0, 
            };
            self.pos = self.data.len();
            return Some(chunk);
        }

        let mut hash = 0u64;
        let start = self.pos;
        let mut end = start + self.config.min_size;
        let max = (start + self.config.max_size).min(self.data.len());
        let avg = start + self.config.avg_size;

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

        while end < max {
            hash = gear::update_hash(hash, self.data[end]);
            if (hash & self.mask_l) == 0 {
                let length = (end + 1) - start;
                self.pos = end + 1;
                return Some(Chunk { offset: start, length, hash });
            }
            end += 1;
        }

        let length = max - start;
        self.pos = max;
        Some(Chunk {
            offset: start,
            length,
            hash,
        })
    }
}
