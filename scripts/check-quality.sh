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
print_step 1 7 "Rust Code Format Check (cargo fmt)"
print_separator
if validate_rust_format; then
    increment_passed
else
    increment_failed
fi

# Check 2: Rust Clippy
print_step 2 7 "Rust Clippy Check (cargo clippy)"
print_separator
if validate_rust_clippy; then
    increment_passed
else
    increment_failed
fi

# Check 3: Rust Tests
print_step 3 7 "Rust Unit Tests (cargo test)"
print_separator
if validate_rust_tests; then
    increment_passed
else
    increment_failed
fi

# Check 4: TypeScript Types
print_step 4 7 "TypeScript Type Check (tsc)"
print_separator
if validate_typescript_types "$PKG_MANAGER"; then
    increment_passed
else
    increment_failed
fi

# Check 5: ESLint
print_step 5 7 "ESLint Check (eslint)"
print_separator
if validate_eslint "$PKG_MANAGER"; then
    increment_passed
else
    increment_failed
fi

# Check 6: Prettier
print_step 6 7 "Prettier Format Check (prettier)"
print_separator
if validate_prettier "$PKG_MANAGER"; then
    increment_passed
else
    increment_failed
fi

# Check 7: Frontend Tests
print_step 7 7 "Frontend Unit Tests (vitest)"
print_separator
if validate_frontend_tests "$PKG_MANAGER"; then
    increment_passed
else
    increment_failed
fi

# ============================================================================
# Summary
# ============================================================================

print_counter_summary

if [[ $UI_COUNTER_FAILED -eq 0 ]]; then
    printf "${COLOR_GREEN}${COLOR_BOLD}✅ All code quality checks passed! 🎉${COLOR_RESET}\n\n"
    printf "${COLOR_GREEN}Safe to commit. Next steps:${COLOR_RESET}\n"
    printf "  ${COLOR_BLUE}git add .${COLOR_RESET}\n"
    printf "  ${COLOR_BLUE}git commit -m \"your message\"${COLOR_RESET}\n"
    printf "  ${COLOR_BLUE}git push${COLOR_RESET}\n\n"
    exit 0
else
    printf "${COLOR_RED}${COLOR_BOLD}❌ %d check(s) failed${COLOR_RESET}\n\n" $UI_COUNTER_FAILED
    printf "${COLOR_YELLOW}Quick fixes:${COLOR_RESET}\n"
    printf "  - Format: ${COLOR_BLUE}cargo fmt && $PKG_MANAGER run format${COLOR_RESET}\n"
    printf "  - Lint: ${COLOR_BLUE}$PKG_MANAGER run lint:fix${COLOR_RESET}\n"
    printf "  - Types: Fix according to tsc error messages\n"
    printf "  - Tests: Fix according to test output\n\n"
    exit 1
fi
