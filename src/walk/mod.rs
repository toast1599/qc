// src/walk/mod.rs

use crate::result::FileResult;
use crate::result::Lang;
use crossbeam_channel;
use ignore::{WalkBuilder, WalkState};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;

mod classify;
mod analyze;
mod io;

use classify::classify_file;
use analyze::{is_binary, count_lines};
use io::map_file;

pub fn parallel_scan(root: &str) -> Vec<FileResult> {
    let (tx, rx) = crossbeam_channel::unbounded(); 

    let pb = ProgressBar::new_spinner().with_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.blue} {msg} [{elapsed_precise}] {pos} files")
            .unwrap(),
    );
    pb.set_message("Auditing");

    WalkBuilder::new(root)
        .hidden(true)
        .git_ignore(true)
        .threads(num_cpus::get())
        .build_parallel()
        .run(|| {
            let tx = tx.clone();
            let pb = pb.clone();
            let mut local_count = 0; // NEW: Local counter for this thread

            Box::new(move |entry| {
                let Ok(entry) = entry else { return WalkState::Continue; };
                if !entry.file_type().map_or(false, |ft| ft.is_file()) {
                    return WalkState::Continue; 
                }

                // FIX: Only update the progress bar every 100 files
                local_count += 1;
                if local_count >= 100 {
                    pb.inc(100);
                    local_count = 0;
                }

                let result = process_file(entry.path());
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
    // 1. Get metadata first to check size without opening/mapping
    let metadata = match std::fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return error_result(path),
    };
    let bytes = metadata.len();

    if bytes == 0 {
        return FileResult {
            path: path.to_path_buf(),
            lang: classify_file(path, &[]),
            code: 0, comment: 0, blank: 0, bytes: 0,
        };
    }

    // 2. Hybrid I/O: mmap is slow for small files
    // Use an enum or separate variables to manage the lifetime of the data
    let (code, comment, blank, lang) = if bytes > 16 * 1024 {
        match map_file(path) {
            Some(mmap) => {
                if is_binary(&mmap) { return binary_result(path, bytes); }
                let l = classify_file(path, &mmap);
                let (co, cm, bl) = count_lines(&mmap, &l);
                (co, cm, bl, l)
            }
            None => return error_result(path),
        }
    } else {
        match std::fs::read(path) {
            Ok(buf) => {
                if is_binary(&buf) { return binary_result(path, bytes); }
                let l = classify_file(path, &buf);
                let (co, cm, bl) = count_lines(&buf, &l);
                (co, cm, bl, l)
            }
            Err(_) => return error_result(path),
        }
    };

    FileResult {
        path: path.to_path_buf(),
        lang,
        code,
        comment,
        blank,
        bytes,
    }
}

/// Result for binary files.
fn binary_result(path: &Path, bytes: u64) -> FileResult {
    FileResult {
        path: path.to_path_buf(),
        lang: Lang::NonUtf8,
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
        // FIX 2: Since we have no content on error, pass an empty slice
        lang: classify_file(path, &[]), 
        code: 0,
        comment: 0,
        blank: 0,
        bytes: 0,
    }
}
