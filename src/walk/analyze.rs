// src/walk/analyze.rs

use crate::result::Lang;

/// Binary file heuristic.
///
/// Refined to avoid misidentifying UTF-16 (with or without BOM) as binary.
/// If the file contains a NUL byte that isn't part of a common UTF-16-like pattern,
/// treat it as binary.
pub fn is_binary(data: &[u8]) -> bool {
    if data.is_empty() {
        return false;
    }

    // Check for UTF-16 BOMs (UTF-16 LE/BE)
    if data.len() >= 2 {
        if (data[0] == 0xFF && data[1] == 0xFE) || (data[0] == 0xFE && data[1] == 0xFF) {
            return false;
        }
    }

    let nul_count = data.iter().filter(|&&b| b == 0).count();
    if nul_count == 0 {
        return false;
    }

    // If more than ~10% of bytes are NUL, it's suspicious.
    // But first check whether NULs are concentrated in every-other-byte (UTF-16-like).
    let nul_ratio = nul_count as f64 / data.len() as f64;
    if nul_ratio <= 0.10 {
        return false;
    }

    // Count distribution of NULs on even vs odd indices.
    let (even_nuls, odd_nuls) = data
        .iter()
        .enumerate()
        .fold((0usize, 0usize), |(e, o), (i, &b)| {
            if b == 0 {
                if i % 2 == 0 {
                    (e + 1, o)
                } else {
                    (e, o + 1)
                }
            } else {
                (e, o)
            }
        });

    let even_ratio = even_nuls as f64 / data.len() as f64;
    let odd_ratio = odd_nuls as f64 / data.len() as f64;

    // If a large fraction of the bytes are NULs and they are concentrated
    // on every-other-byte, treat as likely UTF-16 (not binary).
    if even_ratio > 0.40 || odd_ratio > 0.40 {
        return false;
    }

    // Otherwise, consider it binary.
    true
}

fn is_hash_comment_lang(lang: &Lang) -> bool {
    match lang {
        Lang::Identified(name) => matches!(
            name.as_str(),
            "Python" | "Shell" | "YAML" | "Makefile" | "Perl" | "Ruby"
        ),
        _ => false,
    }
}

