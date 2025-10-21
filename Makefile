# Makefile - Bing Wallpaper Now
#
# 快速参考:
#   make dev              # 启动开发模式
#   make build            # 构建生产版本
#   make test             # 运行所有测试
#   make pre-commit       # 提交前检查
#   make snapshot-patch   # 创建 SNAPSHOT 版本
#   make release          # 发布正式版本
#
# 环境要求:
# - Node.js >= 22 LTS
# - Rust 1.80+ (Edition 2024)
# - pnpm (推荐) 或 npm

# ============================================================================
# 配置变量
# ============================================================================

# 包管理器
PKG_MANAGER := pnpm
ifeq ($(shell command -v pnpm 2> /dev/null),)
	PKG_MANAGER := npm
endif

# 路径
RUST_DIR := src-tauri
RUST_MANIFEST := $(RUST_DIR)/Cargo.toml

# 颜色输出
COLOR_RESET := \033[0m
COLOR_BOLD := \033[1m
COLOR_GREEN := \033[32m
COLOR_YELLOW := \033[33m
COLOR_BLUE := \033[34m
COLOR_CYAN := \033[36m

# ============================================================================
# Phony 目标
# ============================================================================

.PHONY: all dev build bundle
.PHONY: test test-rust test-frontend
.PHONY: fmt lint check pre-commit
.PHONY: clean deps install
.PHONY: snapshot-patch snapshot-minor snapshot-major release release-push version-info
.PHONY: help info

# ============================================================================
# 默认目标
# ============================================================================

all: check test build

# ============================================================================
# 开发命令
# ============================================================================

## dev: 启动 Tauri 开发模式 (热重载)
dev:
	@printf "$(COLOR_BOLD)$(COLOR_BLUE)🚀 启动开发模式...$(COLOR_RESET)\n"
	$(PKG_MANAGER) run tauri dev

# ============================================================================
# 构建命令
# ============================================================================

## build: 构建前端生产版本
build:
	@printf "$(COLOR_BOLD)$(COLOR_GREEN)📦 构建生产版本...$(COLOR_RESET)\n"
	$(PKG_MANAGER) run build

## bundle: 构建 Tauri 完整应用包
bundle:
	@printf "$(COLOR_BOLD)$(COLOR_GREEN)📦 构建 Tauri 应用包...$(COLOR_RESET)\n"
	$(PKG_MANAGER) run tauri build

# ============================================================================
# 依赖管理
# ============================================================================

## install: 安装所有依赖
install: deps

## deps: 安装前端依赖
deps:
	@printf "$(COLOR_BOLD)$(COLOR_BLUE)📦 安装依赖...$(COLOR_RESET)\n"
	$(PKG_MANAGER) install

# ============================================================================
# 测试命令
# ============================================================================

## test: 运行所有测试
test: test-rust test-frontend

## test-rust: 运行 Rust 测试
test-rust:
	@printf "$(COLOR_BOLD)$(COLOR_YELLOW)🧪 运行 Rust 测试...$(COLOR_RESET)\n"
	@cargo test --manifest-path $(RUST_MANIFEST) --quiet

## test-frontend: 运行前端测试
test-frontend:
	@printf "$(COLOR_BOLD)$(COLOR_YELLOW)🧪 运行前端测试...$(COLOR_RESET)\n"
	@$(PKG_MANAGER) run test:frontend

# ============================================================================
# 代码质量
# ============================================================================

## fmt: 格式化所有代码
fmt:
	@printf "$(COLOR_BOLD)$(COLOR_BLUE)✨ 格式化代码...$(COLOR_RESET)\n"
	@cargo fmt --manifest-path $(RUST_MANIFEST)
	@$(PKG_MANAGER) run format

## lint: 运行所有 linter
lint:
	@printf "$(COLOR_BOLD)$(COLOR_YELLOW)🔍 运行代码检查...$(COLOR_RESET)\n"
	@cargo clippy --manifest-path $(RUST_MANIFEST) -- -D warnings
	@$(PKG_MANAGER) run lint

## check: 运行所有质量检查
check:
	@printf "$(COLOR_BOLD)$(COLOR_YELLOW)🔍 运行质量检查...$(COLOR_RESET)\n"
	@cargo fmt --manifest-path $(RUST_MANIFEST) -- --check
	@cargo clippy --manifest-path $(RUST_MANIFEST) -- -D warnings
	@$(PKG_MANAGER) run format:check
	@$(PKG_MANAGER) run lint
	@$(PKG_MANAGER) run typecheck

## pre-commit: 提交前完整 CI 检查 (推荐)
pre-commit:
	@printf "$(COLOR_BOLD)$(COLOR_BLUE)🚀 运行提交前检查...$(COLOR_RESET)\n"
	@./scripts/pre-commit-check.sh

# ============================================================================
# 版本管理 (Maven-like)
# ============================================================================

## snapshot-patch: 创建下一个 patch SNAPSHOT 版本 (0.1.0 -> 0.1.1-SNAPSHOT)
snapshot-patch:
	@printf "$(COLOR_BOLD)$(COLOR_CYAN)📦 创建 patch SNAPSHOT 版本...$(COLOR_RESET)\n"
	@./scripts/version-maven.sh snapshot-patch

## snapshot-minor: 创建下一个 minor SNAPSHOT 版本 (0.1.0 -> 0.2.0-SNAPSHOT)
snapshot-minor:
	@printf "$(COLOR_BOLD)$(COLOR_CYAN)📦 创建 minor SNAPSHOT 版本...$(COLOR_RESET)\n"
	@./scripts/version-maven.sh snapshot-minor

## snapshot-major: 创建下一个 major SNAPSHOT 版本 (0.1.0 -> 1.0.0-SNAPSHOT)
snapshot-major:
	@printf "$(COLOR_BOLD)$(COLOR_CYAN)📦 创建 major SNAPSHOT 版本...$(COLOR_RESET)\n"
	@./scripts/version-maven.sh snapshot-major

## release: 发布当前 SNAPSHOT 为正式版本并打 tag
release:
	@printf "$(COLOR_BOLD)$(COLOR_GREEN)🚀 发布正式版本...$(COLOR_RESET)\n"
	@./scripts/version-maven.sh release

## release-push: 发布正式版本并推送到远程
release-push:
	@printf "$(COLOR_BOLD)$(COLOR_GREEN)🚀 发布并推送正式版本...$(COLOR_RESET)\n"
	@./scripts/version-maven.sh release-push

## version-info: 显示当前版本信息
version-info:
	@./scripts/version-maven.sh info

# ============================================================================
# 清理命令
# ============================================================================

## clean: 清理构建产物
clean:
	@printf "$(COLOR_BOLD)$(COLOR_YELLOW)🧹 清理构建产物...$(COLOR_RESET)\n"
	@cargo clean --manifest-path $(RUST_MANIFEST)
	@rm -rf dist node_modules/.vite

# ============================================================================
# 信息命令
# ============================================================================

## info: 显示项目信息
info:
	@printf "$(COLOR_BOLD)$(COLOR_BLUE)ℹ️  Bing Wallpaper Now$(COLOR_RESET)\n\n"
	@printf "包管理器: $(COLOR_GREEN)$(PKG_MANAGER)$(COLOR_RESET)\n"
	@printf "Rust:     $(COLOR_GREEN)"
	@rustc --version 2>/dev/null || echo "未安装"
	@printf "$(COLOR_RESET)Node.js:  $(COLOR_GREEN)"
	@node --version 2>/dev/null || echo "未安装"
	@printf "$(COLOR_RESET)当前版本: $(COLOR_GREEN)"
	@grep '"version"' package.json | head -1 | sed 's/.*"version": "\(.*\)".*/\1/' || echo "未知"
	@printf "$(COLOR_RESET)\n"

# ============================================================================
# 帮助信息
# ============================================================================

## help: 显示帮助信息
help:
	@printf "$(COLOR_BOLD)Bing Wallpaper Now - Makefile 命令$(COLOR_RESET)\n\n"
	@printf "$(COLOR_BOLD)🚀 开发命令:$(COLOR_RESET)\n"
	@printf "  $(COLOR_GREEN)make dev$(COLOR_RESET)              - 启动开发模式 (热重载)\n"
	@printf "  $(COLOR_GREEN)make build$(COLOR_RESET)            - 构建前端生产版本\n"
	@printf "  $(COLOR_GREEN)make bundle$(COLOR_RESET)           - 构建 Tauri 应用包\n\n"
	@printf "$(COLOR_BOLD)🧪 测试命令:$(COLOR_RESET)\n"
	@printf "  $(COLOR_GREEN)make test$(COLOR_RESET)             - 运行所有测试\n"
	@printf "  $(COLOR_GREEN)make test-rust$(COLOR_RESET)        - 仅运行 Rust 测试\n"
	@printf "  $(COLOR_GREEN)make test-frontend$(COLOR_RESET)    - 仅运行前端测试\n\n"
	@printf "$(COLOR_BOLD)✨ 代码质量:$(COLOR_RESET)\n"
	@printf "  $(COLOR_GREEN)make check$(COLOR_RESET)            - 运行所有质量检查\n"
	@printf "  $(COLOR_GREEN)make pre-commit$(COLOR_RESET)       - 提交前完整检查 (推荐)\n"
	@printf "  $(COLOR_GREEN)make fmt$(COLOR_RESET)              - 格式化所有代码\n"
	@printf "  $(COLOR_GREEN)make lint$(COLOR_RESET)             - 运行所有 linter\n\n"
	@printf "$(COLOR_BOLD)📦 版本管理 (Maven-like):$(COLOR_RESET)\n"
	@printf "  $(COLOR_CYAN)make snapshot-patch$(COLOR_RESET)    - 创建 patch SNAPSHOT (0.1.0 -> 0.1.1-SNAPSHOT)\n"
	@printf "  $(COLOR_CYAN)make snapshot-minor$(COLOR_RESET)    - 创建 minor SNAPSHOT (0.1.0 -> 0.2.0-SNAPSHOT)\n"
	@printf "  $(COLOR_CYAN)make snapshot-major$(COLOR_RESET)    - 创建 major SNAPSHOT (0.1.0 -> 1.0.0-SNAPSHOT)\n"
	@printf "  $(COLOR_GREEN)make release$(COLOR_RESET)           - 发布正式版本并打 tag\n"
	@printf "  $(COLOR_GREEN)make release-push$(COLOR_RESET)      - 发布并推送到远程\n"
	@printf "  $(COLOR_BLUE)make version-info$(COLOR_RESET)      - 显示当前版本信息\n\n"
	@printf "$(COLOR_BOLD)🔧 其他命令:$(COLOR_RESET)\n"
	@printf "  $(COLOR_GREEN)make install$(COLOR_RESET)          - 安装所有依赖\n"
	@printf "  $(COLOR_GREEN)make clean$(COLOR_RESET)            - 清理构建产物\n"
	@printf "  $(COLOR_GREEN)make info$(COLOR_RESET)             - 显示项目信息\n"
	@printf "  $(COLOR_GREEN)make help$(COLOR_RESET)             - 显示此帮助信息\n\n"
	@printf "$(COLOR_BOLD)📝 版本管理工作流:$(COLOR_RESET)\n"
	@printf "  1. 发布 v0.1.0 后:\n"
	@printf "     $(COLOR_CYAN)make snapshot-patch$(COLOR_RESET)  → 创建 0.1.1-SNAPSHOT 用于开发\n\n"
	@printf "  2. 开发新功能...\n\n"
	@printf "  3. 准备发布:\n"
	@printf "     $(COLOR_GREEN)make pre-commit$(COLOR_RESET)    → 运行所有检查\n"
	@printf "     $(COLOR_GREEN)make release-push$(COLOR_RESET)  → 发布 0.1.1 并推送\n\n"
	@printf "  4. GitHub Actions 自动构建并发布到 Releases\n\n"
	@printf "  5. 继续开发:\n"
	@printf "     $(COLOR_CYAN)make snapshot-patch$(COLOR_RESET)  → 创建 0.1.2-SNAPSHOT\n\n"
