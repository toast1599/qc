// src/main.rs
mod args;
mod assets;
mod output;
mod result;
mod walk;

use std::time::Instant;

fn main() {
    let config = args::parse_args();

    println!("Scanning: {} (Top {} files)", config.root, config.top_n);

    let start = Instant::now();
    let mut results = walk::parallel_scan(&config.root);

    if let Some(ref allowed) = config.langs {
        results.retain(|r| allowed.contains(&r.lang));
    }

    let duration = start.elapsed();

    output::print_results(&mut results, config.top_n, duration);
}
