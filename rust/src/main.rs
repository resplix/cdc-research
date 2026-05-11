use std::fs;
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
}
