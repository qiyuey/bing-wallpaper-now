# Makefile - Bing Wallpaper Now
#
# Quick Reference:
#   make dev              # Start development mode
#   make build            # Build production version
#   make test             # Run all tests
#   make pre-commit       # Pre-commit check
#   make snapshot-patch   # Create SNAPSHOT version
#   make release          # Release official version
#
# Requirements:
# - Node.js >= 22 LTS
# - Rust 1.80+ (Edition 2024)
# - pnpm (recommended) or npm

# ============================================================================
# Configuration Variables
# ============================================================================

# Detect operating system
ifeq ($(OS),Windows_NT)
	DETECTED_OS := Windows
else
	UNAME_S := $(shell uname -s)
	ifeq ($(UNAME_S),Linux)
		DETECTED_OS := Linux
	endif
	ifeq ($(UNAME_S),Darwin)
		DETECTED_OS := macOS
	endif
endif

# Select script type and null device based on OS
ifeq ($(DETECTED_OS),Windows)
	SHELL_EXT := .ps1
	SCRIPT_RUNNER := powershell -ExecutionPolicy Bypass -File
	NULL_DEVICE := nul
	ECHO := @echo
else
	SHELL_EXT := .sh
	SCRIPT_RUNNER := bash
	NULL_DEVICE := /dev/null
	ECHO := @echo
endif

# Package manager
PKG_MANAGER := pnpm
ifeq ($(shell command -v pnpm 2> /dev/null),)
	PKG_MANAGER := npm
endif

# Paths
RUST_DIR := src-tauri
RUST_MANIFEST := $(RUST_DIR)/Cargo.toml
VERSION_SCRIPT := scripts/version$(SHELL_EXT)
CHECK_SCRIPT := scripts/check-commit$(SHELL_EXT)

