# Multi-Language Keyword Analyzer Development Tools

.PHONY: help fmt fmt-check build test clean dev api doc

# Default target
help:
	@echo "Available targets:"
	@echo "  fmt         - Format all Rust code"
	@echo "  fmt-check   - Check if code is properly formatted"
	@echo "  build       - Build all workspace members"
	@echo "  test        - Run all tests"
	@echo "  clean       - Clean build artifacts"
	@echo "  dev         - Start development environment"
	@echo "  api         - Start API server"
	@echo "  doc         - Generate documentation"
	@echo "  release     - Build release version"

# Code formatting
fmt:
	@echo "🎨 Formatting Rust code..."
	@docker compose exec rust-dev cargo fmt --all

fmt-check:
	@echo "🔍 Checking code formatting..."
	@docker compose exec rust-dev cargo fmt --all -- --check

# Building
build:
	@echo "🔨 Building all workspace members..."
	@docker compose exec rust-dev cargo build

release:
	@echo "🚀 Building release version..."
	@docker compose exec rust-dev cargo build --release

# Testing
test:
	@echo "🧪 Running all tests..."
	@docker compose exec rust-dev cargo test

# Development
dev:
	@echo "🛠️  Starting development environment..."
	@docker compose up -d rust-dev
	@docker compose exec rust-dev bash

api:
	@echo "🌐 Starting API server..."
	@docker compose up -d api-server

# Documentation
doc:
	@echo "📚 Generating documentation..."
	@docker compose exec rust-dev cargo doc --no-deps --open

# Cleanup
clean:
	@echo "🧹 Cleaning build artifacts..."
	@docker compose exec rust-dev cargo clean

# Linting
clippy:
	@echo "📎 Running clippy..."
	@docker compose exec rust-dev cargo clippy --all -- -D warnings

# Complete development workflow
check: fmt-check clippy test
	@echo "✅ All checks passed!"

# Quick development setup
setup:
	@echo "🔧 Setting up development environment..."
	@docker compose up -d rust-dev
	@docker compose exec rust-dev cargo build
	@echo "✅ Development environment ready!"