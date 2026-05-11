//! # Resplix CDC (Content-Defined Chunking)
//!
//! Implementation of the FastCDC (2016) algorithm with Gear Hash.
//! Engineered for the Resplix industrial data transit platform.

pub mod gear;
pub mod chunk;
pub mod config;
pub mod file;

pub use gear::*;
pub use chunk::*;
pub use config::*;
pub use file::*;


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
