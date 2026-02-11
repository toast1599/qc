// src/result.rs
use std::path::PathBuf;

pub struct FileResult {
    pub path: PathBuf,
    pub extension: String,
    pub code: usize,
    pub comment: usize,
    pub blank: usize,
    pub bytes: u64,
}
