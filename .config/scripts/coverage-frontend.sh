#!/usr/bin/env bash
#
# coverage-frontend.sh
#
# Purpose:
#   Run frontend (React/TypeScript) test coverage using Vitest + V8 coverage provider
#   with optional threshold enforcement.
#
# Requirements:
#   - Node.js 18+
#   - devDependencies: vitest @vitest/coverage-v8 (installed via npm/yarn/pnpm)
#
# Optional Environment Variables:
#   COVERAGE_MIN_LINES        (e.g. 70)
#   COVERAGE_MIN_FUNCTIONS    (e.g. 70)
#   COVERAGE_MIN_BRANCHES     (e.g. 60)
#   COVERAGE_MIN_STATEMENTS   (e.g. 70)
#   COVERAGE_DIR              (default: coverage-frontend)
#   COVERAGE_FAIL_MODE        ("soft" or "hard"; default: soft)
#
# Exit Codes:
#   0  Success (or thresholds soft-failed)
#   1  Hard failure (thresholds not met, hard mode)
#   2  Setup / runtime error (missing tools, etc.)
#
# Usage:
#   bash .config/scripts/coverage-frontend.sh
#
# Notes:
#   - If vitest config file does not specify coverage, flags are injected.
#   - In soft mode, threshold violations are reported but do not break CI.
#   - In hard mode, any threshold violation forces non-zero exit.
#
set -euo pipefail

SCRIPT_NAME="$(basename "$0")"

info()  { printf "\033[1;34m[INFO]\033[0m %s\n" "$*"; }
warn()  { printf "\033[1;33m[WARN]\033[0m %s\n" "$*"; }
error() { printf "\033[1;31m[ERROR]\033[0m %s\n" "$*" >&2; }

# -------------------------
# Resolve project root
# -------------------------
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$ROOT_DIR"

# -------------------------
# Validate prerequisites
# -------------------------
if ! command -v node >/dev/null 2>&1; then
  error "Node.js not found in PATH"
  exit 2
fi

if [ ! -f package.json ]; then
  error "package.json not found at project root: $ROOT_DIR"
  exit 2
fi

# Use local vitest if available; fallback to npx
if [ -f node_modules/.bin/vitest ]; then
  VITEST_BIN="node_modules/.bin/vitest"
else
  VITEST_BIN="npx vitest"
fi

# Ensure vitest & coverage plugin (best-effort install if missing)
if ! node -e "require('fs').accessSync('node_modules/@vitest/coverage-v8/package.json')"; then
  warn "@vitest/coverage-v8 not found, attempting install (dev dependency)"
  if command -v npm >/dev/null 2>&1; then
    npm i -D vitest @vitest/coverage-v8 >/dev/null 2>&1 || {
      error "Failed to auto-install vitest + @vitest/coverage-v8"
      exit 2
    }
  else
    error "npm not found to install vitest coverage plugin"
    exit 2
  fi
fi

# -------------------------
# Configuration / Thresholds
# -------------------------
COVERAGE_DIR="${COVERAGE_DIR:-coverage-frontend}"
FAIL_MODE="${COVERAGE_FAIL_MODE:-soft}"

MIN_LINES="${COVERAGE_MIN_LINES:-}"
MIN_FUNCS="${COVERAGE_MIN_FUNCTIONS:-}"
MIN_BRANCHES="${COVERAGE_MIN_BRANCHES:-}"
MIN_STMTS="${COVERAGE_MIN_STATEMENTS:-}"

THRESHOLDS_SET="false"
[ -n "$MIN_LINES" ]     && THRESHOLDS_SET="true"
[ -n "$MIN_FUNCS" ]     && THRESHOLDS_SET="true"
[ -n "$MIN_BRANCHES" ]  && THRESHOLDS_SET="true"
[ -n "$MIN_STMTS" ]     && THRESHOLDS_SET="true"

info "Frontend coverage output dir: $COVERAGE_DIR"
info "Fail mode: $FAIL_MODE"
[ "$THRESHOLDS_SET" = "true" ] && info "Thresholds (lines=$MIN_LINES funcs=$MIN_FUNCS branches=$MIN_BRANCHES stmts=$MIN_STMTS)" || info "No thresholds set"

# -------------------------
# Build Vitest command
# -------------------------
# We pass coverage args inline to avoid needing a vitest.config.ts for basic usage.
CMD=(
  $VITEST_BIN run
  --coverage.enabled
  --coverage.provider=v8
  "--coverage.reportsDirectory=$COVERAGE_DIR"
  --coverage.reporter=text
  --coverage.reporter=lcov
  --coverage.reporter=json
)

