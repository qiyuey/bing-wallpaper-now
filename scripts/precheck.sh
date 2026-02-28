#!/usr/bin/env bash
# precheck.sh - Quality Gate Pre-check
#
# Shared pre-check script for skills (review, release, etc.)
# Runs `make check` and outputs labeled results.
#
# Usage:
#   bash scripts/precheck.sh [label]
#
# Examples:
#   bash scripts/precheck.sh review
#   bash scripts/precheck.sh release

set -euo pipefail

LABEL="${1:-precheck}"
CHECK_CMD="make check"

echo "[${LABEL}] 开始执行检查：${CHECK_CMD}"
echo "[${LABEL}] 这会运行格式、lint、类型检查与测试。"

if ${CHECK_CMD}; then
  echo "[${LABEL}] 检查通过，可以继续。"
  exit 0
else
  status=$?
  echo "[${LABEL}] 检查失败，退出码: ${status}"
  echo "[${LABEL}] 请先修复失败项。"
  echo "[${LABEL}] 如需快速定位，可单独运行："
  echo "  - cargo fmt --check"
  echo "  - cargo clippy -- -D warnings"
  echo "  - cargo test"
  echo "  - pnpm run typecheck"
  echo "  - pnpm run lint"
  echo "  - pnpm run format:check"
  echo "  - pnpm run test:frontend"
  echo "  - pnpm run lint:md"
  exit "${status}"
fi
