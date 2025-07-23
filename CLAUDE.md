# Multi-Language Keyword Analyzer

A comprehensive keyword analysis tool supporting 5 programming languages with both CLI and API Server interfaces.

## Architecture

This project uses Rust workspace architecture with three main components:

```
├── Cargo.toml (workspace root)
├── shared-lib/ (共有ライブラリ - Core analysis engine)
├── cli/ (CLI binary - Command-line interface)
├── api-server/ (API Server binary - REST API)
├── CLAUDE.md (this file)
├── Dockerfile
└── compose.yml
```

## Docker Development Environment

Start development container:
```bash
docker compose up -d rust-dev
docker compose exec rust-dev bash
```

Start API server:
```bash
docker compose up -d api-server
```

# Development Guidelines

- Uses only Rust standard library for analysis logic - no external dependencies in shared-lib
- API server uses tokio + axum for REST endpoints
- All language analysis logic is shared between CLI and API

## CLI Usage

The CLI provides the original keyword analysis functionality.

### Language Selection

The analyzer supports multiple programming languages. Use the `--language` or `-l` option:

```bash
# Run in Docker container
docker compose up -d rust-dev
docker compose exec rust-dev bash

# Analyze Rust code (default)
cargo run --bin keyword-analyzer -- --language rust
cargo run --bin keyword-analyzer -- -l rust

# Analyze TypeScript/JavaScript code
cargo run --bin keyword-analyzer -- --language js
cargo run --bin keyword-analyzer -- -l ts

# Analyze Ruby code
cargo run --bin keyword-analyzer -- --language ruby
cargo run --bin keyword-analyzer -- -l rb

# Analyze Go code
cargo run --bin keyword-analyzer -- --language go
cargo run --bin keyword-analyzer -- -l golang

# Analyze Python code
cargo run --bin keyword-analyzer -- --language python
cargo run --bin keyword-analyzer -- -l py
```

### Basic Usage

```bash
# Analyze current directory (Rust by default)
cargo run --bin keyword-analyzer

# Analyze specific directory or file
cargo run --bin keyword-analyzer -- --language rust /path/to/rust/project
cargo run --bin keyword-analyzer -- --language js src/
cargo run --bin keyword-analyzer -- --language ruby lib/

# Analyze single files
cargo run --bin keyword-analyzer -- --language rust src/main.rs
cargo run --bin keyword-analyzer -- --language js app.ts
cargo run --bin keyword-analyzer -- --language ruby app.rb
cargo run --bin keyword-analyzer -- --language go main.go

# Analyze GitHub repositories
cargo run --bin keyword-analyzer -- --language rust https://github.com/rust-lang/rust
cargo run --bin keyword-analyzer -- --language js https://github.com/microsoft/typescript
cargo run --bin keyword-analyzer -- --language ruby https://github.com/rails/rails
cargo run --bin keyword-analyzer -- --language go https://github.com/golang/go
```

### Output Formats

```bash
# Plain text output (default)
cargo run --bin keyword-analyzer -- --language rust

# JSON format
cargo run --bin keyword-analyzer -- --language js --format json

# CSV format
cargo run --bin keyword-analyzer -- --language rust --format csv

# HTML format
cargo run --bin keyword-analyzer -- --language rust --format html

# SVG Graph format
cargo run --bin keyword-analyzer -- --language rust --format graph

# Custom output file
cargo run --bin keyword-analyzer -- --language rust --format html --output analysis.html
```

### Command Options

```bash
# Show help
cargo run --bin keyword-analyzer -- --help

# Language selection (long and short forms)
cargo run --bin keyword-analyzer -- --language rust
cargo run --bin keyword-analyzer -- -l js

# Format selection (long and short forms)
cargo run --bin keyword-analyzer -- --format json
cargo run --bin keyword-analyzer -- -f csv

# Output file specification
cargo run --bin keyword-analyzer -- --output results.json
cargo run --bin keyword-analyzer -- -o analysis.html

# Combined options
cargo run --bin keyword-analyzer -- -l ts -f json -o typescript_analysis.json src/
cargo run --bin keyword-analyzer -- -l rb -f csv -o ruby_keywords.csv lib/
cargo run --bin keyword-analyzer -- -l go -f json -o go_analysis.json cmd/
cargo run --bin keyword-analyzer -- -l rust -f html -o rust_report.html src/
cargo run --bin keyword-analyzer -- -l rust -f graph -o rust_chart.svg src/
```

## API Server Usage

The API server provides REST endpoints for keyword analysis.

### Starting the Server

```bash
# Start API server (runs on port 3000)
docker compose up -d api-server

# Check logs
docker compose logs -f api-server
```

### API Endpoints

#### Health Check
```bash
curl http://localhost:3000/health
```

Response:
```json
{
  "success": true,
  "data": "Keyword Analyzer API Server is running",
  "error": null
}
```

#### Supported Languages
```bash
curl http://localhost:3000/languages
```

Response:
```json
{
  "success": true,
  "data": ["rust", "javascript", "ruby", "go", "python"],
  "error": null
}
```

#### Analyze Code (POST)
```bash
curl -X POST http://localhost:3000/analyze \
  -H "Content-Type: application/json" \
  -d '{
    "language": "rust",
    "path": "shared-lib/src/"
  }'
```

#### Analyze Code (GET)
```bash
curl "http://localhost:3000/analyze/rust/shared-lib/src/"
```

