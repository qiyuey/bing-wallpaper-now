# Makefile - Bing Wallpaper Now
#
# Quick Reference:
#   make dev              # Start development mode
#   make check            # Run quality checks
#   make fix              # Auto-fix formatting and lint issues
#   make package          # Build app packages (optional: BUNDLES=dmg)
#   make patch            # Create patch dev version
#   make release          # Release official version
#
# Requirements:
# - Node.js >= 24 LTS
# - Rust 1.80+ (Edition 2024)
# - pnpm 10+ (required, npm is not supported)

# ============================================================================
# Configuration Variables
# ============================================================================

# Package manager (pnpm only, npm is not supported)
PKG_MANAGER := pnpm

# Verify pnpm is available
ifeq ($(shell command -v pnpm 2> /dev/null),)
$(error pnpm is required but not found. Install it via: corepack enable && corepack prepare pnpm@latest --activate)
endif

# Paths
VERSION_SCRIPT := scripts/manage-version.sh
CHECK_SCRIPT := scripts/check-quality.sh

# ============================================================================
# Phony Targets
# ============================================================================

.PHONY: all dev check fix test
.PHONY: build package
.PHONY: clean clean-all deps install
.PHONY: patch minor major release retag
.PHONY: help info

# ============================================================================
# Default Target
# ============================================================================

.DEFAULT_GOAL := help

all: check

# ============================================================================
# Development Commands
# ============================================================================

##@ Development
## dev: Start Tauri development mode (hot reload)
##      Pass MV=x.y.z to mock version for update testing
dev:
	@echo Starting development mode...
	$(if $(MV),@echo "Mock version: $(MV) (via DEV_OVERRIDE_VERSION)")
	$(if $(MV),DEV_OVERRIDE_VERSION=$(MV)) $(PKG_MANAGER) run tauri dev

# ============================================================================
# Dependency Management
# ============================================================================

##@ Dependencies
## install: Install all dependencies
install: deps

## deps: Install frontend dependencies
deps:
	@echo Installing dependencies...
	$(PKG_MANAGER) install

# ============================================================================
# Code Quality
# ============================================================================

##@ Quality
## check: Run all quality checks (format, lint, types, tests)
##        Pass NO_FIX=1 to disable auto-fix (used by release flow)
check:
	@bash $(CHECK_SCRIPT) $(if $(NO_FIX),--no-fix)

## fix: Auto-fix all formatting and lint issues
fix:
	@echo "Auto-fixing all formatting and lint issues..."
	cargo fmt --manifest-path src-tauri/Cargo.toml
	$(PKG_MANAGER) run lint:fix
	$(PKG_MANAGER) run format
	$(PKG_MANAGER) run lint:md:fix
	@echo ""
	@echo "Done. Run 'make check' to verify."

## test: Run all tests (Rust + Frontend)
test:
	$(PKG_MANAGER) test

# ============================================================================
# Version Management
# ============================================================================

##@ Version Management
## patch: Create next patch development version (0.1.0 -> 0.1.1-0)
##        Pass YES=1 to skip confirmation prompt
patch:
	@YES=$(YES) bash $(VERSION_SCRIPT) patch

## minor: Create next minor development version (0.1.0 -> 0.2.0-0)
##        Pass YES=1 to skip confirmation prompt
minor:
	@YES=$(YES) bash $(VERSION_SCRIPT) minor

## major: Create next major development version (0.1.0 -> 1.0.0-0)
##        Pass YES=1 to skip confirmation prompt
major:
	@YES=$(YES) bash $(VERSION_SCRIPT) major

## release: Release current development version (update version, commit, create tag, push)
##         Tags are created on the version update commit (chore(release): X.Y.Z), not on CHANGELOG commit
release:
	@bash $(VERSION_SCRIPT) release

## retag: Re-push current version tag to re-trigger CI/CD build
retag:
	@bash $(VERSION_SCRIPT) retag

# ============================================================================
# Packaging Commands
# ============================================================================

##@ Build & Packaging
## build: Build frontend (TypeScript compile + Vite build)
build:
	$(PKG_MANAGER) run build

## package: Build release packages (optional: BUNDLES=dmg|app|nsis|appimage|deb|rpm)
package:
	@echo "Building release package(s)..."
	@if [ -n "$(BUNDLES)" ]; then \
		echo "Using bundle filter: $(BUNDLES)"; \
		$(PKG_MANAGER) run tauri build -- --bundles $(BUNDLES); \
	else \
		$(PKG_MANAGER) run tauri build; \
	fi

# Example (macOS): make package BUNDLES=dmg

# ============================================================================
# Clean Commands
# ============================================================================

##@ Other
## clean: Clean build artifacts
clean:
	@echo Cleaning build artifacts...
	@rm -rf dist node_modules/.vite

## clean-all: Deep clean including Rust build artifacts
clean-all: clean
	@echo Deep cleaning Rust artifacts...
	@cd src-tauri && cargo clean

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
	@echo -n "Version:     " && node -p "require('./package.json').version" 2>/dev/null || echo "Unknown"
	@echo ""

# ============================================================================
# Help Information
# ============================================================================

## help: Display help information
help:
	@echo ""
	@echo "Bing Wallpaper Now - Makefile Commands"
	@echo ""
	@awk '/^##@ / { \
		group=$$0; \
		sub(/^##@ /, "", group); \
		printf "\n%s:\n", group; \
		next; \
	} \
	/^## [A-Za-z0-9_.-]+:[[:space:]]*/ { \
		line=$$0; \
		sub(/^## /, "", line); \
		target=line; \
		sub(/:.*/, "", target); \
		desc=line; \
		sub(/^[^:]+:[[:space:]]*/, "", desc); \
		printf "  make %-14s %s\n", target, desc; \
	}' $(lastword $(MAKEFILE_LIST))
	@echo ""
	@echo "Workflow:"
	@echo "  1. After release, create development version:"
	@echo "     make patch"
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
