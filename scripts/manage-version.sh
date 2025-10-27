#!/usr/bin/env bash
# manage-version.sh - Development Version Management Script (Fully Automated)
#
# Version format: X.Y.Z or X.Y.Z-0 (development version)
#
# Usage:
#   ./scripts/manage-version.sh snapshot-patch  # Create next patch development version
#   ./scripts/manage-version.sh snapshot-minor  # Create next minor development version
#   ./scripts/manage-version.sh snapshot-major  # Create next major development version
#   ./scripts/manage-version.sh release         # FULLY AUTOMATED: Release and push
#   ./scripts/manage-version.sh rollback        # Rollback last release
#   ./scripts/manage-version.sh info            # Show version information
#
# Workflow:
#   1. After releasing 0.1.0, create 0.1.1-0 for development
#   2. When development is complete, run `make release` (fully automated):
#      - Validates working directory is clean
#      - Runs all pre-commit checks
#      - Updates version from 0.1.1-0 to 0.1.1
#      - Creates release commit and git tag
#      - Pushes to remote (triggers CI/CD)
#      - Automatically creates 0.1.2-0 for next development
#      - Pushes development version to remote

set -euo pipefail

# ============================================================================
# Load Library Functions
# ============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

source "$SCRIPT_DIR/lib/ui.sh"
source "$SCRIPT_DIR/lib/git.sh"
source "$SCRIPT_DIR/lib/version.sh"
source "$SCRIPT_DIR/lib/project.sh"
source "$SCRIPT_DIR/lib/validators.sh"

# ============================================================================
# Initialize
# ============================================================================

init_ui

# ============================================================================
# Create Snapshot (Development Version)
# ============================================================================

create_snapshot() {
    local bump_type=$1
    local current=$(project_get_version)

    if version_is_dev "$current"; then
        print_warning "Current version is already a development version: $current"
        local base=$(version_remove_dev_suffix "$current")
        print_info "Will create new development version based on $base"
    fi

    local base_version=$(version_remove_dev_suffix "$current")
    local next_version=$(version_bump "$base_version" "$bump_type")
    local dev_version=$(version_add_dev_suffix "$next_version")

    print_header "Create Development Version"
    print_separator
    echo ""
    print_table_row "Current version" "$current"
    print_table_row "New version" "$dev_version"
    echo ""

    if ! ask_yes_no "Confirm creating development version?"; then
        print_info "Cancelled"
        exit 0
    fi

    # Validate version format
    if ! version_is_msi_compatible "$dev_version"; then
        exit 1
    fi

    # Update all version files
    project_update_all_versions "$dev_version"

    # Skip git operations for snapshot commands
    # Just update the version files without committing

    print_success "Updated version to: $dev_version"
    print_info "Version files have been modified (not committed)"
    print_info "Modified files:"
    echo "  - $PROJECT_PACKAGE_JSON"
    echo "  - $PROJECT_CARGO_TOML"
    echo "  - $PROJECT_TAURI_CONF"
    echo "  - $PROJECT_CARGO_LOCK"
}

# ============================================================================
# Run Pre-Release Quality Checks
# ============================================================================

run_pre_release_checks() {
    print_header "Running Pre-release Quality Checks"
    print_separator
    echo ""

    # Check if make command exists
    if ! project_has_tool "make"; then
        print_error "make command not found"
        print_info "Please install make or run checks manually"
        exit 1
    fi

    print_info "Running code formatting checks, linting and tests..."
    if ! make check; then
        print_error "Quality checks failed"
        print_info "Please fix the above issues and rerun make release"
        exit 1
    fi

    print_success "All quality checks passed"
    echo ""
}

# ============================================================================
# Release Version (Fully Automated)
# ============================================================================

release_version() {
    local current=$(project_get_version)

    if ! version_is_dev "$current"; then
        print_error "Current version is not a development version: $current"
        print_info "Can only release from development version (X.Y.Z-0)"
        exit 1
    fi

    local release_version=$(version_remove_dev_suffix "$current")
    local tag="$release_version"

    print_header "Release Production Version (Automated)"
    print_separator
    echo ""
    print_table_row "Development version" "$current"
    print_table_row "Release version" "$release_version"
    print_table_row "Git Tag" "$tag"
    echo ""

    # Validate tag format
    if ! validate_tag_format "$tag"; then
        exit 1
    fi

    # Validate CHANGELOG
    if ! validate_changelog "$release_version"; then
        exit 1
    fi
    echo ""

    # Validate version format
    if ! version_is_msi_compatible "$release_version"; then
        exit 1
    fi

    # Run pre-release checks
    run_pre_release_checks

    # Update version (remove -0 suffix)
    print_info "Updating version files to $release_version..."
    project_update_all_versions "$release_version"

    # Commit and tag
    print_info "Creating release commit and tag..."
    git_stage "$PROJECT_PACKAGE_JSON" "$PROJECT_CARGO_TOML" "$PROJECT_TAURI_CONF" "$PROJECT_CARGO_LOCK"
    git_commit "chore(release): $release_version"
    git_create_tag "$tag" "Release $release_version"

    print_success "Created release version: $release_version"
    print_success "Created Git tag: $tag"

    # Push to remote
    echo ""
    print_info "Pushing to remote..."
    git_push
    git_push_tags "origin" "$tag"
    print_success "Pushed to remote, CI will start building"
    echo ""
    print_info "GitHub Actions will automatically build and publish to Releases"

    # Create next development version
    echo ""
    print_info "Creating next development version..."

    local next_version=$(version_bump "$release_version" "patch")
    local dev_version=$(version_add_dev_suffix "$next_version")

    project_update_all_versions "$dev_version"

    git_stage "$PROJECT_PACKAGE_JSON" "$PROJECT_CARGO_TOML" "$PROJECT_TAURI_CONF" "$PROJECT_CARGO_LOCK"
    git_commit "chore(version): bump to $dev_version"

    print_success "Created development version: $dev_version"

    # Push development version
    print_info "Pushing development version to remote..."
    git_push
    print_success "Pushed development version to remote"

    echo ""
    print_success "Release completed successfully!"
    echo ""
    print_table_row "Released" "$release_version"
    print_table_row "Next development" "$dev_version"
    echo ""
    print_info "Ready to start developing new features!"
}

