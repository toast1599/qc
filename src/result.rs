// src/result.rs
use std::path::PathBuf;

/// Language classification for files.

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Lang {
    /// We keep these special cases
    None,
    NonUtf8,
    /// This now covers EVERYTHING in the YAML
    Identified(String), 
}
pub struct FileResult {
    pub path: PathBuf,
    pub lang: Lang,
    pub code: usize,
    pub comment: usize,
    pub blank: usize,
    pub bytes: u64,
}
