use serde::{Deserialize, Serialize};

/// Controls whether and how chunk content hashes are computed.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContentHashMode {
    /// Compute a cryptographic BLAKE3 hash per chunk (content-addressing).
    Blake3,
    /// Skip content hashing (CDC boundary finding only).
    None,
}

/// Controls how cut-point scanning and content hashing are orchestrated.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PipelineMode {
    /// Current implementation: find cut-point (Gear) then hash the resulting chunk slice (BLAKE3).
    /// This is simple and fast for raw cut-point throughput, but reads chunk bytes twice.
    TwoPass,
}

/// Configuration for FastCDC chunking.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Config {
    pub min_size: usize,
    pub avg_size: usize,
    pub max_size: usize,
    pub content_hash_mode: ContentHashMode,
    pub pipeline_mode: PipelineMode,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            min_size: 8 * 1024,      // 8KB
            avg_size: 16 * 1024,     // 16KB
            max_size: 32 * 1024,     // 32KB
            content_hash_mode: ContentHashMode::Blake3,
            pipeline_mode: PipelineMode::TwoPass,
        }
    }
}
