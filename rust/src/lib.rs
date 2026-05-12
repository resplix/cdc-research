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

    #[test]
    fn test_fastcdc_random_data() {
        // Deterministic "random" data — LCG with known seed
        let mut data = vec![0u8; 256 * 1024]; // 256KB
        let mut rng: u64 = 0xDEADBEEF;
        for byte in data.iter_mut() {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            *byte = (rng >> 33) as u8;
        }

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

        assert!(count > 1, "Random data should produce multiple chunks");
        assert_eq!(total_len, data.len());
    }

    #[test]
    fn test_avx2_matches_scalar() {
        // Verify that the AVX2 path produces identical results to scalar
        let mut data = vec![0u8; 64 * 1024]; // 64KB
        let mut rng: u64 = 0xCAFEBABE;
        for byte in data.iter_mut() {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            *byte = (rng >> 33) as u8;
        }

        let mask = 0x0003590703530000u64;

        let scalar_result = gear::find_cutpoint_scalar(&data, 0, data.len(), mask);

        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                let avx2_result = unsafe {
                    gear::find_cutpoint_avx2(&data, 0, data.len(), mask)
                };
                assert_eq!(
                    scalar_result, avx2_result,
                    "AVX2 and Scalar must produce identical cut-points.\n\
                     Scalar: pos={}, hash={:#018x}\n\
                     AVX2:   pos={}, hash={:#018x}",
                    scalar_result.0, scalar_result.1,
                    avx2_result.0, avx2_result.1,
                );
            }
        }

        // Also verify with different start offsets (exercises the scalar tail)
        for offset in [1, 2, 3, 7, 15, 63] {
            let scalar = gear::find_cutpoint_scalar(&data, offset, data.len(), mask);
            let dispatched = gear::find_cutpoint(&data, offset, data.len(), mask);
            assert_eq!(scalar, dispatched, "Mismatch at offset {}", offset);
        }
    }

    #[test]
    fn test_gear_table_uniqueness() {
        // Every entry in the gear table must be unique
        let mut seen = std::collections::HashSet::new();
        for (i, &val) in gear::GEAR_TABLE.iter().enumerate() {
            assert!(seen.insert(val), "Duplicate gear table entry at index {}: {:#018x}", i, val);
            // Verify no degenerate near-zero values
            assert!(val.count_ones() > 10, "Low entropy at index {}: {:#018x} ({} bits set)", i, val, val.count_ones());
        }
    }
}
