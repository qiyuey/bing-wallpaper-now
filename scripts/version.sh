#!/usr/bin/env bash
# SNAPSHOT Version Management Script
#
# Version format: X.Y.Z or X.Y.Z-SNAPSHOT
#
# Usage:
#   ./scripts/version-snapshot.sh snapshot-patch  # Create next patch SNAPSHOT (0.1.0 -> 0.1.1-SNAPSHOT)
#   ./scripts/version-snapshot.sh snapshot-minor  # Create next minor SNAPSHOT (0.1.0 -> 0.2.0-SNAPSHOT)
#   ./scripts/version-snapshot.sh snapshot-major  # Create next major SNAPSHOT (0.1.0 -> 1.0.0-SNAPSHOT)
#   ./scripts/version-snapshot.sh release         # Release current SNAPSHOT version, create tag and push to remote (0.1.1-SNAPSHOT -> 0.1.1)
#
# Workflow:
#   1. After releasing v0.1.0, create 0.1.1-SNAPSHOT for development
#   2. When development is complete, run release to convert to 0.1.1 production version, create tag and push to remote
#   3. After release, create 0.1.2-SNAPSHOT again to continue development
#
# Rollback on Release Failure:
#   If CI build fails after release, you need to rollback:
#   1. Delete local tag: git tag -d vX.Y.Z
#   2. Delete remote tag: git push origin :refs/tags/vX.Y.Z
#   3. Revert commits:
#      - Only revert release commit: git reset --hard HEAD~1
#      - Also created SNAPSHOT: git reset --hard HEAD~2
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
        print_warning "Working directory has uncommitted changes"
        git status -s
        echo ""
        read -p "Continue anyway? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_info "Cancelled"
            exit 0
        fi
    fi
}

# Get current version
get_current_version() {
    grep '"version"' "$PACKAGE_JSON" | head -1 | sed 's/.*"version": "\(.*\)".*/\1/'
}

# Check if is SNAPSHOT version
is_snapshot() {
    [[ $1 == *"-SNAPSHOT" ]]
}

# Remove SNAPSHOT suffix
remove_snapshot() {
    echo "$1" | sed 's/-SNAPSHOT$//'
}

# Add SNAPSHOT suffix
add_snapshot() {
    echo "$1-SNAPSHOT"
}

# Split version number
split_version() {
    local version=$(remove_snapshot "$1")
    MAJOR=$(echo "$version" | cut -d. -f1)
    MINOR=$(echo "$version" | cut -d. -f2)
    PATCH=$(echo "$version" | cut -d. -f3)
}

