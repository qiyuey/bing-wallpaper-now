#!/usr/bin/env bash
#
# Frontend Lint Script
# Centralized wrapper for ESLint (and optional Prettier) with standardized config path.
#
# Features:
# - Uses centralized config: .config/eslint/.eslintrc.cjs
# - Optional auto-install of missing lint dependencies
# - Supports --fix, --pattern override, --prettier check/write, warning threshold
# - JSON parsing to enforce max warnings and fail on errors
#
# Usage:
#   ./lint-frontend.sh [options]
#
# Options:
#   --fix                Apply autofixes (ESLint + Prettier if --prettier used with --fix)
#   --prettier           Also run Prettier (check mode by default; write when combined with --fix)
#   --pattern "<glob>"   Override default file pattern (default: src/**/*.{ts,tsx})
#   --max-warnings <n>   Fail if warnings exceed <n> (default: unlimited unless specified)
#   --no-install         Skip dependency auto-install (assumes ESLint & Prettier already installed)
#   --cache              Enable ESLint caching (to .cache/eslint/)
#   --write              Alias for --fix (for people used to Prettier semantics)
#   --quiet              Suppress non-error warnings output (does NOT change warning counting)
#   --help               Show help
#
# Examples:
#   ./lint-frontend.sh
#   ./lint-frontend.sh --fix
#   ./lint-frontend.sh --prettier --pattern "src/**/*.{ts,tsx,js,jsx}"
#   ./lint-frontend.sh --max-warnings 0
#
# Exit Codes:
#   0 - Success (errors=0 and warnings <= threshold)
#   1 - ESLint errors present
#   2 - Warnings exceeded --max-warnings threshold
#   3 - Missing required tools / config
#   4 - Invalid arguments or runtime failure
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"

CONFIG_ESLINT="${REPO_ROOT}/.config/eslint/.eslintrc.cjs"
PRETTIER_CONFIG="${REPO_ROOT}/.config/prettier/.prettierrc"
PRETTIER_IGNORE="${REPO_ROOT}/.config/prettier/.prettierignore"

DEFAULT_PATTERN='src/**/*.{ts,tsx}'
PATTERN="${DEFAULT_PATTERN}"

RUN_FIX="false"
RUN_PRETTIER="false"
MAX_WARNINGS="-1"
AUTO_INSTALL="true"
USE_CACHE="false"
QUIET="false"

# Parse arguments
while [[ $# -gt 0 ]]; do
  case "$1" in
    --fix|--write)
      RUN_FIX="true"
      shift
      ;;
    --prettier)
      RUN_PRETTIER="true"
      shift
      ;;
    --pattern)
      [[ $# -lt 2 ]] && { echo "ERROR: --pattern requires an argument" >&2; exit 4; }
      PATTERN="$2"
      shift 2
      ;;
    --max-warnings)
      [[ $# -lt 2 ]] && { echo "ERROR: --max-warnings requires a number" >&2; exit 4; }
      MAX_WARNINGS="$2"
      if ! [[ "${MAX_WARNINGS}" =~ ^[0-9]+$ ]]; then
        echo "ERROR: --max-warnings must be a non-negative integer" >&2
        exit 4
      fi
      shift 2
      ;;
    --no-install)
      AUTO_INSTALL="false"
      shift
      ;;
    --cache)
      USE_CACHE="true"
      shift
      ;;
    --quiet)
      QUIET="true"
      shift
      ;;
    --help|-h)
      grep '^# ' "${BASH_SOURCE[0]}" | sed 's/^# //'
      exit 0
      ;;
    *)
      echo "ERROR: Unknown argument: $1" >&2
      exit 4
      ;;
  esac
done

cd "${REPO_ROOT}"

# Sanity checks
if [[ ! -f "${CONFIG_ESLINT}" ]]; then
  echo "ERROR: ESLint config not found at ${CONFIG_ESLINT}" >&2
  exit 3
fi

# Tool detection helpers
need_install() {
  local bin="$1"
  if ! command -v "${bin}" >/dev/null 2>&1; then
    return 0
  fi
  return 1
}

