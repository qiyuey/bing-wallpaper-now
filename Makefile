# Makefile - Bing Wallpaper Now
#
# Quick Reference:
#   make dev              # Start development mode
#   make check            # Run quality checks
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
VERSION_SCRIPT := scripts/manage-version.sh
CHECK_SCRIPT := scripts/check-quality.sh

# ============================================================================
# Phony Targets
# ============================================================================

.PHONY: all dev check
.PHONY: clean deps install
.PHONY: snapshot-patch snapshot-minor snapshot-major release
.PHONY: help info

# ============================================================================
# Default Target
# ============================================================================

all: check

# ============================================================================
# Development Commands
# ============================================================================

## dev: Start Tauri development mode (hot reload)
dev:
	@echo Starting development mode...
	$(PKG_MANAGER) run tauri dev

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
# Code Quality
# ============================================================================

## check: Run all quality checks (format, lint, types, tests)
check:
	@bash $(CHECK_SCRIPT)

# ============================================================================
# Version Management
# ============================================================================

## snapshot-patch: Create next patch development version (0.1.0 -> 0.1.1-0)
snapshot-patch:
	@bash $(VERSION_SCRIPT) snapshot-patch

## snapshot-minor: Create next minor development version (0.1.0 -> 0.2.0-0)
snapshot-minor:
	@bash $(VERSION_SCRIPT) snapshot-minor

## snapshot-major: Create next major development version (0.1.0 -> 1.0.0-0)
snapshot-major:
	@bash $(VERSION_SCRIPT) snapshot-major

## release: Release current development version, tag and push (fully automated)
release:
	@bash $(VERSION_SCRIPT) release

# ============================================================================
# Clean Commands
# ============================================================================

## clean: Clean build artifacts
clean:
	@echo Cleaning build artifacts...
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
	@echo "Development:"
	@echo "  make dev              - Start development mode (hot reload)"
	@echo "  make install          - Install all dependencies"
	@echo ""
	@echo "Quality:"
	@echo "  make check            - Run all quality checks (recommended before commit)"
	@echo ""
	@echo "Version Management:"
	@echo "  make snapshot-patch   - Create patch development version (0.1.0 -> 0.1.1-0)"
	@echo "  make snapshot-minor   - Create minor development version (0.1.0 -> 0.2.0-0)"
	@echo "  make snapshot-major   - Create major development version (0.1.0 -> 1.0.0-0)"
	@echo "  make release          - Release version, tag and push (fully automated)"
	@echo ""
	@echo "Other:"
	@echo "  make clean            - Clean build artifacts"
	@echo "  make info             - Display project information"
	@echo "  make help             - Display this help"
	@echo ""
	@echo "Workflow:"
	@echo "  1. After release, create development version:"
	@echo "     make snapshot-patch"
	@echo ""
	@echo "  2. Develop new features..."
	@echo ""
	@echo "  3. Before commit, run quality checks:"
	@echo "     make check"
	@echo ""
	@echo "  4. Ready to release:"
	@echo "     make release"
	@echo ""
	@echo "  (GitHub Actions will auto-build and publish)"
	@echo ""
