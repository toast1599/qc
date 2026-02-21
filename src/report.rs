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
    pub bytes: u64,
}

#[derive(Serialize)]
pub struct LanguageStat {
    pub lang: Lang,
    pub code: usize,
    pub comment: usize,
    pub blank: usize,
    pub bytes: u64,
}

#[derive(Serialize)]
pub struct FileStat {
    pub path: String,
    pub lang: Lang,
    pub code: usize,
    pub comment: usize,
    pub blank: usize,
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
        bytes: 0,
    };

    let mut langs: HashMap<Lang, (usize, usize, usize, u64)> = HashMap::new();

    for r in results {
        totals.code += r.code;
        totals.comment += r.comment;
        totals.blank += r.blank;
        totals.bytes += r.bytes;

        let entry = langs.entry(r.lang.clone()).or_insert((0, 0, 0, 0));
        entry.0 += r.code;
        entry.1 += r.comment;
        entry.2 += r.blank;
        entry.3 += r.bytes;
    }

    let mut languages: Vec<_> = langs
        .into_iter()
        .map(|(lang, (c, m, b, by))| LanguageStat {
            lang,
            code: c,
            comment: m,
            blank: b,
            bytes: by,
        })
        .collect();

    languages.sort_unstable_by_key(|l| std::cmp::Reverse(l.code + l.comment));

    let mut files: Vec<_> = results
        .iter()
        .map(|r| FileStat {
            path: r.path.display().to_string(),
            lang: r.lang.clone(),
            code: r.code,
            comment: r.comment,
            blank: r.blank,
            bytes: r.bytes,
        })
        .collect();

    files.sort_unstable_by_key(|f| std::cmp::Reverse(f.code + f.comment));
    files.truncate(top_n);

    Report {
        totals,
        languages,
        files,
        elapsed_ms: elapsed.as_millis(),
    }
}
