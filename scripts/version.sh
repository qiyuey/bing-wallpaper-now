#!/usr/bin/env bash
# SNAPSHOT 版本管理脚本
#
# 版本格式：X.Y.Z 或 X.Y.Z-SNAPSHOT
#
# 用法：
#   ./scripts/version-snapshot.sh snapshot-patch  # 创建下一个 patch SNAPSHOT (0.1.0 -> 0.1.1-SNAPSHOT)
#   ./scripts/version-snapshot.sh snapshot-minor  # 创建下一个 minor SNAPSHOT (0.1.0 -> 0.2.0-SNAPSHOT)
#   ./scripts/version-snapshot.sh snapshot-major  # 创建下一个 major SNAPSHOT (0.1.0 -> 1.0.0-SNAPSHOT)
#   ./scripts/version-snapshot.sh release         # 发布当前 SNAPSHOT 版本、打 tag 并推送到远程 (0.1.1-SNAPSHOT -> 0.1.1)
#
# 工作流程：
#   1. 发布 v0.1.0 后，创建 0.1.1-SNAPSHOT 用于开发
#   2. 开发完成后，运行 release 转为 0.1.1 正式版本、打 tag 并推送到远程
#   3. 发布后，再次创建 0.1.2-SNAPSHOT 继续开发

set -euo pipefail

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# 文件路径
PACKAGE_JSON="package.json"
CARGO_TOML="src-tauri/Cargo.toml"
TAURI_CONF="src-tauri/tauri.conf.json"

# 辅助函数
print_info() { echo -e "${BLUE}ℹ${NC} $1"; }
print_success() { echo -e "${GREEN}✓${NC} $1"; }
print_warning() { echo -e "${YELLOW}⚠${NC} $1"; }
print_error() { echo -e "${RED}✗${NC} $1"; }
print_header() { echo -e "${CYAN}${1}${NC}"; }

# 检查是否在 git 仓库中
check_git_repo() {
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        print_error "不在 Git 仓库中"
        exit 1
    fi
}

# 检查工作目录状态
check_working_directory() {
    if [[ -n $(git status -s) ]]; then
        print_warning "工作目录有未提交的更改"
        git status -s
        echo ""
        read -p "是否继续？(y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_info "已取消"
            exit 0
        fi
    fi
}

# 获取当前版本
get_current_version() {
    grep '"version"' "$PACKAGE_JSON" | head -1 | sed 's/.*"version": "\(.*\)".*/\1/'
}

# 检查是否是 SNAPSHOT 版本
is_snapshot() {
    [[ $1 == *"-SNAPSHOT" ]]
}

# 移除 SNAPSHOT 后缀
remove_snapshot() {
    echo "$1" | sed 's/-SNAPSHOT$//'
}

# 添加 SNAPSHOT 后缀
add_snapshot() {
    echo "$1-SNAPSHOT"
}

# 版本号拆分
split_version() {
    local version=$(remove_snapshot "$1")
    MAJOR=$(echo "$version" | cut -d. -f1)
    MINOR=$(echo "$version" | cut -d. -f2)
    PATCH=$(echo "$version" | cut -d. -f3)
}

# 计算下一个版本
calculate_next_version() {
    local current=$1
    local bump_type=$2

    local base_version=$(remove_snapshot "$current")
    split_version "$base_version"

    case "$bump_type" in
        patch)
            PATCH=$((PATCH + 1))
            ;;
        minor)
            MINOR=$((MINOR + 1))
            PATCH=0
            ;;
        major)
            MAJOR=$((MAJOR + 1))
            MINOR=0
            PATCH=0
            ;;
        *)
            print_error "无效的版本类型: $bump_type"
            exit 1
            ;;
    esac

    echo "${MAJOR}.${MINOR}.${PATCH}"
}

# 更新所有版本文件
update_version_files() {
    local new_version=$1

    print_info "更新 $PACKAGE_JSON..."
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "s/\"version\": \".*\"/\"version\": \"$new_version\"/" "$PACKAGE_JSON"
    else
        sed -i "s/\"version\": \".*\"/\"version\": \"$new_version\"/" "$PACKAGE_JSON"
    fi

    print_info "更新 $CARGO_TOML..."
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "s/^version = \".*\"/version = \"$new_version\"/" "$CARGO_TOML"
    else
        sed -i "s/^version = \".*\"/version = \"$new_version\"/" "$CARGO_TOML"
    fi

    print_info "更新 $TAURI_CONF..."
    if command -v jq &> /dev/null; then
        jq ".version = \"$new_version\"" "$TAURI_CONF" > "${TAURI_CONF}.tmp"
        mv "${TAURI_CONF}.tmp" "$TAURI_CONF"
    else
        if [[ "$OSTYPE" == "darwin"* ]]; then
            sed -i '' "s/\"version\": \".*\"/\"version\": \"$new_version\"/" "$TAURI_CONF"
        else
            sed -i "s/\"version\": \".*\"/\"version\": \"$new_version\"/" "$TAURI_CONF"
        fi
    fi

    print_info "更新 Cargo.lock..."
    cargo update -p bing-wallpaper-now --manifest-path src-tauri/Cargo.toml --quiet 2>/dev/null || true

    print_success "版本文件已更新为 $new_version"
}

