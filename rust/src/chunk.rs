use crate::gear;
use crate::config::Config;
use std::io::Read;

/// A chunk of data identified by CDC.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Chunk {
    pub offset: usize,
    pub length: usize,
    pub rolling_hash: u64,
    pub content_hash: [u8; 32],
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

        if !self.eof && (self.len - self.pos) < self.config.max_size {
            let _ = self.fill_buffer();
        }

        let remaining = self.len - self.pos;
        if remaining == 0 {
            return None;
        }

        let start = self.pos;
        let mut end: usize;
        let mut hash = 0u64;

        if remaining <= self.config.min_size {
            end = self.len;
        } else {
            end = start + self.config.min_size;
            let max = (start + self.config.max_size).min(self.len);
            let avg = start + self.config.avg_size;

            let limit_s = avg.min(max);
            let mut found = false;
            while end < limit_s {
                hash = gear::update_hash(hash, self.buffer[end]);
                if (hash & self.mask_s) == 0 {
                    end = end + 1;
                    found = true;
                    break;
                }
                end += 1;
            }

            if !found {
                while end < max {
                    hash = gear::update_hash(hash, self.buffer[end]);
                    if (hash & self.mask_l) == 0 {
                        end = end + 1;
                        found = true;
                        break;
                    }
                    end += 1;
                }
            }
            
            if !found {
                end = max;
            }
        }

        let length = end - start;
        let chunk_data = &self.buffer[start..end];
        let content_hash = blake3::hash(chunk_data).into();

        self.pos = end;
        Some(Chunk {
            offset: start,
            length,
            rolling_hash: hash,
            content_hash,
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

        let start = self.pos;
        let remaining = self.data.len() - start;
        let mut end: usize;
        let mut hash = 0u64;

        if remaining <= self.config.min_size {
            end = self.data.len();
        } else {
            let mut found = false;
            let start_scan = start + self.config.min_size;
            let max_scan = (start + self.config.max_size).min(self.data.len());
            let avg_scan = (start + self.config.avg_size).min(max_scan);

            // 1. Scan with small mask (Normalized Distribution)
            let (new_pos, h) = gear::find_cutpoint(self.data, start_scan, avg_scan, self.mask_s);
            if new_pos < avg_scan || (h & self.mask_s) == 0 {
                end = new_pos;
                hash = h;
                found = true;
            }

            // 2. Scan with large mask if no cut-point found
            if !found {
                let (new_pos, h) = gear::find_cutpoint(self.data, avg_scan, max_scan, self.mask_l);
                end = new_pos;
                hash = h;
            }
        }

        let length = end - start;
        let chunk_data = &self.data[start..end];
        let content_hash = blake3::hash(chunk_data).into();

        self.pos = end;
        Some(Chunk {
            offset: start,
            length,
            rolling_hash: hash,
            content_hash,
        })
    }
}
