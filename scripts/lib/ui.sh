#!/usr/bin/env bash
# ui.sh - UI and Output Utilities
#
# Provides reusable functions for terminal output formatting:
# - Color definitions
# - Print functions (success, error, warning, info)
# - Progress indicators
# - Separators and headers
#
# Usage:
#   source "$(dirname "${BASH_SOURCE[0]}")/lib/ui.sh"
#   print_success "Operation completed"
#   print_error "Something went wrong"

# ============================================================================
# Color Definitions
# ============================================================================

export COLOR_RESET='\033[0m'
export COLOR_BOLD='\033[1m'
export COLOR_DIM='\033[2m'

# Foreground colors
export COLOR_BLACK='\033[30m'
export COLOR_RED='\033[31m'
export COLOR_GREEN='\033[32m'
export COLOR_YELLOW='\033[33m'
export COLOR_BLUE='\033[34m'
export COLOR_MAGENTA='\033[35m'
export COLOR_CYAN='\033[36m'
export COLOR_WHITE='\033[37m'

# Bright foreground colors
export COLOR_BRIGHT_RED='\033[91m'
export COLOR_BRIGHT_GREEN='\033[92m'
export COLOR_BRIGHT_YELLOW='\033[93m'
export COLOR_BRIGHT_BLUE='\033[94m'
export COLOR_BRIGHT_MAGENTA='\033[95m'
export COLOR_BRIGHT_CYAN='\033[96m'

# ============================================================================
# Print Functions
# ============================================================================

# Print success message
# Usage: print_success "Operation completed"
print_success() {
    printf "${COLOR_GREEN}✅ %s${COLOR_RESET}\n" "$1"
}

# Print error message
# Usage: print_error "Operation failed"
print_error() {
    printf "${COLOR_RED}❌ %s${COLOR_RESET}\n" "$1" >&2
}

# Print warning message
# Usage: print_warning "Potential issue detected"
print_warning() {
    printf "${COLOR_YELLOW}⚠️  %s${COLOR_RESET}\n" "$1"
}

# Print info message
# Usage: print_info "Processing data..."
print_info() {
    printf "${COLOR_BLUE}ℹ️  %s${COLOR_RESET}\n" "$1"
}

# Print debug message (only if DEBUG=1)
# Usage: print_debug "Variable value: $var"
print_debug() {
    if [[ "${DEBUG:-0}" == "1" ]]; then
        printf "${COLOR_DIM}🐛 %s${COLOR_RESET}\n" "$1" >&2
    fi
}

# ============================================================================
# Header and Separator Functions
# ============================================================================

# Print section header
# Usage: print_header "Build Process"
print_header() {
    printf "\n${COLOR_BOLD}${COLOR_CYAN}%s${COLOR_RESET}\n" "$1"
}