# ============================================================================
# Show Version Information
# ============================================================================

show_version_info() {
    local current=$(project_get_version)

    print_header "Version Information"
    print_separator
    echo ""
    print_table_row "Current version" "$current"

    if version_is_dev "$current"; then
        local release=$(version_remove_dev_suffix "$current")
        print_table_row "Type" "Development version (-0 suffix)"
        print_table_row "Release version" "$release (when released)"
    else
        print_table_row "Type" "Release (production version)"
        print_warning "Recommend creating next development version to continue development"
    fi

    echo ""
    print_info "Recent Git tags:"
    git_list_tags | head -3
    echo ""
}

# ============================================================================
# Rollback Last Release
# ============================================================================

rollback_release() {
    print_header "Rollback Last Release"
    print_separator
    echo ""

    # Get the latest tag
    local latest_tag=$(git_latest_tag)

    if [[ -z "$latest_tag" ]]; then
        print_error "No tags found in repository"
        exit 1
    fi

    print_warning "This will rollback the release: $latest_tag"
    echo ""
    print_info "Actions to be performed:"
    echo "  1. Delete local tag: $latest_tag"
    echo "  2. Delete remote tag: $latest_tag"
    echo "  3. Reset to 2 commits before (release + snapshot)"
    echo "  4. Force push to remote"
    echo ""
    print_warning "This operation is DESTRUCTIVE and cannot be undone!"
    print_warning "Make sure you understand what you're doing."
    echo ""

    # Show recent commits
    print_info "Recent commits (will reset to HEAD~2):"
    git log --oneline -5
    echo ""

    if ! ask_confirmation "Are you absolutely sure you want to rollback?"; then
        print_info "Rollback cancelled"
        exit 0
    fi

    echo ""
    print_step 1 4 "Deleting local tag $latest_tag"
    if git_delete_tag "$latest_tag"; then
        print_success "Local tag deleted"
    else
        print_error "Failed to delete local tag"
        exit 1
    fi

    echo ""
    print_step 2 4 "Deleting remote tag $latest_tag"
    if git_delete_tag_remote "$latest_tag"; then
        print_success "Remote tag deleted"
    else
        print_warning "Failed to delete remote tag (may not exist)"
    fi

    echo ""
    print_step 3 4 "Resetting to HEAD~2"
    if git_reset "HEAD~2" "hard"; then
        print_success "Reset to HEAD~2 completed"
    else
        print_error "Failed to reset"
        exit 1
    fi

    echo ""
    print_step 4 4 "Force pushing to remote"
    if git_force_push; then
        print_success "Force push completed"
    else
        print_error "Failed to force push"
        print_warning "You may need to manually push: git push origin main --force"
        exit 1
    fi

    echo ""
    print_success "Rollback completed successfully!"

    local current=$(project_get_version)
    print_info "Current version after rollback: $current"
    echo ""
    print_info "You can now fix the issues and run 'make release' again"
}

# ============================================================================
# Main Function
# ============================================================================

main() {
    git_require_repo

    if [[ $# -eq 0 ]]; then
        show_version_info
        echo ""
        print_info "Usage:"
        echo "  $0 snapshot-patch      # Create next patch development version"
        echo "  $0 snapshot-minor      # Create next minor development version"
        echo "  $0 snapshot-major      # Create next major development version"
        echo "  $0 release             # Release current development version"
        echo "  $0 rollback            # Rollback last release"
        echo "  $0 info                # Show version information"
        echo ""
        exit 0
    fi

    # Only release command needs working directory check
    # Snapshot commands can run with uncommitted changes
    if [[ "$1" == "release" ]]; then
        git_require_clean
    fi

    case "$1" in
        snapshot-patch)
            create_snapshot "patch"
            ;;
        snapshot-minor)
            create_snapshot "minor"
            ;;
        snapshot-major)
            create_snapshot "major"
            ;;
        release)
            release_version
            ;;
        rollback)
            rollback_release
            ;;
        info)
            show_version_info
            ;;
        *)
            print_error "Unknown command: $1"
            print_info "Usage: $0 <snapshot-patch|snapshot-minor|snapshot-major|release|rollback|info>"
            exit 1
            ;;
    esac
}

main "$@"
