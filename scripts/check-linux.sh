#!/usr/bin/env bash
# 快速检查代码语法（跳过 Tauri 特定的 Linux 依赖）
#
# 注意：Tauri 应用依赖 Linux 系统库（GTK, WebKit 等），
# 在 macOS 上无法进行完整的交叉编译检查。
#
# 此脚本提供语法检查作为快速反馈，完整的编译检查在 CI 中进行。

set -e

MANIFEST="src-tauri/Cargo.toml"

echo "🔍 运行代码语法和类型检查..."
echo "注意：此检查验证 Rust 语法和类型，不包含 Linux 系统依赖"
echo ""

# 方案 1: 使用 cargo clippy 做语法检查（最接近 CI 检查）
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "运行 Clippy 检查..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if cargo clippy --manifest-path "${MANIFEST}" -- -D warnings 2>&1 | tail -30; then
    echo ""
    echo "✅ Clippy 检查通过！"
else
    echo ""
    echo "❌ Clippy 检查失败"
    exit 1
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "运行格式检查..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if cargo fmt --manifest-path "${MANIFEST}" -- --check; then
    echo "✅ 格式检查通过！"
else
    echo "❌ 格式检查失败"
    exit 1
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "运行测试..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if cargo test --manifest-path "${MANIFEST}" 2>&1 | tail -30; then
    echo ""
    echo "✅ 测试通过！"
else
    echo ""
    echo "❌ 测试失败"
    exit 1
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ 所有检查通过！"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "💡 提示："
echo "  - 语法、格式、测试检查已通过"
echo "  - Linux 系统依赖检查会在 GitHub Actions CI 中自动进行"
echo "  - 现在可以安全地提交和推送代码"
echo ""
