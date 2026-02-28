use crate::result::{FileResult, Lang};
use serde::Serialize;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Serialize)]
pub struct Report {
    pub totals: Totals,
    pub languages: Vec<LanguageStat>,
    pub files: Vec<FileStat>,
    pub elapsed_ms: u128,
}

#[derive(Serialize)]
pub struct Totals {
    pub code: usize,
    pub comment: usize,
    pub blank: usize,
    pub physical_lines: usize,
    pub bytes: u64,
}

#[derive(Serialize)]
pub struct LanguageStat {
    pub lang: Lang,
    pub code: usize,
    pub comment: usize,
    pub blank: usize,
    pub physical_lines: usize,
    pub bytes: u64,
}

#[derive(Serialize)]
pub struct FileStat {
    pub path: String,
    pub lang: Lang,
    pub code: usize,
    pub comment: usize,
    pub blank: usize,
    pub physical_lines: usize,
    pub bytes: u64,
}

pub fn build_report(
    results: &[FileResult],
    elapsed: Duration,
    top_n: usize,
) -> Report {
    let mut totals = Totals {
        code: 0,
        comment: 0,
        blank: 0,
        physical_lines: 0,
        bytes: 0,
    };

    let mut langs: HashMap<Lang, (usize, usize, usize, usize, u64)> = HashMap::new();

    for r in results {
        totals.code += r.code;
        totals.comment += r.comment;
        totals.blank += r.blank;
        totals.physical_lines += r.physical_lines;
        totals.bytes += r.bytes;

        let entry = langs.entry(r.lang.clone()).or_insert((0, 0, 0, 0, 0));
        entry.0 += r.code;
        entry.1 += r.comment;
        entry.2 += r.blank;
        entry.3 += r.physical_lines;
        entry.4 += r.bytes;
    }

    let mut languages: Vec<_> = langs
        .into_iter()
        .map(|(lang, (c, m, b, p, by))| LanguageStat {
            lang,
            code: c,
            comment: m,
            blank: b,
            physical_lines: p,
            bytes: by,
        })
        .collect();

    // Deterministic ordering:
    // 1. Most non-blank lines
    // 2. Language name (stable)
    languages.sort_unstable_by(|a, b| {
        let la = match &a.lang {
            Lang::Identified(s) => s.as_str(),
            Lang::NonUtf8 => "[Binary]",
            Lang::None => "None",
        };
        let lb = match &b.lang {
            Lang::Identified(s) => s.as_str(),
            Lang::NonUtf8 => "[Binary]",
            Lang::None => "None",
        };

        (b.code + b.comment)
            .cmp(&(a.code + a.comment))
            .then_with(|| la.cmp(lb))
    });

    let mut files: Vec<_> = results
        .iter()
        .map(|r| FileStat {
            path: r.path.display().to_string(),
            lang: r.lang.clone(),
            code: r.code,
            comment: r.comment,
            blank: r.blank,
            physical_lines: r.physical_lines,
            bytes: r.bytes,
        })
        .collect();

    // Deterministic ordering:
    // 1. Largest file size (bytes)
    // 2. Path (stable, human-meaningful)
    files.sort_unstable_by(|a, b| {
        b.bytes
            .cmp(&a.bytes)
            .then_with(|| a.path.cmp(&b.path))
    });

    files.truncate(top_n);

    Report {
        totals,
        languages,
        files,
        elapsed_ms: elapsed.as_millis(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn top_files_are_sorted_by_size_desc_then_path() {
        let results = vec![
            FileResult {
                path: PathBuf::from("b.rs"),
                lang: Lang::Identified("Rust".to_string()),
                code: 10,
                comment: 0,
                blank: 0,
                physical_lines: 10,
                bytes: 100,
            },
            FileResult {
                path: PathBuf::from("a.rs"),
                lang: Lang::Identified("Rust".to_string()),
                code: 1,
                comment: 0,
                blank: 0,
                physical_lines: 1,
                bytes: 100,
            },
            FileResult {
                path: PathBuf::from("c.rs"),
                lang: Lang::Identified("Rust".to_string()),
                code: 1,
                comment: 0,
                blank: 0,
                physical_lines: 1,
                bytes: 10,
            },
        ];

        let report = build_report(&results, Duration::from_millis(1), 3);
        let paths: Vec<_> = report.files.iter().map(|f| f.path.as_str()).collect();

        assert_eq!(paths, vec!["a.rs", "b.rs", "c.rs"]);
    }
}
