// src/main.rs
mod walk;
mod result;
mod output;
mod args; // 1. Declare the module

use std::time::Instant;

fn main() {
    // 2. Call your dedicated parser
    let config = args::parse_args();

    println!("Scanning: {} (Top {} files)", config.root, config.top_n);

    let start = Instant::now();
    // 3. Use values from the config struct
    let mut results = walk::parallel_scan(&config.root);
    let duration = start.elapsed();

    output::print_results(&mut results, config.top_n, duration);
}
