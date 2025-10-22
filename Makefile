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

# Package manager
PKG_MANAGER := pnpm
ifeq ($(shell command -v pnpm 2> /dev/null),)
	PKG_MANAGER := npm
endif

# Paths
RUST_DIR := src-tauri
RUST_MANIFEST := $(RUST_DIR)/Cargo.toml
VERSION_SCRIPT := scripts/version.sh
CHECK_SCRIPT := scripts/check-commit.sh

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
	@echo Starting development mode...
	$(PKG_MANAGER) run tauri dev

# ============================================================================
# Build Commands
# ============================================================================

## build: Build frontend production version
build:
	@echo Building production version...
	$(PKG_MANAGER) run build

## bundle: Build complete Tauri application package
bundle:
	@echo Building Tauri application package...
	$(PKG_MANAGER) run tauri build

# ============================================================================
# Dependency Management
# ============================================================================

## install: Install all dependencies
install: deps

## deps: Install frontend dependencies
deps:
	@echo Installing dependencies...
	$(PKG_MANAGER) install

# ============================================================================
# Test Commands
# ============================================================================

## test: Run all tests
test: test-rust test-frontend

## test-rust: Run Rust tests
test-rust:
	@echo Running Rust tests...
	@cargo test --manifest-path $(RUST_MANIFEST) --quiet

## test-frontend: Run frontend tests
test-frontend:
	@echo Running frontend tests...
	@$(PKG_MANAGER) run test:frontend

# ============================================================================
# Code Quality
# ============================================================================

## fmt: Format all code
fmt:
	@echo Formatting code...
	@cargo fmt --manifest-path $(RUST_MANIFEST)
	@$(PKG_MANAGER) run format

## lint: Run all linters
lint:
	@echo Running code checks...
	@cargo clippy --manifest-path $(RUST_MANIFEST) -- -D warnings
	@$(PKG_MANAGER) run lint

## check: Run all quality checks
check:
	@echo Running quality checks...
	@cargo fmt --manifest-path $(RUST_MANIFEST) -- --check
	@cargo clippy --manifest-path $(RUST_MANIFEST) -- -D warnings
	@$(PKG_MANAGER) run format:check
	@$(PKG_MANAGER) run lint
	@$(PKG_MANAGER) run typecheck

## pre-commit: Complete CI checks before commit (recommended)
pre-commit:
	@echo Running pre-commit checks...
	@bash $(CHECK_SCRIPT)

# ============================================================================
# Version Management (Development Versions)
# ============================================================================

## snapshot-patch: Create next patch development version (0.1.0 -> 0.1.1-0)
snapshot-patch:
	@echo Creating patch development version...
	@bash $(VERSION_SCRIPT) snapshot-patch

## snapshot-minor: Create next minor development version (0.1.0 -> 0.2.0-0)
snapshot-minor:
	@echo Creating minor development version...
	@bash $(VERSION_SCRIPT) snapshot-minor

## snapshot-major: Create next major development version (0.1.0 -> 1.0.0-0)
snapshot-major:
	@echo Creating major development version...
	@bash $(VERSION_SCRIPT) snapshot-major

## release: Release current development version as official version, tag and push to remote
release:
	@echo Releasing official version...
	@bash $(VERSION_SCRIPT) release

## version-info: Display current version information
version-info:
	@bash $(VERSION_SCRIPT) info

# ============================================================================
# Clean Commands
# ============================================================================

## clean: Clean build artifacts
clean:
	@echo Cleaning build artifacts...
	@cargo clean --manifest-path $(RUST_MANIFEST)
	@rm -rf dist node_modules/.vite

# ============================================================================
# Info Commands
# ============================================================================

## info: Display project information
info:
	@echo ""
	@echo "Bing Wallpaper Now - Project Info"
	@echo ""
	@echo "Pkg Manager: $(PKG_MANAGER)"
	@echo -n "Rust:        " && rustc --version 2>/dev/null || echo "Not installed"
	@echo -n "Node.js:     " && node --version 2>/dev/null || echo "Not installed"
	@echo -n "Version:     " && grep '"version"' package.json | head -1 | sed 's/.*"version": "\(.*\)".*/\1/' 2>/dev/null || echo "Unknown"
	@echo ""

# ============================================================================
# Help Information
# ============================================================================

## help: Display help information
help:
	@echo ""
	@echo "Bing Wallpaper Now - Makefile Commands"
	@echo ""
	@echo "Development Commands:"
	@echo "  make dev              - Start development mode (hot reload)"
	@echo "  make build            - Build frontend production version"
	@echo "  make bundle           - Build Tauri application package"
	@echo ""
	@echo "Test Commands:"
	@echo "  make test             - Run all tests"
	@echo "  make test-rust        - Run Rust tests only"
	@echo "  make test-frontend    - Run frontend tests only"
	@echo ""
	@echo "Code Quality:"
	@echo "  make check            - Run all quality checks"
	@echo "  make pre-commit       - Complete pre-commit checks (recommended)"
	@echo "  make fmt              - Format all code"
	@echo "  make lint             - Run all linters"
	@echo ""
	@echo "Version Management:"
	@echo "  make snapshot-patch   - Create patch development version (0.1.0 -> 0.1.1-0)"
	@echo "  make snapshot-minor   - Create minor development version (0.1.0 -> 0.2.0-0)"
	@echo "  make snapshot-major   - Create major development version (0.1.0 -> 1.0.0-0)"
	@echo "  make release          - Release official version, tag and push"
	@echo "  make version-info     - Display current version information"
	@echo ""
	@echo "Other Commands:"
	@echo "  make install          - Install all dependencies"
	@echo "  make clean            - Clean build artifacts"
	@echo "  make info             - Display project information"
	@echo "  make help             - Display this help information"
	@echo ""
	@echo "Version Management Workflow:"
	@echo "  1. After releasing v0.1.0:"
	@echo "     make snapshot-patch  -> Create 0.1.1-0 for development"
	@echo ""
	@echo "  2. Develop new features..."
	@echo ""
	@echo "  3. Prepare for release:"
	@echo "     make pre-commit    -> Run all checks"
	@echo "     make release       -> Release 0.1.1 and push to remote"
	@echo "                          (Will ask to auto-create 0.1.2-0)"
	@echo ""
	@echo "  4. GitHub Actions will auto-build and publish to Releases"
	@echo ""
