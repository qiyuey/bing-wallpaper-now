#!/usr/bin/env bash
# git.sh - Git Operations Utilities
#
# Provides reusable functions for Git operations:
# - Repository checks
# - Status and diff operations
# - Tag management
# - Commit operations
# - Branch operations
#
# Usage:
#   source "$(dirname "${BASH_SOURCE[0]}")/lib/git.sh"
#   if git_is_repo; then
#       git_check_clean
#   fi

# ============================================================================
# Repository Checks
# ============================================================================

# Check if current directory is a git repository
# Returns: 0 if in git repo, 1 otherwise
# Usage: if git_is_repo; then ... fi
git_is_repo() {
    git rev-parse --git-dir > /dev/null 2>&1
}

# Ensure we're in a git repository (exits if not)
# Usage: git_require_repo
git_require_repo() {
    if ! git_is_repo; then
        if type print_error &>/dev/null; then
            print_error "Not in a Git repository"
        else
            echo "Error: Not in a Git repository" >&2
        fi
        exit 1
    fi
}

# ============================================================================
# Working Directory Status
# ============================================================================

# Check if working directory is clean (no uncommitted changes)
# Returns: 0 if clean, 1 if has changes
# Usage: if git_is_clean; then ... fi
git_is_clean() {
    [[ -z $(git status --porcelain) ]]
}

# Check if working directory has uncommitted changes
# Returns: 0 if has changes, 1 if clean
# Usage: if git_has_changes; then ... fi
git_has_changes() {
    [[ -n $(git status --porcelain) ]]
}

# Require clean working directory (exits if dirty)
# Usage: git_require_clean
git_require_clean() {
    if git_has_changes; then
        if type print_error &>/dev/null; then
            print_error "Working directory has uncommitted changes"
            git status --short
            print_info "Please commit or stash changes first"
        else
            echo "Error: Working directory has uncommitted changes" >&2
            git status --short
        fi
        exit 1
    fi
}

# Get list of modified files
# Returns: List of modified file paths
# Usage: modified_files=$(git_get_modified_files)
git_get_modified_files() {
    git status --porcelain | awk '{print $2}'
}

# Get list of staged files
# Returns: List of staged file paths
# Usage: staged_files=$(git_get_staged_files)
git_get_staged_files() {
    git diff --cached --name-only
}

# ============================================================================
# Branch Operations
# ============================================================================

# Get current branch name
# Returns: Current branch name
# Usage: branch=$(git_current_branch)
git_current_branch() {
    git rev-parse --abbrev-ref HEAD
}

# Get default/main branch name
# Returns: main or master
# Usage: main_branch=$(git_main_branch)
git_main_branch() {
    if git show-ref --verify --quiet refs/heads/main; then
        echo "main"
    elif git show-ref --verify --quiet refs/heads/master; then
        echo "master"
    else
        echo "main"  # Default to main if neither exists
    fi
}

# Check if on main/master branch
# Returns: 0 if on main branch, 1 otherwise
# Usage: if git_is_main_branch; then ... fi
git_is_main_branch() {
    local current=$(git_current_branch)
    local main=$(git_main_branch)
    [[ "$current" == "$main" ]]
}

# ============================================================================
# Tag Operations
# ============================================================================

# Check if tag exists locally
# Args: $1 - tag name
# Returns: 0 if exists, 1 otherwise
# Usage: if git_tag_exists "1.0.0"; then ... fi
git_tag_exists() {
    local tag="$1"
    git show-ref --verify --quiet "refs/tags/$tag"
}

# Check if tag exists on remote
# Args: $1 - tag name, $2 - remote (default: origin)
# Returns: 0 if exists, 1 otherwise
# Usage: if git_tag_exists_remote "1.0.0"; then ... fi
git_tag_exists_remote() {
    local tag="$1"
    local remote="${2:-origin}"
    git ls-remote --tags "$remote" | grep -q "refs/tags/$tag$"
}

# Get latest tag
# Args: $1 - pattern (optional, e.g., "v*")
# Returns: Latest tag name
# Usage: latest=$(git_latest_tag)
git_latest_tag() {
    local pattern="${1:-*}"
    git tag --list "$pattern" --sort=-v:refname | head -1
}

# Get all tags sorted by version
# Args: $1 - pattern (optional)
# Returns: List of tags, one per line
# Usage: git_list_tags | head -5
git_list_tags() {
    local pattern="${1:-*}"
    git tag --list "$pattern" --sort=-v:refname
}

# Create annotated tag
# Args: $1 - tag name, $2 - message
# Usage: git_create_tag "1.0.0" "Release version 1.0.0"
# Note: Tag is created on HEAD (current commit), tags do not use 'v' prefix
git_create_tag() {
    local tag="$1"
    local message="$2"
    # Explicitly create tag on HEAD to ensure it's on the current commit
    git tag -a "$tag" -m "$message" HEAD
}

