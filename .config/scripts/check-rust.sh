#!/usr/bin/env bash
#
# Rust Quality Gate Script
#
# Performs a consolidated set of Rust quality checks for the Tauri backend:
# 1. Formatting (cargo fmt --check)
# 2. Clippy (deny all warnings)
# 3. Tests (unit/integration)
# 4. Optional network tests (BING_TEST=1 env)
# 5. Optional feature matrix or all-features runs
#
# Location: .config/scripts/check-rust.sh
#
# Typical usage:
#   bash .config/scripts/check-rust.sh
#   bash .config/scripts/check-rust.sh --all-features
#   bash .config/scripts/check-rust.sh --features "foo,bar"
#   bash .config/scripts/check-rust.sh --fix          # auto-fix fmt & attempt clippy suggestions
#   BING_TEST=1 bash .config/scripts/check-rust.sh    # include ignored network tests
#
# Exit codes:
#   0 -> success
#   1 -> usage error / unknown flag
#   >1 -> underlying cargo failure
#
# Notes:
# - Clippy runs with -D warnings to enforce zero-warning policy.
# - Network tests are ignored by default; only run if BING_TEST=1 is set.
# - The script is idempotent & safe to re-run locally or in CI.
#
# Future extensions (placeholder):
# - Add coverage integration (tarpaulin) gating.
# - Add per-module lint exceptions (controlled allow lists).
# - Add differential (git diff) filtered clippy runs for faster PR feedback.

set -euo pipefail

#######################################
# Color helpers
#######################################
color() {
  local code="$1"; shift
  printf "\033[%sm%s\033[0m" "${code}" "$*"
}
green() { color "32" "$*"; }
red()   { color "31" "$*"; }
yellow(){ color "33" "$*"; }
blue()  { color "34" "$*"; }

#######################################
# Usage
#######################################
usage() {
  cat <<'EOF'
Rust quality gate script.

Flags:
  --all-features        Run cargo commands with --all-features.
  --features "<list>"   Comma-separated feature list (mutually exclusive with --all-features).
  --fix                 Apply formatting and attempt clippy auto-fixes (experimental).
  --help                Show this help.

Environment:
  BING_TEST=1           Also run ignored network tests (Bing API).

Examples:
  bash .config/scripts/check-rust.sh
  bash .config/scripts/check-rust.sh --all-features
  bash .config/scripts/check-rust.sh --features "native,experimental"
  BING_TEST=1 bash .config/scripts/check-rust.sh --all-features
EOF
}

#######################################
# Parse flags
#######################################
ALL_FEATURES=0
FEATURE_LIST=""
AUTO_FIX=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --all-features)
      ALL_FEATURES=1
      ;;
    --features)
      shift || { red "Missing value for --features"; exit 1; }
      FEATURE_LIST="$1"
      ;;
    --fix)
      AUTO_FIX=1
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      red "Unknown argument: $1"
      usage
      exit 1
      ;;
  esac
  shift
done

if [[ $ALL_FEATURES -eq 1 && -n "$FEATURE_LIST" ]]; then
  red "Cannot use --all-features and --features together."
  exit 1
fi

#######################################
# Determine project paths
#######################################
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
CRATE_DIR="${REPO_ROOT}/src-tauri"

if [[ ! -f "${CRATE_DIR}/Cargo.toml" ]]; then
  red "Cargo.toml not found at expected location: ${CRATE_DIR}"
  exit 1
fi

#######################################
# Build cargo common flags
#######################################
CARGO_FEATURE_FLAGS=()
if [[ $ALL_FEATURES -eq 1 ]]; then
  CARGO_FEATURE_FLAGS+=(--all-features)
elif [[ -n "$FEATURE_LIST" ]]; then
  # Normalize commas/spaces
  CLEAN_FEATURES="$(echo "${FEATURE_LIST}" | tr -d ' ' )"
  CARGO_FEATURE_FLAGS+=(--features "${CLEAN_FEATURES}")
fi

#######################################
# Step wrappers
#######################################
run_step() {
  local name="$1"; shift
  local cmd=("$@")
  echo -e "$(blue "==>") $(yellow "${name}")"
  if "${cmd[@]}"; then
    echo "    $(green "✓ Success")"
  else
    echo "    $(red "✗ Failed")"
    return 1
  fi
}

