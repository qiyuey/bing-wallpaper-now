#!/usr/bin/env bash
#
# coverage-rust.sh
#
# Purpose:
#   Generate line coverage for the Rust backend (Tauri crate) using cargo-tarpaulin,
#   store machine-readable reports, and optionally enforce a minimum threshold.
#
# Usage:
#   bash .config/scripts/coverage-rust.sh [--threshold <PERCENT>] [--features "<FEATURES>"] [--open] [--fail-on-zero] [--] [TARPAULIN_EXTRA_ARGS...]
#
# Examples:
#   bash .config/scripts/coverage-rust.sh
#   bash .config/scripts/coverage-rust.sh --threshold 70
#   bash .config/scripts/coverage-rust.sh --features "native-wp macos" --threshold 65
#   COVERAGE_THRESHOLD=75 bash .config/scripts/coverage-rust.sh
#
# Behavior:
#   - Ensures cargo-tarpaulin is installed.
#   - Runs coverage for the crate at `src-tauri/Cargo.toml`.
#   - Produces JSON and XML reports under `coverage/rust/`.
#   - Prints a summary and enforces an optional threshold.
#   - Ignores network-bound ignored tests (those marked with #[ignore]) by default.
#
# Notes:
#   - Set BING_TEST=1 to include ignored network tests (not recommended for deterministic CI).
#   - Threshold can be passed via --threshold or env COVERAGE_THRESHOLD.
#   - If jq is available, parsing uses jq; otherwise falls back to text parsing.
#   - This script is idempotent; re-running overwrites prior reports.
#
# Exit Codes:
#   0  Success (coverage computed and threshold satisfied or not set)
#   1  Coverage below threshold
#   2  cargo-tarpaulin invocation failure
#   3  Unable to parse coverage results
#
# License: MIT (same as project)
#

set -euo pipefail

# ---------------------------
# Color helpers
# ---------------------------
if [[ -t 1 ]]; then
  _BOLD=$'\e[1m'
  _RED=$'\e[31m'
  _GREEN=$'\e[32m'
  _YELLOW=$'\e[33m'
  _BLUE=$'\e[34m'
  _RESET=$'\e[0m'
else
  _BOLD=''
  _RED=''
  _GREEN=''
  _YELLOW=''
  _BLUE=''
  _RESET=''
fi

# ---------------------------
# Default configuration
# ---------------------------
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
CRATE_MANIFEST="${ROOT_DIR}/src-tauri/Cargo.toml"
OUTPUT_DIR="${ROOT_DIR}/coverage/rust"
THRESHOLD="${COVERAGE_THRESHOLD:-}"   # from env or overridden by --threshold
FEATURES=""
OPEN_REPORT="false"
FAIL_ON_ZERO="false"

# ---------------------------
# Argument parsing
# ---------------------------
print_usage() {
  sed -n '1,70p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
}

EXTRA_ARGS=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --threshold)
      shift
      THRESHOLD="${1:-}"
      ;;
    --features)
      shift
      FEATURES="${1:-}"
      ;;
    --open)
      OPEN_REPORT="true"
      ;;
    --fail-on-zero)
      FAIL_ON_ZERO="true"
      ;;
    --help|-h)
      print_usage
      exit 0
      ;;
    --)
      shift
      # Everything after -- is passed directly to tarpaulin
      EXTRA_ARGS+=("$@")
      break
      ;;
    *)
      # Pass through unknown args to tarpaulin
      EXTRA_ARGS+=("$1")
      ;;
  esac
  shift || true
done

# Normalize threshold
if [[ -n "${THRESHOLD}" ]]; then
  if ! [[ "${THRESHOLD}" =~ ^[0-9]+([.][0-9]+)?$ ]]; then
    echo "${_RED}Invalid threshold '${THRESHOLD}'. Must be a numeric percentage (e.g. 70 or 72.5).${_RESET}"
    exit 1
  fi
fi

# ---------------------------
# Pre-flight checks
# ---------------------------
if [[ ! -f "${CRATE_MANIFEST}" ]]; then
  echo "${_RED}Rust manifest not found at: ${CRATE_MANIFEST}${_RESET}"
  exit 2
fi

mkdir -p "${OUTPUT_DIR}"

echo "${_BLUE}${_BOLD}==> Rust coverage (tarpaulin) starting...${_RESET}"
echo "Root:        ${ROOT_DIR}"
echo "Crate:       ${CRATE_MANIFEST}"
echo "Output dir:  ${OUTPUT_DIR}"
[[ -n "${THRESHOLD}" ]] && echo "Threshold:   ${THRESHOLD}%"
[[ -n "${FEATURES}" ]] && echo "Features:    ${FEATURES}"
echo "Include ignored tests (BING_TEST=1)? $( [[ "${BING_TEST:-}" == "1" ]] && echo 'Yes' || echo 'No' )"

# ---------------------------
# Install cargo-tarpaulin if missing
# ---------------------------
if ! command -v cargo-tarpaulin >/dev/null 2>&1; then
  echo "${_YELLOW}cargo-tarpaulin not found; installing (may take a moment)...${_RESET}"
  cargo install cargo-tarpaulin
fi

# ---------------------------
# Build tarpaulin command
# ---------------------------
CMD=(cargo tarpaulin
     --manifest-path "${CRATE_MANIFEST}"
     --out Xml
     --out Json
     --timeout 180
     --exclude-files "src-tauri/target/*"
     --ignore-tests
)

