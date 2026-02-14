// src/result.rs
use std::path::PathBuf;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Lang {
    C,
    H,
    Rs,
    Py,
    Sh,
    Json,
    Yaml,
    Doc,
    None,
    NonUtf8,
    Other(String),
}

pub struct FileResult {
    pub path: PathBuf,
    pub lang: Lang,
    pub code: usize,
    pub comment: usize,
    pub blank: usize,
    pub bytes: u64,
}
