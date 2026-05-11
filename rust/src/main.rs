use std::fs;
use resplix_cdc::{Config, test_insertion};

fn main() {
    // Load config from TOML
    let config_content = fs::read_to_string("config.toml")
        .expect("Could not read config.toml");
    let config: Config = toml::from_str(&config_content)
        .expect("Could not parse config.toml");

    println!("Resplix CDC Research Cluster - Rust Implementation");
    println!("Config Loaded: {:?}", config);

    // Create a large-ish string for testing
    let mut original = String::new();
    for i in 0..5000 {
        original.push_str(&format!("This is a line of text for CDC research. Index: {} ", i));
    }

    // Run insertion test
    test_insertion(&original, "INSERTED_DATA_HERE_TO_SHIFT_BYTES", config);
}
