#!/usr/bin/env bash
# Development Version Management Script (Fully Automated)
#
# Version format: X.Y.Z or X.Y.Z-0 (development version)
#
# Usage:
#   ./scripts/version.sh snapshot-patch  # Create next patch development version (0.1.0 -> 0.1.1-0)
#   ./scripts/version.sh snapshot-minor  # Create next minor development version (0.1.0 -> 0.2.0-0)
#   ./scripts/version.sh snapshot-major  # Create next major development version (0.1.0 -> 1.0.0-0)
#   ./scripts/version.sh release         # FULLY AUTOMATED: Release, push, and create next snapshot (0.1.1-0 -> 0.1.1 -> 0.1.2-0)
#
# Workflow:
#   1. After releasing 0.1.0, create 0.1.1-0 for development
#   2. When development is complete, run `make release` (fully automated):
#      - Validates working directory is clean (no uncommitted changes)
#      - Runs all pre-commit checks (format, lint, tests)
#      - Updates version from 0.1.1-0 to 0.1.1
#      - Creates release commit and git tag
#      - Pushes to remote (triggers CI/CD)
#      - Automatically creates 0.1.2-0 for next development
#      - Pushes development version to remote
#
# Release Requirements:
#   1. Working directory must be clean (no uncommitted changes) - will error and exit
#   2. CHANGELOG.md must contain entry for the release version - will error and exit
#   3. All pre-commit checks must pass - will error and exit
#
# Rollback on Release Failure:
#   If CI build fails after release, you need to rollback:
#   1. Delete local tag: git tag -d X.Y.Z
#   2. Delete remote tag: git push origin :refs/tags/X.Y.Z
#   3. Revert commits (2 commits: release + next snapshot):
#      git reset --hard HEAD~2
#   4. Force push: git push origin main --force-with-lease
#   5. Fix issues and rerun: make release

set -euo pipefail

# Color definitions
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# File paths
PACKAGE_JSON="package.json"
CARGO_TOML="src-tauri/Cargo.toml"
TAURI_CONF="src-tauri/tauri.conf.json"

# Helper functions
print_info() { echo -e "${BLUE}ℹ${NC} $1"; }
print_success() { echo -e "${GREEN}✓${NC} $1"; }
print_warning() { echo -e "${YELLOW}⚠${NC} $1"; }
print_error() { echo -e "${RED}✗${NC} $1"; }
print_header() { echo -e "${CYAN}${1}${NC}"; }

# Check if in git repository
check_git_repo() {
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        print_error "Not in a Git repository"
        exit 1
    fi
}

# Check working directory status
check_working_directory() {
    if [[ -n $(git status -s) ]]; then
        print_error "Working directory has uncommitted changes"
        git status -s
        print_info "Please commit or stash changes before release"
        exit 1
    fi
}

# Get current version
get_current_version() {
    grep '"version"' "$PACKAGE_JSON" | head -1 | sed 's/.*"version": "\(.*\)".*/\1/'
}

# Check if is development version
is_dev_version() {
    [[ $1 == *"-0" ]]
}

# Remove development suffix
remove_dev_suffix() {
    echo "$1" | sed 's/-0$//'
}

# Add development suffix
add_dev_suffix() {
    echo "$1-0"
}

# Split version number
split_version() {
    local version=$(remove_dev_suffix "$1")
    MAJOR=$(echo "$version" | cut -d. -f1)
    MINOR=$(echo "$version" | cut -d. -f2)
    PATCH=$(echo "$version" | cut -d. -f3)
}

# Calculate next version
calculate_next_version() {
    local current=$1
    local bump_type=$2

    local base_version=$(remove_dev_suffix "$current")
    split_version "$base_version"

    case "$bump_type" in
        patch)
            PATCH=$((PATCH + 1))
            ;;
        minor)
            MINOR=$((MINOR + 1))
            PATCH=0
            ;;
        major)
            MAJOR=$((MAJOR + 1))
            MINOR=0
            PATCH=0
            ;;
        *)
            print_error "Invalid version type: $bump_type"
            exit 1
            ;;
    esac

    echo "${MAJOR}.${MINOR}.${PATCH}"
}

# Validate version format (MSI compatibility)
validate_version_format() {
    local version=$1

    # MSI requirement: pre-release identifier must be numeric only (e.g. 1.0.0 or 1.0.0-0)
    # Cannot contain letter suffixes (e.g. 1.0.0-alpha, 1.0.0-beta)
    if [[ "$version" =~ -[^0-9] ]]; then
        print_error "Version '$version' contains non-numeric pre-release identifier"
        print_error "MSI build requires pre-release identifiers to be numeric only (e.g. 1.0.0 or 1.0.0-0)"
        print_error "Current version contains letter suffix, which will cause Windows MSI build failure"
        return 1
    fi
    return 0
}

