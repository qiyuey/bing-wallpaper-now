#!/usr/bin/env bash
# project.sh - Project Configuration and Utilities
#
# Provides project-specific configuration and tool detection:
# - Project paths and file locations
# - Package manager detection
# - Tool availability checks
# - Version file reading/writing
#
# Usage:
#   source "$(dirname "${BASH_SOURCE[0]}")/lib/project.sh"
#   pkg_manager=$(project_detect_package_manager)
#   current_version=$(project_get_version)

# ============================================================================
# Project Paths Configuration
# ============================================================================

# Get project root directory
# Returns: Absolute path to project root
# Usage: root=$(project_get_root)
project_get_root() {
    git rev-parse --show-toplevel 2>/dev/null || pwd
}

# Project file paths (relative to project root)
export PROJECT_PACKAGE_JSON="package.json"
export PROJECT_CARGO_TOML="src-tauri/Cargo.toml"
export PROJECT_TAURI_CONF="src-tauri/tauri.conf.json"
export PROJECT_CARGO_LOCK="src-tauri/Cargo.lock"
export PROJECT_CHANGELOG="CHANGELOG.md"

# Get absolute path to project file
# Args: $1 - relative file path
# Returns: Absolute path
# Usage: package_json=$(project_get_file_path "$PROJECT_PACKAGE_JSON")
project_get_file_path() {
    local relative_path="$1"
    local root=$(project_get_root)
    echo "${root}/${relative_path}"
}

# Check if project file exists
# Args: $1 - relative file path
# Returns: 0 if exists, 1 otherwise
# Usage: if project_file_exists "$PROJECT_PACKAGE_JSON"; then ... fi
project_file_exists() {
    local relative_path="$1"
    local abs_path=$(project_get_file_path "$relative_path")
    [[ -f "$abs_path" ]]
}

# ============================================================================
# Package Manager Detection
# ============================================================================

# Detect package manager (pnpm only, npm is not supported)
# Returns: "pnpm"
# Usage: PKG_MANAGER=$(project_detect_package_manager)
project_detect_package_manager() {
    if command -v pnpm &> /dev/null; then
        echo "pnpm"
    else
        if type print_error &>/dev/null; then
            print_error "pnpm is required but not found"
            print_info "Install via: corepack enable && corepack prepare pnpm@latest --activate"
        else
            echo "Error: pnpm is required but not found" >&2
            echo "Install via: corepack enable && corepack prepare pnpm@latest --activate" >&2
        fi
        exit 1
    fi
}

# Check if specific package manager is available
# Args: $1 - package manager name
# Returns: 0 if available, 1 otherwise
# Usage: if project_has_package_manager "pnpm"; then ... fi
project_has_package_manager() {
    local pm="$1"
    command -v "$pm" &> /dev/null
}

# ============================================================================
# Tool Detection
# ============================================================================

# Check if command/tool is available
# Args: $1 - command name
# Returns: 0 if available, 1 otherwise
# Usage: if project_has_tool "cargo"; then ... fi
project_has_tool() {
    local tool="$1"
    command -v "$tool" &> /dev/null
}

# Require tool to be available (exits if not)
# Args: $1 - tool name, $2 - optional error message
# Usage: project_require_tool "cargo" "Rust toolchain required"
project_require_tool() {
    local tool="$1"
    local message="${2:-$tool is required but not found}"

    if ! project_has_tool "$tool"; then
        if type print_error &>/dev/null; then
            print_error "$message"
        else
            echo "Error: $message" >&2
        fi
        exit 1
    fi
}

# Get tool version
# Args: $1 - tool name, $2 - version flag (default: --version)
# Returns: Tool version string
# Usage: version=$(project_get_tool_version "node")
project_get_tool_version() {
    local tool="$1"
    local flag="${2:---version}"

    if project_has_tool "$tool"; then
        "$tool" "$flag" 2>/dev/null | head -1
    else
        echo "not installed"
    fi
}

# ============================================================================
# Version File Operations
# ============================================================================

# Get current version from package.json
# Returns: Version string
# Usage: version=$(project_get_version)
project_get_version() {
    local package_json=$(project_get_file_path "$PROJECT_PACKAGE_JSON")

    if [[ ! -f "$package_json" ]]; then
        if type print_error &>/dev/null; then
            print_error "package.json not found"
        fi
        return 1
    fi

    grep '"version"' "$package_json" | head -1 | sed 's/.*"version": "\(.*\)".*/\1/'
}

# Update version in package.json (top-level .version only)
# Args: $1 - new version
# Requires: jq
# Usage: project_update_package_json_version "1.2.3"
project_update_package_json_version() {
    local new_version="$1"
    local file=$(project_get_file_path "$PROJECT_PACKAGE_JSON")
    local temp_file="${file}.tmp"

    jq --arg v "$new_version" '.version = $v' "$file" > "$temp_file"
    mv "$temp_file" "$file"
}

