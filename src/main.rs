// src/main.rs
mod args;
mod assets;
mod output;
mod result;
mod walk;
mod report;

use std::time::Instant;
use std::fs;
use crate::args::OutputFormat;

fn main() {
    let config = args::parse_args();

    println!("Scanning: {} (Top {} files)", config.root, config.top_n);

    let start = Instant::now();
    let mut results = walk::parallel_scan(&config.root);

    if let Some(ref allowed) = config.langs {
        results.retain(|r| allowed.contains(&r.lang));
    }

    let duration = start.elapsed();

    let report = report::build_report(&results, duration, config.top_n);

    match config.format {
        OutputFormat::Text => {
            output::text::print_results(&mut results, config.top_n, duration);
        }
        OutputFormat::Json => {
            let json = output::json::render_json(&report);

            if let Some(path) = &config.json_out {
                fs::write(path, json).unwrap_or_else(|e| {
                    eprintln!(
                        "\x1b[31;1mError:\x1b[0m Failed to write JSON to {}: {}",
                        path.display(),
                        e
                    );
                    std::process::exit(1);
                });
            } else {
                println!("{}", json);
            }
        }
    }
}