# Update all version files
update_version_files() {
    local new_version=$1

    # Validate version format
    if ! validate_version_format "$new_version"; then
        exit 1
    fi

    print_info "Updating $PACKAGE_JSON..."
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "s/\"version\": \".*\"/\"version\": \"$new_version\"/" "$PACKAGE_JSON"
    else
        sed -i "s/\"version\": \".*\"/\"version\": \"$new_version\"/" "$PACKAGE_JSON"
    fi

    print_info "Updating $CARGO_TOML..."
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "s/^version = \".*\"/version = \"$new_version\"/" "$CARGO_TOML"
    else
        sed -i "s/^version = \".*\"/version = \"$new_version\"/" "$CARGO_TOML"
    fi

    print_info "Updating $TAURI_CONF..."
    if command -v jq &> /dev/null; then
        jq ".version = \"$new_version\"" "$TAURI_CONF" > "${TAURI_CONF}.tmp"
        mv "${TAURI_CONF}.tmp" "$TAURI_CONF"
    else
        if [[ "$OSTYPE" == "darwin"* ]]; then
            sed -i '' "s/\"version\": \".*\"/\"version\": \"$new_version\"/" "$TAURI_CONF"
        else
            sed -i "s/\"version\": \".*\"/\"version\": \"$new_version\"/" "$TAURI_CONF"
        fi
    fi

    print_info "Updating Cargo.lock..."
    cargo update -p bing-wallpaper-now --manifest-path src-tauri/Cargo.toml --quiet 2>/dev/null || true

    print_success "Version files updated to $new_version"
}

# Create development version
create_snapshot() {
    local bump_type=$1
    local current=$(get_current_version)

    if is_dev_version "$current"; then
        print_warning "Current version is already a development version: $current"
        local base=$(remove_dev_suffix "$current")
        print_info "Will create new development version based on $base"
    fi

    local base_version=$(remove_dev_suffix "$current")
    local next_version=$(calculate_next_version "$base_version" "$bump_type")
    local dev_version=$(add_dev_suffix "$next_version")

    print_header "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    print_header "  Create Development Version"
    print_header "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    print_info "Current version: $current"
    print_info "New version:     $dev_version"
    echo ""

    read -p "Confirm creating development version? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Cancelled"
        exit 0
    fi

    update_version_files "$dev_version"

    git add "$PACKAGE_JSON" "$CARGO_TOML" "$TAURI_CONF" "src-tauri/Cargo.lock"
    git commit -m "chore(version): bump to $dev_version"

    print_success "Created development version: $dev_version"
    print_info "Ready to start developing new features!"
}

# Validate tag format (must not start with 'v')
validate_tag_format() {
    local tag=$1

    if [[ $tag == v* ]]; then
        print_error "Tag format error: tag must not start with 'v'"
        print_error "Expected: $tag (without 'v' prefix)"
        print_info "Project uses tags like: 0.1.0, 0.1.1, 0.1.2"
        print_info "Not: v0.1.0, v0.1.1, v0.1.2"
        return 1
    fi

    print_success "Tag format validation passed: $tag"
    return 0
}

# Validate CHANGELOG is updated
validate_changelog() {
    local version=$1

    if [ ! -f "CHANGELOG.md" ]; then
        print_error "CHANGELOG.md file not found"
        return 1
    fi

    if ! grep -q "## \[$version\]" CHANGELOG.md; then
        print_error "Version [$version] not found in CHANGELOG.md"
        print_info "Please add the following content to CHANGELOG.md first:"
        echo ""
        echo "  ## [$version]"
        echo ""
        echo "  ### Added/Changed/Fixed"
        echo "  - Your changelog notes..."
        echo ""
        print_info "Then rerun make release"
        return 1
    fi

    print_success "CHANGELOG.md validation passed"
    return 0
}

# Run pre-release quality checks
run_pre_release_checks() {
    print_header "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    print_header "  Running Pre-release Quality Checks"
    print_header "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""

    # Check if make command exists
    if ! command -v make &> /dev/null; then
        print_error "make command not found"
        print_info "Please install make or run checks manually: cargo fmt --check && cargo clippy && cargo test"
        exit 1
    fi

    print_info "Running code formatting checks, linting and tests..."
    if ! make pre-commit; then
        print_error "Quality checks failed"
        print_info "Please fix the above issues and rerun make release"
        exit 1
    fi

    print_success "All quality checks passed"
    echo ""
}