# Color output (ANSI codes not needed on Windows PowerShell)
ifneq ($(DETECTED_OS),Windows)
	COLOR_RESET := \033[0m
	COLOR_BOLD := \033[1m
	COLOR_GREEN := \033[32m
	COLOR_YELLOW := \033[33m
	COLOR_BLUE := \033[34m
	COLOR_CYAN := \033[36m
else
	COLOR_RESET :=
	COLOR_BOLD :=
	COLOR_GREEN :=
	COLOR_YELLOW :=
	COLOR_BLUE :=
	COLOR_CYAN :=
endif

# ============================================================================
# Phony Targets
# ============================================================================

.PHONY: all dev build bundle
.PHONY: test test-rust test-frontend
.PHONY: fmt lint check pre-commit
.PHONY: clean deps install
.PHONY: snapshot-patch snapshot-minor snapshot-major release version-info
.PHONY: help info

# ============================================================================
# Default Target
# ============================================================================

all: check test build

# ============================================================================
# Development Commands
# ============================================================================

## dev: Start Tauri development mode (hot reload)
dev:
	$(ECHO) "Starting development mode..."
	$(PKG_MANAGER) run tauri dev

# ============================================================================
# Build Commands
# ============================================================================

## build: Build frontend production version
build:
	$(ECHO) "Building production version..."
	$(PKG_MANAGER) run build

## bundle: Build complete Tauri application package
bundle:
	$(ECHO) "Building Tauri application package..."
	$(PKG_MANAGER) run tauri build

# ============================================================================
# Dependency Management
# ============================================================================

## install: Install all dependencies
install: deps

## deps: Install frontend dependencies
deps:
	$(ECHO) "Installing dependencies..."
	$(PKG_MANAGER) install

# ============================================================================
# Test Commands
# ============================================================================

## test: Run all tests
test: test-rust test-frontend

## test-rust: Run Rust tests
test-rust:
	$(ECHO) "Running Rust tests..."
	@cargo test --manifest-path $(RUST_MANIFEST) --quiet

## test-frontend: Run frontend tests
test-frontend:
	$(ECHO) "Running frontend tests..."
	@$(PKG_MANAGER) run test:frontend

# ============================================================================
# Code Quality
# ============================================================================

## fmt: Format all code
fmt:
	$(ECHO) "Formatting code..."
	@cargo fmt --manifest-path $(RUST_MANIFEST)
	@$(PKG_MANAGER) run format

## lint: Run all linters
lint:
	$(ECHO) "Running code checks..."
	@cargo clippy --manifest-path $(RUST_MANIFEST) -- -D warnings
	@$(PKG_MANAGER) run lint

## check: Run all quality checks
check:
	$(ECHO) "Running quality checks..."
	@cargo fmt --manifest-path $(RUST_MANIFEST) -- --check
	@cargo clippy --manifest-path $(RUST_MANIFEST) -- -D warnings
	@$(PKG_MANAGER) run format:check
	@$(PKG_MANAGER) run lint
	@$(PKG_MANAGER) run typecheck

## pre-commit: Complete CI checks before commit (recommended)
pre-commit:
	$(ECHO) "Running pre-commit checks..."
	@$(SCRIPT_RUNNER) $(CHECK_SCRIPT)

# ============================================================================
# Version Management (SNAPSHOT)
# ============================================================================

## snapshot-patch: Create next patch SNAPSHOT version (0.1.0 -> 0.1.1-SNAPSHOT)
snapshot-patch:
	$(ECHO) "Creating patch SNAPSHOT version..."
	@$(SCRIPT_RUNNER) $(VERSION_SCRIPT) snapshot-patch

## snapshot-minor: Create next minor SNAPSHOT version (0.1.0 -> 0.2.0-SNAPSHOT)
snapshot-minor:
	$(ECHO) "Creating minor SNAPSHOT version..."
	@$(SCRIPT_RUNNER) $(VERSION_SCRIPT) snapshot-minor

## snapshot-major: Create next major SNAPSHOT version (0.1.0 -> 1.0.0-SNAPSHOT)
snapshot-major:
	$(ECHO) "Creating major SNAPSHOT version..."
	@$(SCRIPT_RUNNER) $(VERSION_SCRIPT) snapshot-major

## release: Release current SNAPSHOT as official version, tag and push to remote
release:
	$(ECHO) "Releasing official version..."
	@$(SCRIPT_RUNNER) $(VERSION_SCRIPT) release

## version-info: Display current version information
version-info:
	@$(SCRIPT_RUNNER) $(VERSION_SCRIPT) info

# ============================================================================
# Clean Commands
# ============================================================================

## clean: Clean build artifacts
clean:
	$(ECHO) "Cleaning build artifacts..."
	@cargo clean --manifest-path $(RUST_MANIFEST)
	@rm -rf dist node_modules/.vite

# ============================================================================
# Info Commands
# ============================================================================

## info: Display project information
info:
	$(ECHO) ""
	$(ECHO) "Bing Wallpaper Now - Project Info"
	$(ECHO) ""
	$(ECHO) "OS:           $(DETECTED_OS)"
	$(ECHO) "Pkg Manager: $(PKG_MANAGER)"
	@echo -n "Rust:        " && rustc --version 2>$(NULL_DEVICE) || echo "Not installed"
	@echo -n "Node.js:     " && node --version 2>$(NULL_DEVICE) || echo "Not installed"
	@echo -n "Version:     " && grep '"version"' package.json | head -1 | sed 's/.*"version": "\(.*\)".*/\1/' 2>$(NULL_DEVICE) || echo "Unknown"
	$(ECHO) ""

# ============================================================================
# Help Information
# ============================================================================

## help: Display help information
help:
	$(ECHO) ""
	$(ECHO) "Bing Wallpaper Now - Makefile Commands"
	$(ECHO) ""
	$(ECHO) "Development Commands:"
	$(ECHO) "  make dev              - Start development mode (hot reload)"
	$(ECHO) "  make build            - Build frontend production version"
	$(ECHO) "  make bundle           - Build Tauri application package"
	$(ECHO) ""
	$(ECHO) "Test Commands:"
	$(ECHO) "  make test             - Run all tests"
	$(ECHO) "  make test-rust        - Run Rust tests only"
	$(ECHO) "  make test-frontend    - Run frontend tests only"
	$(ECHO) ""
	$(ECHO) "Code Quality:"
	$(ECHO) "  make check            - Run all quality checks"
	$(ECHO) "  make pre-commit       - Complete pre-commit checks (recommended)"
	$(ECHO) "  make fmt              - Format all code"
	$(ECHO) "  make lint             - Run all linters"
	$(ECHO) ""
	$(ECHO) "Version Management (SNAPSHOT):"
	$(ECHO) "  make snapshot-patch    - Create patch SNAPSHOT (0.1.0 -> 0.1.1-SNAPSHOT)"
	$(ECHO) "  make snapshot-minor    - Create minor SNAPSHOT (0.1.0 -> 0.2.0-SNAPSHOT)"
	$(ECHO) "  make snapshot-major    - Create major SNAPSHOT (0.1.0 -> 1.0.0-SNAPSHOT)"
	$(ECHO) "  make release           - Release official version, tag and push"
	$(ECHO) "  make version-info      - Display current version information"
	$(ECHO) ""
	$(ECHO) "Other Commands:"
	$(ECHO) "  make install          - Install all dependencies"
	$(ECHO) "  make clean            - Clean build artifacts"
	$(ECHO) "  make info             - Display project information"
	$(ECHO) "  make help             - Display this help information"
	$(ECHO) ""
	$(ECHO) "Version Management Workflow:"
	$(ECHO) "  1. After releasing v0.1.0:"
	$(ECHO) "     make snapshot-patch  -> Create 0.1.1-SNAPSHOT for development"
	$(ECHO) ""
	$(ECHO) "  2. Develop new features..."
	$(ECHO) ""
	$(ECHO) "  3. Prepare for release:"
	$(ECHO) "     make pre-commit    -> Run all checks"
	$(ECHO) "     make release       -> Release 0.1.1 and push to remote"
	$(ECHO) "                          (Will ask to auto-create 0.1.2-SNAPSHOT)"
	$(ECHO) ""
	$(ECHO) "  4. GitHub Actions will auto-build and publish to Releases"
	$(ECHO) ""