Response Format:
```json
{
  "success": true,
  "data": {
    "language": "rust",
    "file_count": 6,
    "total_keywords": 1331,
    "keyword_counts": {
      "let": 263,
      "fn": 110,
      "if": 103,
      "..."
    },
    "files_analyzed": ["shared-lib/src/lib.rs", "..."]
  },
  "error": null
}
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

### Ruby
- **File Extensions**: `.rb`, `.rake`, `.gemspec`, plus special files (`Rakefile`, `Gemfile`, `Guardfile`, `Capfile`, `Vagrantfile`)
- **Keywords Tracked**: 75+ keywords including core Ruby keywords, metaprogramming, and common methods
- **Skip Directories**: `vendor/`, `tmp/`, `log/`, `.bundle/`, `.git/`, `target/`, `node_modules/`

### Go
- **File Extensions**: `.go`
- **Keywords Tracked**: 64+ keywords including language keywords, built-in types, constants, and functions
- **Skip Directories**: `vendor/`, `bin/`, `pkg/`, `.git/`, `target/`, `node_modules/`, `.vscode/`, `.idea/`

### Python
- **File Extensions**: `.py`, `.pyw`, plus special files (`__init__.py`, `setup.py`, `conftest.py`)
- **Keywords Tracked**: 75+ keywords including built-in functions, exceptions, and Python 3.x features
- **Skip Directories**: `__pycache__/`, `.venv/`, `venv/`, `env/`, `.git/`, `target/`, `node_modules/`

## Features

- **Multi-language support**: Rust, TypeScript/JavaScript, Ruby, Go, and Python analysis
- **Comprehensive keyword tracking**: Latest language specifications supported
- **Recursive directory scanning** with intelligent directory skipping
- **Multiple output formats** (Plain, JSON, CSV, HTML, SVG Graph) for integration with other tools
- **File output support**: Save results to custom files or use auto-generated timestamped filenames
- **GitHub repository support** - directly analyze any public GitHub repository by URL
- **Real-time progress display** - shows current file being processed and count
- **Automatic cleanup** - temporary directories are cleaned up after analysis
- **REST API** - HTTP endpoints for integration with web applications
- **Docker support** - containerized development and deployment

## GitHub Repository Analysis

Both CLI and API support direct GitHub URL analysis:

```bash
# CLI
cargo run --bin keyword-analyzer -- --language rust https://github.com/rust-lang/rust
cargo run --bin keyword-analyzer -- --language js https://github.com/microsoft/typescript

# API
curl -X POST http://localhost:3000/analyze \
  -H "Content-Type: application/json" \
  -d '{
    "language": "rust", 
    "path": "https://github.com/rust-lang/rust"
  }'
```

## File Output

When using JSON, CSV, HTML, or Graph formats, the tool automatically saves results to files:

### Automatic File Naming
If no output file is specified, the tool generates timestamped filenames:
- Format: `keyword_analysis_{language}_{timestamp}.{extension}`
- Examples: `keyword_analysis_rust_1753282196.json`, `keyword_analysis_python_1753282200.svg`

### Custom File Paths
Use `--output` or `-o` to specify custom output paths:

```bash
# CLI
cargo run --bin keyword-analyzer -- --format json --output my_analysis.json

# API returns analysis data as JSON (no file output)
```

### Output Format Details

**JSON Output:**
- Structured data perfect for programmatic analysis
- Includes file count, total keywords, and keyword counts

**CSV Output:**
- Spreadsheet-compatible format
- Easy to import into Excel, Google Sheets, or data analysis tools

**HTML Output:**
- Visual, interactive report with charts and styling
- Professional presentation format
- Includes progress bars showing keyword distribution
- Mobile-responsive design

**SVG Graph Output:**
- Interactive bar chart visualization
- Top 20 keywords with color coding
- Hover effects and tooltips
- Scalable vector graphics

## Testing

Run the comprehensive test suite:

```bash
# Run all tests
docker compose exec rust-dev cargo test

# Run tests with output
docker compose exec rust-dev cargo test -- --no-capture

# Run specific workspace tests
docker compose exec rust-dev cargo test -p keyword-analyzer-shared
docker compose exec rust-dev cargo test -p keyword-analyzer-cli
docker compose exec rust-dev cargo test -p keyword-analyzer-api
```

The test suite includes:
- **40+ unit tests** covering all major functionality
- **Language-specific tests** for all supported languages
- **Edge case testing** for error handling and input validation
- **Shared library tests** for core analysis engine

## Building

```bash
# Build all workspace members
docker compose exec rust-dev cargo build

# Build specific binary
docker compose exec rust-dev cargo build --bin keyword-analyzer
docker compose exec rust-dev cargo build --bin api-server

# Build release version
docker compose exec rust-dev cargo build --release
```

## Development

### Project Structure
- `shared-lib/`: Core analysis engine (language-agnostic)
- `cli/`: Command-line interface using shared library
- `api-server/`: REST API server using shared library

### Adding New Languages
1. Create new module in `shared-lib/src/`
2. Add language enum variant in `shared-lib/src/lib.rs`
3. Implement analysis functions following existing patterns
4. Add to CLI argument parser in `cli/src/main.rs`
5. Add to API language parser in `api-server/src/main.rs`
6. Add comprehensive tests

### Contributing Guidelines
- Follow existing code patterns
- Add tests for new functionality
- Update documentation
- Use `cargo fmt` for formatting
- Run `cargo clippy` for linting