// src/walk/classify.rs

use crate::assets::{EXTENSION_LOOKUP, FILENAME_LOOKUP};
use crate::result::Lang;
use std::path::Path;

pub fn classify_file(path: &Path, content: &[u8]) -> Lang {
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    // 1. Exact filename match (e.g. Makefile)
    if let Some(lang_name) = FILENAME_LOOKUP.get(filename) {
        return Lang::Identified(lang_name.clone());
    }

    // 2. Shebang-based detection (author intent beats extension)
    if content.starts_with(b"#!") {
        if let Some(lang) = guess_shebang(content) {
            return lang;
        }
    }

    // 3. Extension-based match (case-insensitive)
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        let ext = ext.to_ascii_lowercase();
        if let Some(lang_name) = EXTENSION_LOOKUP.get(&ext) {
            return Lang::Identified((*lang_name).to_string());
        }
    }

    Lang::None
}

fn guess_shebang(content: &[u8]) -> Option<Lang> {
    let line = content.split(|&b| b == b'\n').next()?;
    let line = String::from_utf8_lossy(line);

    // Strip "#!" and split
    let mut parts = line
        .trim_start_matches("#!")
        .trim()
        .split_whitespace();

    // Handle /usr/bin/env correctly:
    // #!/usr/bin/env python -O  -> python
    // #!/bin/bash               -> bash
    let interp = match parts.next()? {
        "env" => parts.next()?,
        other => other,
    };

    let interp = Path::new(interp)
        .file_name()
        .and_then(|s| s.to_str())?
        .to_ascii_lowercase();

    match interp.as_str() {
        "python" | "python3" | "python2" => {
            Some(Lang::Identified("Python".to_string()))
        }
        "sh" | "bash" | "zsh" | "dash" => {
            Some(Lang::Identified("Shell".to_string()))
        }
        _ => None,
    }
}
