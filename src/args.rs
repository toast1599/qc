// src/args.rs

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process;

use crate::result::Lang;

const EX_USAGE: i32 = 67; // yes, really

// ---- Canonical language names ----
// These MUST match classifier output.
// Centralizing them removes accidental string drift.
const LANG_RUST: &str = "Rust";
const LANG_PYTHON: &str = "Python";
const LANG_C: &str = "C";
const LANG_SHELL: &str = "Shell";
const LANG_YAML: &str = "YAML";
const LANG_MARKDOWN: &str = "Markdown";
const LANG_JS: &str = "JavaScript";
const LANG_JAVA: &str = "Java";

#[derive(Debug)]
pub struct Config {
    pub root: String,
    pub top_n: usize,
    pub langs: Option<HashSet<Lang>>,
    pub format: OutputFormat,
    pub json_out: Option<PathBuf>,
}

pub fn parse_args() -> Config {
    let args: Vec<String> = std::env::args().collect();

    let mut root: Option<String> = None;
    let mut top_n: usize = 10;
    let mut langs: HashSet<Lang> = HashSet::new();
    let mut format = OutputFormat::Text;
    let mut json_out: Option<PathBuf> = None;
    let mut end_of_options = false;

    let mut iter = args.iter().skip(1).peekable();

    while let Some(arg) = iter.next() {
        if arg == "--" {
            end_of_options = true;
            continue;
        }

        if !end_of_options {
            match arg.as_str() {
                "-h" | "--help" => {
                    print_help();
                    process::exit(0);
                }
                "-v" | "--version" => {
                    println!("qc version {}", env!("CARGO_PKG_VERSION"));
                    process::exit(0);
                }
                "--json" => {
                    format = OutputFormat::Json;
                    continue;
                }
                "--json-out" => {
                    format = OutputFormat::Json;
                    let path = iter.next().unwrap_or_else(|| {
                        eprintln!(
                            "\x1b[31;1mError:\x1b[0m --json-out requires a path"
                        );
                        process::exit(EX_USAGE);
                    });
                    json_out = Some(PathBuf::from(path));
                    continue;
                }
                _ => {}
            }

            // Language flags
            if let Some(lang) = flag_to_lang(arg) {
                langs.insert(lang);
                continue;
            }

            // Numeric shorthand: -5
            if arg.starts_with('-')
                && arg.len() > 1
                && arg[1..].chars().all(|c| c.is_ascii_digit())
            {
                match arg[1..].parse::<usize>() {
                    Ok(val) => {
                        top_n = val;
                        continue;
                    }
                    Err(_) => {
                        eprintln!(
                            "\x1b[31;1mError:\x1b[0m Invalid number '{}'",
                            arg
                        );
                        process::exit(EX_USAGE);
                    }
                }
            }
        }

        // Path handling (paths may contain commas, dashes, etc.)
        if root.is_none() {
            if Path::new(arg).exists() {
                root = Some(arg.clone());
                continue;
            } else {
                eprintln!(
                    "\x1b[31;1mError:\x1b[0m Path '{}' does not exist.",
                    arg
                );
                process::exit(EX_USAGE);
            }
        }

        if !end_of_options && arg.starts_with('-') {
            eprintln!(
                "\x1b[31;1mError:\x1b[0m Unknown option '{}'",
                arg
            );
            process::exit(EX_USAGE);
        }
    }

    Config {
        root: root.unwrap_or_else(|| ".".to_string()),
        top_n,
        langs: if langs.is_empty() { None } else { Some(langs) },
        format,
        json_out,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
}

/// Maps CLI flags to canonical language identifiers.
/// These identifiers MUST match classifier output exactly.
fn flag_to_lang(arg: &str) -> Option<Lang> {
    let name = match arg {
        "--rs" | "--rust" => LANG_RUST,
        "--py" | "--python" => LANG_PYTHON,
        "--c" => LANG_C,
        "--sh" => LANG_SHELL,
        "--yaml" | "--yml" => LANG_YAML,
        "--md" | "--doc" => LANG_MARKDOWN,
        "--js" | "--javascript" => LANG_JS,
        "--java" => LANG_JAVA,
        _ => return None,
    };

    Some(Lang::Identified(name.to_string()))
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
    println!("  --json               Output results as JSON");
    println!("  --json-out <path>    Write JSON output to file");

    println!("\nLanguage filters (may be combined):");
    println!("  --rs, --rust");
    println!("  --c");
    println!("  --py, --python");
    println!("  --sh");
    println!("  --yaml, --yml");
    println!("  --doc, --md");
    println!("  --js, --javascript");
    println!("  --java");

    println!("\nUse `--` to separate options from paths (e.g. qc -- Makefile)");
}