# Calculate next version
calculate_next_version() {
    local current=$1
    local bump_type=$2

    local base_version=$(remove_snapshot "$current")
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

    # MSI requirement: pre-release identifier must be numeric only (e.g. 1.0.0 or 1.0.0-123)
    # Cannot contain letter suffixes (e.g. 1.0.0-alpha, 1.0.0-SNAPSHOT)
    if [[ "$version" =~ -[^0-9] ]]; then
        print_error "Version '$version' contains non-numeric pre-release identifier"
        print_error "MSI build requires pre-release identifiers to be numeric only (e.g. 1.0.0 or 1.0.0-123)"
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

# Create SNAPSHOT version
create_snapshot() {
    local bump_type=$1
    local current=$(get_current_version)

    if is_snapshot "$current"; then
        print_warning "Current version is already SNAPSHOT: $current"
        local base=$(remove_snapshot "$current")
        print_info "Will create new SNAPSHOT based on $base"
    fi

    local base_version=$(remove_snapshot "$current")
    local next_version=$(calculate_next_version "$base_version" "$bump_type")
    local snapshot_version=$(add_snapshot "$next_version")

    print_header "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    print_header "  Create SNAPSHOT Version"
    print_header "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    print_info "Current version: $current"
    print_info "New version:     $snapshot_version"
    echo ""

    read -p "Confirm creating SNAPSHOT version? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Cancelled"
        exit 0
    fi

    update_version_files "$snapshot_version"

    git add "$PACKAGE_JSON" "$CARGO_TOML" "$TAURI_CONF" "src-tauri/Cargo.lock"
    git commit -m "chore(version): bump to $snapshot_version"

    print_success "Created SNAPSHOT version: $snapshot_version"
    print_info "Ready to start developing new features!"
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
        print_warning "make command not found, skipping quality checks"
        print_warning "Recommended to run manually: make pre-commit"
        echo ""
        read -p "Continue with release? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_info "Cancelled"
            exit 0
        fi
        return 0
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

# Release version (pushes to remote by default)
release_version() {
    local current=$(get_current_version)

    if ! is_snapshot "$current"; then
        print_error "Current version is not SNAPSHOT: $current"
        print_info "Can only release from SNAPSHOT version"
        exit 1
    fi

    local release_version=$(remove_snapshot "$current")
    local tag="v$release_version"

    print_header "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    print_header "  Release Production Version"
    print_header "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    print_info "SNAPSHOT version: $current"
    print_info "Release version:  $release_version"
    print_info "Git Tag:          $tag"
    echo ""

    # Validate CHANGELOG
    if ! validate_changelog "$release_version"; then
        exit 1
    fi
    echo ""

    # Run pre-release checks
    run_pre_release_checks

    read -p "Confirm releasing version? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "Cancelled"
        exit 0
    fi

    # Update version (remove SNAPSHOT)
    update_version_files "$release_version"

    # Commit and tag
    git add "$PACKAGE_JSON" "$CARGO_TOML" "$TAURI_CONF" "src-tauri/Cargo.lock"
    git commit -m "chore(release): $release_version"
    git tag -a "$tag" -m "Release $release_version"

    print_success "Created release version: $release_version"
    print_success "Created Git tag: $tag"

    echo ""
    read -p "Push to remote immediately? (Y/n) " -n 1 -r
    echo
    local pushed=false
    if [[ ! $REPLY =~ ^[Nn]$ ]]; then
        print_info "Pushing to remote..."
        git push origin main
        git push origin "$tag"
        print_success "Pushed to remote, CI will start building"
        pushed=true
        echo ""
        print_info "GitHub Actions will automatically build and publish to Releases"
    else
        print_info "Skipped push, manually push later:"
        echo "  git push origin main && git push origin $tag"
    fi

    # Ask if create next SNAPSHOT version
    echo ""
    read -p "Create next patch SNAPSHOT version? (y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo ""
        print_info "Creating next SNAPSHOT version..."

        local next_version=$(calculate_next_version "$release_version" "patch")
        local snapshot_version=$(add_snapshot "$next_version")

        update_version_files "$snapshot_version"

        git add "$PACKAGE_JSON" "$CARGO_TOML" "$TAURI_CONF" "src-tauri/Cargo.lock"
        git commit -m "chore(version): bump to $snapshot_version"

        print_success "Created SNAPSHOT version: $snapshot_version"

        if [ "$pushed" = true ]; then
            echo ""
            read -p "Push SNAPSHOT version to remote? (y/N) " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                git push origin main
                print_success "Pushed SNAPSHOT version to remote"
            else
                print_info "Push manually later: git push origin main"
            fi
        fi

        echo ""
        print_success "Ready to start developing new features!"
    else
        echo ""
        print_info "Create SNAPSHOT version manually later: make snapshot-patch"
    fi
}

# Show current version information
show_version_info() {
    local current=$(get_current_version)

    print_header "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    print_header "  Version Information"
    print_header "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    print_info "Current version: $current"

    if is_snapshot "$current"; then
        local release=$(remove_snapshot "$current")
        print_info "Type:            SNAPSHOT (development version)"
        print_info "Release version: $release (when released)"
    else
        print_info "Type:            Release (production version)"
        print_warning "Recommend creating next SNAPSHOT version to continue development"
    fi

    echo ""
    print_info "Recent Git tags:"
    git tag --sort=-v:refname | head -3
    echo ""
}

# Main function
main() {
    check_git_repo

    if [ $# -eq 0 ]; then
        show_version_info
        echo ""
        print_info "Usage:"
        echo "  $0 snapshot-patch      # Create next patch SNAPSHOT"
        echo "  $0 snapshot-minor      # Create next minor SNAPSHOT"
        echo "  $0 snapshot-major      # Create next major SNAPSHOT"
        echo "  $0 release             # Release current SNAPSHOT version, create tag and push to remote"
        echo ""
        exit 0
    fi

    check_working_directory

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
        info)
            show_version_info
            ;;
        *)
            print_error "Unknown command: $1"
            print_info "Usage: $0 <snapshot-patch|snapshot-minor|snapshot-major|release|info>"
            exit 1
            ;;
    esac
}

main "$@"