# If user explicitly wants ignored tests (network) they set BING_TEST=1
if [[ "${BING_TEST:-}" == "1" ]]; then
  # Remove --ignore-tests to include #[ignore] tests
  CMD=("${CMD[@]/--ignore-tests}")
fi

# Add features if provided
if [[ -n "${FEATURES}" ]]; then
  CMD+=(--features "${FEATURES}")
fi

# Pass-through extra args
if [[ ${#EXTRA_ARGS[@]} -gt 0 ]]; then
  CMD+=("${EXTRA_ARGS[@]}")
fi

echo "${_BLUE}Running: ${_RESET}${CMD[*]}"
set +e
TARP_LOG="$("${CMD[@]}" 2>&1)"
TARP_EXIT=$?
set -e

echo "${TARP_LOG}" > "${OUTPUT_DIR}/tarpaulin-run.log"

if [[ ${TARP_EXIT} -ne 0 ]]; then
  echo "${_RED}Tarpaulin failed (exit ${TARP_EXIT}). See ${OUTPUT_DIR}/tarpaulin-run.log${_RESET}"
  exit 2
fi

# ---------------------------
# Parse coverage result
# ---------------------------
JSON_REPORT="${OUTPUT_DIR}/tarpaulin-report.json"
XML_REPORT="${OUTPUT_DIR}/tarpaulin-report.xml"

# Tarpaulin names output files in current working directory by default
# Move them if present
[[ -f "tarpaulin-report.json" ]] && mv "tarpaulin-report.json" "${JSON_REPORT}"
[[ -f "tarpaulin-report.xml"  ]] && mv "tarpaulin-report.xml"  "${XML_REPORT}"

LINE_COV=""

if [[ -f "${JSON_REPORT}" ]]; then
  if command -v jq >/dev/null 2>&1; then
    # Attempt jq parse (structure may vary by tarpaulin version; fallback to grep if needed)
    LINE_COV="$(jq -r '.coverage.line // .line // .["line"] // empty' "${JSON_REPORT}" 2>/dev/null || true)"
    # Tarpaulin may store coverage as an object with overall summary; fallback parsing
  fi
fi

if [[ -z "${LINE_COV}" ]]; then
  # Fallback textual parse from tarpaulin log output
  # Looks for a pattern like "lines: 85.71%"
  LINE_COV="$(grep -Eo 'lines:[[:space:]]+[0-9]+(\.[0-9]+)?%' "${OUTPUT_DIR}/tarpaulin-run.log" | tail -n1 | sed -E 's/.*lines:[[:space:]]+([0-9]+(\.[0-9]+)?)%.*/\1/' || true)"
fi

if [[ -z "${LINE_COV}" ]]; then
  echo "${_RED}Unable to parse line coverage percentage.${_RESET}"
  exit 3
fi

echo "${_BOLD}Line Coverage:${_RESET} ${LINE_COV}%"

# Fail on zero if requested
if [[ "${FAIL_ON_ZERO}" == "true" ]]; then
  awk 'BEGIN { cov='"${LINE_COV}"'; if (cov < 0.01) exit 1 }' || {
    echo "${_RED}Coverage appears to be ~0%; build or instrumentation failure?${_RESET}"
    exit 1
  }
fi

# Threshold enforcement
if [[ -n "${THRESHOLD}" ]]; then
  awk 'BEGIN { got='"${LINE_COV}"'; want='"${THRESHOLD}"'; if (got + 0 < want + 0) exit 1 }' || {
    echo "${_RED}Coverage (${LINE_COV}%) below threshold (${THRESHOLD}%).${_RESET}"
    exit 1
  }
  echo "${_GREEN}Coverage (${LINE_COV}%) meets threshold (${THRESHOLD}%).${_RESET}"
else
  echo "${_YELLOW}No threshold set; skipping enforcement.${_RESET}"
fi

# ---------------------------
# Optional: open XML report (e.g. in coverage viewer)
# ---------------------------
if [[ "${OPEN_REPORT}" == "true" && -f "${XML_REPORT}" ]]; then
  if command -v xdg-open >/dev/null 2>&1; then
    xdg-open "${XML_REPORT}" >/dev/null 2>&1 || true
  elif command -v open >/dev/null 2>&1; then
    open "${XML_REPORT}" >/dev/null 2>&1 || true
  fi
fi

# ---------------------------
# Summary file
# ---------------------------
SUMMARY_FILE="${OUTPUT_DIR}/summary.json"
cat > "${SUMMARY_FILE}" <<EOF
{
  "line_coverage_percent": ${LINE_COV},
  "threshold_enforced": $( [[ -n "${THRESHOLD}" ]] && echo true || echo false ),
  "threshold": $( [[ -n "${THRESHOLD}" ]] && echo "${THRESHOLD}" || echo null ),
  "manifest": "$(basename "${CRATE_MANIFEST}")",
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "ignored_tests_included": $( [[ "${BING_TEST:-}" == "1" ]] && echo true || echo false )
}
EOF

echo "${_BLUE}Report artifacts:${_RESET}"
echo "  JSON:    ${JSON_REPORT}"
echo "  XML:     ${XML_REPORT}"
echo "  Summary: ${SUMMARY_FILE}"
echo "  Log:     ${OUTPUT_DIR}/tarpaulin-run.log"

echo "${_GREEN}${_BOLD}==> Rust coverage complete.${_RESET}"
exit 0
