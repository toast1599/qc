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
    let (tx, rx) = crossbeam_channel::bounded(num_cpus::get() * 64);

    let pb = ProgressBar::new_spinner().with_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.blue} {msg} [{elapsed_precise}] {pos} files")
            .unwrap(),
    );
    pb.set_message("Auditing");

    // --- THE FIX IS HERE ---
    WalkBuilder::new(root)
        .hidden(true)       // This skips .git, .gitignore, and hidden system files
        .git_ignore(true)    // This skips anything listed in your .gitignore (like /target)
        .git_global(true)    // Optional: respects your global git settings
        .threads(num_cpus::get())
        .build_parallel()
        .run(|| {
            let tx = tx.clone();
            let pb = pb.clone();

            Box::new(move |entry| {
                let Ok(entry) = entry else { return WalkState::Continue; };
                
                // Ensure we only look at files, not directory entries
                if !entry.file_type().map_or(false, |ft| ft.is_file()) {
                    return WalkState::Continue; 
                }

                let path = entry.path();
                let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
                
                // Skip the lockfile as requested
                if name == "Cargo.lock" || name == ".DS_Store" {
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
    let mmap = match map_file(path) {
        Some(m) => m,
        None => return error_result(path),
    };

    let bytes = mmap.len() as u64;

    if bytes == 0 {
        return FileResult {
            path: path.to_path_buf(),
            lang: classify_file(path, &[]),
            code: 0,
            comment: 0,
            blank: 0,
            bytes: 0,
        };
    }

    if is_binary(&mmap) {
        return binary_result(path, bytes);
    }

    let lang = classify_file(path, &mmap); 
    let (code, comment, blank) = count_lines(&mmap, &lang);
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
