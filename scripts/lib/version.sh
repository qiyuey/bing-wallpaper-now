#!/usr/bin/env bash
# version.sh - Semantic Version Utilities
#
# Provides reusable functions for semantic version parsing and manipulation:
# - Version parsing (major.minor.patch)
# - Version comparison
# - Version increment (bump major/minor/patch)
# - Development version handling (-0 suffix)
# - Version validation
#
# Usage:
#   source "$(dirname "${BASH_SOURCE[0]}")/lib/version.sh"
#   if version_is_valid "1.2.3"; then
#       next=$(version_bump_patch "1.2.3")
#   fi

# ============================================================================
# Version Validation
# ============================================================================

# Check if version string is valid semantic version
# Args: $1 - version string
# Returns: 0 if valid, 1 otherwise
# Usage: if version_is_valid "1.2.3"; then ... fi
version_is_valid() {
    local version="$1"
    # Matches: X.Y.Z or X.Y.Z-N (where N is numeric pre-release)
    [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9]+)?$ ]]
}

# Validate version format (MSI compatibility check)
# MSI requires pre-release identifiers to be numeric only
# Args: $1 - version string
# Returns: 0 if valid for MSI, 1 otherwise
# Usage: if version_is_msi_compatible "1.2.3-0"; then ... fi
version_is_msi_compatible() {
    local version="$1"

    # Check for non-numeric pre-release identifier
    if [[ "$version" =~ -[^0-9] ]]; then
        if type print_error &>/dev/null; then
            print_error "Version '$version' contains non-numeric pre-release identifier"
            print_error "MSI build requires numeric pre-release only (e.g., 1.0.0 or 1.0.0-0)"
        fi
        return 1
    fi

    return 0
}

# ============================================================================
# Development Version Detection
# ============================================================================

# Check if version is a development version (ends with -0)
# Args: $1 - version string
# Returns: 0 if development version, 1 otherwise
# Usage: if version_is_dev "1.2.3-0"; then ... fi
version_is_dev() {
    local version="$1"
    [[ "$version" == *"-0" ]]
}

# Check if version is a release version (no -0 suffix)
# Args: $1 - version string
# Returns: 0 if release version, 1 otherwise
# Usage: if version_is_release "1.2.3"; then ... fi
version_is_release() {
    local version="$1"
    [[ "$version" != *"-0" ]] && version_is_valid "$version"
}

# ============================================================================
# Development Suffix Operations
# ============================================================================

# Remove development suffix (-0) from version
# Args: $1 - version string
# Returns: Version without -0 suffix
# Usage: release_version=$(version_remove_dev_suffix "1.2.3-0")
version_remove_dev_suffix() {
    local version="$1"
    echo "${version%-0}"
}

# Add development suffix (-0) to version
# Args: $1 - version string
# Returns: Version with -0 suffix
# Usage: dev_version=$(version_add_dev_suffix "1.2.3")
version_add_dev_suffix() {
    local version="$1"
    echo "${version}-0"
}

# ============================================================================
# Version Parsing
# ============================================================================

# Parse version into components
# Args: $1 - version string
# Sets global variables: VERSION_MAJOR, VERSION_MINOR, VERSION_PATCH, VERSION_PRERELEASE
# Note: Uses global state; not safe for concurrent or nested calls.
# Usage: version_parse "1.2.3-0"
#        echo $VERSION_MAJOR  # 1
#        echo $VERSION_MINOR  # 2
#        echo $VERSION_PATCH  # 3
#        echo $VERSION_PRERELEASE  # 0
version_parse() {
    local version="$1"

    # Remove development suffix for parsing
    local base_version=$(version_remove_dev_suffix "$version")

    VERSION_MAJOR=$(echo "$base_version" | cut -d. -f1)
    VERSION_MINOR=$(echo "$base_version" | cut -d. -f2)
    VERSION_PATCH=$(echo "$base_version" | cut -d. -f3)

    # Check if has pre-release
    if [[ "$version" == *"-"* ]]; then
        VERSION_PRERELEASE=$(echo "$version" | cut -d- -f2)
    else
        VERSION_PRERELEASE=""
    fi

    export VERSION_MAJOR VERSION_MINOR VERSION_PATCH VERSION_PRERELEASE
}

# Get major version number
# Args: $1 - version string
# Returns: Major version number
# Usage: major=$(version_get_major "1.2.3")
version_get_major() {
    local version="$1"
    version_parse "$version"
    echo "$VERSION_MAJOR"
}

# Get minor version number
# Args: $1 - version string
# Returns: Minor version number
# Usage: minor=$(version_get_minor "1.2.3")
version_get_minor() {
    local version="$1"
    version_parse "$version"
    echo "$VERSION_MINOR"
}

# Get patch version number
# Args: $1 - version string
# Returns: Patch version number
# Usage: patch=$(version_get_patch "1.2.3")
version_get_patch() {
    local version="$1"
    version_parse "$version"
    echo "$VERSION_PATCH"
}

# ============================================================================
# Version Increment (Bump)
# ============================================================================

