#!/usr/bin/env bash
# validate-changelog.sh - CHANGELOG Validation Script
#
# Validates that CHANGELOG.md is properly updated for the current version
#
# Usage:
#   ./scripts/validate-changelog.sh
#
# Exit codes:
#   0 - Validation passed
#   1 - Validation failed

set -euo pipefail

# Color output
COLOR_RESET='\033[0m'
COLOR_GREEN='\033[32m'
COLOR_YELLOW='\033[33m'
COLOR_RED='\033[31m'

# Print messages
print_success() { printf "${COLOR_GREEN}✅ $1${COLOR_RESET}\n"; }
print_error() { printf "${COLOR_RED}❌ $1${COLOR_RESET}\n"; }
print_warning() { printf "${COLOR_YELLOW}⚠️  $1${COLOR_RESET}\n"; }
print_info() { printf "ℹ️  $1\n"; }

# Get current version from package.json
get_current_version() {
    grep '"version"' package.json | head -1 | sed 's/.*"version": "\(.*\)".*/\1/'
}

# Check if version is a development version
is_dev_version() {
    [[ $1 == *"-0" ]]
}

# Remove development suffix
remove_dev_suffix() {
    echo "$1" | sed 's/-0$//'
}

# Main validation function
validate_changelog() {
    local current_version=$(get_current_version)

    # Check if version is a development version (ends with -0)
    if is_dev_version "$current_version"; then
        # For development versions, check the release version
        local release_version=$(remove_dev_suffix "$current_version")

        print_info "Current version: $current_version (development)"
        print_info "Will validate CHANGELOG for release version: $release_version"

        # Check if CHANGELOG.md exists
        if [ ! -f "CHANGELOG.md" ]; then
            print_error "CHANGELOG.md file not found"
            return 1
        fi

        # Check if release version entry exists
        if ! grep -q "## \[$release_version\]" CHANGELOG.md; then
            print_error "Version [$release_version] not found in CHANGELOG.md"
            echo ""
            print_info "Before running 'make release', please add the following to CHANGELOG.md:"
            echo ""
            echo "  ## [$release_version]"
            echo ""
            echo "  ### Added"
            echo "  - New feature 1"
            echo "  - New feature 2"
            echo ""
            echo "  ### Changed"
            echo "  - Changed feature 1"
            echo ""
            echo "  ### Fixed"
            echo "  - Bug fix 1"
            echo ""
            return 1
        else
            print_success "CHANGELOG entry found for release version $release_version"
            return 0
        fi
    else
        # For release versions, check the current version
        print_info "Current version: $current_version (release)"

        # Check if CHANGELOG.md exists
        if [ ! -f "CHANGELOG.md" ]; then
            print_error "CHANGELOG.md file not found"
            return 1
        fi

        # Check if current version entry exists
        if ! grep -q "## \[$current_version\]" CHANGELOG.md; then
            print_error "Version [$current_version] not found in CHANGELOG.md"
            print_warning "Please add changelog entry for release version $current_version"
            return 1
        else
            print_success "CHANGELOG entry found for version $current_version"
            return 0
        fi
    fi
}

# Run validation
if validate_changelog; then
    exit 0
else
    exit 1
fi
