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
## Note: AWS_LC_SYS_NO_ASM=1 workaround for GCC 9.x bug with aws-lc-sys
build:
	AWS_LC_SYS_NO_ASM=1 $(CARGO) build

## Build release version (optimized)
## Note: If GCC < 10, install clang: sudo apt install clang
## Then use: CC=clang make release
release:
	@if command -v clang >/dev/null 2>&1; then \
		CC=clang $(CARGO) build --release; \
	else \
		echo "Warning: clang not found. GCC 9.x has a bug with aws-lc-sys."; \
		echo "Install clang (sudo apt install clang) or upgrade GCC to 10+"; \
		$(CARGO) build --release; \
	fi

## Build all crates
build-all:
	AWS_LC_SYS_NO_ASM=1 $(CARGO) build --workspace

# ─────────────────────────────────────────────────────────────
# Run
# ─────────────────────────────────────────────────────────────

## Run the application (debug)
run:
	AWS_LC_SYS_NO_ASM=1 $(CARGO) run

## Run the application (release)
run-release:
	@if command -v clang >/dev/null 2>&1; then \
		CC=clang $(CARGO) run --release; \
	else \
		$(CARGO) run --release; \
	fi

## Run with live reload (requires cargo-watch)
dev:
	AWS_LC_SYS_NO_ASM=1 $(CARGO) watch -x run

## Run with environment variables from .env
run-env:
	@if [ -f .env ]; then \
		export $$(cat .env | xargs) && AWS_LC_SYS_NO_ASM=1 $(CARGO) run; \
	else \
		echo "No .env file found"; \
		AWS_LC_SYS_NO_ASM=1 $(CARGO) run; \
	fi

# ─────────────────────────────────────────────────────────────
# Test
# ─────────────────────────────────────────────────────────────

## Run all tests
test:
	AWS_LC_SYS_NO_ASM=1 $(CARGO) test --workspace

## Run tests with output
test-verbose:
	AWS_LC_SYS_NO_ASM=1 $(CARGO) test --workspace -- --nocapture

## Run tests for a specific crate
test-crate:
	@echo "Usage: make test-crate CRATE=tor-capture-core"
	@if [ -n "$(CRATE)" ]; then AWS_LC_SYS_NO_ASM=1 $(CARGO) test -p $(CRATE); fi

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
	AWS_LC_SYS_NO_ASM=1 $(CARGO) clippy --workspace --all-targets -- -D warnings

## Run all checks (fmt + clippy + test)
check: fmt-check clippy test
	@echo "All checks passed!"

## Run cargo check (fast compile check)
check-compile:
	AWS_LC_SYS_NO_ASM=1 $(CARGO) check --workspace

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

## Install the binary locally (requires clang due to GCC bug)
install:
	@if command -v clang >/dev/null 2>&1; then \
		CC=clang $(CARGO) install --path .; \
	else \
		echo "Error: clang not found. GCC 9.x has a bug with aws-lc-sys."; \
		echo "Install clang: sudo apt install clang"; \
		exit 1; \
	fi

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
