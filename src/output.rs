// src/output.rs
use crate::result::{FileResult, Lang};
use std::{collections::HashMap, time::Duration};

/// Maps Lang to (Color, Label)
fn get_color(lang: &Lang) -> (&'static str, &'static str) {
    match lang {
        Lang::C => ("\x1b[34m", "C"),
        Lang::H => ("\x1b[36m", "H"),
        Lang::Rs => ("\x1b[38;5;208m", "RS"),
        Lang::Py => ("\x1b[33m", "PY"),
        Lang::Sh => ("\x1b[32m", "SH"),
        Lang::Json => ("\x1b[35m", "JSON"),
        Lang::Yaml => ("\x1b[35m", "YAML"),
        Lang::Doc => ("\x1b[37m", "DOC"),
        Lang::None => ("\x1b[90m", "NONE"),
        Lang::NonUtf8 => ("\x1b[90m", "BIN"),
        Lang::Other(_) => ("\x1b[90m", "OTHER"),
    }
}

pub fn format_size(b: u64) -> String {
    let (v, u) = if b >= 1073741824 {
        (b as f64 / 1073741824.0, "GB")
    } else if b >= 1048576 {
        (b as f64 / 1048576.0, "MB")
    } else if b >= 1024 {
        (b as f64 / 1024.0, "KB")
    } else {
        (b as f64, "B")
    };
    format!("{:.2} {}", v, u)
}

pub fn print_results(results: &mut [FileResult], top_n: usize, elapsed: Duration) {
    if results.is_empty() {
        return;
    }

    let (t_code, t_comm, t_blnk) = results.iter().fold(
        (0, 0, 0),
        |a, r| (a.0 + r.code, a.1 + r.comment, a.2 + r.blank),
    );

    let mut langs: HashMap<&Lang, (usize, usize, usize, u64)> = HashMap::new();
    for r in results.iter() {
        let e = langs.entry(&r.lang).or_insert((0, 0, 0, 0));
        *e = (e.0 + r.code, e.1 + r.comment, e.2 + r.blank, e.3 + r.bytes);
    }

    let mut stats: Vec<_> = langs.into_iter().collect();
    stats.sort_unstable_by_key(|s| std::cmp::Reverse(s.1.0 + s.1.1));

    let (blu, cyn, bld, rst) = ("\x1b[34;1m", "\x1b[36;1m", "\x1b[1m", "\x1b[0m");

    println!("\n{}--- 📊 AUDIT RESULTS ---{}", blu, rst);
    println!(
        "Total Lines: {} (Code: {}, Comm: {}, Blank: {})\nElapsed: {:?}",
        t_code + t_comm + t_blnk,
        t_code,
        t_comm,
        t_blnk,
        elapsed
    );

    println!("\n{}--- 📚 LANGUAGE BREAKDOWN ---{}", cyn, rst);
    println!(
        "{:<12} | {:>10} | {:>8} | {:>8} | {:>10}\n{}",
        "LANG",
        "CODE",
        "COMM",
        "BLANK",
        "SIZE",
        "-".repeat(60)
    );

    for (lang, (co, cm, bl, by)) in stats.iter().take(15) {
        let (clr, lbl) = get_color(lang);
        println!(
            "{}{:<12}{} | {:>10} | {:>8} | {:>8} | {:>10}",
            clr,
            lbl,
            rst,
            co,
            cm,
            bl,
            format_size(*by)
        );
    }

    results.sort_unstable_by_key(|r| std::cmp::Reverse(r.code + r.comment));
    println!(
        "\n{}--- 🏆 TOP {} LARGEST FILES ---{}\n{}{:>12} | {:>10} | {}{}\n{}",
        blu,
        top_n,
        rst,
        bld,
        "LINES",
        "SIZE",
        "PATH",
        rst,
        "-".repeat(80)
    );

    for r in results.iter().take(top_n) {
        let (clr, _) = get_color(&r.lang);
        println!(
            "{:>12} | {:>10} | {}{}{}",
            r.code + r.comment,
            format_size(r.bytes),
            clr,
            r.path.display(),
            rst
        );
    }

    print_heatmap(t_code, t_comm, t_blnk);
}

fn print_heatmap(code: usize, comm: usize, blnk: usize) {
    let total = code + comm + blnk;
    if total == 0 {
        println!("No data collected.");
        return;
    }

    let total = total as f64;
    let width: usize = 60;

    let c_w = ((code as f64 / total) * width as f64).round() as usize;
    let m_w = ((comm as f64 / total) * width as f64).round() as usize;
    let b_w = width.saturating_sub(c_w + m_w);

    println!(
        "\n\x1b[35;1m--- 📊 COMPOSITION --- \x1b[0m\n  [\x1b[32m{}\x1b[33m{}\x1b[37m{}\x1b[0m]",
        "█".repeat(c_w),
        "█".repeat(m_w),
        "█".repeat(b_w)
    );
    println!(
        "  \x1b[32m■\x1b[0m Code ({:.1}%)  \x1b[33m■\x1b[0m Comm ({:.1}%)  \x1b[37m■\x1b[0m Blank ({:.1}%)",
        (code as f64 / total) * 100.0,
        (comm as f64 / total) * 100.0,
        (blnk as f64 / total) * 100.0
    );
}
