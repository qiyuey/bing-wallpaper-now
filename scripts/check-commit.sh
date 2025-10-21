#!/usr/bin/env bash
# pre-commit-check.sh - 提交前本地 CI 检查
#
# 此脚本在提交前运行所有 CI 检查，确保代码能够通过 GitHub Actions CI
# 避免提交后 CI 失败的循环
#
# 使用方法:
#   ./scripts/pre-commit-check.sh
#   或者 make pre-commit

set -euo pipefail

# 颜色输出
COLOR_RESET='\033[0m'
COLOR_BOLD='\033[1m'
COLOR_GREEN='\033[32m'
COLOR_YELLOW='\033[33m'
COLOR_BLUE='\033[34m'
COLOR_RED='\033[31m'

# 检查计数器
CHECKS_PASSED=0
CHECKS_FAILED=0

# 打印分隔符
print_separator() {
    printf "${COLOR_BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${COLOR_RESET}\n"
}

# 打印检查标题
print_check() {
    printf "\n${COLOR_BOLD}${COLOR_BLUE}🔍 $1${COLOR_RESET}\n"
    print_separator
}

# 打印成功消息
print_success() {
    printf "${COLOR_GREEN}✅ $1${COLOR_RESET}\n"
    CHECKS_PASSED=$((CHECKS_PASSED + 1))
}

# 打印失败消息
print_error() {
    printf "${COLOR_RED}❌ $1${COLOR_RESET}\n"
    CHECKS_FAILED=$((CHECKS_FAILED + 1))
}

# 打印警告消息
print_warning() {
    printf "${COLOR_YELLOW}⚠️  $1${COLOR_RESET}\n"
}

# 检测包管理器
detect_package_manager() {
    if command -v pnpm &> /dev/null; then
        echo "pnpm"
    elif command -v npm &> /dev/null; then
        echo "npm"
    else
        print_error "未找到 npm 或 pnpm"
        exit 1
    fi
}

PKG_MANAGER=$(detect_package_manager)

printf "${COLOR_BOLD}${COLOR_BLUE}╔══════════════════════════════════════════════════════════════╗${COLOR_RESET}\n"
printf "${COLOR_BOLD}${COLOR_BLUE}║         Bing Wallpaper Now - 提交前检查 (Pre-Commit)        ║${COLOR_RESET}\n"
printf "${COLOR_BOLD}${COLOR_BLUE}╚══════════════════════════════════════════════════════════════╝${COLOR_RESET}\n\n"

printf "${COLOR_YELLOW}此脚本将运行所有 CI 检查，确保代码能够通过 GitHub Actions${COLOR_RESET}\n"
printf "${COLOR_YELLOW}包管理器: ${PKG_MANAGER}${COLOR_RESET}\n\n"

# ============================================================================
# 1. Rust 代码格式检查
# ============================================================================
print_check "1/8 Rust 代码格式检查 (cargo fmt)"

if cargo fmt --manifest-path src-tauri/Cargo.toml -- --check; then
    print_success "Rust 代码格式正确"
else
    print_error "Rust 代码格式不正确，请运行: cargo fmt --manifest-path src-tauri/Cargo.toml"
fi

# ============================================================================
# 2. Rust Clippy 检查
# ============================================================================
print_check "2/8 Rust Clippy 检查 (cargo clippy)"

if cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings; then
    print_success "Clippy 检查通过"
else
    print_error "Clippy 检查失败，请修复 Rust 代码问题"
fi

# ============================================================================
# 3. Rust 测试
# ============================================================================
print_check "3/8 Rust 单元测试 (cargo test)"

if cargo test --manifest-path src-tauri/Cargo.toml --quiet; then
    print_success "Rust 测试通过"
else
    print_error "Rust 测试失败，请修复测试问题"
fi

# ============================================================================
# 4. TypeScript 类型检查
# ============================================================================
print_check "4/8 TypeScript 类型检查 (tsc)"

if $PKG_MANAGER run typecheck; then
    print_success "TypeScript 类型检查通过"
else
    print_error "TypeScript 类型检查失败，请修复类型错误"
fi

# ============================================================================
# 5. ESLint 检查
# ============================================================================
print_check "5/8 ESLint 检查 (eslint)"

if $PKG_MANAGER run lint; then
    print_success "ESLint 检查通过"
else
    print_error "ESLint 检查失败，请运行: $PKG_MANAGER run lint:fix"
fi

# ============================================================================
# 6. Prettier 格式检查
# ============================================================================
print_check "6/8 Prettier 格式检查 (prettier)"

if $PKG_MANAGER run format:check; then
    print_success "Prettier 格式检查通过"
else
    print_error "Prettier 格式检查失败，请运行: $PKG_MANAGER run format"
fi

# ============================================================================
# 7. 前端测试
# ============================================================================
print_check "7/8 前端单元测试 (vitest)"

if $PKG_MANAGER run test:frontend; then
    print_success "前端测试通过"
else
    print_error "前端测试失败，请修复测试问题"
fi

# ============================================================================
# 8. 前端构建
# ============================================================================
print_check "8/8 前端构建检查 (vite build)"

if $PKG_MANAGER run build; then
    print_success "前端构建成功"
else
    print_error "前端构建失败，请修复构建问题"
fi

# ============================================================================
# 汇总结果
# ============================================================================
printf "\n"
print_separator
printf "${COLOR_BOLD}检查汇总:${COLOR_RESET}\n"
print_separator

TOTAL_CHECKS=$((CHECKS_PASSED + CHECKS_FAILED))
printf "总检查项: ${COLOR_BOLD}%d${COLOR_RESET}\n" $TOTAL_CHECKS
printf "通过: ${COLOR_GREEN}${COLOR_BOLD}%d${COLOR_RESET}\n" $CHECKS_PASSED
printf "失败: ${COLOR_RED}${COLOR_BOLD}%d${COLOR_RESET}\n" $CHECKS_FAILED

if [ $CHECKS_FAILED -eq 0 ]; then
    printf "\n${COLOR_GREEN}${COLOR_BOLD}✅ 所有检查通过！可以安全提交代码 🎉${COLOR_RESET}\n"
    printf "${COLOR_GREEN}建议使用以下命令提交:${COLOR_RESET}\n"
    printf "  ${COLOR_BLUE}git add .${COLOR_RESET}\n"
    printf "  ${COLOR_BLUE}git commit${COLOR_RESET}\n"
    printf "  ${COLOR_BLUE}git push${COLOR_RESET}\n\n"
    exit 0
else
    printf "\n${COLOR_RED}${COLOR_BOLD}❌ 有 %d 项检查失败，请修复后再提交${COLOR_RESET}\n" $CHECKS_FAILED
    printf "${COLOR_YELLOW}提示:${COLOR_RESET}\n"
    printf "  - 格式问题: ${COLOR_BLUE}cargo fmt && $PKG_MANAGER run format${COLOR_RESET}\n"
    printf "  - Lint 问题: ${COLOR_BLUE}$PKG_MANAGER run lint:fix${COLOR_RESET}\n"
    printf "  - 类型问题: 根据 tsc 错误提示修复\n"
    printf "  - 测试问题: 根据测试输出修复\n\n"
    exit 1
fi