# Delete local tag
# Args: $1 - tag name
# Usage: git_delete_tag "1.0.0"
git_delete_tag() {
    local tag="$1"
    git tag -d "$tag"
}

# Delete remote tag
# Args: $1 - tag name, $2 - remote (default: origin)
# Usage: git_delete_tag_remote "1.0.0"
git_delete_tag_remote() {
    local tag="$1"
    local remote="${2:-origin}"
    git push "$remote" ":refs/tags/$tag"
}

# ============================================================================
# Commit Operations
# ============================================================================

# Stage files
# Args: $@ - files to stage
# Usage: git_stage file1.txt file2.txt
git_stage() {
    git add "$@"
}

# Stage all changes
# Usage: git_stage_all
git_stage_all() {
    git add -A
}

# Create commit
# Args: $1 - commit message
# Usage: git_commit "feat: add new feature"
git_commit() {
    local message="$1"
    git commit -m "$message"
}

# Create commit with staged files
# Args: $1 - commit message
# Usage: git_commit_staged "fix: resolve bug"
git_commit_staged() {
    local message="$1"
    if [[ -z $(git_get_staged_files) ]]; then
        if type print_warning &>/dev/null; then
            print_warning "No staged files to commit"
        fi
        return 1
    fi
    git commit -m "$message"
}

# Amend last commit
# Args: $1 - new message (optional)
# Usage: git_amend "Updated commit message"
git_amend() {
    local message="$1"
    if [[ -n "$message" ]]; then
        git commit --amend -m "$message"
    else
        git commit --amend --no-edit
    fi
}

# Get last commit hash
# Args: $1 - short format (default: true)
# Returns: Commit hash
# Usage: hash=$(git_last_commit)
git_last_commit() {
    local short="${1:-true}"
    if [[ "$short" == "true" ]]; then
        git rev-parse --short HEAD
    else
        git rev-parse HEAD
    fi
}

# Get last commit message
# Returns: Last commit message
# Usage: message=$(git_last_commit_message)
git_last_commit_message() {
    git log -1 --pretty=%B
}

# ============================================================================
# Push/Pull Operations
# ============================================================================

# Push to remote
# Args: $1 - remote (default: origin), $2 - branch (default: current)
# Usage: git_push
git_push() {
    local remote="${1:-origin}"
    local branch="${2:-$(git_current_branch)}"
    git push "$remote" "$branch"
}

# Push tags to remote
# Args: $1 - remote (default: origin), $2 - tag (optional, pushes all if not specified)
# Usage: git_push_tags
git_push_tags() {
    local remote="${1:-origin}"
    local tag="${2:-}"

    if [[ -n "$tag" ]]; then
        git push "$remote" "$tag"
    else
        git push "$remote" --tags
    fi
}

# Force push with lease
# Args: $1 - remote (default: origin), $2 - branch (default: current)
# Usage: git_force_push
git_force_push() {
    local remote="${1:-origin}"
    local branch="${2:-$(git_current_branch)}"
    git push "$remote" "$branch" --force-with-lease
}

# Pull from remote
# Args: $1 - remote (default: origin), $2 - branch (default: current)
# Usage: git_pull
git_pull() {
    local remote="${1:-origin}"
    local branch="${2:-$(git_current_branch)}"
    git pull "$remote" "$branch"
}

# ============================================================================
# History Operations
# ============================================================================

# Reset to specific commit
# Args: $1 - commit hash or HEAD~N, $2 - mode (soft/mixed/hard, default: mixed)
# Usage: git_reset "HEAD~1" "hard"
git_reset() {
    local target="$1"
    local mode="${2:-mixed}"

    case "$mode" in
        soft)
            git reset --soft "$target"
            ;;
        hard)
            git reset --hard "$target"
            ;;
        *)
            git reset "$target"
            ;;
    esac
}

# Get commit count
# Args: $1 - branch (default: current)
# Returns: Number of commits
# Usage: count=$(git_commit_count)
git_commit_count() {
    local branch="${1:-HEAD}"
    git rev-list --count "$branch"
}

# Get commits since tag
# Args: $1 - tag name
# Returns: List of commit messages
# Usage: git_commits_since "1.0.0"
git_commits_since() {
    local tag="$1"
    git log "${tag}..HEAD" --oneline
}

# ============================================================================
# Remote Operations
# ============================================================================

# Get remote URL
# Args: $1 - remote name (default: origin)
# Returns: Remote URL
# Usage: url=$(git_remote_url)
git_remote_url() {
    local remote="${1:-origin}"
    git remote get-url "$remote"
}

# Check if remote exists
# Args: $1 - remote name
# Returns: 0 if exists, 1 otherwise
# Usage: if git_remote_exists "origin"; then ... fi
git_remote_exists() {
    local remote="$1"
    git remote | grep -q "^${remote}$"
}