# Optionally attach thresholds (Vitest's built-in enforcement if all provided)
# We only attach those flags we have values for.
[ -n "$MIN_LINES" ]    && CMD+=("--coverage.lines=$MIN_LINES")
[ -n "$MIN_FUNCS" ]    && CMD+=("--coverage.functions=$MIN_FUNCS")
[ -n "$MIN_BRANCHES" ] && CMD+=("--coverage.branches=$MIN_BRANCHES")
[ -n "$MIN_STMTS" ]    && CMD+=("--coverage.statements=$MIN_STMTS")

info "Executing: ${CMD[*]}"
# Allow vitest to complete even if thresholds fail (we handle below)
set +e
"${CMD[@]}"
VITEST_EXIT=$?
set -e

if [ $VITEST_EXIT -ne 0 ]; then
  warn "Vitest exited with code $VITEST_EXIT (may be test failure or threshold failure)"
  # Continue to threshold parsing for more context.
fi

# -------------------------
# Parse coverage results (coverage-final.json from c8)
# -------------------------
COVERAGE_JSON_FILE="$COVERAGE_DIR/coverage-final.json"
if [ ! -f "$COVERAGE_JSON_FILE" ]; then
  error "Coverage JSON file not found at $COVERAGE_JSON_FILE"
  # If tests ran but coverage file missing, treat as failure in hard mode
  [ "$FAIL_MODE" = "hard" ] && exit 1 || exit 0
fi

# Node script to aggregate global metrics across all files
read -r -d '' NODE_SCRIPT <<'EOF' || true
const fs = require('fs');
const path = process.argv[2];
const data = JSON.parse(fs.readFileSync(path, 'utf8'));

let total = {
  lines: { covered: 0, total: 0 },
  functions: { covered: 0, total: 0 },
  branches: { covered: 0, total: 0 },
  statements: { covered: 0, total: 0 },
};

for (const file of Object.values(data)) {
  for (const k of ['lines', 'functions', 'branches', 'statements']) {
    const m = file[k];
    if (!m) continue;
    total[k].covered += m.covered;
    total[k].total += m.total;
  }
}

function pct(obj) {
  return obj.total === 0 ? 100 : (obj.covered / obj.total) * 100;
}

const summary = {
  lines: pct(total.lines),
  functions: pct(total.functions),
  branches: pct(total.branches),
  statements: pct(total.statements),
};

process.stdout.write(JSON.stringify(summary));
EOF

SUMMARY_JSON="$(node -e "$NODE_SCRIPT" "$COVERAGE_JSON_FILE")"
LINES_ACTUAL="$(node -e "console.log(JSON.parse(process.argv[1]).lines.toFixed(2))" "$SUMMARY_JSON")"
FUNCS_ACTUAL="$(node -e "console.log(JSON.parse(process.argv[1]).functions.toFixed(2))" "$SUMMARY_JSON")"
BRANCHES_ACTUAL="$(node -e "console.log(JSON.parse(process.argv[1]).branches.toFixed(2))" "$SUMMARY_JSON")"
STMTS_ACTUAL="$(node -e "console.log(JSON.parse(process.argv[1]).statements.toFixed(2))" "$SUMMARY_JSON")"

printf "\nCoverage Summary (aggregated):\n"
printf "  Lines:      %s%%\n" "$LINES_ACTUAL"
printf "  Functions:  %s%%\n" "$FUNCS_ACTUAL"
printf "  Branches:   %s%%\n" "$BRANCHES_ACTUAL"
printf "  Statements: %s%%\n" "$STMTS_ACTUAL"

# -------------------------
# Threshold enforcement (manual soft/hard mode)
# -------------------------
violations=0

check_threshold () {
  local name="$1" actual="$2" required="$3"
  if [ -z "$required" ]; then
    return 0
  fi
  awk_script="BEGIN { exit ($actual + 0 >= $required + 0) ? 0 : 1 }"
  if ! awk "$awk_script"; then
    warn "Threshold violation: $name actual=$actual < required=$required"
    violations=$((violations + 1))
  fi
}

check_threshold "Lines"      "$LINES_ACTUAL"    "$MIN_LINES"
check_threshold "Functions"  "$FUNCS_ACTUAL"    "$MIN_FUNCS"
check_threshold "Branches"   "$BRANCHES_ACTUAL" "$MIN_BRANCHES"
check_threshold "Statements" "$STMTS_ACTUAL"    "$MIN_STMTS"

if [ $violations -gt 0 ]; then
  if [ "$FAIL_MODE" = "hard" ]; then
    error "Coverage thresholds not met ($violations violation(s)); hard fail."
    exit 1
  else
    warn "Coverage thresholds not met ($violations violation(s)); soft mode -> not failing build."
    exit 0
  fi
else
  info "All coverage thresholds satisfied."
fi

# If vitest had failed earlier (non-zero), reflect that now unless soft thresholds:
if [ $VITEST_EXIT -ne 0 ]; then
  error "Vitest reported non-zero exit code ($VITEST_EXIT). Failing."
  exit $VITEST_EXIT
fi

exit 0
