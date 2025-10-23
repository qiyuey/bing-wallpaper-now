#!/usr/bin/env bash
# check-commit.sh - Pre-Commit Code Quality Checks
#
# This script runs quick code quality checks before commit:
# - Code formatting (Rust + TypeScript)
# - Linting (Clippy + ESLint)
# - Type checking (TypeScript)
# - Unit tests (Rust + Frontend)
#
# For complete pre-release validation, use: make pre-release
#
# Usage:
#   ./scripts/check-commit.sh
#   or make pre-commit

set -euo pipefail

# Color output
COLOR_RESET='\033[0m'
COLOR_BOLD='\033[1m'
COLOR_GREEN='\033[32m'
COLOR_YELLOW='\033[33m'
COLOR_BLUE='\033[34m'
COLOR_RED='\033[31m'

# Check counters
CHECKS_PASSED=0
CHECKS_FAILED=0

# Print separator
print_separator() {
    printf "${COLOR_BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${COLOR_RESET}\n"
}

# Print check title
print_check() {
    printf "\n${COLOR_BOLD}${COLOR_BLUE}🔍 $1${COLOR_RESET}\n"
    print_separator
}

# Print success message
print_success() {
    printf "${COLOR_GREEN}✅ $1${COLOR_RESET}\n"
    CHECKS_PASSED=$((CHECKS_PASSED + 1))
}

# Print error message
print_error() {
    printf "${COLOR_RED}❌ $1${COLOR_RESET}\n"
    CHECKS_FAILED=$((CHECKS_FAILED + 1))
}

# Print warning message
print_warning() {
    printf "${COLOR_YELLOW}⚠️  $1${COLOR_RESET}\n"
}

# Detect package manager
detect_package_manager() {
    if command -v pnpm &> /dev/null; then
        echo "pnpm"
    elif command -v npm &> /dev/null; then
        echo "npm"
    else
        print_error "npm or pnpm not found"
        exit 1
    fi
}

PKG_MANAGER=$(detect_package_manager)

printf "${COLOR_BOLD}${COLOR_BLUE}╔══════════════════════════════════════════════════════════════╗${COLOR_RESET}\n"
printf "${COLOR_BOLD}${COLOR_BLUE}║     Bing Wallpaper Now - Code Quality Checks                ║${COLOR_RESET}\n"
printf "${COLOR_BOLD}${COLOR_BLUE}╚══════════════════════════════════════════════════════════════╝${COLOR_RESET}\n\n"

printf "${COLOR_YELLOW}Quick code quality checks (format, lint, types, tests)${COLOR_RESET}\n"
printf "${COLOR_YELLOW}Package Manager: ${PKG_MANAGER}${COLOR_RESET}\n\n"

# ============================================================================
# 1. Rust Code Format Check
# ============================================================================
print_check "1/8 Rust Code Format Check (cargo fmt)"

if cargo fmt --manifest-path src-tauri/Cargo.toml -- --check; then
    print_success "Rust code format correct"
else
    print_error "Rust code format incorrect, please run: cargo fmt --manifest-path src-tauri/Cargo.toml"
fi

# ============================================================================
# 2. Rust Clippy Check
# ============================================================================
print_check "2/8 Rust Clippy Check (cargo clippy)"

if cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings; then
    print_success "Clippy check passed"
else
    print_error "Clippy check failed, please fix Rust code issues"
fi

# ============================================================================
# 3. Rust Tests
# ============================================================================
print_check "3/8 Rust Unit Tests (cargo test)"

if cargo test --manifest-path src-tauri/Cargo.toml --quiet; then
    print_success "Rust tests passed"
else
    print_error "Rust tests failed, please fix test issues"
fi

# ============================================================================
# 4. TypeScript Type Check
# ============================================================================
print_check "4/8 TypeScript Type Check (tsc)"

if $PKG_MANAGER run typecheck; then
    print_success "TypeScript type check passed"
else
    print_error "TypeScript type check failed, please fix type errors"
fi

# ============================================================================
# 5. ESLint Check
# ============================================================================
print_check "5/8 ESLint Check (eslint)"

if $PKG_MANAGER run lint; then
    print_success "ESLint check passed"
else
    print_error "ESLint check failed, please run: $PKG_MANAGER run lint:fix"
fi

# ============================================================================
# 6. Prettier Format Check
# ============================================================================
print_check "6/8 Prettier Format Check (prettier)"

if $PKG_MANAGER run format:check; then
    print_success "Prettier format check passed"
else
    print_error "Prettier format check failed, please run: $PKG_MANAGER run format"
fi

# ============================================================================
# 7. Frontend Tests
# ============================================================================
print_check "7/7 Frontend Unit Tests (vitest)"

if $PKG_MANAGER run test:frontend; then
    print_success "Frontend tests passed"
else
    print_error "Frontend tests failed, please fix test issues"
fi

# ============================================================================
# Summary Results
# ============================================================================
printf "\n"
print_separator
printf "${COLOR_BOLD}Check Summary:${COLOR_RESET}\n"
print_separator

TOTAL_CHECKS=$((CHECKS_PASSED + CHECKS_FAILED))
printf "Total Checks: ${COLOR_BOLD}%d${COLOR_RESET}\n" $TOTAL_CHECKS
printf "Passed: ${COLOR_GREEN}${COLOR_BOLD}%d${COLOR_RESET}\n" $CHECKS_PASSED
printf "Failed: ${COLOR_RED}${COLOR_BOLD}%d${COLOR_RESET}\n" $CHECKS_FAILED

if [ $CHECKS_FAILED -eq 0 ]; then
    printf "\n${COLOR_GREEN}${COLOR_BOLD}✅ All code quality checks passed! 🎉${COLOR_RESET}\n"
    printf "${COLOR_GREEN}Safe to commit. Next steps:${COLOR_RESET}\n"
    printf "  ${COLOR_BLUE}git add .${COLOR_RESET}\n"
    printf "  ${COLOR_BLUE}git commit -m \"your message\"${COLOR_RESET}\n"
    printf "  ${COLOR_BLUE}git push${COLOR_RESET}\n\n"
    printf "${COLOR_YELLOW}Note: Before release, run ${COLOR_BLUE}make pre-release${COLOR_RESET} for complete validation${COLOR_RESET}\n\n"
    exit 0
else
    printf "\n${COLOR_RED}${COLOR_BOLD}❌ %d check(s) failed${COLOR_RESET}\n" $CHECKS_FAILED
    printf "${COLOR_YELLOW}Quick fixes:${COLOR_RESET}\n"
    printf "  - Format: ${COLOR_BLUE}cargo fmt && $PKG_MANAGER run format${COLOR_RESET}\n"
    printf "  - Lint: ${COLOR_BLUE}$PKG_MANAGER run lint:fix${COLOR_RESET}\n"
    printf "  - Types: Fix according to tsc error messages\n"
    printf "  - Tests: Fix according to test output\n\n"
    exit 1
fi