# Print major header with box
# Usage: print_major_header "Release Process"
print_major_header() {
    local text="$1"
    local length=${#text}
    local padding=4
    local total_length=$((length + padding * 2))

    printf "\n${COLOR_BOLD}${COLOR_BLUE}"
    printf '╔'
    printf '═%.0s' $(seq 1 $total_length)
    printf '╗\n'
    printf "║%*s%s%*s║\n" $padding "" "$text" $padding ""
    printf '╚'
    printf '═%.0s' $(seq 1 $total_length)
    printf '╝'
    printf "${COLOR_RESET}\n\n"
}

# Print separator line
# Usage: print_separator
print_separator() {
    printf "${COLOR_BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${COLOR_RESET}\n"
}

# Print thin separator
# Usage: print_thin_separator
print_thin_separator() {
    printf "${COLOR_DIM}────────────────────────────────────────────────────────────${COLOR_RESET}\n"
}

# ============================================================================
# Step and Progress Functions
# ============================================================================

# Print step indicator
# Usage: print_step 1 5 "Compiling code"
print_step() {
    local current=$1
    local total=$2
    local description=$3
    printf "\n${COLOR_BOLD}${COLOR_BLUE}[%d/%d]${COLOR_RESET} ${COLOR_BOLD}%s${COLOR_RESET}\n" \
        "$current" "$total" "$description"
}

# Print progress indicator (spinning wheel)
# Usage: print_progress "Loading..."
print_progress() {
    local message="$1"
    printf "${COLOR_CYAN}⏳ %s${COLOR_RESET}" "$message"
}

# Print completion checkmark
# Usage: print_complete
print_complete() {
    printf " ${COLOR_GREEN}✓${COLOR_RESET}\n"
}

# ============================================================================
# Prompt Functions
# ============================================================================

# Ask yes/no question (auto-confirms when YES=1)
# Usage: if ask_yes_no "Continue?"; then ... fi
ask_yes_no() {
    local question="$1"
    local default="${2:-N}"

    if [[ "${YES:-0}" == "1" ]]; then
        printf "${COLOR_YELLOW}❓ %s → auto-confirmed (YES=1)${COLOR_RESET}\n" "$question"
        return 0
    fi

    if [[ "$default" == "Y" ]]; then
        printf "${COLOR_YELLOW}❓ %s [Y/n] ${COLOR_RESET}" "$question"
    else
        printf "${COLOR_YELLOW}❓ %s [y/N] ${COLOR_RESET}" "$question"
    fi

    read -r response

    if [[ -z "$response" ]]; then
        response="$default"
    fi

    [[ "$response" =~ ^[Yy]$ ]]
}

# Ask for confirmation (requires typing "yes")
# Usage: if ask_confirmation "Delete all files?"; then ... fi
ask_confirmation() {
    local question="$1"
    printf "${COLOR_BRIGHT_RED}⚠️  %s${COLOR_RESET}\n" "$question"
    printf "${COLOR_YELLOW}Type 'yes' to confirm: ${COLOR_RESET}"
    read -r response
    [[ "$response" == "yes" ]]
}

# ============================================================================
# Counter Functions
# ============================================================================

# Initialize counters (use in scripts that track success/failure)
# Usage: init_counters
init_counters() {
    export UI_COUNTER_PASSED=0
    export UI_COUNTER_FAILED=0
    export UI_COUNTER_SKIPPED=0
}

# Increment passed counter
# Usage: increment_passed
increment_passed() {
    UI_COUNTER_PASSED=$((UI_COUNTER_PASSED + 1))
}

# Increment failed counter
# Usage: increment_failed
increment_failed() {
    UI_COUNTER_FAILED=$((UI_COUNTER_FAILED + 1))
}

# Increment skipped counter
# Usage: increment_skipped
increment_skipped() {
    UI_COUNTER_SKIPPED=$((UI_COUNTER_SKIPPED + 1))
}

# Print counter summary
# Usage: print_counter_summary
print_counter_summary() {
    local total=$((UI_COUNTER_PASSED + UI_COUNTER_FAILED + UI_COUNTER_SKIPPED))

    printf "\n"
    print_separator
    printf "${COLOR_BOLD}Summary:${COLOR_RESET}\n"
    print_separator
    printf "Total:   ${COLOR_BOLD}%d${COLOR_RESET}\n" "$total"
    printf "Passed:  ${COLOR_GREEN}${COLOR_BOLD}%d${COLOR_RESET}\n" "$UI_COUNTER_PASSED"
    printf "Failed:  ${COLOR_RED}${COLOR_BOLD}%d${COLOR_RESET}\n" "$UI_COUNTER_FAILED"

    if [[ $UI_COUNTER_SKIPPED -gt 0 ]]; then
        printf "Skipped: ${COLOR_YELLOW}${COLOR_BOLD}%d${COLOR_RESET}\n" "$UI_COUNTER_SKIPPED"
    fi

    printf "\n"
}

# ============================================================================
# Table Functions
# ============================================================================

# Print table row
# Usage: print_table_row "Key" "Value"
print_table_row() {
    local key="$1"
    local value="$2"
    local key_width=20

    printf "%-${key_width}s ${COLOR_BOLD}%s${COLOR_RESET}\n" "$key:" "$value"
}

# ============================================================================
# Utility Functions
# ============================================================================

# Disable color output (useful for CI or non-interactive environments)
# Usage: disable_colors
disable_colors() {
    COLOR_RESET=''
    COLOR_BOLD=''
    COLOR_DIM=''
    COLOR_BLACK=''
    COLOR_RED=''
    COLOR_GREEN=''
    COLOR_YELLOW=''
    COLOR_BLUE=''
    COLOR_MAGENTA=''
    COLOR_CYAN=''
    COLOR_WHITE=''
    COLOR_BRIGHT_RED=''
    COLOR_BRIGHT_GREEN=''
    COLOR_BRIGHT_YELLOW=''
    COLOR_BRIGHT_BLUE=''
    COLOR_BRIGHT_MAGENTA=''
    COLOR_BRIGHT_CYAN=''
}

# Auto-detect if colors should be disabled
# Usage: auto_detect_colors
auto_detect_colors() {
    if [[ ! -t 1 ]] || [[ "${NO_COLOR:-}" == "1" ]] || [[ "${TERM:-}" == "dumb" ]]; then
        disable_colors
    fi
}

# Initialize UI (call this at the start of scripts)
# Usage: init_ui
init_ui() {
    auto_detect_colors
}
