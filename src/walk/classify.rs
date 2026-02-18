use crate::assets::{EXTENSION_LOOKUP, FILENAME_LOOKUP};
use crate::result::Lang;
use std::path::Path;

pub fn classify_file(path: &Path, content: &[u8]) -> Lang {
    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    if let Some(lang_name) = FILENAME_LOOKUP.get(filename) {
        return Lang::Identified(lang_name.to_string());
    }

    // FIX: Remove format! and to_lowercase()
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        // We now lookup using the raw extension string.
        // We lowercase it once, but only if necessary.
        if let Some(lang_name) = EXTENSION_LOOKUP.get(&ext.to_lowercase()) {
            return Lang::Identified(lang_name.to_string());
        }
    }

    if content.starts_with(b"#!") {
        if let Some(lang_name) = guess_shebang(content) {
            return Lang::Identified(lang_name);
        }
    }

    Lang::None
}

fn guess_shebang(content: &[u8]) -> Option<String> {
    let line = content.split(|&b| b == b'\n').next()?;
    let s = String::from_utf8_lossy(line);
    if s.contains("python") {
        return Some("Python".into());
    }
    if s.contains("sh") {
        return Some("Shell".into());
    }
    None
}
