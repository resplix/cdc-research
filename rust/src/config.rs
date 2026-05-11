use serde::{Deserialize, Serialize};

/// Configuration for FastCDC chunking.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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
