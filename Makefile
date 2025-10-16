# Makefile - Unified local developer commands for Bing Wallpaper Now
#
# Usage examples:
#   make dev            # Run front-end dev server (Vite)
#   make tauri-dev      # Run Tauri dev (frontend + backend)
#   make build          # Type-check and build frontend assets
#   make test           # Run Rust tests
#   make ci             # Run local CI (typecheck + build + rust tests)
#   make bundle         # Build production Tauri bundles
#   make clean          # Remove build artifacts
#
# Notes:
# - Targets are phony to avoid conflicts with files of same name.
# - Ensure Node.js (>=18) and Rust toolchain are installed.
# - On Linux, Tauri may require system packages:
#     sudo apt-get install -y libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev patchelf

# -------------------------------------------------------------------
# Configuration
# -------------------------------------------------------------------

FRONTEND_DIR := .
RUST_MANIFEST := src-tauri/Cargo.toml

# -------------------------------------------------------------------
# Phony targets
# -------------------------------------------------------------------

.PHONY: dev tauri-dev build typecheck test rust-test ci bundle clean format help

# -------------------------------------------------------------------
# Frontend / Dev
# -------------------------------------------------------------------

dev:
	@echo "[dev] Starting Vite development server..."
	npm run dev

tauri-dev:
	@echo "[tauri-dev] Starting Tauri development (frontend + Rust backend)..."
	npm run tauri dev

# -------------------------------------------------------------------
# Build / Type Check
# -------------------------------------------------------------------

typecheck:
	@echo "[typecheck] Running TypeScript no-emit check..."
	npm run typecheck

build:
	@echo "[build] Building frontend (typecheck + vite build)..."
	npm run build

# -------------------------------------------------------------------
# Testing
# -------------------------------------------------------------------

test: rust-test

rust-test:
	@echo "[rust-test] Running Rust tests..."
	cargo test --manifest-path $(RUST_MANIFEST) -- --nocapture

# Combined local CI sequence
ci:
	@echo "[ci] Running local CI pipeline (typecheck + build + rust tests)..."
	npm run ci

# -------------------------------------------------------------------
# Tauri Bundle
# -------------------------------------------------------------------

bundle:
	@echo "[bundle] Building Tauri release bundles..."
	npm run tauri build

# -------------------------------------------------------------------
# Utilities
# -------------------------------------------------------------------

format:
	@echo "[format] (Placeholder) Add prettier/clippy/eslint format commands here"
	@echo "Skipping (no formatter configured)."

fmt-check:
	@echo "[fmt-check] Verifying Rust formatting..."
	cargo fmt --manifest-path $(RUST_MANIFEST) -- --check

lint-rust:
	@echo "[lint-rust] Running clippy lints (deny warnings)..."
	cargo clippy --manifest-path $(RUST_MANIFEST) -- -D warnings

check:
	@echo "[check] Full quality gate (typecheck + fmt + clippy + tests)"
	npm run typecheck
	make fmt-check
	make lint-rust
	make rust-test

clean:
	@echo "[clean] Removing dist and Rust target artifacts..."
	rm -rf dist
	cargo clean --manifest-path $(RUST_MANIFEST)

# -------------------------------------------------------------------
# Help
# -------------------------------------------------------------------

help:
	@echo "Available targets:"
	@echo "  dev          - Run Vite dev server"
	@echo "  tauri-dev    - Run Tauri dev (frontend + backend)"
	@echo "  typecheck    - TypeScript no-emit type checking"
	@echo "  build        - Type-check and build frontend"
	@echo "  test         - Run Rust tests (alias to rust-test)"
	@echo "  rust-test    - Run Rust tests only"
	@echo "  ci           - Run local CI pipeline (typecheck + build + tests)"
	@echo "  bundle       - Build Tauri bundles for release"
	@echo "  format       - Placeholder for code formatting"
	@echo "  fmt-check    - Check Rust formatting"
	@echo "  lint-rust    - Run clippy (deny warnings)"
	@echo "  check        - Run full quality gate"
	@echo "  clean        - Clean build artifacts"
	@echo "  help         - Show this help message"
	@echo ""
	@echo "Example: make check"