if [[ "${AUTO_INSTALL}" == "true" ]]; then
  INSTALL_LIST=()
  need_install eslint && INSTALL_LIST+=("eslint")
  need_install prettier && [[ "${RUN_PRETTIER}" == "true" ]] && INSTALL_LIST+=("prettier")
  # eslint plugins (best effort; they may already exist or be unnecessary if using workspace)
  for pkg in @typescript-eslint/parser @typescript-eslint/eslint-plugin eslint-plugin-react eslint-plugin-react-hooks eslint-plugin-import eslint-config-prettier eslint-plugin-prettier; do
    if ! (npm ls --depth=0 "${pkg}" >/dev/null 2>&1); then
      INSTALL_LIST+=("${pkg}")
    fi
  done
  if [[ ${#INSTALL_LIST[@]} -gt 0 ]]; then
    echo "Installing missing lint dependencies: ${INSTALL_LIST[*]}"
    npm i -D "${INSTALL_LIST[@]}"
  fi
fi

if ! command -v eslint >/dev/null 2>&1; then
  echo "ERROR: eslint not available; install or omit --no-install" >&2
  exit 3
fi
if [[ "${RUN_PRETTIER}" == "true" ]] && ! command -v prettier >/dev/null 2>&1; then
  echo "ERROR: prettier requested but not available; install or omit --prettier" >&2
  exit 3
fi

# Prettier step (check or write)
if [[ "${RUN_PRETTIER}" == "true" ]]; then
  echo "==> Running Prettier (${RUN_FIX})"
  PRETTIER_ARGS=(--config "${PRETTIER_CONFIG}")
  [[ -f "${PRETTIER_IGNORE}" ]] && PRETTIER_ARGS+=(--ignore-path "${PRETTIER_IGNORE}")
  PRETTIER_TARGETS=( "src/**/*.{ts,tsx,js,jsx,css,scss,md,json}" )

  if [[ "${RUN_FIX}" == "true" ]]; then
    npx prettier "${PRETTIER_ARGS[@]}" --write "${PRETTIER_TARGETS[@]}"
  else
    npx prettier "${PRETTIER_ARGS[@]}" --check "${PRETTIER_TARGETS[@]}"
  fi
fi

# ESLint run (JSON output for counting)
TMP_DIR="$(mktemp -d)"
REPORT_JSON="${TMP_DIR}/eslint-report.json"
REPORT_TEXT="${TMP_DIR}/eslint-stylish.txt"

echo "==> Running ESLint on pattern: ${PATTERN}"
ESLINT_ARGS=(-c "${CONFIG_ESLINT}" "${PATTERN}" --format json --output-file "${REPORT_JSON}")
# Also produce human-friendly output
ESLINT_STYLISH_ARGS=(-c "${CONFIG_ESLINT}" "${PATTERN}")

[[ "${RUN_FIX}" == "true" ]] && ESLINT_ARGS+=(--fix) && ESLINT_STYLISH_ARGS+=(--fix)
[[ "${USE_CACHE}" == "true" ]] && ESLINT_ARGS+=(--cache --cache-location .cache/eslint/) && ESLINT_STYLISH_ARGS+=(--cache --cache-location .cache/eslint/)
[[ "${QUIET}" == "true" ]] && ESLINT_ARGS+=(--quiet) && ESLINT_STYLISH_ARGS+=(--quiet)

# Stylish (for readability) - ignore its exit code until after JSON parsing to enforce thresholds
if ! npx eslint "${ESLINT_STYLISH_ARGS[@]}" > "${REPORT_TEXT}" 2>&1; then
  # We'll re-evaluate after parsing JSON
  true
fi

# JSON run (guarantee fresh results after potential fixes)
if ! npx eslint "${ESLINT_ARGS[@]}" >/dev/null 2>&1; then
  # Continue; errors counted below
  true
fi

if [[ ! -s "${REPORT_JSON}" ]]; then
  echo "ERROR: ESLint JSON report missing or empty" >&2
  cat "${REPORT_TEXT}" || true
  rm -rf "${TMP_DIR}"
  exit 4
fi

# Parse counts with node
ESLINT_COUNTS="$(node <<'NODE' "${REPORT_JSON}" "${MAX_WARNINGS}"
const fs = require('fs');
const path = process.argv[2];
const maxWarn = parseInt(process.argv[3],10);
let data;
try {
  data = JSON.parse(fs.readFileSync(path,'utf8'));
} catch (e) {
  console.error('Failed to parse ESLint JSON:', e.message);
  process.exit(99);
}
if (!Array.isArray(data)) {
  console.error('Unexpected ESLint JSON structure');
  process.exit(99);
}
let errors = 0, warnings = 0;
for (const f of data) {
  errors += f.errorCount || 0;
  warnings += f.warningCount || 0;
}
process.stdout.write(JSON.stringify({errors, warnings, maxWarn}));
NODE
)"

ERROR_COUNT="$(echo "${ESLINT_COUNTS}" | node -pe 'JSON.parse(fs.readFileSync(0,"utf8")).errors')"
WARNING_COUNT="$(echo "${ESLINT_COUNTS}" | node -pe 'JSON.parse(fs.readFileSync(0,"utf8")).warnings')"
# shellcheck disable=SC2091
MAX_WARN_PARSED="$(echo "${ESLINT_COUNTS}" | node -pe 'JSON.parse(fs.readFileSync(0,"utf8")).maxWarn')"

echo "==> ESLint Summary: errors=${ERROR_COUNT} warnings=${WARNING_COUNT} (threshold=${MAX_WARN_PARSED})"

# Show stylish output if not quiet
if [[ "${QUIET}" != "true" ]]; then
  echo "---- ESLint Detailed Output (stylish) ----"
  cat "${REPORT_TEXT}"
  echo "------------------------------------------"
fi

EXIT_CODE=0
if (( ERROR_COUNT > 0 )); then
  echo "FAIL: ESLint errors present (${ERROR_COUNT})."
  EXIT_CODE=1
elif (( MAX_WARN_PARSED >= 0 )) && (( WARNING_COUNT > MAX_WARN_PARSED )); then
  echo "FAIL: Warning count ${WARNING_COUNT} exceeds threshold ${MAX_WARN_PARSED}."
  EXIT_CODE=2
else
  echo "SUCCESS: Lint passed (errors=0, warnings=${WARNING_COUNT})."
fi

rm -rf "${TMP_DIR}"
exit "${EXIT_CODE}"
