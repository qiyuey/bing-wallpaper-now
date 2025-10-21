#!/usr/bin/env bash
# 快速检查 Linux 编译（GitHub Actions 使用的平台）

set -e

MANIFEST="src-tauri/Cargo.toml"
TARGET="x86_64-unknown-linux-gnu"

echo "🐧 检查 Linux 编译..."

# 确保 target 已安装
if ! rustup target list --installed | grep -q "^${TARGET}$"; then
    echo "📦 安装 ${TARGET}..."
    rustup target add "${TARGET}"
fi

echo ""
echo "🔍 运行编译检查..."
if cargo check --manifest-path "${MANIFEST}" --target "${TARGET}"; then
    echo ""
    echo "✅ Linux 编译检查通过！"
    exit 0
else
    echo ""
    echo "❌ Linux 编译检查失败"
    exit 1
fi
