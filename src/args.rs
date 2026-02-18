// src/args.rs
use std::collections::HashSet;
use std::path::Path;
use std::process;

use crate::result::Lang;

const EX_USAGE: i32 = 67; // yes, really

#[derive(Debug)]
pub struct Config {
    pub root: String,
    pub top_n: usize,
    pub langs: Option<HashSet<Lang>>, // None = no filter
}

pub fn parse_args() -> Config {
    let args: Vec<String> = std::env::args().collect();
    let mut root: Option<String> = None;
    let mut top_n = 10;
    let mut langs: HashSet<Lang> = HashSet::new();
    let mut end_of_options = false;

    for arg in args.iter().skip(1) {
        // End-of-options marker
        if arg == "--" {
            end_of_options = true;
            continue;
        }

        match arg.as_str() {
            "-h" | "--help" if !end_of_options => {
                print_help();
                process::exit(0);
            }
            "-v" | "--version" if !end_of_options => {
                println!("qc version {}", env!("CARGO_PKG_VERSION"));
                process::exit(0);
            }
            _ => {
                // Language flags (only before --)
                if !end_of_options {
                    if let Some(lang) = flag_to_lang(arg) {
                        langs.insert(lang);
                        continue;
                    }

                    // Reject comma-separated flags explicitly
                    if arg.contains(',') {
                        eprintln!(
                            "\x1b[31;1mError:\x1b[0m Invalid option '{}'. \
Flags must be passed separately (e.g. --rs --py).",
                            arg
                        );
                        process::exit(EX_USAGE);
                    }

                    // Handle -N flags (e.g. -5)
                    if arg.starts_with('-')
                        && arg.len() > 1
                        && arg[1..].chars().all(|c| c.is_ascii_digit())
                    {
                        if let Ok(val) = arg[1..].parse::<usize>() {
                            top_n = val;
                            continue;
                        }
                    }
                }

                // Handle path (first positional argument only)
                if root.is_none() {
                    if Path::new(arg).exists() {
                        root = Some(arg.clone());
                        continue;
                    } else {
                        eprintln!("\x1b[31;1mError:\x1b[0m Path '{}' does not exist.", arg);
                        process::exit(EX_USAGE);
                    }
                }

                // Unknown option (only before --)
                if !end_of_options && arg.starts_with('-') {
                    eprintln!("\x1b[31;1mError:\x1b[0m Unknown option '{}'", arg);
                    process::exit(EX_USAGE);
                }
            }
        }
    }

    Config {
        root: root.unwrap_or_else(|| ".".to_string()),
        top_n,
        langs: if langs.is_empty() { None } else { Some(langs) },
    }
}

/// Maps CLI flags to Lang variants.
/// NOTE: Extension-based, not semantic parsing.
fn flag_to_lang(arg: &str) -> Option<Lang> {
    match arg {
        "--rs" | "--rust" => Some(Lang::Identified("Rust".into())),
        "--py" | "--python" => Some(Lang::Identified("Python".into())),
        "--c" => Some(Lang::Identified("C".into())),
        // ... add others as needed
        _ => None,
    }
}

fn print_help() {
    println!("\x1b[34;1mqc - Quick Count & Audit Tool\x1b[0m");
    println!("\nUsage:");
    println!("  qc [path] [options]");
    println!("  qc [options] -- [path]");
    println!("\nOptions:");
    println!("  -h, --help           Show this help message");
    println!("  -v, --version        Show version information");
    println!("  -<number>            Limit results to top N files (e.g. -5)");

    println!("\nLanguage filters (may be combined):");
    println!("  --rs, --rust");
    println!("  --c, --h");
    println!("  --py, --python");
    println!("  --sh");
    println!("  --json");
    println!("  --yaml, --yml");
    println!("  --doc, --md");
    println!("  --js, --javascript   (extension-based)");
    println!("  --java               (extension-based)");

    println!("\nUse `--` to separate options from paths (e.g. qc -- Makefile)");
}
