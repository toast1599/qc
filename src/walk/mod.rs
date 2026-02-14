// src/walk.rs
use crate::result::FileResult;
use crossbeam_channel::unbounded;
use ignore::{WalkBuilder, WalkState};
use indicatif::{ProgressBar, ProgressStyle};
use memmap2::Mmap;
use std::{fs::File, path::Path};

mod classify;
use classify::classify_extension;

pub fn parallel_scan(root: &str) -> Vec<FileResult> {
    let (tx, rx) = unbounded();

    let pb = ProgressBar::new_spinner().with_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.blue} {msg} [{elapsed_precise}] {pos} files")
            .unwrap(),
    );
    pb.set_message("Auditing");

    WalkBuilder::new(root)
        .threads(num_cpus::get())
        .build_parallel()
        .run(|| {
            let tx = tx.clone();
            let pb = pb.clone();

            Box::new(move |entry| {
                let Ok(entry) = entry else {
                    return WalkState::Continue;
                };

                let path = entry.path();
                if !path.is_file() {
                    return WalkState::Continue;
                }

                pb.inc(1);

                let result = process_file(path);
                let _ = tx.send(result);

                WalkState::Continue
            })
        });

    pb.finish_with_message("Done");
    drop(tx);

    rx.into_iter().collect()
}

/// Processes a single file and always returns a FileResult.
/// Errors are represented explicitly as zeroed results.
fn process_file(path: &Path) -> FileResult {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return error_result(path),
    };

    let mmap = match unsafe { Mmap::map(&file) } {
        Ok(m) => m,
        Err(_) => return error_result(path),
    };

    if is_binary(&mmap) {
        return binary_result(path, mmap.len() as u64);
    }

    let (code, comment, blank) = count_lines(&mmap);

    FileResult {
        path: path.to_path_buf(),
        lang: classify_extension(path),
        code,
        comment,
        blank,
        bytes: mmap.len() as u64,
    }
}

/// Binary heuristic: NUL byte detection.
fn is_binary(data: &[u8]) -> bool {
    data.iter().any(|&b| b == 0)
}

/// Result for binary files.
fn binary_result(path: &Path, bytes: u64) -> FileResult {
    FileResult {
        path: path.to_path_buf(),
        lang: classify_extension(path),
        code: 0,
        comment: 0,
        blank: 0,
        bytes,
    }
}

/// Result for unreadable or unmappable files.
fn error_result(path: &Path) -> FileResult {
    FileResult {
        path: path.to_path_buf(),
        lang: classify_extension(path),
        code: 0,
        comment: 0,
        blank: 0,
        bytes: 0,
    }
}

/// Fast, byte-level line counting.
/// This is a heuristic, not a language parser.
fn count_lines(data: &[u8]) -> (usize, usize, usize) {
    let mut code = 0;
    let mut comment = 0;
    let mut blank = 0;
    let mut in_block = false;

    for line in data.split(|&b| b == b'\n') {
        let first = line.iter().position(|&b| !b.is_ascii_whitespace());

        match first {
            None => {
                if in_block {
                    comment += 1;
                } else {
                    blank += 1;
                }
            }
            Some(pos) => {
                let rest = &line[pos..];

                if in_block {
                    comment += 1;
                    if rest.windows(2).any(|w| w == b"*/") {
                        in_block = false;
                    }
                } else if rest.starts_with(b"//") || rest.starts_with(b"#") {
                    comment += 1;
                } else if rest.starts_with(b"/*") {
                    comment += 1;
                    if !rest.windows(2).any(|w| w == b"*/") {
                        in_block = true;
                    }
                } else {
                    code += 1;
                }
            }
        }
    }

    (code, comment, blank)
}
