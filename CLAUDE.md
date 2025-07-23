# Docker Development Environment

Start development container:
```bash
docker compose up -d
docker compose exec rust-dev bash
```

# Development Guidelines

- Use only Rust standard library - no external dependencies
- Implement functionality using built-in Rust features only

# Multi-Language Keyword Analyzer Usage

## Language Selection

The analyzer supports multiple programming languages. Use the `--language` or `-l` option to specify the target language.

```bash
# Run in Docker container
docker compose up -d
docker compose exec rust-dev bash

# Analyze Rust code (default)
cargo run -- --language rust
cargo run -- -l rust

# Analyze TypeScript/JavaScript code
cargo run -- --language typescript
cargo run -- -l ts
```

## Basic Usage

```bash
# Analyze current directory (Rust by default)
cargo run

# Analyze specific directory or file
cargo run -- --language rust /path/to/rust/project
cargo run -- --language typescript src/

# Analyze single files
cargo run -- --language rust src/main.rs
cargo run -- --language typescript app.ts

# Analyze GitHub repositories
cargo run -- --language rust https://github.com/rust-lang/rust
cargo run -- --language typescript https://github.com/microsoft/typescript
```

## Using Built Binary

```bash
# Build release version
cargo build --release

# Run built binary directly
./target/release/app

# With arguments
./target/release/app --language rust --format json
./target/release/app --language typescript /path/to/typescript/project
./target/release/app --language rust https://github.com/rust-lang/rust
./target/release/app --help
## Output Formats

```bash
# Plain text output (default)
cargo run -- --language rust

# JSON format
cargo run -- --language typescript --format json

# CSV format
cargo run -- --language rust --format csv
```

## Command Options

```bash
# Show help
cargo run -- --help

# Language selection (long and short forms)
cargo run -- --language rust
cargo run -- -l typescript

# Format selection (long and short forms)
cargo run -- --format json
cargo run -- -f csv

# Combined options
cargo run -- -l ts -f json src/
```

## Supported Languages

### Rust
- **File Extensions**: `.rs`
- **Keywords Tracked**: 60+ keywords including primitives, control flow, and reserved words
- **Skip Directories**: `target/`, `.git/`, `node_modules/`

### TypeScript/JavaScript  
- **File Extensions**: `.ts`, `.tsx`, `.js`, `.jsx`
- **Keywords Tracked**: 88+ keywords including ES2023, TypeScript 5.x features
- **Skip Directories**: `node_modules/`, `dist/`, `build/`, `.git/`, `target/`

## Features

- **Multi-language support**: Rust and TypeScript/JavaScript analysis
- **Comprehensive keyword tracking**: Latest language specifications supported
- **Recursive directory scanning** with intelligent directory skipping
- **Multiple output formats** (Plain, JSON, CSV) for integration with other tools
- **GitHub repository support** - directly analyze any public GitHub repository by URL
- **Real-time progress display** - shows current file being processed and count
- **Automatic cleanup** - temporary directories are cleaned up after analysis

## GitHub Repository Analysis

```bash
# Analyze popular Rust projects
cargo run -- --language rust https://github.com/rust-lang/rust
cargo run -- --language rust https://github.com/tokio-rs/tokio
cargo run -- --language rust https://github.com/serde-rs/serde

# Analyze popular TypeScript projects
cargo run -- --language typescript https://github.com/microsoft/typescript
cargo run -- --language typescript https://github.com/angular/angular
cargo run -- --language typescript https://github.com/vuejs/vue

# With different output formats
cargo run -- --language rust --format json https://github.com/actix/actix-web
cargo run -- --language typescript --format csv https://github.com/nestjs/nest
```

The tool automatically:
- Clones the repository to a temporary directory using `git clone --depth 1`
- Analyzes all files matching the selected language (`.rs` for Rust, `.ts/.tsx/.js/.jsx` for TypeScript)
- Displays real-time progress showing files being processed
- Cleans up the temporary directory after analysis

## Example Outputs

### Rust Analysis
```
=== Rust Keyword Analysis Results ===
Files analyzed: 3
Total keywords found: 425

let          : 97
fn           : 39
if           : 34
mut          : 21
use          : 17
...
```

### TypeScript Analysis
```
=== Rust Keyword Analysis Results ===
Files analyzed: 1
Total keywords found: 21

this         : 3
string       : 3
interface    : 1
class        : 1
async        : 1
...
```

## Testing

Run the comprehensive test suite:

```bash
# Run all tests
docker compose exec rust-dev cargo test

# Run tests with output
docker compose exec rust-dev cargo test -- --no-capture

# Run specific test module
docker compose exec rust-dev cargo test rust::tests
docker compose exec rust-dev cargo test typescript::tests
```

The test suite includes:
- **20 unit tests** covering all major functionality
- **Language-specific tests** for both Rust and TypeScript modules
- **Edge case testing** for error handling and input validation
- **Mock tests** for network-dependent functionality (avoiding authentication prompts)