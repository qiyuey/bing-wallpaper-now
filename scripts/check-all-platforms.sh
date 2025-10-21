#!/usr/bin/env bash
# 跨平台编译检查脚本
# 用于在本地快速验证代码在 Linux、Windows、macOS 上的编译情况

set -e

MANIFEST="src-tauri/Cargo.toml"
TARGETS=(
    "x86_64-unknown-linux-gnu"      # Linux (GitHub Actions 使用)
    "x86_64-pc-windows-msvc"        # Windows
    "x86_64-apple-darwin"           # macOS Intel
    "aarch64-apple-darwin"          # macOS Apple Silicon
)

echo "🔍 开始跨平台编译检查..."
echo ""

# 检查是否已安装必要的 target
echo "📦 检查并安装必要的编译目标..."
for target in "${TARGETS[@]}"; do
    if ! rustup target list --installed | grep -q "^${target}$"; then
        echo "   安装 ${target}..."
        rustup target add "${target}"
    else
        echo "   ✅ ${target} (已安装)"
    fi
done

echo ""
echo "🧪 开始编译检查..."
echo ""

failed=0
for target in "${TARGETS[@]}"; do
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "🎯 Target: ${target}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    if cargo check --manifest-path "${MANIFEST}" --target "${target}" 2>&1 | tail -20; then
        echo "✅ ${target} 编译检查通过"
    else
        echo "❌ ${target} 编译检查失败"
        failed=$((failed + 1))
    fi
    echo ""
done

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 结果汇总"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "总计: ${#TARGETS[@]} 个平台"
echo "成功: $((${#TARGETS[@]} - failed)) 个"
echo "失败: ${failed} 个"

if [ ${failed} -eq 0 ]; then
    echo ""
    echo "🎉 所有平台编译检查通过！"
    exit 0
else
    echo ""
    echo "⚠️  有 ${failed} 个平台编译检查失败"
    exit 1
fi
