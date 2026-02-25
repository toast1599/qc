// src/output/text.rs

use crate::assets::LANG_MAP;
use crate::report::{LanguageStat, Report};
use crate::result::Lang;

const WIDTH_BAR: usize = 60;
const MAX_LANG_ROWS: usize = 15;
const MAX_COMPOSITION_LANGS: usize = 8;
const MAX_COMPOSITION_LABELS: usize = 5;

const CLR_RESET: &str = "\x1b[0m";
const CLR_GREY: &str = "\x1b[90m";
const CLR_DEFAULT: &str = "\x1b[38;5;250m";

fn get_color_and_label(lang: &Lang) -> (String, &str) {
    match lang {
        Lang::Identified(name) => {
            if let Some(data) = LANG_MAP.get(name) {
                if let Some(hex) = &data.color {
                    return (hex_to_ansi(hex), name);
                }
            }
            (CLR_DEFAULT.to_string(), name)
        }
        Lang::None => (CLR_GREY.to_string(), "None"),
        Lang::NonUtf8 => (CLR_GREY.to_string(), "[Binary]"),
    }
}

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
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let (v, u) = match bytes as f64 {
        b if b >= GB => (b / GB, "GB"),
        b if b >= MB => (b / MB, "MB"),
        b if b >= KB => (b / KB, "KB"),
        b => (b, "B"),
    };

    format!("{:.2} {}", v, u)
}

pub fn print_report(report: &Report) {
    print_summary(report);
    print_language_breakdown(report);
    print_top_files(report);
    print_heatmap(
        report.totals.code,
        report.totals.comment,
        report.totals.blank,
    );
    print_language_composition(&report.languages);
}

fn print_summary(report: &Report) {
    println!("\n\x1b[34;1m--- 📊 AUDIT RESULTS ---{}\n", CLR_RESET);
    println!(
        "Total Lines: {} (Code: {}, Comm*: {}, Blank: {})\nElapsed: {} ms",
        report.totals.code + report.totals.comment + report.totals.blank,
        report.totals.code,
        report.totals.comment,
        report.totals.blank,
        report.elapsed_ms
    );
}

fn print_language_breakdown(report: &Report) {
    println!("\n\x1b[36;1m--- 📚 LANGUAGE BREAKDOWN ---{}\n", CLR_RESET);
    println!(
        "{:<12} | {:>10} | {:>8} | {:>8} | {:>10}\n{}",
        "LANG",
        "CODE",
        "COMM*",
        "BLANK",
        "SIZE",
        "-".repeat(60)
    );

    for lang in report.languages.iter().take(MAX_LANG_ROWS) {
        let (clr, lbl) = get_color_and_label(&lang.lang);
        println!(
            "{}{:<12}{} | {:>10} | {:>8} | {:>8} | {:>10}",
            clr,
            lbl,
            CLR_RESET,
            lang.code,
            lang.comment,
            lang.blank,
            format_size(lang.bytes)
        );
    }
}

fn print_top_files(report: &Report) {
    println!(
        "\n\x1b[34;1m--- 🏆 TOP {} LARGEST FILES ---{}\n\x1b[1m{:>12} | {:>10} | PATH{}\n{}",
        report.files.len(),
        CLR_RESET,
        "LINES",
        "SIZE",
        CLR_RESET,
        "-".repeat(80)
    );

    for f in &report.files {
        let (clr, _) = get_color_and_label(&f.lang);
        println!(
            "{:>12} | {:>10} | {}{}{}",
            f.code + f.comment,
            format_size(f.bytes),
            clr,
            f.path,
            CLR_RESET
        );
    }
}

pub fn print_language_composition(stats: &[LanguageStat]) {
    let filtered: Vec<_> = stats
        .iter()
        .filter(|l| !matches!(l.lang, Lang::None | Lang::NonUtf8))
        .collect();

    let total_bytes: u64 = filtered.iter().map(|l| l.bytes).sum();
    if total_bytes == 0 {
        return;
    }

    println!("\n\x1b[1m--- 📊 LANGUAGE COMPOSITION ---{}\n  [", CLR_RESET);

    let mut remaining = WIDTH_BAR;

    for (i, lang) in filtered.iter().take(MAX_COMPOSITION_LANGS).enumerate() {
        let (clr, _) = get_color_and_label(&lang.lang);
        let w = if i + 1 == MAX_COMPOSITION_LANGS || i + 1 == filtered.len() {
            remaining
        } else {
            ((lang.bytes as f64 / total_bytes as f64) * WIDTH_BAR as f64).round() as usize
        };

        let w = w.min(remaining);
        remaining -= w;

        print!("{}{}", clr, "█".repeat(w));
        if remaining == 0 {
            break;
        }
    }

    if remaining > 0 {
        print!("{}{}", CLR_GREY, "█".repeat(remaining));
    }

    println!("{}]", CLR_RESET);
    print!("  ");

    for lang in filtered.iter().take(MAX_COMPOSITION_LABELS) {
        let (clr, lbl) = get_color_and_label(&lang.lang);
        let pct = (lang.bytes as f64 / total_bytes as f64) * 100.0;
        print!("{}{} ({:.1}%)  ", clr, lbl, pct);
    }

    println!("{}\n", CLR_RESET);
}

fn print_heatmap(code: usize, comm: usize, blank: usize) {
    let total = code + comm + blank;
    if total == 0 {
        println!("No data collected.");
        return;
    }

    let total_f = total as f64;
    let c_w = ((code as f64 / total_f) * WIDTH_BAR as f64).floor() as usize;
    let m_w = ((comm as f64 / total_f) * WIDTH_BAR as f64).floor() as usize;
    let b_w = WIDTH_BAR.saturating_sub(c_w + m_w);

    println!(
        "\n\x1b[35;1m--- 📊 COMPOSITION ---{}\n  [\x1b[32m{}\x1b[33m{}\x1b[37m{}\x1b[0m]",
        CLR_RESET,
        "█".repeat(c_w),
        "█".repeat(m_w),
        "█".repeat(b_w)
    );

    println!(
        "  \x1b[32m■\x1b[0m Code ({:.1}%)  \
         \x1b[33m■\x1b[0m Comm* ({:.1}%)  \
         \x1b[37m■\x1b[0m Blank ({:.1}%)",
        (code as f64 / total_f) * 100.0,
        (comm as f64 / total_f) * 100.0,
        (blank as f64 / total_f) * 100.0
    );
}
