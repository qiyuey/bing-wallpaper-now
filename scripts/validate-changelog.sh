#!/usr/bin/env bash
# validate-changelog.sh - CHANGELOG Validation Script
#
# Validates that CHANGELOG.md is properly updated for a version
#
# Usage:
#   ./scripts/validate-changelog.sh [version]
#
# Arguments:
#   version  - Optional. Version to validate. If not provided, uses current version from package.json
#
# Exit codes:
#   0 - Validation passed
#   1 - Validation failed
#
# Examples:
#   ./scripts/validate-changelog.sh        # Validate current version
#   ./scripts/validate-changelog.sh 0.1.9  # Validate specific version

set -euo pipefail

# ============================================================================
# Load Library Functions
# ============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

source "$SCRIPT_DIR/lib/ui.sh"
source "$SCRIPT_DIR/lib/version.sh"
source "$SCRIPT_DIR/lib/project.sh"
source "$SCRIPT_DIR/lib/validators.sh"

# ============================================================================
# Initialize
# ============================================================================

init_ui

# ============================================================================
# Main Validation
# ============================================================================

main() {
    local target_version="${1:-}"

    # If no version specified, validate for current project version
    if [[ -z "$target_version" ]]; then
        if validate_changelog; then
            exit 0
        else
            exit 1
        fi
    else
        # Validate for specified version
        print_info "Validating CHANGELOG for version: $target_version"
        if validate_changelog_has_version "$target_version"; then
            exit 0
        else
            exit 1
        fi
    fi
}

# Run validation when script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
