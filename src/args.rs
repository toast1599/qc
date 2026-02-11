// src/args.rs
use std::path::Path;
use std::process;

#[derive(Debug)]
pub struct Config {
    pub root: String,
    pub top_n: usize,
}

pub fn parse_args() -> Config {
    let args: Vec<String> = std::env::args().collect();
    let mut root: Option<String> = None;
    let mut top_n = 10;

    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                process::exit(0);
            }
            "-v" | "--version" => {
                println!("qc version 0.1.1");
                process::exit(0);
            }
            _ => {
                // Handle -N flags
                if arg.starts_with('-') && arg.len() > 1 && arg.chars().nth(1).unwrap().is_ascii_digit() {
                    if let Ok(val) = arg[1..].parse::<usize>() {
                        top_n = val;
                    }
                } 
                // Handle Paths
                else if !arg.starts_with('-') && root.is_none() {
                    if Path::new(arg).exists() {
                        root = Some(arg.clone());
                    } else {
                        eprintln!("\x1b[31;1mError:\x1b[0m Path '{}' does not exist.", arg);
                        process::exit(1);
                    }
                }
            }
        }
    }

    Config {
        root: root.unwrap_or_else(|| ".".to_string()),
        top_n,
    }
}

fn print_help() {
    println!("\x1b[34;1mqc - Quick Count & Audit Tool\x1b[0m");
    println!("\nUsage:");
    println!("  qc [path] [-number]");
    println!("\nOptions:");
    println!("  -h, --help       Show this help message");
    println!("  -v, --version    Show version information");
    println!("  -<number>        Limit results to top N files (e.g. -5)");
}
