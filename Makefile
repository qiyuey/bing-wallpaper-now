# Makefile - Bing Wallpaper Now (Tauri 2)
#
# 快速参考:
#   make dev            # 启动 Tauri 开发模式 (热重载)
#   make build          # 构建生产版本
#   make test           # 运行所有测试
#   make clean          # 清理构建产物
#   make fmt            # 格式化代码
#   make check          # 代码质量检查
#
# 环境要求:
# - Node.js >= 18 (推荐使用 pnpm)
# - Rust toolchain (stable)
# - macOS: 已安装 Xcode Command Line Tools
# - Linux: libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev patchelf

# ============================================================================
# 配置变量
# ============================================================================

# 包管理器 (支持 pnpm/npm/yarn)
PKG_MANAGER := pnpm
ifeq ($(shell command -v pnpm 2> /dev/null),)
	PKG_MANAGER := npm
endif

# Rust 相关路径
RUST_DIR := src-tauri
RUST_MANIFEST := $(RUST_DIR)/Cargo.toml
RUST_TARGET_DIR := $(RUST_DIR)/target

# 前端相关路径
DIST_DIR := dist
NODE_MODULES := node_modules

# 构建选项
RUST_PROFILE ?= debug
TAURI_BUILD_FLAGS ?=

# 颜色输出
COLOR_RESET := \033[0m
COLOR_BOLD := \033[1m
COLOR_GREEN := \033[32m
COLOR_YELLOW := \033[33m
COLOR_BLUE := \033[34m

# ============================================================================
# Phony 目标
# ============================================================================

.PHONY: all dev build bundle install
.PHONY: test test-rust test-frontend typecheck
.PHONY: fmt fmt-rust fmt-frontend fmt-check
.PHONY: lint lint-rust lint-frontend
.PHONY: check check-rust check-frontend
.PHONY: clean clean-rust clean-frontend clean-all
.PHONY: deps deps-rust deps-frontend
.PHONY: cargo-update cargo-tree
.PHONY: help info

# ============================================================================
# 默认目标
# ============================================================================

all: check test build

# ============================================================================
# 开发命令
# ============================================================================

## dev: 启动 Tauri 开发模式 (前端 + 后端热重载)
dev:
	@printf "$(COLOR_BOLD)$(COLOR_BLUE)🚀 启动 Tauri 开发模式...$(COLOR_RESET)\n"
	$(PKG_MANAGER) run tauri dev

## dev-frontend: 仅启动前端开发服务器
dev-frontend:
	@printf "$(COLOR_BOLD)$(COLOR_BLUE)⚡ 启动 Vite 开发服务器...$(COLOR_RESET)\n"
	$(PKG_MANAGER) run dev

## dev-rust: 仅编译 Rust 代码 (开发模式)
dev-rust:
	@printf "$(COLOR_BOLD)$(COLOR_BLUE)🦀 编译 Rust 代码 (debug)...$(COLOR_RESET)\n"
	cargo build --manifest-path $(RUST_MANIFEST)

# ============================================================================
# 构建命令
# ============================================================================

## build: 构建生产版本 (前端 + 类型检查)
build: typecheck
	@printf "$(COLOR_BOLD)$(COLOR_GREEN)📦 构建前端生产版本...$(COLOR_RESET)\n"
	$(PKG_MANAGER) run build

## bundle: 构建 Tauri 完整应用包
bundle:
	@printf "$(COLOR_BOLD)$(COLOR_GREEN)📦 构建 Tauri 应用包...$(COLOR_RESET)\n"
	$(PKG_MANAGER) run tauri build $(TAURI_BUILD_FLAGS)

## build-rust: 仅构建 Rust 代码 (release 模式)
build-rust:
	@printf "$(COLOR_BOLD)$(COLOR_GREEN)🦀 编译 Rust 代码 (release)...$(COLOR_RESET)\n"
	cargo build --manifest-path $(RUST_MANIFEST) --release

# ============================================================================
# 安装依赖
# ============================================================================

## install: 安装所有依赖 (Node + Rust)
install: deps

## deps: 安装前端和 Rust 依赖
deps: deps-frontend deps-rust

## deps-frontend: 安装前端依赖
deps-frontend:
	@printf "$(COLOR_BOLD)$(COLOR_BLUE)📦 安装前端依赖...$(COLOR_RESET)\n"
	$(PKG_MANAGER) install

## deps-rust: 更新 Rust 依赖
deps-rust:
	@printf "$(COLOR_BOLD)$(COLOR_BLUE)🦀 检查 Rust 依赖...$(COLOR_RESET)\n"
	cargo fetch --manifest-path $(RUST_MANIFEST)

# ============================================================================
# 测试命令
# ============================================================================

## test: 运行所有测试 (Rust + 前端)
test: test-rust test-frontend

## test-rust: 运行 Rust 测试
test-rust:
	@printf "$(COLOR_BOLD)$(COLOR_YELLOW)🧪 运行 Rust 测试...$(COLOR_RESET)\n"
	cargo test --manifest-path $(RUST_MANIFEST) -- --nocapture

## test-frontend: 运行前端测试
test-frontend:
	@printf "$(COLOR_BOLD)$(COLOR_YELLOW)🧪 运行前端测试...$(COLOR_RESET)\n"
	$(PKG_MANAGER) run test:frontend

## test-coverage: 生成测试覆盖率报告
test-coverage:
	@printf "$(COLOR_BOLD)$(COLOR_YELLOW)📊 生成测试覆盖率...$(COLOR_RESET)\n"
	cargo tarpaulin --manifest-path $(RUST_MANIFEST) --out Html

## typecheck: TypeScript 类型检查
typecheck:
	@printf "$(COLOR_BOLD)$(COLOR_YELLOW)🔍 TypeScript 类型检查...$(COLOR_RESET)\n"
	$(PKG_MANAGER) run typecheck

# ============================================================================
# 代码格式化
# ============================================================================

## fmt: 格式化所有代码 (Rust + 前端)
fmt: fmt-rust fmt-frontend

## fmt-rust: 格式化 Rust 代码
fmt-rust:
	@printf "$(COLOR_BOLD)$(COLOR_BLUE)✨ 格式化 Rust 代码...$(COLOR_RESET)\n"
	cargo fmt --manifest-path $(RUST_MANIFEST)

## fmt-frontend: 格式化前端代码
fmt-frontend:
	@printf "$(COLOR_BOLD)$(COLOR_BLUE)✨ 格式化前端代码...$(COLOR_RESET)\n"
	$(PKG_MANAGER) run format || true

## fmt-check: 检查代码格式 (不修改文件)
fmt-check: fmt-check-rust fmt-check-frontend

## fmt-check-rust: 检查 Rust 代码格式
fmt-check-rust:
	@printf "$(COLOR_BOLD)$(COLOR_YELLOW)🔍 检查 Rust 代码格式...$(COLOR_RESET)\n"
	cargo fmt --manifest-path $(RUST_MANIFEST) -- --check

## fmt-check-frontend: 检查前端代码格式
fmt-check-frontend:
	@printf "$(COLOR_BOLD)$(COLOR_YELLOW)🔍 检查前端代码格式...$(COLOR_RESET)\n"
	$(PKG_MANAGER) run format:check || true

# ============================================================================
# 代码检查 (Linting)
# ============================================================================

## lint: 运行所有 linter (Rust + 前端)
lint: lint-rust lint-frontend

## lint-rust: 运行 Rust clippy
lint-rust:
	@printf "$(COLOR_BOLD)$(COLOR_YELLOW)🔍 运行 Rust Clippy...$(COLOR_RESET)\n"
	cargo clippy --manifest-path $(RUST_MANIFEST) -- -D warnings

## lint-frontend: 运行前端 ESLint
lint-frontend:
	@printf "$(COLOR_BOLD)$(COLOR_YELLOW)🔍 运行 ESLint...$(COLOR_RESET)\n"
	$(PKG_MANAGER) run lint || true

## lint-fix: 自动修复 lint 问题
lint-fix:
	@printf "$(COLOR_BOLD)$(COLOR_BLUE)🔧 自动修复 lint 问题...$(COLOR_RESET)\n"
	cargo clippy --manifest-path $(RUST_MANIFEST) --fix --allow-dirty --allow-staged
	$(PKG_MANAGER) run lint:fix || true

# ============================================================================
# 代码质量检查
# ============================================================================

## check: 运行所有质量检查 (类型检查 + lint + 格式检查)
check: check-rust check-frontend

## check-rust: Rust 代码质量检查
check-rust: fmt-check-rust lint-rust
	@printf "$(COLOR_BOLD)$(COLOR_YELLOW)🔍 Rust 编译检查...$(COLOR_RESET)\n"
	cargo check --manifest-path $(RUST_MANIFEST)

## check-frontend: 前端代码质量检查
check-frontend: fmt-check-frontend typecheck lint-frontend

# ============================================================================
# 清理命令
# ============================================================================

## clean: 清理所有构建产物
clean: clean-rust clean-frontend

## clean-rust: 清理 Rust 构建产物
clean-rust:
	@printf "$(COLOR_BOLD)$(COLOR_YELLOW)🧹 清理 Rust 构建产物...$(COLOR_RESET)\n"
	cargo clean --manifest-path $(RUST_MANIFEST)

## clean-frontend: 清理前端构建产物
clean-frontend:
	@printf "$(COLOR_BOLD)$(COLOR_YELLOW)🧹 清理前端构建产物...$(COLOR_RESET)\n"
	rm -rf $(DIST_DIR)

## clean-all: 深度清理 (包括依赖)
clean-all: clean
	@printf "$(COLOR_BOLD)$(COLOR_YELLOW)🧹 清理所有依赖...$(COLOR_RESET)\n"
	rm -rf $(NODE_MODULES)
	rm -rf $(RUST_TARGET_DIR)

# ============================================================================
# 依赖管理
# ============================================================================

## cargo-update: 更新 Rust 依赖
cargo-update:
	@printf "$(COLOR_BOLD)$(COLOR_BLUE)🔄 更新 Rust 依赖...$(COLOR_RESET)\n"
	cargo update --manifest-path $(RUST_MANIFEST)

## cargo-tree: 显示 Rust 依赖树
cargo-tree:
	@printf "$(COLOR_BOLD)$(COLOR_BLUE)🌳 显示依赖树...$(COLOR_RESET)\n"
	cargo tree --manifest-path $(RUST_MANIFEST)

## cargo-audit: Rust 安全审计
cargo-audit:
	@printf "$(COLOR_BOLD)$(COLOR_YELLOW)🔒 运行安全审计...$(COLOR_RESET)\n"
	cargo audit --manifest-path $(RUST_MANIFEST)

# ============================================================================
# 工具命令
# ============================================================================

## doc: 生成 Rust 文档
doc:
	@printf "$(COLOR_BOLD)$(COLOR_BLUE)📚 生成 Rust 文档...$(COLOR_RESET)\n"
	cargo doc --manifest-path $(RUST_MANIFEST) --no-deps --open

## info: 显示项目信息
info:
	@printf "$(COLOR_BOLD)$(COLOR_BLUE)ℹ️  项目信息$(COLOR_RESET)\n"
	@printf "包管理器: $(COLOR_GREEN)$(PKG_MANAGER)$(COLOR_RESET)\n"
	@printf "Rust 版本: $(COLOR_GREEN)"
	@rustc --version
	@printf "$(COLOR_RESET)"
	@printf "Node 版本: $(COLOR_GREEN)"
	@node --version
	@printf "$(COLOR_RESET)"
	@printf "Tauri CLI: $(COLOR_GREEN)"
	@$(PKG_MANAGER) run tauri --version | head -n 1
	@printf "$(COLOR_RESET)"

# ============================================================================
# CI/CD 相关
# ============================================================================

## ci: CI 流程 (检查 + 测试 + 构建)
ci: check test build
	@printf "$(COLOR_BOLD)$(COLOR_GREEN)✅ CI 检查通过$(COLOR_RESET)\n"

## ci-bundle: CI 完整构建流程
ci-bundle: check test bundle
	@printf "$(COLOR_BOLD)$(COLOR_GREEN)✅ CI 构建完成$(COLOR_RESET)\n"

# ============================================================================
# 帮助信息
# ============================================================================

## help: 显示帮助信息
help:
	@printf "$(COLOR_BOLD)Bing Wallpaper Now - Makefile 命令$(COLOR_RESET)\n\n"
	@printf "$(COLOR_BOLD)开发命令:$(COLOR_RESET)\n"
	@printf "  $(COLOR_GREEN)make dev$(COLOR_RESET)              - 启动 Tauri 开发模式 (热重载)\n"
	@printf "  $(COLOR_GREEN)make dev-frontend$(COLOR_RESET)     - 仅启动前端开发服务器\n"
	@printf "  $(COLOR_GREEN)make dev-rust$(COLOR_RESET)         - 仅编译 Rust 代码\n\n"
	@printf "$(COLOR_BOLD)构建命令:$(COLOR_RESET)\n"
	@printf "  $(COLOR_GREEN)make build$(COLOR_RESET)            - 构建前端生产版本\n"
	@printf "  $(COLOR_GREEN)make bundle$(COLOR_RESET)           - 构建 Tauri 完整应用包\n"
	@printf "  $(COLOR_GREEN)make build-rust$(COLOR_RESET)       - 仅构建 Rust (release)\n\n"
	@printf "$(COLOR_BOLD)测试命令:$(COLOR_RESET)\n"
	@printf "  $(COLOR_GREEN)make test$(COLOR_RESET)             - 运行所有测试\n"
	@printf "  $(COLOR_GREEN)make test-rust$(COLOR_RESET)        - 运行 Rust 测试\n"
	@printf "  $(COLOR_GREEN)make test-frontend$(COLOR_RESET)    - 运行前端测试\n"
	@printf "  $(COLOR_GREEN)make typecheck$(COLOR_RESET)        - TypeScript 类型检查\n\n"
	@printf "$(COLOR_BOLD)代码质量:$(COLOR_RESET)\n"
	@printf "  $(COLOR_GREEN)make check$(COLOR_RESET)            - 运行所有质量检查\n"
	@printf "  $(COLOR_GREEN)make fmt$(COLOR_RESET)              - 格式化所有代码\n"
	@printf "  $(COLOR_GREEN)make lint$(COLOR_RESET)             - 运行所有 linter\n"
	@printf "  $(COLOR_GREEN)make lint-fix$(COLOR_RESET)         - 自动修复 lint 问题\n\n"
	@printf "$(COLOR_BOLD)清理命令:$(COLOR_RESET)\n"
	@printf "  $(COLOR_GREEN)make clean$(COLOR_RESET)            - 清理构建产物\n"
	@printf "  $(COLOR_GREEN)make clean-all$(COLOR_RESET)        - 深度清理 (包括依赖)\n\n"
	@printf "$(COLOR_BOLD)其他命令:$(COLOR_RESET)\n"
	@printf "  $(COLOR_GREEN)make install$(COLOR_RESET)          - 安装所有依赖\n"
	@printf "  $(COLOR_GREEN)make doc$(COLOR_RESET)              - 生成 Rust 文档\n"
	@printf "  $(COLOR_GREEN)make info$(COLOR_RESET)             - 显示项目信息\n"
	@printf "  $(COLOR_GREEN)make ci$(COLOR_RESET)               - 运行 CI 流程\n"
	@printf "  $(COLOR_GREEN)make help$(COLOR_RESET)             - 显示此帮助信息\n\n"
	@printf "$(COLOR_BOLD)示例:$(COLOR_RESET)\n"
	@printf "  make dev                  # 开始开发\n"
	@printf "  make check test           # 代码检查和测试\n"
	@printf "  make bundle               # 构建应用包\n"
	@printf "  RUST_PROFILE=release make dev-rust\n\n"