# Update version in Cargo.toml
# Args: $1 - new version
# Usage: project_update_cargo_toml_version "1.2.3"
project_update_cargo_toml_version() {
    local new_version="$1"
    local file=$(project_get_file_path "$PROJECT_CARGO_TOML")

    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "s/^version = \".*\"/version = \"$new_version\"/" "$file"
    else
        sed -i "s/^version = \".*\"/version = \"$new_version\"/" "$file"
    fi
}

# Update version in tauri.conf.json
# Args: $1 - new version
# Requires: jq
# Usage: project_update_tauri_conf_version "1.2.3"
project_update_tauri_conf_version() {
    local new_version="$1"
    local file=$(project_get_file_path "$PROJECT_TAURI_CONF")
    local temp_file="${file}.tmp"

    jq --arg v "$new_version" '.version = $v' "$file" > "$temp_file"
    mv "$temp_file" "$file"
}

# Update Cargo.lock
# Usage: project_update_cargo_lock
project_update_cargo_lock() {
    local manifest=$(project_get_file_path "$PROJECT_CARGO_TOML")
    cargo update -p bing-wallpaper-now --manifest-path "$manifest" --quiet 2>/dev/null || true
}

# Update version in all project files
# Args: $1 - new version
# Usage: project_update_all_versions "1.2.3"
project_update_all_versions() {
    local new_version="$1"

    if type print_info &>/dev/null; then
        print_info "Updating version to $new_version in all project files..."
    fi

    project_update_package_json_version "$new_version"
    project_update_cargo_toml_version "$new_version"
    project_update_tauri_conf_version "$new_version"
    project_update_cargo_lock

    if type print_success &>/dev/null; then
        print_success "All version files updated to $new_version"
    fi
}

# ============================================================================
# Project Information
# ============================================================================

# Get project name from package.json
# Returns: Project name
# Usage: name=$(project_get_name)
project_get_name() {
    local package_json=$(project_get_file_path "$PROJECT_PACKAGE_JSON")

    if [[ ! -f "$package_json" ]]; then
        echo "unknown"
        return
    fi

    grep '"name"' "$package_json" | head -1 | sed 's/.*"name": "\(.*\)".*/\1/'
}

# Get project description from package.json
# Returns: Project description
# Usage: desc=$(project_get_description)
project_get_description() {
    local package_json=$(project_get_file_path "$PROJECT_PACKAGE_JSON")

    if [[ ! -f "$package_json" ]]; then
        echo ""
        return
    fi

    grep '"description"' "$package_json" | head -1 | sed 's/.*"description": "\(.*\)".*/\1/'
}

# Print project information
# Usage: project_print_info
project_print_info() {
    local name=$(project_get_name)
    local version=$(project_get_version)
    local desc=$(project_get_description)
    local pkg_manager=$(project_detect_package_manager)
    local root=$(project_get_root)

    if type print_table_row &>/dev/null; then
        echo ""
        print_table_row "Project Name" "$name"
        print_table_row "Version" "$version"
        print_table_row "Description" "$desc"
        print_table_row "Package Manager" "$pkg_manager"
        print_table_row "Project Root" "$root"
        print_table_row "Node.js" "$(project_get_tool_version node)"
        print_table_row "Rust" "$(project_get_tool_version rustc)"
        echo ""
    else
        echo ""
        echo "Project: $name"
        echo "Version: $version"
        echo "Description: $desc"
        echo "Package Manager: $pkg_manager"
        echo "Root: $root"
        echo ""
    fi
}

# ============================================================================
# OS Detection
# ============================================================================

# Check if running on macOS
# Returns: 0 if macOS, 1 otherwise
# Usage: if project_is_macos; then ... fi
project_is_macos() {
    [[ "$OSTYPE" == "darwin"* ]]
}

# Check if running on Linux
# Returns: 0 if Linux, 1 otherwise
# Usage: if project_is_linux; then ... fi
project_is_linux() {
    [[ "$OSTYPE" == "linux-gnu"* ]]
}

# Check if running on Windows (Git Bash/WSL)
# Returns: 0 if Windows, 1 otherwise
# Usage: if project_is_windows; then ... fi
project_is_windows() {
    [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" || "$OSTYPE" == "cygwin" ]]
}

# Get OS name
# Returns: "macos", "linux", "windows", or "unknown"
# Usage: os=$(project_get_os)
project_get_os() {
    if project_is_macos; then
        echo "macos"
    elif project_is_linux; then
        echo "linux"
    elif project_is_windows; then
        echo "windows"
    else
        echo "unknown"
    fi
}
