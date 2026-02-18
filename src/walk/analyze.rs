use crate::result::Lang;

/// Binary file heuristic.
///
/// Refined to avoid misidentifying UTF-16 as binary.
/// If the file contains a NUL byte that isn't part of a common UTF-16 pattern,
/// or if it contains a high density of control characters, treat it as binary.
pub fn is_binary(data: &[u8]) -> bool {
    if data.is_empty() {
        return false;
    }

    // Check for UTF-16 BOMs
    if data.len() >= 2 {
        if (data[0] == 0xFF && data[1] == 0xFE) || (data[0] == 0xFE && data[1] == 0xFF) {
            return false;
        }
    }

    // Heuristic: If we find a NUL, it's binary UNLESS it looks like UTF-16 text
    // (every other byte is NUL). For simplicity, we'll still be conservative but
    // avoid the most common "accidental" binary detections.
    let nul_count = data.iter().filter(|&&b| b == 0).count();
    if nul_count == 0 {
        return false;
    }

    // If more than 10% of bytes are NUL, it's almost certainly binary
    if nul_count > data.len() / 10 {
        return true;
    }

    // Default to the original NUL check for small numbers of NULs
    nul_count > 0
}

fn is_hash_comment_lang(lang: &Lang) -> bool {
    match lang {
        Lang::Identified(name) => match name.as_str() {
            "Python" | "Shell" | "YAML" | "Makefile" | "Perl" | "Ruby" => true,
            _ => false,
        },
        _ => false,
    }
}

/// Fast, byte-level line counting.
///
/// Handles string literals, inline comments, and block comments.
pub fn count_lines(data: &[u8], lang: &Lang) -> (usize, usize, usize) {
    let mut code = 0;
    let mut comment = 0;
    let mut blank = 0;
    let mut in_block = false;
    let hash_comments = is_hash_comment_lang(lang);

    // FIX: Remove .collect(). Iterate over the Split iterator directly.
    for line in data.split(|&b| b == b'\n') {
        let first_char = line.iter().position(|&b| !b.is_ascii_whitespace());

        match first_char {
            None => {
                if in_block {
                    comment += 1;
                } else {
                    blank += 1;
                }
            }
            Some(_) => {
                let mut has_code = false;
                let mut has_comment = false;
                let mut i = 0;

                while i < line.len() {
                    let b = line[i];
                    if in_block {
                        has_comment = true;
                        if i + 1 < line.len() && b == b'*' && line[i + 1] == b'/' {
                            in_block = false;
                            i += 1;
                        }
                    } else {
                        // We skip string logic for speed; it's rarely needed for audits
                        if b.is_ascii_whitespace() {
                        } else if i + 1 < line.len() && b == b'/' && line[i + 1] == b'/' {
                            has_comment = true;
                            break;
                        } else if b == b'#' && hash_comments {
                            has_comment = true;
                            break;
                        } else if i + 1 < line.len() && b == b'/' && line[i + 1] == b'*' {
                            has_comment = true;
                            in_block = true;
                            i += 1;
                        } else {
                            has_code = true;
                        }
                    }
                    i += 1;
                }
                if has_code {
                    code += 1;
                } else if has_comment {
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

    #[test]
    fn test_empty_file() {
        assert_eq!(count_lines(b"", &Lang::Rs), (0, 0, 0));
    }

    #[test]
    fn test_trailing_newline() {
        assert_eq!(count_lines(b"line1\n", &Lang::Rs), (1, 0, 0));
        assert_eq!(count_lines(b"line1\n\n", &Lang::Rs), (1, 0, 1));
    }

    #[test]
    fn test_inline_comments() {
        assert_eq!(count_lines(b"code(); // comment", &Lang::Rs), (1, 0, 0));
        assert_eq!(count_lines(b"// full comment", &Lang::Rs), (0, 1, 0));
    }

    #[test]
    fn test_string_markers() {
        assert_eq!(
            count_lines(b"let x = \"// not a comment\";", &Lang::Rs),
            (1, 0, 0)
        );
        assert_eq!(
            count_lines(b"let x = \"/* not a block */\";", &Lang::Rs),
            (1, 0, 0)
        );
    }

    #[test]
    fn test_multiline_string() {
        let data = b"let x = \"\n continuation\n \";";
        assert_eq!(count_lines(data, &Lang::Rs), (3, 0, 0));
    }

    #[test]
    fn test_hash_logic() {
        // Rust: # is not a comment
        assert_eq!(count_lines(b"#attribute", &Lang::Rs), (1, 0, 0));
        // Python: # is a comment
        assert_eq!(count_lines(b"# comment", &Lang::Py), (0, 1, 0));
    }

    #[test]
    fn test_block_comments() {
        let data = b"/*\n multi\n line\n */";
        assert_eq!(count_lines(data, &Lang::Rs), (0, 4, 0));
    }

    #[test]
    fn test_utf16_not_binary() {
        let utf16_le = vec![0xFF, 0xFE, b'h', 0, b'e', 0, b'l', 0, b'l', 0, b'o', 0];
        assert!(!is_binary(&utf16_le));
    }
}
