// src/walk/mod.rs

use crate::result::{FileResult, Lang};
use crossbeam_channel;
use ignore::{WalkBuilder, WalkState};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::Path;

mod analyze;
mod classify;
mod io;

use analyze::{count_lines, is_binary};
use classify::classify_file;
use io::map_file;

pub fn parallel_scan(root: &str) -> Vec<FileResult> {
    // Bounded channel to avoid unbounded memory growth
    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

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
        .threads(threads)
        .build_parallel()
        .run(|| {
            let tx = tx.clone();
            let pb = pb.clone();
            let mut local_count = 0usize;

            Box::new(move |entry| {
                let Ok(entry) = entry else {
                    return WalkState::Continue;
                };

                if !entry.file_type().map_or(false, |ft| ft.is_file()) {
                    return WalkState::Continue;
                }

                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);

                local_count += 1;
                if local_count >= 500 {
                    pb.inc(local_count as u64);
                    local_count = 0;
                }

                let result = process_file(entry.path(), size);
                let _ = tx.send(result);

                WalkState::Continue
            })
        });

    // Flush any remaining per-thread progress
    pb.finish_with_message("Done");

    drop(tx);
    rx.into_iter().collect()
}

fn process_file(path: &Path, bytes: u64) -> FileResult {
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

    if bytes > 16 * 1024 {
        if let Some(mmap) = map_file(path) {
            return analyze_data(path, &mmap, bytes);
        }
    }

    if let Ok(buf) = fs::read(path) {
        return analyze_data(path, &buf, bytes);
    }

    error_result(path, bytes)
}

fn analyze_data(path: &Path, data: &[u8], bytes: u64) -> FileResult {
    if is_binary(data) {
        return binary_result(path, bytes);
    }

    let lang = classify_file(path, data);
    let (code, comment, blank) = count_lines(data, &lang);

    FileResult {
        path: path.to_path_buf(),
        lang,
        code,
        comment,
        blank,
        bytes,
    }
}

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

fn error_result(path: &Path, bytes: u64) -> FileResult {
    FileResult {
        path: path.to_path_buf(),
        lang: Lang::None, // unreadable stays "unknown"
        code: 0,
        comment: 0,
        blank: 0,
        bytes,
    }
}
