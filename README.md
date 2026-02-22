# qc (Quick Count) 🚀

A very fast source code auditor and line counter built in Rust. 

`qc` is designed to give you an instant breakdown of your codebase composition. It uses **Memory Mapping (Mmap)** and **Parallel Directory Walking** to scan thousands of files in milliseconds.

## Benchmarks
In a head-to-head comparison against `tokei` on the Linux Kernel source tree, `qc` performed **~1.8x faster** over 100 runs.

| Tool | Mean Time | Speedup
| :--- | :--- | :---
| **qc** | **568.9 ms** | **1.81x**
| tokei | 1032 ms | 1.00x

## Features

* **High Performance:** Leverages all CPU cores and memory-mapped I/O for maximum throughput.
* **Deep Insights:** Provides line counts for Code, Comments, and Blank lines.
* **Language Breakdown:** Automatically identifies and categorizes files by extension.
* **Smart Composition:** Visualizes the ratio of code vs. comments with a terminal heatmap.
* **Respectful:** Automatically respects `.gitignore` and hidden files (powered by `ignore`).

## Installation

### Pre-built Binaries
Download the latest binary for your OS from the [Releases](https://github.com/toast1599/qc/releases) page.

### From Source
Ensure you have the Rust toolchain installed.

1. Clone the repository:
   ```bash
   git clone https://github.com/toast1599/qc.git
   cd qc
   ```

2. Build and install using the Makefile:
   ```bash
   make install
   ```
   *This will place the `qc` binary in your `~/.cargo/bin` directory.*

## Usage

Run `qc` in any directory to start an audit:

```bash
qc .
```

### Options
* `qc [path]` - Scan a specific directory.
* `qc -<number>` - Limit the "Top Files" list to N results (e.g., `qc -5`).
* `qc -h [or] qc --help` - Show the help message.

## Performance
`qc` is built to be faster than traditional `wc -l` loops by using byte-level scanning and avoiding unnecessary UTF-8 validation where possible.

## License
This project is licensed under **GPL-3.0-or-later** - see the [LICENSE](LICENSE) file for details.
