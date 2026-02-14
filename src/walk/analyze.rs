// src/walk/analyze.rs

/// Binary file heuristic.
/// If the file contains a NUL byte, treat it as binary.
#[inline]
pub fn is_binary(data: &[u8]) -> bool {
    data.iter().any(|&b| b == 0)
}

/// Fast, byte-level line counting.
/// This is a heuristic and not a language-aware parser.
///
/// Returns (code, comment, blank).
pub fn count_lines(data: &[u8]) -> (usize, usize, usize) {
    let mut code = 0;
    let mut comment = 0;
    let mut blank = 0;
    let mut in_block = false;

    for line in data.split(|&b| b == b'\n') {
        let first = line.iter().position(|&b| !b.is_ascii_whitespace());

        match first {
            None => {
                if in_block {
                    comment += 1;
                } else {
                    blank += 1;
                }
            }
            Some(pos) => {
                let rest = &line[pos..];

                if in_block {
                    comment += 1;
                    if rest.windows(2).any(|w| w == b"*/") {
                        in_block = false;
                    }
                } else if rest.starts_with(b"//") || rest.starts_with(b"#") {
                    comment += 1;
                } else if rest.starts_with(b"/*") {
                    comment += 1;
                    if !rest.windows(2).any(|w| w == b"*/") {
                        in_block = true;
                    }
                } else {
                    code += 1;
                }
            }
        }
    }

    (code, comment, blank)
}
