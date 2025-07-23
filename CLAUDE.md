# Docker Development Environment

Start development container:
```bash
docker compose up -d
docker compose exec rust-dev bash
```

# Development Guidelines

- Use only Rust standard library - no external dependencies
- Implement functionality using built-in Rust features only

# Rust Keyword Analyzer Usage

## Basic Usage

```bash
# Run in Docker container
docker compose up -d
docker compose exec rust-dev bash

# Analyze current directory (default)
cargo run

# Analyze specific directory or file
cargo run -- /path/to/rust/project
cargo run -- src/main.rs
```

## Using Built Binary

```bash
# Build release version
cargo build --release

# Run built binary directly
./target/release/app

# With arguments
./target/release/app --format json
./target/release/app /path/to/rust/project
./target/release/app --help
## Output Formats

```bash
# Plain text output (default)
cargo run

# JSON format
cargo run -- --format json

# CSV format
cargo run -- --format csv
```

## Command Options

```bash
# Show help
cargo run -- --help

# Short format option
cargo run -- -f json
```

## Features

- Analyzes all Rust keyword usage in `.rs` files
- Supports recursive directory scanning
- Skips `target/`, `.git/`, and `node_modules/` directories
- Tracks 60+ Rust keywords including primitives, control flow, and reserved words
- Multiple output formats for integration with other tools