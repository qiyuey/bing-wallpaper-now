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
#   ./scripts/check-quality.sh            # with auto-fix on failure
#   ./scripts/check-quality.sh --no-fix   # strict mode, no auto-fix
#   make check                            # same as above (default)
#   make check NO_FIX=1                   # strict mode via Make

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

# Parse arguments: --no-fix disables auto-fix (used by release flow)
NO_FIX=""
for arg in "$@"; do
    case "$arg" in
        --no-fix) NO_FIX="1" ;;
    esac
done

# Detect package manager
PKG_MANAGER=$(project_detect_package_manager)

# Array to track failed checks
FAILED_CHECKS=()

# Variables to track checks that have been auto-fixed (to prevent infinite retry)
AUTO_FIXED_RUST_FORMAT=""
AUTO_FIXED_ESLINT=""
AUTO_FIXED_PRETTIER=""
AUTO_FIXED_MARKDOWN=""

# ============================================================================
# Header
# ============================================================================

print_major_header "Code Quality Checks"
printf "${COLOR_YELLOW}Running comprehensive quality checks (format, lint, types, tests)${COLOR_RESET}\n"
printf "${COLOR_YELLOW}Package Manager: ${PKG_MANAGER}${COLOR_RESET}\n"
if [[ -n "$NO_FIX" ]]; then
    printf "${COLOR_YELLOW}Mode: strict (no auto-fix)${COLOR_RESET}\n"
fi
echo ""

# ============================================================================
# Helper Functions
# ============================================================================

# Auto-fix Rust format
auto_fix_rust_format() {
    local manifest=$(project_get_file_path "$PROJECT_CARGO_TOML" 2>/dev/null || echo "src-tauri/Cargo.toml")
    printf "${COLOR_YELLOW}  → Auto-fixing Rust format...${COLOR_RESET}\n"
    if cargo fmt --manifest-path "$manifest"; then
        printf "${COLOR_GREEN}  ✓ Rust format auto-fixed${COLOR_RESET}\n"
        return 0
    else
        printf "${COLOR_RED}  ✗ Failed to auto-fix Rust format${COLOR_RESET}\n"
        return 1
    fi
}

# Auto-fix ESLint
auto_fix_eslint() {
    printf "${COLOR_YELLOW}  → Auto-fixing ESLint issues...${COLOR_RESET}\n"
    if $PKG_MANAGER run lint:fix; then
        printf "${COLOR_GREEN}  ✓ ESLint issues auto-fixed${COLOR_RESET}\n"
        return 0
    else
        printf "${COLOR_RED}  ✗ Failed to auto-fix ESLint issues${COLOR_RESET}\n"
        return 1
    fi
}

# Auto-fix Prettier
auto_fix_prettier() {
    printf "${COLOR_YELLOW}  → Auto-fixing Prettier format...${COLOR_RESET}\n"
    if $PKG_MANAGER run format; then
        printf "${COLOR_GREEN}  ✓ Prettier format auto-fixed${COLOR_RESET}\n"
        return 0
    else
        printf "${COLOR_RED}  ✗ Failed to auto-fix Prettier format${COLOR_RESET}\n"
        return 1
    fi
}

# Auto-fix Markdown lint
auto_fix_markdown() {
    printf "${COLOR_YELLOW}  → Auto-fixing Markdown issues...${COLOR_RESET}\n"
    if $PKG_MANAGER run lint:md:fix; then
        printf "${COLOR_GREEN}  ✓ Markdown issues auto-fixed${COLOR_RESET}\n"
        return 0
    else
        printf "${COLOR_RED}  ✗ Failed to auto-fix Markdown issues${COLOR_RESET}\n"
        return 1
    fi
}

# ============================================================================
# Run All Quality Checks
# ============================================================================

# Check 1: Rust Code Format
print_step 1 8 "Rust Code Format Check (cargo fmt)"
print_separator
if validate_rust_format; then
    increment_passed
elif [[ -n "$NO_FIX" ]]; then
    increment_failed
    FAILED_CHECKS+=("Rust Code Format (cargo fmt)")
else
    if [[ -z "$AUTO_FIXED_RUST_FORMAT" ]]; then
        AUTO_FIXED_RUST_FORMAT="1"
        auto_fix_rust_format
        printf "${COLOR_YELLOW}  → Re-checking after auto-fix...${COLOR_RESET}\n"
        if validate_rust_format; then
            increment_passed
        else
            increment_failed
            FAILED_CHECKS+=("Rust Code Format (cargo fmt)")
        fi
    else
        increment_failed
        FAILED_CHECKS+=("Rust Code Format (cargo fmt)")
    fi
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
elif [[ -n "$NO_FIX" ]]; then
    increment_failed
    FAILED_CHECKS+=("ESLint Check")
else
    if [[ -z "$AUTO_FIXED_ESLINT" ]]; then
        AUTO_FIXED_ESLINT="1"
        auto_fix_eslint
        printf "${COLOR_YELLOW}  → Re-checking after auto-fix...${COLOR_RESET}\n"
        if validate_eslint "$PKG_MANAGER"; then
            increment_passed
        else
            increment_failed
            FAILED_CHECKS+=("ESLint Check")
        fi
    else
        increment_failed
        FAILED_CHECKS+=("ESLint Check")
    fi
fi

# Check 6: Prettier
print_step 6 8 "Prettier Format Check (prettier)"
print_separator
if validate_prettier "$PKG_MANAGER"; then
    increment_passed
elif [[ -n "$NO_FIX" ]]; then
    increment_failed
    FAILED_CHECKS+=("Prettier Format Check")
else
    if [[ -z "$AUTO_FIXED_PRETTIER" ]]; then
        AUTO_FIXED_PRETTIER="1"
        auto_fix_prettier
        printf "${COLOR_YELLOW}  → Re-checking after auto-fix...${COLOR_RESET}\n"
        if validate_prettier "$PKG_MANAGER"; then
            increment_passed
        else
            increment_failed
            FAILED_CHECKS+=("Prettier Format Check")
        fi
    else
        increment_failed
        FAILED_CHECKS+=("Prettier Format Check")
    fi
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
elif [[ -n "$NO_FIX" ]]; then
    increment_failed
    FAILED_CHECKS+=("Markdown Lint Check")
else
    if [[ -z "$AUTO_FIXED_MARKDOWN" ]]; then
        AUTO_FIXED_MARKDOWN="1"
        auto_fix_markdown
        printf "${COLOR_YELLOW}  → Re-checking after auto-fix...${COLOR_RESET}\n"
        if validate_markdown_lint "$PKG_MANAGER"; then
            increment_passed
        else
            increment_failed
            FAILED_CHECKS+=("Markdown Lint Check")
        fi
    else
        increment_failed
        FAILED_CHECKS+=("Markdown Lint Check")
    fi
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
