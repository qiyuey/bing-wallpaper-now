#!/usr/bin/env bash
# manage-version.sh - Development Version Management Script (Fully Automated)
#
# Version format: X.Y.Z or X.Y.Z-0 (development version)
#
# Usage:
#   ./scripts/manage-version.sh patch      # Create next patch development version
#   ./scripts/manage-version.sh minor      # Create next minor development version
#   ./scripts/manage-version.sh major      # Create next major development version
#   ./scripts/manage-version.sh release    # Release and push (no auto snapshot)
#   ./scripts/manage-version.sh retag      # Re-push current version tag (re-trigger CI)
#   ./scripts/manage-version.sh info       # Show version information
#
# Workflow:
#   1. After releasing 0.1.0, create 0.1.1-0 for development: make patch
#   2. When development is complete, run `make release`:
#      - Validates working directory is clean
#      - Runs all pre-commit checks
#      - Updates version from 0.1.1-0 to 0.1.1
#      - Creates release commit and git tag
#      - Pushes to remote (triggers CI/CD)
#   3. Manually create next dev version: make patch

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
project_require_tool "jq" "jq is required for JSON manipulation. Install via: brew install jq (macOS) or apt install jq (Linux)"

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

    # Check for other uncommitted files before updating version
    # git status --porcelain outputs: " M file" (modified), "M  file" (staged), "?? file" (untracked)
    local other_changes=$(git status --porcelain 2>/dev/null | grep -v -E "(package\.json|src-tauri/Cargo\.toml|src-tauri/tauri\.conf\.json|src-tauri/Cargo\.lock)" || true)
    
    # Update all version files
    project_update_all_versions "$dev_version"

    # Check if there are other uncommitted files (excluding version files)
    if [[ -n "$other_changes" ]]; then
        print_success "Updated version to: $dev_version"
        print_warning "Working directory has other uncommitted changes"
        echo ""
        print_info "Uncommitted files (excluding version files):"
        echo "$other_changes" | sed 's/^/  /'
        echo ""
        print_info "Version files have been modified (not committed)"
        print_info "Please commit other changes first, then commit version files separately"
        print_info "Modified version files:"
        echo "  - $PROJECT_PACKAGE_JSON"
        echo "  - $PROJECT_CARGO_TOML"
        echo "  - $PROJECT_TAURI_CONF"
        echo "  - $PROJECT_CARGO_LOCK"
    else
        # No other changes, auto-commit version files
        print_info "No other uncommitted changes detected"
        print_info "Staging version files..."
        git_stage "$PROJECT_PACKAGE_JSON" "$PROJECT_CARGO_TOML" "$PROJECT_TAURI_CONF" "$PROJECT_CARGO_LOCK"
        
        # Generate commit message based on bump type
        local commit_msg
        case "$bump_type" in
            patch)
                commit_msg="chore: bump version to $dev_version (patch)"
                ;;
            minor)
                commit_msg="chore: bump version to $dev_version (minor)"
                ;;
            major)
                commit_msg="chore: bump version to $dev_version (major)"
                ;;
            *)
                commit_msg="chore: bump version to $dev_version"
                ;;
        esac
        
        print_info "Creating commit..."
        git_commit "$commit_msg"
        print_success "Updated version to: $dev_version"
        print_success "Version files have been committed"
        print_info "Commit message: $commit_msg"
    fi
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
    if ! make check NO_FIX=1; then
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
    trap 'echo ""; print_warning "发布流程中断。请检查工作区状态: git status"; exit 1' INT TERM

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

    # Commit version changes
    print_info "Creating release commit..."
    git_stage "$PROJECT_PACKAGE_JSON" "$PROJECT_CARGO_TOML" "$PROJECT_TAURI_CONF" "$PROJECT_CARGO_LOCK"
    git_commit "chore(release): $release_version"
    print_success "Created release version: $release_version"

    # Create tag on the version update commit (HEAD)
    # This ensures the tag points to the commit with the updated version numbers,
    # not the CHANGELOG commit that comes before it
    print_info "Creating Git tag on version update commit..."
    git_create_tag "$tag" "Release $release_version"
    print_success "Created Git tag: $tag (on commit: chore(release): $release_version)"

    # Push to remote
    echo ""
    print_info "Pushing to remote..."
    git_push
    git_push_tags "origin" "$tag"
    print_success "Pushed to remote, CI will start building"
    echo ""
    print_info "GitHub Actions will automatically build and publish to Releases"

    trap - INT TERM

    echo ""
    print_success "Release completed successfully!"
    echo ""
    print_table_row "Released" "$release_version"
    print_table_row "Git tag" "$tag"
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
# Re-push Version Tag
# ============================================================================

retag_version() {
    print_header "Re-push Version Tag"
    print_separator
    echo ""

    local current=$(project_get_version)

    # Check if current version is a release version (not dev)
    if version_is_dev "$current"; then
        print_error "Current version is a development version: $current"
        print_info "Cannot re-push dev version tags"
        print_info "This command is only for release versions (without -0 suffix)"
        exit 1
    fi

    local tag="$current"

    print_info "Current release version: $current"
    print_info "Will force-push tag: $tag"
    echo ""

    # Check if tag exists locally
    if ! git_tag_exists "$tag"; then
        print_error "Tag '$tag' does not exist locally"
        print_info "Create the tag first with: git tag $tag"
        exit 1
    fi

    local head_commit
    head_commit=$(git rev-parse HEAD)
    local tag_commit
    tag_commit=$(git rev-parse "$tag")

    if [[ "$head_commit" != "$tag_commit" ]]; then
        print_info "Updating local tag to point at current commit..."
        if git tag -a -f "$tag" -m "Release $current" HEAD; then
            print_success "Tag $tag now points to $(git rev-parse --short HEAD)"
        else
            print_error "Failed to update local tag"
            exit 1
        fi
    else
        print_info "Local tag already points to current commit"
    fi

    print_warning "This will force-push the tag to remote"
    print_info "Use this to re-trigger CI/CD builds for the same version"
    echo ""

    # Push tag with force
    print_info "Force-pushing tag $tag to remote..."
    if git push origin "refs/tags/${tag}" --force; then
        print_success "Tag pushed successfully!"
        echo ""
        print_info "GitHub Actions will start building for tag: $tag"
        print_info "Check progress at: https://github.com/$(git remote get-url origin | sed 's/.*github.com[:/]\(.*\)\.git/\1/')/actions"
    else
        print_error "Failed to push tag"
        exit 1
    fi
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
        echo "  $0 patch      # Create next patch development version"
        echo "  $0 minor      # Create next minor development version"
        echo "  $0 major      # Create next major development version"
        echo "  $0 release    # Release current development version"
        echo "  $0 retag      # Re-push current version tag (re-trigger CI)"
        echo "  $0 info       # Show version information"
        echo ""
        exit 0
    fi

    # Only release command needs working directory check
    # Snapshot commands can run with uncommitted changes
    if [[ "$1" == "release" ]]; then
        git_require_clean
    fi

    case "$1" in
        patch|snapshot-patch)
            create_snapshot "patch"
            ;;
        minor|snapshot-minor)
            create_snapshot "minor"
            ;;
        major|snapshot-major)
            create_snapshot "major"
            ;;
        release)
            release_version
            ;;
        retag)
            retag_version
            ;;
        info)
            show_version_info
            ;;
        *)
            print_error "Unknown command: $1"
            print_info "Usage: $0 <patch|minor|major|release|retag|info>"
            exit 1
            ;;
    esac
}

main "$@"
