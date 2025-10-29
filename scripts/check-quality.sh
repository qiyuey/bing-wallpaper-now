#!/usr/bin/env bash
# check-quality.sh - Pre-Commit Code Quality Checks
#
# Runs comprehensive code quality checks before commit:
# - Code formatting (Rust + TypeScript)
# - Linting (Clippy + ESLint)
# - Type checking (TypeScript)
# - Unit tests (Rust + Frontend)
#
# Usage:
#   ./scripts/check-quality.sh
#   make pre-commit

set -euo pipefail

# ============================================================================
# Load Library Functions
# ============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

source "$SCRIPT_DIR/lib/ui.sh"
source "$SCRIPT_DIR/lib/project.sh"
source "$SCRIPT_DIR/lib/validators.sh"

# ============================================================================
# Initialize
# ============================================================================

init_ui
init_counters

# Detect package manager
PKG_MANAGER=$(project_detect_package_manager)

# Array to track failed checks
FAILED_CHECKS=()

# ============================================================================
# Header
# ============================================================================

print_major_header "Code Quality Checks"
printf "${COLOR_YELLOW}Running comprehensive quality checks (format, lint, types, tests)${COLOR_RESET}\n"
printf "${COLOR_YELLOW}Package Manager: ${PKG_MANAGER}${COLOR_RESET}\n\n"

# ============================================================================
# Run All Quality Checks
# ============================================================================

# Check 1: Rust Code Format
print_step 1 8 "Rust Code Format Check (cargo fmt)"
print_separator
if validate_rust_format; then
    increment_passed
else
    increment_failed
    FAILED_CHECKS+=("Rust Code Format (cargo fmt)")
fi

# Check 2: Rust Clippy
print_step 2 8 "Rust Clippy Check (cargo clippy)"
print_separator
if validate_rust_clippy; then
    increment_passed
else
    increment_failed
    FAILED_CHECKS+=("Rust Clippy (cargo clippy)")
fi

# Check 3: Rust Tests
print_step 3 8 "Rust Unit Tests (cargo test)"
print_separator
if validate_rust_tests; then
    increment_passed
else
    increment_failed
    FAILED_CHECKS+=("Rust Unit Tests (cargo test)")
fi

# Check 4: TypeScript Types
print_step 4 8 "TypeScript Type Check (tsc)"
print_separator
if validate_typescript_types "$PKG_MANAGER"; then
    increment_passed
else
    increment_failed
    FAILED_CHECKS+=("TypeScript Type Check (tsc)")
fi

# Check 5: ESLint
print_step 5 8 "ESLint Check (eslint)"
print_separator
if validate_eslint "$PKG_MANAGER"; then
    increment_passed
else
    increment_failed
    FAILED_CHECKS+=("ESLint Check")
fi

# Check 6: Prettier
print_step 6 8 "Prettier Format Check (prettier)"
print_separator
if validate_prettier "$PKG_MANAGER"; then
    increment_passed
else
    increment_failed
    FAILED_CHECKS+=("Prettier Format Check")
fi

# Check 7: Frontend Tests
print_step 7 8 "Frontend Unit Tests (vitest)"
print_separator
if validate_frontend_tests "$PKG_MANAGER"; then
    increment_passed
else
    increment_failed
    FAILED_CHECKS+=("Frontend Unit Tests (vitest)")
fi

# Check 8: Markdown Lint
print_step 8 8 "Markdown Lint Check (markdownlint)"
print_separator
if validate_markdown_lint "$PKG_MANAGER"; then
    increment_passed
else
    increment_failed
    FAILED_CHECKS+=("Markdown Lint Check")
fi

# ============================================================================
# Summary
# ============================================================================

print_counter_summary

if [[ $UI_COUNTER_FAILED -eq 0 ]]; then
    printf "${COLOR_GREEN}${COLOR_BOLD}✅ All code quality checks passed! 🎉${COLOR_RESET}\n\n"
    exit 0
else
    printf "${COLOR_RED}${COLOR_BOLD}❌ %d check(s) failed${COLOR_RESET}\n\n" $UI_COUNTER_FAILED

    # List all failed checks
    printf "${COLOR_RED}${COLOR_BOLD}Failed checks:${COLOR_RESET}\n"
    for check in "${FAILED_CHECKS[@]}"; do
        printf "  ${COLOR_RED}✗ %s${COLOR_RESET}\n" "$check"
    done
    exit 1
fi
