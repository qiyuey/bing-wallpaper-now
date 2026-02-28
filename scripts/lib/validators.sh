#!/usr/bin/env bash
# validators.sh - Validation Utilities
#
# Provides reusable validation functions:
# - CHANGELOG validation
# - Version format validation
# - Tag format validation
# - Code quality checks
#
# Usage:
#   source "$(dirname "${BASH_SOURCE[0]}")/lib/validators.sh"
#   if validate_changelog_has_version "1.2.3"; then
#       echo "CHANGELOG is up to date"
#   fi

# ============================================================================
# CHANGELOG Validation
# ============================================================================

# Check if CHANGELOG.md exists
# Returns: 0 if exists, 1 otherwise
# Usage: if validate_changelog_exists; then ... fi
validate_changelog_exists() {
    local changelog=$(project_get_file_path "$PROJECT_CHANGELOG" 2>/dev/null || echo "CHANGELOG.md")

    if [[ ! -f "$changelog" ]]; then
        if type print_error &>/dev/null; then
            print_error "CHANGELOG.md not found"
        fi
        return 1
    fi

    return 0
}

# Check if CHANGELOG has entry for specific version
# Args: $1 - version string, $2 - quiet mode (optional, default: false)
# Returns: 0 if entry exists, 1 otherwise
# Usage: if validate_changelog_has_version "1.2.3"; then ... fi
validate_changelog_has_version() {
    local version="$1"
    local quiet="${2:-false}"

    if ! validate_changelog_exists; then
        return 1
    fi

    local changelog=$(project_get_file_path "$PROJECT_CHANGELOG" 2>/dev/null || echo "CHANGELOG.md")

    # Check for version entry: ## X.Y.Z
    if ! grep -q "^## $version" "$changelog"; then
        if [[ "$quiet" != "true" ]]; then
            if type print_error &>/dev/null; then
                print_error "Version $version not found in CHANGELOG.md"
                print_info "Please add the following to CHANGELOG.md:"
                echo ""
                echo "  ## $version"
                echo ""
                echo "  ### Added"
                echo "  - New feature 1"
                echo ""
                echo "  ### Changed"
                echo "  - Changed item 1"
                echo ""
                echo "  ### Fixed"
                echo "  - Bug fix 1"
                echo ""
            fi
        fi
        return 1
    fi

    if [[ "$quiet" != "true" ]]; then
        if type print_success &>/dev/null; then
            print_success "CHANGELOG entry found for version $version"
        fi
    fi

    return 0
}

# Validate CHANGELOG for current or specified version
# Args: $1 - version (optional, uses project version if not specified)
# Returns: 0 if valid, 1 otherwise
# Usage: validate_changelog "1.2.3"
validate_changelog() {
    local target_version="${1:-}"

    # If no version specified, use current version from project
    if [[ -z "$target_version" ]]; then
        if type project_get_version &>/dev/null; then
            local current_version=$(project_get_version)

            # If development version, check for release version
            if type version_is_dev &>/dev/null && version_is_dev "$current_version"; then
                local release_version=$(version_remove_dev_suffix "$current_version")

                if type print_info &>/dev/null; then
                    print_info "Current version: $current_version (development)"
                    print_info "Validating CHANGELOG for release version: $release_version"
                fi

                validate_changelog_has_version "$release_version"
                return $?
            else
                if type print_info &>/dev/null; then
                    print_info "Current version: $current_version (release)"
                fi

                validate_changelog_has_version "$current_version"
                return $?
            fi
        else
            if type print_error &>/dev/null; then
                print_error "Cannot determine version"
            fi
            return 1
        fi
    else
        # Validate specified version
        if type print_info &>/dev/null; then
            print_info "Validating CHANGELOG for version: $target_version"
        fi
        validate_changelog_has_version "$target_version"
        return $?
    fi
}

# ============================================================================
# Tag Format Validation
# ============================================================================

