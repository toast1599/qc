use std::path::Path;
use crate::result::Lang;
use crate::assets::{FILENAME_LOOKUP, EXTENSION_LOOKUP};

pub fn classify_file(path: &Path, content: &[u8]) -> Lang {
    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    
    // Check filenames
    if let Some(lang_name) = FILENAME_LOOKUP.get(filename) {
        return Lang::Identified(lang_name.to_string());
    }

    // Check EXTENSIONS,
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let dotted_ext = format!(".{}", ext.to_lowercase());
        if let Some(lang_name) = EXTENSION_LOOKUP.get(&dotted_ext) {
            return Lang::Identified(lang_name.to_string());
        }
    }
    // 3. Shebang fallback for extensionless files
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
    if s.contains("python") { return Some("Python".into()); }
    if s.contains("sh") { return Some("Shell".into()); }
    None
}
