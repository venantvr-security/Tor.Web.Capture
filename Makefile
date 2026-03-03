# Tor.Web.Capture - Makefile
# ===========================

.PHONY: all build release run dev test clean fmt clippy check setup help

# Variables
CARGO := cargo
BINARY := tor-web-capture
TARGET_DIR := target

# Default target
all: build

# ─────────────────────────────────────────────────────────────
# Build
# ─────────────────────────────────────────────────────────────

## Build debug version
build:
	$(CARGO) build

## Build release version (optimized)
release:
	$(CARGO) build --release

## Build all crates
build-all:
	$(CARGO) build --workspace

# ─────────────────────────────────────────────────────────────
# Run
# ─────────────────────────────────────────────────────────────

## Run the application (debug)
run:
	$(CARGO) run

## Run the application (release)
run-release:
	$(CARGO) run --release

## Run with live reload (requires cargo-watch)
dev:
	$(CARGO) watch -x run

## Run with environment variables from .env
run-env:
	@if [ -f .env ]; then \
		export $$(cat .env | xargs) && $(CARGO) run; \
	else \
		echo "No .env file found"; \
		$(CARGO) run; \
	fi

# ─────────────────────────────────────────────────────────────
# Test
# ─────────────────────────────────────────────────────────────

## Run all tests
test:
	$(CARGO) test --workspace

## Run tests with output
test-verbose:
	$(CARGO) test --workspace -- --nocapture

## Run tests for a specific crate
test-crate:
	@echo "Usage: make test-crate CRATE=tor-capture-core"
	@if [ -n "$(CRATE)" ]; then $(CARGO) test -p $(CRATE); fi

# ─────────────────────────────────────────────────────────────
# Quality
# ─────────────────────────────────────────────────────────────

## Format code
fmt:
	$(CARGO) fmt --all

## Check formatting
fmt-check:
	$(CARGO) fmt --all -- --check

## Run clippy linter
clippy:
	$(CARGO) clippy --workspace --all-targets -- -D warnings

## Run all checks (fmt + clippy + test)
check: fmt-check clippy test
	@echo "All checks passed!"

## Run cargo check (fast compile check)
check-compile:
	$(CARGO) check --workspace

# ─────────────────────────────────────────────────────────────
# Documentation
# ─────────────────────────────────────────────────────────────

## Generate documentation
doc:
	$(CARGO) doc --workspace --no-deps

## Generate and open documentation
doc-open:
	$(CARGO) doc --workspace --no-deps --open

# ─────────────────────────────────────────────────────────────
# Clean
# ─────────────────────────────────────────────────────────────

## Clean build artifacts
clean:
	$(CARGO) clean

## Clean and rebuild
rebuild: clean build

# ─────────────────────────────────────────────────────────────
# Setup
# ─────────────────────────────────────────────────────────────

## Setup development environment
setup:
	@echo "Installing Rust toolchain components..."
	rustup component add rustfmt clippy
	@echo "Installing cargo extensions..."
	$(CARGO) install cargo-watch || true
	@echo "Creating config directory if needed..."
	@mkdir -p data
	@echo "Setup complete!"

## Install the binary locally
install:
	$(CARGO) install --path .

## Copy default config if not exists
init-config:
	@if [ ! -f config/local.toml ]; then \
		cp config/default.toml config/local.toml 2>/dev/null || true; \
		echo "Created config/local.toml"; \
	else \
		echo "config/local.toml already exists"; \
	fi

# ─────────────────────────────────────────────────────────────
# Database
# ─────────────────────────────────────────────────────────────

## Create database directory
db-init:
	@mkdir -p data
	@echo "Database directory created at ./data"

# ─────────────────────────────────────────────────────────────
# Docker (optional)
# ─────────────────────────────────────────────────────────────

## Build Docker image
docker-build:
	docker build -t tor-web-capture .

## Run in Docker
docker-run:
	docker run -p 8080:8080 -v $(PWD)/data:/app/data tor-web-capture

# ─────────────────────────────────────────────────────────────
# Help
# ─────────────────────────────────────────────────────────────

## Show this help
help:
	@echo "Tor.Web.Capture - Available targets:"
	@echo ""
	@echo "  Build:"
	@echo "    make build        - Build debug version"
	@echo "    make release      - Build release version"
	@echo "    make build-all    - Build all workspace crates"
	@echo ""
	@echo "  Run:"
	@echo "    make run          - Run application (debug)"
	@echo "    make run-release  - Run application (release)"
	@echo "    make dev          - Run with live reload"
	@echo ""
	@echo "  Test:"
	@echo "    make test         - Run all tests"
	@echo "    make test-verbose - Run tests with output"
	@echo ""
	@echo "  Quality:"
	@echo "    make fmt          - Format code"
	@echo "    make clippy       - Run linter"
	@echo "    make check        - Run all checks"
	@echo ""
	@echo "  Other:"
	@echo "    make setup        - Setup dev environment"
	@echo "    make clean        - Clean build artifacts"
	@echo "    make doc          - Generate documentation"
	@echo "    make help         - Show this help"
