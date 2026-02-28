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
    if content.starts_with(b"#!") && let Some(lang) = guess_shebang(content) {
        return lang;
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
    let mut parts = line.trim_start_matches("#!").split_whitespace();

    // Handle /usr/bin/env correctly:
    // #!/usr/bin/env python -O  -> python
    // #!/bin/bash               -> bash
    let interp = normalize_interp(parts.next()?)?;
    let interp = if interp == "env" {
        normalize_interp(parts.next()?)?
    } else {
        interp
    };

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

fn normalize_interp(token: &str) -> Option<String> {
    Path::new(token)
        .file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shebang_env_python_is_detected() {
        let detected = guess_shebang(b"#!/usr/bin/env python -O\nprint(1)\n");
        assert_eq!(detected, Some(Lang::Identified("Python".to_string())));
    }

    #[test]
    fn shebang_env_bash_is_detected() {
        let detected = guess_shebang(b"#!/usr/bin/env bash\nset -e\n");
        assert_eq!(detected, Some(Lang::Identified("Shell".to_string())));
    }

    #[test]
    fn shebang_direct_path_is_detected() {
        let detected = guess_shebang(b"#!/bin/bash\necho hi\n");
        assert_eq!(detected, Some(Lang::Identified("Shell".to_string())));
    }
}
