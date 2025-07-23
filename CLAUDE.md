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

The analyzer supports multiple programming languages. Use the `--language` or `-l` option with `rust`/`rs` for Rust, `js`/`ts` for JavaScript/TypeScript, `ruby`/`rb` for Ruby, or `go`/`golang` for Go.

```bash
# Run in Docker container
docker compose up -d
docker compose exec rust-dev bash

# Analyze Rust code (default)
cargo run -- --language rust
cargo run -- -l rust

# Analyze TypeScript/JavaScript code
cargo run -- --language js
cargo run -- -l ts

# Analyze Ruby code
cargo run -- --language ruby
cargo run -- -l rb

# Analyze Go code
cargo run -- --language go
cargo run -- -l golang
```

## Basic Usage

```bash
# Analyze current directory (Rust by default)
cargo run

# Analyze specific directory or file
cargo run -- --language rust /path/to/rust/project
cargo run -- --language js src/
cargo run -- --language ruby lib/

# Analyze single files
cargo run -- --language rust src/main.rs
cargo run -- --language js app.ts
cargo run -- --language ruby app.rb
cargo run -- --language go main.go

# Analyze GitHub repositories
cargo run -- --language rust https://github.com/rust-lang/rust
cargo run -- --language js https://github.com/microsoft/typescript
cargo run -- --language ruby https://github.com/rails/rails
cargo run -- --language go https://github.com/golang/go
```

## Using Built Binary

```bash
# Build release version
cargo build --release

# Run built binary directly
./target/release/app

# With arguments
./target/release/app --language rust --format json
./target/release/app --language js /path/to/typescript/project
./target/release/app --language ruby /path/to/ruby/project
./target/release/app --language go /path/to/go/project
./target/release/app --language rust https://github.com/rust-lang/rust
./target/release/app --help
## Output Formats

```bash
# Plain text output (default)
cargo run -- --language rust

# JSON format
cargo run -- --language js --format json

# CSV format
cargo run -- --language rust --format csv

# HTML format
cargo run -- --language rust --format html

# Ruby with different formats
cargo run -- --language ruby --format json

# Go with different formats
cargo run -- --language go --format csv
```

## Command Options

```bash
# Show help
cargo run -- --help

# Language selection (long and short forms)
cargo run -- --language rust
cargo run -- -l js
cargo run -- -l ruby
cargo run -- -l go

# Format selection (long and short forms)
cargo run -- --format json
cargo run -- -f csv

# Combined options
cargo run -- -l ts -f json src/
cargo run -- -l rb -f csv lib/  
cargo run -- -l go -f json cmd/
cargo run -- -l rust -f html src/
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

## Features

- **Multi-language support**: Rust, TypeScript/JavaScript, Ruby, and Go analysis
- **Comprehensive keyword tracking**: Latest language specifications supported
- **Recursive directory scanning** with intelligent directory skipping
- **Multiple output formats** (Plain, JSON, CSV, HTML) for integration with other tools
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
cargo run -- --language js https://github.com/microsoft/typescript
cargo run -- --language js https://github.com/angular/angular
cargo run -- --language js https://github.com/vuejs/vue

# Analyze popular Ruby projects
cargo run -- --language ruby https://github.com/rails/rails
cargo run -- --language ruby https://github.com/jekyll/jekyll
cargo run -- --language ruby https://github.com/discourse/discourse

# Analyze popular Go projects
cargo run -- --language go https://github.com/golang/go
cargo run -- --language go https://github.com/kubernetes/kubernetes
cargo run -- --language go https://github.com/docker/docker

# With different output formats
cargo run -- --language rust --format json https://github.com/actix/actrix-web
cargo run -- --language js --format csv https://github.com/nestjs/nest
cargo run -- --language ruby --format json https://github.com/fastlane/fastlane
cargo run -- --language go --format json https://github.com/hashicorp/terraform
cargo run -- --language rust --format html https://github.com/rust-lang/rust
```

The tool automatically:
- Clones the repository to a temporary directory using `git clone --depth 1`
- Analyzes all files matching the selected language (`.rs` for Rust, `.ts/.tsx/.js/.jsx` for TypeScript/JavaScript, `.rb/.rake/.gemspec` for Ruby, `.go` for Go)
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

### Ruby Analysis
```
=== Rust Keyword Analysis Results ===
Files analyzed: 2
Total keywords found: 67

def          : 12
end          : 12
class        : 4
if           : 6
attr_accessor: 2
module       : 1
...
```

### Go Analysis
```
=== Rust Keyword Analysis Results ===
Files analyzed: 3
Total keywords found: 89

func         : 15
var          : 12
if           : 8
for          : 6
return       : 5
package      : 3
import       : 2
struct       : 2
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
docker compose exec rust-dev cargo test javascript::tests
docker compose exec rust-dev cargo test ruby::tests
docker compose exec rust-dev cargo test golang::tests
```

The test suite includes:
- **40+ unit tests** covering all major functionality
- **Language-specific tests** for Rust, JavaScript, Ruby, and Go modules
- **Edge case testing** for error handling and input validation
- **Mock tests** for network-dependent functionality (avoiding authentication prompts)