# Validate tag format (must not start with 'v')
# Args: $1 - tag name
# Returns: 0 if valid, 1 otherwise
# Usage: if validate_tag_format "1.0.0"; then ... fi
validate_tag_format() {
    local tag="$1"

    if [[ $tag == v* ]]; then
        if type print_error &>/dev/null; then
            print_error "Tag format error: tag must not start with 'v'"
            print_error "Expected: ${tag#v} (without 'v' prefix)"
            print_info "Project uses tags like: 0.1.0, 0.1.1, 0.1.2"
            print_info "Not: v0.1.0, v0.1.1, v0.1.2"
        fi
        return 1
    fi

    # Check if tag is valid version format
    if type version_is_valid &>/dev/null && ! version_is_valid "$tag"; then
        if type print_error &>/dev/null; then
            print_error "Tag format error: '$tag' is not a valid version"
            print_info "Expected format: X.Y.Z (e.g., 1.0.0, 0.1.9)"
        fi
        return 1
    fi

    if type print_success &>/dev/null; then
        print_success "Tag format validation passed: $tag"
    fi

    return 0
}

# ============================================================================
# Code Quality Validators
# ============================================================================

# Run Rust format check
# Returns: 0 if formatted correctly, 1 otherwise
# Usage: if validate_rust_format; then ... fi
validate_rust_format() {
    local manifest=$(project_get_file_path "$PROJECT_CARGO_TOML" 2>/dev/null || echo "src-tauri/Cargo.toml")

    if type print_info &>/dev/null; then
        print_info "Checking Rust code format..."
    fi

    if cargo fmt --manifest-path "$manifest" -- --check; then
        if type print_success &>/dev/null; then
            print_success "Rust code format correct"
        fi
        return 0
    else
        if type print_error &>/dev/null; then
            print_error "Rust code format incorrect"
            print_info "Run: cargo fmt --manifest-path $manifest"
        fi
        return 1
    fi
}

# Run Rust clippy check
# Returns: 0 if passed, 1 otherwise
# Usage: if validate_rust_clippy; then ... fi
validate_rust_clippy() {
    local manifest=$(project_get_file_path "$PROJECT_CARGO_TOML" 2>/dev/null || echo "src-tauri/Cargo.toml")

    if type print_info &>/dev/null; then
        print_info "Running Clippy checks..."
    fi

    if cargo clippy --manifest-path "$manifest" -- -D warnings; then
        if type print_success &>/dev/null; then
            print_success "Clippy checks passed"
        fi
        return 0
    else
        if type print_error &>/dev/null; then
            print_error "Clippy checks failed"
        fi
        return 1
    fi
}

# Run Rust tests
# Returns: 0 if passed, 1 otherwise
# Usage: if validate_rust_tests; then ... fi
validate_rust_tests() {
    local manifest=$(project_get_file_path "$PROJECT_CARGO_TOML" 2>/dev/null || echo "src-tauri/Cargo.toml")

    if type print_info &>/dev/null; then
        print_info "Running Rust tests..."
    fi

    if cargo test --manifest-path "$manifest" --quiet; then
        if type print_success &>/dev/null; then
            print_success "Rust tests passed"
        fi
        return 0
    else
        if type print_error &>/dev/null; then
            print_error "Rust tests failed"
        fi
        return 1
    fi
}

# Run TypeScript type check
# Args: $1 - package manager (optional)
# Returns: 0 if passed, 1 otherwise
# Usage: if validate_typescript_types "$PKG_MANAGER"; then ... fi
validate_typescript_types() {
    local pkg_manager="${1:-}"

    if [[ -z "$pkg_manager" ]]; then
        pkg_manager=$(project_detect_package_manager 2>/dev/null || echo "pnpm")
    fi

    if type print_info &>/dev/null; then
        print_info "Running TypeScript type check..."
    fi

    if $pkg_manager run typecheck; then
        if type print_success &>/dev/null; then
            print_success "TypeScript type check passed"
        fi
        return 0
    else
        if type print_error &>/dev/null; then
            print_error "TypeScript type check failed"
        fi
        return 1
    fi
}