# Release version (fully automated: push to remote and create next snapshot)
release_version() {
    local current=$(get_current_version)

    if ! is_dev_version "$current"; then
        print_error "Current version is not a development version: $current"
        print_info "Can only release from development version (X.Y.Z-0)"
        exit 1
    fi

    local release_version=$(remove_dev_suffix "$current")
    local tag="$release_version"

    print_header "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    print_header "  Release Production Version (Automated)"
    print_header "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    print_info "Development version: $current"
    print_info "Release version:     $release_version"
    print_info "Git Tag:             $tag"
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

    # Run pre-release checks
    run_pre_release_checks

    # Update version (remove SNAPSHOT)
    print_info "Updating version files to $release_version..."
    update_version_files "$release_version"

    # Commit and tag
    print_info "Creating release commit and tag..."
    git add "$PACKAGE_JSON" "$CARGO_TOML" "$TAURI_CONF" "src-tauri/Cargo.lock"
    git commit -m "chore(release): $release_version"
    git tag -a "$tag" -m "Release $release_version"

    print_success "Created release version: $release_version"
    print_success "Created Git tag: $tag"

    # Automatically push to remote
    echo ""
    print_info "Pushing to remote..."
    git push origin main
    git push origin "$tag"
    print_success "Pushed to remote, CI will start building"
    echo ""
    print_info "GitHub Actions will automatically build and publish to Releases"

    # Automatically create next development version
    echo ""
    print_info "Creating next development version..."

    local next_version=$(calculate_next_version "$release_version" "patch")
    local dev_version=$(add_dev_suffix "$next_version")

    update_version_files "$dev_version"

    git add "$PACKAGE_JSON" "$CARGO_TOML" "$TAURI_CONF" "src-tauri/Cargo.lock"
    git commit -m "chore(version): bump to $dev_version"

    print_success "Created development version: $dev_version"

    # Automatically push development version
    print_info "Pushing development version to remote..."
    git push origin main
    print_success "Pushed development version to remote"

    echo ""
    print_success "Release completed successfully!"
    print_info "Released: $release_version"
    print_info "Next development version: $dev_version"
    print_info "Ready to start developing new features!"
}

# Show current version information
show_version_info() {
    local current=$(get_current_version)

    print_header "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    print_header "  Version Information"
    print_header "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    print_info "Current version: $current"

    if is_dev_version "$current"; then
        local release=$(remove_dev_suffix "$current")
        print_info "Type:            Development version (-0 suffix)"
        print_info "Release version: $release (when released)"
    else
        print_info "Type:            Release (production version)"
        print_warning "Recommend creating next development version to continue development"
    fi

    echo ""
    print_info "Recent Git tags:"
    git tag --sort=-v:refname | head -3
    echo ""
}

# Rollback last release
rollback_release() {
    print_header "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    print_header "  Rollback Last Release"
    print_header "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""

    # Get the latest tag
    local latest_tag=$(git tag --sort=-v:refname | head -1)

    if [ -z "$latest_tag" ]; then
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

    read -p "Are you absolutely sure you want to rollback? (yes/NO) " -r
    echo
    if [[ ! $REPLY == "yes" ]]; then
        print_info "Rollback cancelled"
        exit 0
    fi

    echo ""
    print_info "Step 1/4: Deleting local tag $latest_tag..."
    if git tag -d "$latest_tag"; then
        print_success "Local tag deleted"
    else
        print_error "Failed to delete local tag"
        exit 1
    fi

    echo ""
    print_info "Step 2/4: Deleting remote tag $latest_tag..."
    if git push origin ":refs/tags/$latest_tag"; then
        print_success "Remote tag deleted"
    else
        print_warning "Failed to delete remote tag (may not exist)"
    fi

    echo ""
    print_info "Step 3/4: Resetting to HEAD~2..."
    if git reset --hard HEAD~2; then
        print_success "Reset to HEAD~2 completed"
    else
        print_error "Failed to reset"
        exit 1
    fi

    echo ""
    print_info "Step 4/4: Force pushing to remote..."
    if git push origin main --force-with-lease; then
        print_success "Force push completed"
    else
        print_error "Failed to force push"
        print_warning "You may need to manually push: git push origin main --force"
        exit 1
    fi

    echo ""
    print_success "Rollback completed successfully!"

    local current=$(get_current_version)
    print_info "Current version after rollback: $current"
    echo ""
    print_info "You can now fix the issues and run 'make release' again"
}

# Main function
main() {
    check_git_repo

    if [ $# -eq 0 ]; then
        show_version_info
        echo ""
        print_info "Usage:"
        echo "  $0 snapshot-patch      # Create next patch development version"
        echo "  $0 snapshot-minor      # Create next minor development version"
        echo "  $0 snapshot-major      # Create next major development version"
        echo "  $0 release             # Release current development version, create tag and push to remote"
        echo "  $0 rollback            # Rollback last release (delete tag and reset to HEAD~2)"
        echo ""
        exit 0
    fi

    # Rollback doesn't need working directory check (it will reset anyway)
    if [[ "$1" != "rollback" && "$1" != "info" ]]; then
        check_working_directory
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
