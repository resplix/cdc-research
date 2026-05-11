use std::fs;
use resplix_cdc::{Config, compare_dedupe, read_file};

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

    // Experiment 1: Byte-Shift Resilience
    println!("\n[EXPERIMENT 1: Byte-Shift Resilience]");
    let base = read_file("../tests/data/random_base.txt").unwrap();
    let shifted = read_file("../tests/data/random_shifted.txt").unwrap();
    
    println!("Comparing 'random_base.txt' vs 'random_shifted.txt' (1-byte shift at start)");
    compare_dedupe(&base, &shifted, config);

    // Experiment 2: Modification Resilience
    println!("\n[EXPERIMENT 2: Tail Modification Resilience]");
    let base_large = read_file("../tests/data/large_base.txt").unwrap();
    let modified_large = read_file("../tests/data/large_modified.txt").unwrap();
    
    println!("Comparing 'large_base.txt' vs 'large_modified.txt' (appended content)");
    compare_dedupe(&base_large, &modified_large, config);

    // Experiment 3: Sensitivity Analysis (Dynamic Config)
    println!("\n[EXPERIMENT 3: Sensitivity Analysis]");
    let mut small_config = config;
    small_config.min_size = 2048;
    small_config.avg_size = 4096;
    small_config.max_size = 8192;
    
    println!("Using smaller chunks (Avg 4KB) on base file:");
    compare_dedupe(&base, &base, small_config);
}