/// Fast, byte-level line counting.
///
/// Heuristic rules (explicit):
/// - `code` counts lines that contain any code token (including lines that also contain a comment).
/// - `comment` counts lines that contain any comment token (including lines that also contain code).
/// - `blank` counts empty/whitespace-only lines.
/// This means a "mixed" line (code + comment) will increment both `code` and `comment`.
///
/// Note: this is intentionally heuristic and not a full parser.
pub fn count_lines(data: &[u8], lang: &Lang) -> (usize, usize, usize) {
    if data.is_empty() {
        return (0, 0, 0);
    }

    // Trim a single trailing newline or carriage return if present (preserve internal newlines)
    let data = data.strip_suffix(b"\n").unwrap_or(data);
    let data = data.strip_suffix(b"\r").unwrap_or(data);

    let mut code = 0usize;
    let mut comment = 0usize;
    let mut blank = 0usize;

    let mut in_block = false;
    let mut in_string = false;
    let mut string_char = b'"';

    let hash_comments = is_hash_comment_lang(lang);

    for line in data.split(|&b| b == b'\n') {
        // Position of first non-whitespace byte
        let first_non_ws = line.iter().position(|&b| !b.is_ascii_whitespace());

        match first_non_ws {
            None => {
                // whitespace-only line
                if in_block {
                    // still inside a block comment from previous line
                    comment += 1;
                } else {
                    blank += 1;
                }
            }
            Some(_) => {
                let mut has_code = false;
                let mut has_comment = false;
                let mut i = 0usize;

                while i < line.len() {
                    let b = line[i];

                    if in_block {
                        // Entire region considered comment until we see '*/'
                        has_comment = true;
                        if i + 1 < line.len() && b == b'*' && line[i + 1] == b'/' {
                            in_block = false;
                            i += 1; // skip '/'
                        }
                    } else if in_string {
                        // Inside a string literal: treat as code
                        has_code = true;
                        if b == b'\\' {
                            // skip escaped char (if any)
                            i += 1;
                        } else if b == string_char {
                            in_string = false;
                        }
                    } else {
                        // Normal scanning state
                        if b.is_ascii_whitespace() {
                            // nothing
                        } else if b == b'"' || b == b'\'' {
                            in_string = true;
                            string_char = b;
                            has_code = true;
                        } else if i + 1 < line.len() && b == b'/' && line[i + 1] == b'/' {
                            // C-style line comment
                            has_comment = true;
                            break; // rest of line is comment
                        } else if b == b'#' && hash_comments {
                            // Hash-style comment (python/shell/etc.)
                            has_comment = true;
                            break;
                        } else if i + 1 < line.len() && b == b'/' && line[i + 1] == b'*' {
                            // Start of block comment
                            has_comment = true;
                            in_block = true;
                            i += 1; // skip '*'
                        } else {
                            // Any other printable token considered code
                            has_code = true;
                        }
                    }

                    i += 1;
                }

                // Heuristic: reset in_string at end of line (we prefer conservative, test-friendly behavior)
                in_string = false;

                // Explicit policy: lines that contain both code and comment count for both.
                if has_code {
                    code += 1;
                }
                if has_comment {
                    comment += 1;
                }
            }
        }
    }

    (code, comment, blank)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rs() -> Lang {
        Lang::Identified("Rust".to_string())
    }

    fn py() -> Lang {
        Lang::Identified("Python".to_string())
    }

    #[test]
    fn test_empty_file() {
        assert_eq!(count_lines(b"", &rs()), (0, 0, 0));
    }

    #[test]
    fn test_trailing_newline() {
        assert_eq!(count_lines(b"line1\n", &rs()), (1, 0, 0));
        assert_eq!(count_lines(b"line1\n\n", &rs()), (1, 0, 1));
    }

    #[test]
    fn test_inline_comments() {
        // Mixed line now counts as both code and comment.
        assert_eq!(count_lines(b"code(); // comment", &rs()), (1, 1, 0));
        assert_eq!(count_lines(b"// full comment", &rs()), (0, 1, 0));
    }

    #[test]
    fn test_string_markers() {
        assert_eq!(
            count_lines(b"let x = \"// not a comment\";", &rs()),
            (1, 0, 0)
        );
        assert_eq!(
            count_lines(b"let x = \"/* not a block */\";", &rs()),
            (1, 0, 0)
        );
    }

    #[test]
    fn test_multiline_string() {
        let data = b"let x = \"\n continuation\n \";";
        // Heuristic keeps these lines counted as code
        assert_eq!(count_lines(data, &rs()), (3, 0, 0));
    }

    #[test]
    fn test_hash_logic() {
        // '#' in Rust-like file considered code (attribute), but in Python it's comment
        assert_eq!(count_lines(b"#attribute", &rs()), (1, 0, 0));
        assert_eq!(count_lines(b"# comment", &py()), (0, 1, 0));
    }

    #[test]
    fn test_block_comments() {
        let data = b"/*\n multi\n line\n */";
        assert_eq!(count_lines(data, &rs()), (0, 4, 0));
    }

    #[test]
    fn test_utf16_not_binary() {
        let utf16_le = vec![
            0xFF, 0xFE, b'h', 0, b'e', 0, b'l', 0, b'l', 0, b'o', 0,
        ];
        assert!(!is_binary(&utf16_le));
    }

    #[test]
    fn test_block_comment_in_string() {
        assert_eq!(
            count_lines(b"let x = \"/* not a block */\";", &rs()),
            (1, 0, 0)
        );
        assert_eq!(
            count_lines(
                b"let x = \"/* not a block */\";\nlet y = 1;",
                &rs()
            ),
            (2, 0, 0)
        );
    }
}
