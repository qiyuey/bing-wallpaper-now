#!/usr/bin/env bash
# Tauri 版本管理脚本（类似 npm version）
#
# 用法：
#   ./scripts/version.sh patch    # 0.1.0 -> 0.1.1
#   ./scripts/version.sh minor    # 0.1.0 -> 0.2.0
#   ./scripts/version.sh major    # 0.1.0 -> 1.0.0
#   ./scripts/version.sh 1.2.3    # 设置为指定版本
#
# 功能：
#   1. 更新 package.json
#   2. 更新 src-tauri/Cargo.toml
#   3. 更新 src-tauri/tauri.conf.json
#   4. 生成 Cargo.lock
#   5. Git commit
#   6. 创建 Git tag
#   7. 可选：自动推送

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 文件路径
PACKAGE_JSON="package.json"
CARGO_TOML="src-tauri/Cargo.toml"
TAURI_CONF="src-tauri/tauri.conf.json"

# 辅助函数
print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

# 检查是否在 git 仓库中
if ! git rev-parse --git-dir > /dev/null 2>&1; then
    print_error "不在 Git 仓库中"
    exit 1
fi

# 检查工作目录是否干净
if [[ -n $(git status -s) ]]; then
    print_warning "工作目录有未提交的更改"
    read -p "是否继续？(y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "已取消"
        exit 0
    fi
fi

# 获取当前版本
get_current_version() {
    grep '"version"' "$PACKAGE_JSON" | head -1 | sed 's/.*"version": "\(.*\)".*/\1/'
}

# 版本号拆分
split_version() {
    local version=$1
    MAJOR=$(echo "$version" | cut -d. -f1)
    MINOR=$(echo "$version" | cut -d. -f2)
    PATCH=$(echo "$version" | cut -d. -f3)
}

# 计算新版本
calculate_new_version() {
    local current=$1
    local bump_type=$2

    split_version "$current"

    case "$bump_type" in
        major)
            MAJOR=$((MAJOR + 1))
            MINOR=0
            PATCH=0
            ;;
        minor)
            MINOR=$((MINOR + 1))
            PATCH=0
            ;;
        patch)
            PATCH=$((PATCH + 1))
            ;;
        *)
            # 假设是完整版本号
            if [[ $bump_type =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
                echo "$bump_type"
                return
            else
                print_error "无效的版本类型: $bump_type"
                print_info "用法: $0 <major|minor|patch|x.y.z>"
                exit 1
            fi
            ;;
    esac

    echo "${MAJOR}.${MINOR}.${PATCH}"
}

# 更新 package.json
update_package_json() {
    local new_version=$1
    print_info "更新 $PACKAGE_JSON..."

    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        sed -i '' "s/\"version\": \".*\"/\"version\": \"$new_version\"/" "$PACKAGE_JSON"
    else
        # Linux
        sed -i "s/\"version\": \".*\"/\"version\": \"$new_version\"/" "$PACKAGE_JSON"
    fi

    print_success "$PACKAGE_JSON 已更新为 $new_version"
}

# 更新 Cargo.toml
update_cargo_toml() {
    local new_version=$1
    print_info "更新 $CARGO_TOML..."

    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        sed -i '' "s/^version = \".*\"/version = \"$new_version\"/" "$CARGO_TOML"
    else
        # Linux
        sed -i "s/^version = \".*\"/version = \"$new_version\"/" "$CARGO_TOML"
    fi

    print_success "$CARGO_TOML 已更新为 $new_version"
}

# 更新 tauri.conf.json
update_tauri_conf() {
    local new_version=$1
    print_info "更新 $TAURI_CONF..."

    # 使用 jq 或 sed（jq 更可靠）
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

    print_success "$TAURI_CONF 已更新为 $new_version"
}

# 更新 Cargo.lock
update_cargo_lock() {
    print_info "更新 Cargo.lock..."
    cargo update -p bing-wallpaper-now --manifest-path src-tauri/Cargo.toml > /dev/null 2>&1
    print_success "Cargo.lock 已更新"
}

# Git 提交和打标签
git_commit_and_tag() {
    local version=$1
    local tag="v$version"

    print_info "添加文件到 Git..."
    git add "$PACKAGE_JSON" "$CARGO_TOML" "$TAURI_CONF" "src-tauri/Cargo.lock"

    print_info "创建提交..."
    git commit -m "chore(release): $version"
    print_success "已创建提交"

    print_info "创建标签 $tag..."
    git tag -a "$tag" -m "Release $version"
    print_success "已创建标签 $tag"

    echo ""
    print_success "版本已更新为 $version"
    echo ""
    print_info "下一步："
    echo "  git push origin main"
    echo "  git push origin $tag"
    echo ""

    read -p "是否立即推送到远程？(y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        print_info "推送到远程..."
        git push origin main
        git push origin "$tag"
        print_success "已推送到远程"
    else
        print_warning "请记得手动推送："
        echo "  git push origin main && git push origin $tag"
    fi
}

# 主逻辑
main() {
    if [ $# -eq 0 ]; then
        print_error "缺少版本参数"
        print_info "用法: $0 <major|minor|patch|x.y.z>"
        exit 1
    fi

    local bump_type=$1
    local current_version=$(get_current_version)
    local new_version=$(calculate_new_version "$current_version" "$bump_type")

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  Tauri 版本更新"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    print_info "当前版本: $current_version"
    print_info "新版本:   $new_version"
    echo ""

    read -p "确认更新版本？(y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        print_info "已取消"
        exit 0
    fi

    echo ""

    # 执行更新
    update_package_json "$new_version"
    update_cargo_toml "$new_version"
    update_tauri_conf "$new_version"
    update_cargo_lock

    echo ""

    git_commit_and_tag "$new_version"

    echo ""
    print_success "完成！"
}

main "$@"