# Run ESLint check
# Args: $1 - package manager (optional)
# Returns: 0 if passed, 1 otherwise
# Usage: if validate_eslint "$PKG_MANAGER"; then ... fi
validate_eslint() {
    local pkg_manager="${1:-}"

    if [[ -z "$pkg_manager" ]]; then
        pkg_manager=$(project_detect_package_manager 2>/dev/null || echo "pnpm")
    fi

    if type print_info &>/dev/null; then
        print_info "Running ESLint..."
    fi

    if $pkg_manager run lint; then
        if type print_success &>/dev/null; then
            print_success "ESLint checks passed"
        fi
        return 0
    else
        if type print_error &>/dev/null; then
            print_error "ESLint checks failed"
            print_info "Run: $pkg_manager run lint:fix"
        fi
        return 1
    fi
}

# Run Prettier format check
# Args: $1 - package manager (optional)
# Returns: 0 if passed, 1 otherwise
# Usage: if validate_prettier "$PKG_MANAGER"; then ... fi
validate_prettier() {
    local pkg_manager="${1:-}"

    if [[ -z "$pkg_manager" ]]; then
        pkg_manager=$(project_detect_package_manager 2>/dev/null || echo "pnpm")
    fi

    if type print_info &>/dev/null; then
        print_info "Running Prettier format check..."
    fi

    if $pkg_manager run format:check; then
        if type print_success &>/dev/null; then
            print_success "Prettier format check passed"
        fi
        return 0
    else
        if type print_error &>/dev/null; then
            print_error "Prettier format check failed"
            print_info "Run: $pkg_manager run format"
        fi
        return 1
    fi
}

# Run frontend tests
# Args: $1 - package manager (optional)
# Returns: 0 if passed, 1 otherwise
# Usage: if validate_frontend_tests "$PKG_MANAGER"; then ... fi
validate_frontend_tests() {
    local pkg_manager="${1:-}"

    if [[ -z "$pkg_manager" ]]; then
        pkg_manager=$(project_detect_package_manager 2>/dev/null || echo "pnpm")
    fi

    if type print_info &>/dev/null; then
        print_info "Running frontend tests..."
    fi

    if $pkg_manager run test:frontend; then
        if type print_success &>/dev/null; then
            print_success "Frontend tests passed"
        fi
        return 0
    else
        if type print_error &>/dev/null; then
            print_error "Frontend tests failed"
        fi
        return 1
    fi
}

# Run Markdown lint check
# Args: $1 - package manager (optional)
# Returns: 0 if passed, 1 otherwise
# Usage: if validate_markdown_lint "$PKG_MANAGER"; then ... fi
validate_markdown_lint() {
    local pkg_manager="${1:-}"

    if [[ -z "$pkg_manager" ]]; then
        pkg_manager=$(project_detect_package_manager 2>/dev/null || echo "pnpm")
    fi

    if type print_info &>/dev/null; then
        print_info "Running Markdown lint..."
    fi

    if $pkg_manager run lint:md; then
        if type print_success &>/dev/null; then
            print_success "Markdown lint passed"
        fi
        return 0
    else
        if type print_error &>/dev/null; then
            print_error "Markdown lint failed"
            print_info "Run: $pkg_manager run lint:md:fix"
        fi
        return 1
    fi
}

# Run all quality checks
# Args: $1 - package manager (optional)
# Returns: 0 if all passed, 1 if any failed
# Usage: if validate_all_quality_checks "$PKG_MANAGER"; then ... fi
validate_all_quality_checks() {
    local pkg_manager="${1:-}"
    local failed=0

    if [[ -z "$pkg_manager" ]]; then
        pkg_manager=$(project_detect_package_manager 2>/dev/null || echo "pnpm")
    fi

    # Initialize counters if ui.sh is loaded
    if type init_counters &>/dev/null; then
        init_counters
    fi

    # Run all checks
    validate_rust_format || failed=$((failed + 1))
    validate_rust_clippy || failed=$((failed + 1))
    validate_rust_tests || failed=$((failed + 1))
    validate_typescript_types "$pkg_manager" || failed=$((failed + 1))
    validate_eslint "$pkg_manager" || failed=$((failed + 1))
    validate_prettier "$pkg_manager" || failed=$((failed + 1))
    validate_frontend_tests "$pkg_manager" || failed=$((failed + 1))

    return $failed
}
