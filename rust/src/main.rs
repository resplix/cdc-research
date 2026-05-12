use std::fs;
use std::time::Instant;
use resplix_cdc::{Config, compare_dedupe, read_file, test_corruption, test_reordering, print_stats, FastCDC, Chunker};

fn main() {
    // Load config
    let config_content = fs::read_to_string("config.toml")
        .expect("Could not read config.toml");
    let config: Config = toml::from_str(&config_content)
        .expect("Could not parse config.toml");

    println!("====================================================");
    println!("/// RESPLIX CDC EXPERIMENTAL RESEARCH RUNNER");
    println!("====================================================");
    println!("Config: {:?}", config);

    let base_rand = read_file("../tests/data/random_base.txt").unwrap();
    let shifted_rand = read_file("../tests/data/random_shifted.txt").unwrap();
    let base_large = read_file("../tests/data/large_base.txt").unwrap();
    let modified_large = read_file("../tests/data/large_modified.txt").unwrap();

    // Experiment 1: Byte-Shift Resilience
    println!("\n[EXPERIMENT 1: Byte-Shift Resilience]");
    compare_dedupe(&base_rand, &shifted_rand, config);

    // Experiment 2: Corruption Resilience
    println!("\n[EXPERIMENT 2: Single-Byte Corruption]");
    test_corruption(&base_rand, config);

    // Experiment 3: Block Reordering
    println!("\n[EXPERIMENT 3: Block Reordering]");
    test_reordering(&base_rand, config);

    // Experiment 4: Statistical Distribution
    println!("\n[EXPERIMENT 4: Statistical Distribution Analysis]");
    let mut chunks = Vec::new();
    let mut cdc = FastCDC::new(&base_rand, config);
    while let Some(c) = cdc.next_chunk() { chunks.push(c); }
    print_stats(&chunks);

    // Experiment 5: Large File Modification Resilience
    println!("\n[EXPERIMENT 5: Large File Modification (115KB)]");
    compare_dedupe(&base_large, &modified_large, config);

    // Experiment 6: Throughput Measurement
    println!("\n[EXPERIMENT 6: Throughput Benchmark]");
    measure_throughput(&base_large, config);
}

/// Measures single-core CDC throughput in MB/s.
fn measure_throughput(data: &[u8], config: Config) {
    let iterations = 1000;
    let start = Instant::now();

    for _ in 0..iterations {
        let mut cdc = FastCDC::new(data, config);
        while let Some(chunk) = cdc.next_chunk() {
            std::hint::black_box(&chunk);
        }
    }

    let elapsed = start.elapsed();
    let total_bytes = data.len() as f64 * iterations as f64;
    let mb_per_sec = (total_bytes / (1024.0 * 1024.0)) / elapsed.as_secs_f64();

    println!("--- Throughput Report ---");
    println!("Data Size:    {:.2} KB", data.len() as f64 / 1024.0);
    println!("Iterations:   {}", iterations);
    println!("Total Time:   {:.3} ms", elapsed.as_secs_f64() * 1000.0);
    println!("Throughput:   {:.2} MB/s", mb_per_sec);
    println!("Per-Iter:     {:.3} µs", elapsed.as_secs_f64() * 1_000_000.0 / iterations as f64);

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            println!("SIMD Path:    AVX2 (256-bit gather)");
        } else if is_x86_feature_detected!("sse4.1") {
            println!("SIMD Path:    SSE4.1");
        } else {
            println!("SIMD Path:    Scalar");
        }
    }
}