#######################################
# fmt (check or fix)
#######################################
do_fmt() {
  if [[ $AUTO_FIX -eq 1 ]]; then
    cargo fmt --manifest-path "${CRATE_DIR}/Cargo.toml"
  else
    cargo fmt --manifest-path "${CRATE_DIR}/Cargo.toml" -- --check
  fi
}

#######################################
# clippy
#######################################
do_clippy() {
  if [[ $AUTO_FIX -eq 1 ]]; then
    # Attempt experimental fixes (not all lints auto-fixable)
    cargo clippy --manifest-path "${CRATE_DIR}/Cargo.toml" \
      "${CARGO_FEATURE_FLAGS[@]}" \
      --fix --allow-dirty --allow-staged -- -D warnings || {
        yellow "Clippy fix pass encountered failures; rerunning without --fix to surface diagnostics."
        cargo clippy --manifest-path "${CRATE_DIR}/Cargo.toml" \
          "${CARGO_FEATURE_FLAGS[@]}" \
          -- -D warnings
      }
  else
    cargo clippy --manifest-path "${CRATE_DIR}/Cargo.toml" \
      "${CARGO_FEATURE_FLAGS[@]}" \
      -- -D warnings
  fi
}

#######################################
# tests (normal + optional network)
#######################################
do_tests() {
  cargo test --manifest-path "${CRATE_DIR}/Cargo.toml" \
    "${CARGO_FEATURE_FLAGS[@]}" \
    --all-targets -- --nocapture
}

do_network_tests() {
  # Ignored network tests
  cargo test --manifest-path "${CRATE_DIR}/Cargo.toml" \
    "${CARGO_FEATURE_FLAGS[@]}" \
    -- --ignored --nocapture
}

#######################################
# Summary tracking
#######################################
declare -a SUMMARY_OK
declare -a SUMMARY_FAIL

record_result() {
  local name="$1" status="$2"
  if [[ $status -eq 0 ]]; then
    SUMMARY_OK+=("$name")
  else
    SUMMARY_FAIL+=("$name")
  fi
}

#######################################
# Execution
#######################################
START_TS=$(date +%s)

echo "$(blue "Rust Quality Checks")"
echo "Repository root: ${REPO_ROOT}"
echo "Crate dir      : ${CRATE_DIR}"
echo "Features       : $( [[ $ALL_FEATURES -eq 1 ]] && echo 'ALL' || echo "${FEATURE_LIST:-<none>}")"
echo "Auto-fix mode  : $( [[ $AUTO_FIX -eq 1 ]] && echo 'ENABLED' || echo 'DISABLED')"
echo "Network tests  : $( [[ "${BING_TEST:-}" == "1" ]] && echo 'ENABLED' || echo 'DISABLED')"
echo

# 1. Formatting
if run_step "Formatting (cargo fmt)" do_fmt; then
  record_result "fmt" 0
else
  record_result "fmt" 1
fi

# 2. Clippy
if run_step "Clippy (deny warnings)" do_clippy; then
  record_result "clippy" 0
else
  record_result "clippy" 1
fi

# 3. Tests
if run_step "Tests (all targets)" do_tests; then
  record_result "tests" 0
else
  record_result "tests" 1
fi

# 4. Optional network tests
if [[ "${BING_TEST:-}" == "1" ]]; then
  if run_step "Network tests (ignored group)" do_network_tests; then
    record_result "network-tests" 0
  else
    record_result "network-tests" 1
  fi
fi

END_TS=$(date +%s)
DURATION=$((END_TS - START_TS))

#######################################
# Summary
#######################################
echo
echo "$(blue "Summary")"
for ok in "${SUMMARY_OK[@]:-}"; do
  echo "  $(green "[OK]") ${ok}"
done
for fail in "${SUMMARY_FAIL[@]:-}"; do
  echo "  $(red "[FAIL]") ${fail}"
done
echo
echo "Total duration: ${DURATION}s"

if [[ ${#SUMMARY_FAIL[@]} -gt 0 ]]; then
  red "One or more steps failed."
  exit 2
fi

green "All Rust quality checks passed."
exit 0
