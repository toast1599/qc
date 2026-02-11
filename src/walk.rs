// src/walk.rs
use crate::result::FileResult;
use crossbeam_channel::unbounded;
use ignore::{WalkBuilder, WalkState};
use indicatif::{ProgressBar, ProgressStyle};
use memmap2::Mmap;
use std::{fs::File, path::Path};

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
            let (tx, pb) = (tx.clone(), pb.clone());

            Box::new(move |entry| {
                let Ok(entry) = entry else {
                    return WalkState::Continue;
                };

                let path = entry.path();
                if !path.is_file() {
                    return WalkState::Continue;
                }

                pb.inc(1);

                match process_file(path) {
                    Ok(result) => {
                        let _ = tx.send(result);
                    }
                    Err(result) => {
                        let _ = tx.send(result);
                    }
                }

                WalkState::Continue
            })
        });

    pb.finish_with_message("Done");
    drop(tx);
    rx.into_iter().collect()
}

/// Processes a single file and always returns a FileResult.
/// Errors are surfaced, not silently ignored.
fn process_file(path: &Path) -> Result<FileResult, FileResult> {
    let file = File::open(path).map_err(|_| error_result(path))?;

    let mmap = unsafe { Mmap::map(&file) }.map_err(|_| error_result(path))?;

    // Heuristic: if file contains NUL bytes, treat as binary
    if mmap.iter().any(|&b| b == 0) {
        return Ok(binary_result(path, mmap.len() as u64));
    }

    let (code, comment, blank) = count_lines(&mmap);

    Ok(FileResult {
        path: path.to_path_buf(),
        extension: classify_extension(path),
        code,
        comment,
        blank,
        bytes: mmap.len() as u64,
    })
}

/// Explicit, lossless extension classification.
fn classify_extension(path: &Path) -> String {
    match path.extension() {
        None => "none".into(),
        Some(ext) => match ext.to_str() {
            Some(s) => s.to_lowercase(),
            None => "non-utf8".into(),
        },
    }
}

/// Result for binary files: counted, but not misclassified.
fn binary_result(path: &Path, bytes: u64) -> FileResult {
    FileResult {
        path: path.to_path_buf(),
        extension: classify_extension(path),
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
        extension: classify_extension(path),
        code: 0,
        comment: 0,
        blank: 0,
        bytes: 0,
    }
}

/// Fast, byte-level line counting.
/// NOTE: This is a heuristic and not a language parser.
/// Comment detection is approximate by design.
fn count_lines(data: &[u8]) -> (usize, usize, usize) {
    let (mut code, mut comment, mut blank, mut in_block) = (0, 0, 0, false);

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