# 创建 SNAPSHOT 版本
create_snapshot() {
    local bump_type=$1
    local current=$(get_current_version)

    if is_snapshot "$current"; then
        print_warning "当前已经是 SNAPSHOT 版本: $current"
        local base=$(remove_snapshot "$current")
        print_info "将基于 $base 创建新的 SNAPSHOT"
    fi

    local base_version=$(remove_snapshot "$current")
    local next_version=$(calculate_next_version "$base_version" "$bump_type")
    local snapshot_version=$(add_snapshot "$next_version")

    print_header "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    print_header "  创建 SNAPSHOT 版本"
    print_header "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    print_info "当前版本: $current"
    print_info "新版本:   $snapshot_version"
    echo ""

    read -p "确认创建 SNAPSHOT 版本？(y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "已取消"
        exit 0
    fi

    update_version_files "$snapshot_version"

    git add "$PACKAGE_JSON" "$CARGO_TOML" "$TAURI_CONF" "src-tauri/Cargo.lock"
    git commit -m "chore(version): bump to $snapshot_version"

    print_success "已创建 SNAPSHOT 版本: $snapshot_version"
    print_info "可以开始新功能的开发了！"
}

# 发布版本（默认推送到远程）
release_version() {
    local current=$(get_current_version)

    if ! is_snapshot "$current"; then
        print_error "当前不是 SNAPSHOT 版本: $current"
        print_info "只能从 SNAPSHOT 版本发布"
        exit 1
    fi

    local release_version=$(remove_snapshot "$current")
    local tag="v$release_version"

    print_header "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    print_header "  发布正式版本"
    print_header "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    print_info "SNAPSHOT 版本: $current"
    print_info "发布版本:      $release_version"
    print_info "Git Tag:       $tag"
    echo ""

    read -p "确认发布版本？(y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "已取消"
        exit 0
    fi

    # 更新版本号（移除 SNAPSHOT）
    update_version_files "$release_version"

    # 提交并打 tag
    git add "$PACKAGE_JSON" "$CARGO_TOML" "$TAURI_CONF" "src-tauri/Cargo.lock"
    git commit -m "chore(release): $release_version"
    git tag -a "$tag" -m "Release $release_version"

    print_success "已创建发布版本: $release_version"
    print_success "已创建 Git 标签: $tag"

    echo ""
    read -p "是否立即推送到远程？(Y/n) " -n 1 -r
    echo
    local pushed=false
    if [[ ! $REPLY =~ ^[Nn]$ ]]; then
        print_info "推送到远程..."
        git push origin main
        git push origin "$tag"
        print_success "已推送到远程，CI 将开始构建"
        pushed=true
        echo ""
        print_info "GitHub Actions 将自动构建并发布到 Releases"
    else
        print_info "跳过推送，下次手动推送："
        echo "  git push origin main && git push origin $tag"
    fi

    # 询问是否创建下一个 SNAPSHOT 版本
    echo ""
    read -p "是否创建下一个 patch SNAPSHOT 版本？(y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo ""
        print_info "创建下一个 SNAPSHOT 版本..."

        local next_version=$(calculate_next_version "$release_version" "patch")
        local snapshot_version=$(add_snapshot "$next_version")

        update_version_files "$snapshot_version"

        git add "$PACKAGE_JSON" "$CARGO_TOML" "$TAURI_CONF" "src-tauri/Cargo.lock"
        git commit -m "chore(version): bump to $snapshot_version"

        print_success "已创建 SNAPSHOT 版本: $snapshot_version"

        if [ "$pushed" = true ]; then
            echo ""
            read -p "是否推送 SNAPSHOT 版本到远程？(y/N) " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                git push origin main
                print_success "已推送 SNAPSHOT 版本到远程"
            else
                print_info "稍后手动推送: git push origin main"
            fi
        fi

        echo ""
        print_success "可以开始新功能的开发了！"
    else
        echo ""
        print_info "稍后可以手动创建 SNAPSHOT 版本: make snapshot-patch"
    fi
}

# 显示当前版本信息
show_version_info() {
    local current=$(get_current_version)

    print_header "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    print_header "  版本信息"
    print_header "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    print_info "当前版本: $current"

    if is_snapshot "$current"; then
        local release=$(remove_snapshot "$current")
        print_info "类型:     SNAPSHOT (开发版本)"
        print_info "发布版本: $release (发布时)"
    else
        print_info "类型:     Release (正式版本)"
        print_warning "建议创建下一个 SNAPSHOT 版本以继续开发"
    fi

    echo ""
    print_info "最近的 Git 标签:"
    git tag --sort=-v:refname | head -3
    echo ""
}

# 主函数
main() {
    check_git_repo

    if [ $# -eq 0 ]; then
        show_version_info
        echo ""
        print_info "用法:"
        echo "  $0 snapshot-patch      # 创建下一个 patch SNAPSHOT"
        echo "  $0 snapshot-minor      # 创建下一个 minor SNAPSHOT"
        echo "  $0 snapshot-major      # 创建下一个 major SNAPSHOT"
        echo "  $0 release             # 发布当前 SNAPSHOT 版本、打 tag 并推送到远程"
        echo ""
        exit 0
    fi

    check_working_directory

    case "$1" in
        snapshot-patch)
            create_snapshot "patch"
            ;;
        snapshot-minor)
            create_snapshot "minor"
            ;;
        snapshot-major)
            create_snapshot "major"
            ;;
        release)
            release_version
            ;;
        info)
            show_version_info
            ;;
        *)
            print_error "未知命令: $1"
            print_info "用法: $0 <snapshot-patch|snapshot-minor|snapshot-major|release|info>"
            exit 1
            ;;
    esac
}

main "$@"
