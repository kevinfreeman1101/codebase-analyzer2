# codebase-analyzer2

A lightweight Rust tool to analyze code projects and generate compact JSON summaries for Grok 3. Supports multiple file types, parallel processing, and detailed metrics.

## Features

- Analyzes `.py`, `.rs`, `.c`, `.cpp`, `.h`, `.json` files.
- Metrics: lines, size, complexity, documentation %, dependencies, variables, performance hotspots.
- CLI: `codebase-analyzer2 [path] [extensions] [output_file]`.
- Fast: Parallel processing with Rayon.

## Installation

### Linux (Debian/Ubuntu)

#### Linux: Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

#### Linux: Install from Source

```bash
git clone https://github.com/kevinfreeman1101/codebase-analyzer2
cd codebase-analyzer2
cargo install --path .
```

#### Linux: Run

- Via Cargo: cargo run -- [path] [extensions] [output_file]
- Installed: codebase-analyzer2 [path] [extensions] [output_file]

### Windows

#### Windows: Install Rust

- Download an run rustup-init.exe from rustup.rs.
- Follow prompts; add Rust to PATH when asked.
- Restart your terminal (e.g., PowerShell).

#### Windows: Install from Source

```powershell
git clone https://github.com/kevinfreeman1101/codebase-analyzer2
cd codebase-analyzer2
cargo install --path .
```

#### Windows: Run

- Via Cargo: cargo run -- [path] [extensions] [output_file]
- Installed: codebase-analyzer2.exe [path] [extensions] [output_file]

### Linux/Windows: Pre-Built Binary 

- Download the latest release binary from https://github.com/kevinfreeman1101/codebase-analyzer2/releases (e.g., codebase-analyzer2-linux, codebase-analyzer2-windows.exe).
- Place it in your PATH:
  - Linux: mv codebase-analyzer2-linux ~/.local/bin/codebase-analyzer2 && chmod +x ~/.local/bin/codebase-analyzer2
  - Windows: Move codebase-analyzer2-windows.exe to C:\Users\<YourUser>\.cargo\bin or another PATH directory.
- Run: codebase-analyzer2 [path] [extensions] [output_file]

## Usage

```bash
codebase-analyzer2 ./my_project py,rs,c summary.json
```

- path: Directory to analyze (default: .).
- extensions: Comma-separated file types (default: py,rs,c,cpp,h,json).
- output_file: JSON output file (default: summary.json).
- Help: codebase-analyzer2 --help

### Example Output

```json
{
  "ts": "2025-03-22T04:15:29.475046842+00:00",
  "version": "0.1.0",
  "files_count": 3,
  "lines": 12,
  "size": 160,
  "path": "./test",
  "exts": ["py", "rs", "c"],
  "errors": [],
  "files": { ... }
}
```

## Building a Release Binary

- 1. Compile

```bash
cargo build --release
```

- 2. Find binary:
  - Linux: ./target/release/codebase-analyzer2
  - Windows: .\target\release\codebase-analyzer2.exe
- 3. Share via your repo's Releases page

## License

MIT
