// src/walk/classify.rs
use crate::result::Lang;
use std::path::Path;

/// Classify a file path into a language category.
pub fn classify_extension(path: &Path) -> Lang {
    let Some(ext) = path.extension() else {
        return Lang::None;
    };

    let Some(ext) = ext.to_str() else {
        return Lang::NonUtf8;
    };

    match ext.to_ascii_lowercase().as_str() {
        "c" => Lang::C,
        "h" => Lang::H,
        "rs" => Lang::Rs,
        "py" => Lang::Py,
        "sh" => Lang::Sh,
        "json" => Lang::Json,
        "yaml" | "yml" => Lang::Yaml,
        "md" | "rst" => Lang::Doc,
        other => Lang::Other(other.to_string()),
    }
}