# Bump patch version
# Args: $1 - version string
# Returns: New version with incremented patch
# Usage: new_version=$(version_bump_patch "1.2.3")  # Returns: 1.2.4
version_bump_patch() {
    local version="$1"
    local base_version=$(version_remove_dev_suffix "$version")

    version_parse "$base_version"

    local new_patch=$((VERSION_PATCH + 1))
    echo "${VERSION_MAJOR}.${VERSION_MINOR}.${new_patch}"
}

# Bump minor version (resets patch to 0)
# Args: $1 - version string
# Returns: New version with incremented minor
# Usage: new_version=$(version_bump_minor "1.2.3")  # Returns: 1.3.0
version_bump_minor() {
    local version="$1"
    local base_version=$(version_remove_dev_suffix "$version")

    version_parse "$base_version"

    local new_minor=$((VERSION_MINOR + 1))
    echo "${VERSION_MAJOR}.${new_minor}.0"
}

# Bump major version (resets minor and patch to 0)
# Args: $1 - version string
# Returns: New version with incremented major
# Usage: new_version=$(version_bump_major "1.2.3")  # Returns: 2.0.0
version_bump_major() {
    local version="$1"
    local base_version=$(version_remove_dev_suffix "$version")

    version_parse "$base_version"

    local new_major=$((VERSION_MAJOR + 1))
    echo "${new_major}.0.0"
}

# Bump version by type
# Args: $1 - version string, $2 - bump type (major/minor/patch)
# Returns: New version
# Usage: new_version=$(version_bump "1.2.3" "minor")
version_bump() {
    local version="$1"
    local bump_type="$2"

    case "$bump_type" in
        major)
            version_bump_major "$version"
            ;;
        minor)
            version_bump_minor "$version"
            ;;
        patch)
            version_bump_patch "$version"
            ;;
        *)
            if type print_error &>/dev/null; then
                print_error "Invalid bump type: $bump_type (use: major, minor, or patch)"
            fi
            return 1
            ;;
    esac
}

# ============================================================================
# Version Comparison
# ============================================================================

# Compare two versions
# Args: $1 - version1, $2 - version2
# Returns: 0 if v1 == v2, 1 if v1 > v2, 2 if v1 < v2
# Usage: version_compare "1.2.3" "1.2.4" && echo "equal"
version_compare() {
    local v1="$1"
    local v2="$2"

    # Remove dev suffix for comparison
    v1=$(version_remove_dev_suffix "$v1")
    v2=$(version_remove_dev_suffix "$v2")

    if [[ "$v1" == "$v2" ]]; then
        return 0
    fi

    # Use sort to compare versions
    local sorted=$(printf "%s\n%s" "$v1" "$v2" | sort -V | head -1)

    if [[ "$sorted" == "$v1" ]]; then
        return 2  # v1 < v2
    else
        return 1  # v1 > v2
    fi
}

# Check if version1 is greater than version2
# Args: $1 - version1, $2 - version2
# Returns: 0 if v1 > v2, 1 otherwise
# Usage: if version_gt "1.2.4" "1.2.3"; then ... fi
version_gt() {
    local rc=0
    version_compare "$1" "$2" || rc=$?
    [[ $rc -eq 1 ]]
}

# Check if version1 is less than version2
# Args: $1 - version1, $2 - version2
# Returns: 0 if v1 < v2, 1 otherwise
# Usage: if version_lt "1.2.3" "1.2.4"; then ... fi
version_lt() {
    local rc=0
    version_compare "$1" "$2" || rc=$?
    [[ $rc -eq 2 ]]
}

# Check if version1 equals version2
# Args: $1 - version1, $2 - version2
# Returns: 0 if v1 == v2, 1 otherwise
# Usage: if version_eq "1.2.3" "1.2.3"; then ... fi
version_eq() {
    local rc=0
    version_compare "$1" "$2" || rc=$?
    [[ $rc -eq 0 ]]
}

# Check if version1 is greater than or equal to version2
# Args: $1 - version1, $2 - version2
# Returns: 0 if v1 >= v2, 1 otherwise
# Usage: if version_gte "1.2.4" "1.2.3"; then ... fi
version_gte() {
    local rc=0
    version_compare "$1" "$2" || rc=$?
    [[ $rc -eq 0 || $rc -eq 1 ]]
}

# Check if version1 is less than or equal to version2
# Args: $1 - version1, $2 - version2
# Returns: 0 if v1 <= v2, 1 otherwise
# Usage: if version_lte "1.2.3" "1.2.4"; then ... fi
version_lte() {
    local rc=0
    version_compare "$1" "$2" || rc=$?
    [[ $rc -eq 0 || $rc -eq 2 ]]
}

# ============================================================================
# Version Formatting
# ============================================================================

# Format version for display
# Args: $1 - version string
# Returns: Formatted version string
# Usage: formatted=$(version_format "1.2.3-0")  # Returns: "1.2.3-0 (development)"
version_format() {
    local version="$1"

    if version_is_dev "$version"; then
        echo "$version (development)"
    else
        echo "$version (release)"
    fi
}

# Get version type string
# Args: $1 - version string
# Returns: "development" or "release"
# Usage: type=$(version_get_type "1.2.3-0")
version_get_type() {
    local version="$1"

    if version_is_dev "$version"; then
        echo "development"
    else
        echo "release"
    fi
}
