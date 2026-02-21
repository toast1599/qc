// src/output/text.rs
use crate::assets::LANG_MAP;
use crate::result::{FileResult, Lang};
use std::collections::HashMap;
use std::time::Duration;

fn get_color_and_label(lang: &Lang) -> (String, String) {
    match lang {
        Lang::Identified(name) => {
            let mut color_code = "\x1b[38;5;250m".to_string(); // Default grey

            if let Some(data) = LANG_MAP.get(name) {
                if let Some(hex) = &data.color {
                    color_code = hex_to_ansi(hex);
                }
            }

            (color_code, name.clone())
        }
        Lang::None => ("\x1b[90m".to_string(), "None".to_string()),
        Lang::NonUtf8 => ("\x1b[90m".to_string(), "[Binary]".to_string()),
    }
}

/// Hex to ANSI truecolor converter
fn hex_to_ansi(hex: &str) -> String {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return "\x1b[37m".to_string();
    }

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);

    format!("\x1b[38;2;{};{};{}m", r, g, b)
}

pub fn format_size(bytes: u64) -> String {
    let (v, u) = if bytes >= 1 << 30 {
        (bytes as f64 / (1 << 30) as f64, "GB")
    } else if bytes >= 1 << 20 {
        (bytes as f64 / (1 << 20) as f64, "MB")
    } else if bytes >= 1 << 10 {
        (bytes as f64 / (1 << 10) as f64, "KB")
    } else {
        (bytes as f64, "B")
    };

    format!("{:.2} {}", v, u)
}

pub fn print_results(results: &mut [FileResult], top_n: usize, elapsed: Duration) {
    if results.is_empty() {
        return;
    }

    let (t_code, t_comm, t_blnk) = results.iter().fold((0, 0, 0), |a, r| {
        (a.0 + r.code, a.1 + r.comment, a.2 + r.blank)
    });

    let mut langs: HashMap<Lang, (usize, usize, usize, u64)> = HashMap::new();
    for r in results.iter() {
        let entry = langs.entry(r.lang.clone()).or_insert((0, 0, 0, 0));
        entry.0 += r.code;
        entry.1 += r.comment;
        entry.2 += r.blank;
        entry.3 += r.bytes;
    }

    let mut stats: Vec<_> = langs.into_iter().collect();
    stats.sort_unstable_by_key(|(_, v)| std::cmp::Reverse(v.0 + v.1));

    let (blu, cyn, bld, rst) = ("\x1b[34;1m", "\x1b[36;1m", "\x1b[1m", "\x1b[0m");

    println!("\n{}--- 📊 AUDIT RESULTS ---{}", blu, rst);
    println!(
        "Total Lines: {} (Code: {}, Comm*: {}, Blank: {})\nElapsed: {:?}",
        t_code + t_comm + t_blnk,
        t_code,
        t_comm,
        t_blnk,
        elapsed
    );

    println!("{}--- 📚 LANGUAGE BREAKDOWN ---{}", cyn, rst);
    println!(
        "{:<12} | {:>10} | {:>8} | {:>8} | {:>10}\n{}",
        "LANG",
        "CODE",
        "COMM*",
        "BLANK",
        "SIZE",
        "-".repeat(60)
    );

    for (lang, (co, cm, bl, by)) in stats.iter().take(15) {
        let (clr, lbl) = get_color_and_label(lang);
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
        "\n{}--- 🏆 TOP {} LARGEST FILES ---{}\n{}{:>12} | {:>10} | PATH{}\n{}",
        blu,
        top_n,
        rst,
        bld,
        "LINES",
        "SIZE",
        rst,
        "-".repeat(80)
    );

    for r in results.iter().take(top_n) {
        let (clr, _) = get_color_and_label(&r.lang);
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
    print_language_composition(&stats);
}

pub fn print_language_composition(stats: &[(Lang, (usize, usize, usize, u64))]) {
    let filtered: Vec<_> = stats
        .iter()
        .filter(|(lang, _)| !matches!(lang, Lang::None | Lang::NonUtf8))
        .collect();

    let total_bytes: u64 = filtered.iter().map(|(_, v)| v.3).sum();
    if total_bytes == 0 {
        return;
    }

    let width = 60;
    println!("\n\x1b[1m--- 📊 LANGUAGE COMPOSITION ---\x1b[0m");
    print!("  [");

    let mut used = 0;
    for (lang, v) in filtered.iter().take(8) {
        let (clr, _) = get_color_and_label(lang);
        let w = ((v.3 as f64 / total_bytes as f64) * width as f64).round() as usize;
        let w = w.max(1);

        if used + w <= width {
            print!("{}{}", clr, "█".repeat(w));
            used += w;
        }
    }

    if used < width {
        print!("\x1b[90m{}", "█".repeat(width - used));
    }
    println!("\x1b[0m]");

    print!("  ");
    for (lang, v) in filtered.iter().take(5) {
        let (clr, lbl) = get_color_and_label(lang);
        let pct = (v.3 as f64 / total_bytes as f64) * 100.0;
        print!("{}{} ({:.1}%)  ", clr, lbl, pct);
    }
    println!("\x1b[0m\n");
}

fn print_heatmap(code: usize, comm: usize, blnk: usize) {
    let total = code + comm + blnk;
    if total == 0 {
        println!("No data collected.");
        return;
    }

    let total = total as f64;
    let width: usize = 60;

    let c_w = ((code as f64 / total) * width as f64).floor() as usize;
    let m_w = ((comm as f64 / total) * width as f64).floor() as usize;
    let b_w = width.saturating_sub(c_w + m_w);

    println!(
        "\n\x1b[35;1m--- 📊 COMPOSITION --- \x1b[0m\n  [\x1b[32m{}\x1b[33m{}\x1b[37m{}\x1b[0m]",
        "█".repeat(c_w),
        "█".repeat(m_w),
        "█".repeat(b_w)
    );

    println!(
        "  \x1b[32m■\x1b[0m Code ({:.1}%)  \x1b[33m■\x1b[0m Comm* ({:.1}%)  \x1b[37m■\x1b[0m Blank ({:.1}%)",
        (code as f64 / total) * 100.0,
        (comm as f64 / total) * 100.0,
        (blnk as f64 / total) * 100.0
    );
}
