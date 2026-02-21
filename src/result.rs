use serde::Serialize;
use std::path::PathBuf;

/// Language classification for files.
///
/// `Identified` owns its name to avoid lifetime lies,
/// pointer-identity bugs, and accidental UB-by-design.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "name")]
pub enum Lang {
    None,
    NonUtf8,
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
