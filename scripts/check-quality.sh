#!/usr/bin/env bash
# 代码质量检查脚本
#
# 功能：运行 Clippy、格式检查和单元测试，验证代码质量
# 用途：日常开发中的快速反馈，建议每次修改代码后运行
#
# 注意：此脚本不涉及跨平台编译检查，完整的平台兼容性验证在 CI 中进行

set -e

MANIFEST="src-tauri/Cargo.toml"

echo "🔍 运行代码质量检查..."
echo "包括：Clippy 检查、格式验证、单元测试"
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
echo "  - 代码质量检查已全部通过"
echo "  - 跨平台兼容性检查会在 GitHub Actions CI 中自动进行"
echo "  - 现在可以安全地提交和推送代码"
echo ""
