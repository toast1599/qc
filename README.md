# qc (Quick Count) 🚀

A high-performance source code auditor and line counter built in modern Rust. 

`qc` is designed to provide an instant, high-fidelity breakdown of codebase composition. By leveraging **Memory Mapping (Mmap)** and **Parallel Directory Walking**, it can scan tens of thousands of files in milliseconds.

## ⚡ Benchmarks
In head-to-head comparisons against `tokei` on the Linux Kernel source tree, `qc` performed **~1.8x faster** over 100 runs.

| Tool | Mean Time | Speedup |
| :--- | :--- | :--- |
| **qc** | **568.9 ms** | **1.81x** |
| tokei | 1032 ms | 1.00x |

## ✨ Features

* **Modern Rust Core:** Built on the **Rust 2024 Edition** for maximum safety and performance.
* **Hybrid I/O Engine:** Automatically switches between standard `fs::read` and `Mmap` based on file size to optimize throughput.
* **Deep Heuristics:** Identifies files via extensions, filenames, and **shebang detection** (e.g., `#!/bin/bash`).
* **Line Composition:** Accurately distinguishes between Code, Comments, and Blank lines using a fast, byte-level state machine.
* **Terminal Visuals:** Includes a color-coded language composition bar and a code/comment heatmap.
* **Rich Output:** Supports human-readable text output or machine-readable **JSON** for CI/CD integration.
* **Respectful:** Powered by the `ignore` crate to automatically honor `.gitignore`, `.ignore`, and hidden file rules.

## 🛠 Installation

### From Source
Ensure you have the latest Rust toolchain installed (1.85+ recommended for Edition 2024).

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/toast1599/qc.git
    cd qc
    ```

2.  **Build and Install:**
    ```bash
    cargo install --path .
    ```
    *Or use the provided `Makefile` to automate the build process.*

## 🚀 Usage

Run `qc` in any directory to start an audit:

```bash
qc .
```

### Options
* `qc [path]` — Scan a specific directory (defaults to `.`).
* `qc -<number>` — Limit the "Top Files" list (e.g., `qc -20`).
* `qc --json` — Output results as a JSON object.
* `qc --json-out <path>` — Save JSON results directly to a file.
* `qc --rs --py` — Filter results to specific languages (e.g., Rust and Python).

## 📊 Performance Philosophy
Unlike traditional counters that rely on heavy AST parsing, `qc` uses a **byte-level scanning heuristic**. This avoids the overhead of full UTF-8 validation while maintaining high accuracy for line-type classification, making it ideal for massive monorepos where speed is the primary constraint.

## ⚖️ License
This project is licensed under **GPL-3.0-or-later** — see the [LICENSE](LICENSE) file for details.
