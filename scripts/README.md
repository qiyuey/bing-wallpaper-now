# Scripts 目录

本目录包含项目开发和构建相关的脚本工具。

---

## 📋 脚本列表

- **`version.sh`** - SNAPSHOT 版本管理
- **`check-commit.sh`** - 提交前完整 CI 检查
- **`check-cross.sh`** - 跨平台编译检查

---

## 🏷️ 版本管理

### `version.sh` - SNAPSHOT 版本管理

基于 SNAPSHOT 的版本管理工作流，类似 Maven 的版本管理方式。

**使用方法：**

```bash
# 使用 Makefile（推荐）
make snapshot-patch    # 0.1.0 -> 0.1.1-SNAPSHOT
make snapshot-minor    # 0.1.0 -> 0.2.0-SNAPSHOT
make snapshot-major    # 0.1.0 -> 1.0.0-SNAPSHOT
make release           # 0.1.1-SNAPSHOT -> 0.1.1 并推送
make version-info      # 查看当前版本信息

# 或直接使用脚本
./scripts/version.sh snapshot-patch
./scripts/version.sh snapshot-minor
./scripts/version.sh snapshot-major
./scripts/version.sh release
./scripts/version.sh info
```

**工作流程：**

1. 发布 v0.1.0 后，创建 0.1.1-SNAPSHOT 用于开发
2. 开发完成后，运行 release 转为 0.1.1 正式版本、打 tag 并推送到远程
3. 发布后，再次创建 0.1.2-SNAPSHOT 继续开发

**自动完成的操作：**

- ✅ 更新 `package.json` 版本号
- ✅ 更新 `src-tauri/Cargo.toml` 版本号
- ✅ 更新 `src-tauri/tauri.conf.json` 版本号
- ✅ 更新 `Cargo.lock`
- ✅ Git 提交（`chore(version): x.y.z-SNAPSHOT` 或 `chore(release): x.y.z`）
- ✅ 创建 Git 标签（仅 release）
- ✅ 推送到远程（release 时可选）

---

## 🔍 代码质量检查

### `check-commit.sh` - 提交前完整检查 ⚡ (推荐)

在本地运行完整的 CI 检查流程，包括格式、Lint、类型检查和测试。

**使用方法：**

```bash
# 使用 Makefile（推荐）
make pre-commit

# 或直接运行脚本
./scripts/check-commit.sh
```

**包含的检查：**

- ✅ **Rust 格式检查**: `cargo fmt --check`
- ✅ **Rust Clippy**: 捕获常见错误和代码异味
- ✅ **Rust 测试**: 验证核心功能
- ✅ **TypeScript 类型检查**: `tsc --noEmit`
- ✅ **前端 ESLint**: 检查 TypeScript/React 代码质量
- ✅ **前端格式检查**: Prettier 格式验证
- ✅ **前端测试**: 运行所有前端测试
- ✅ **前端构建**: 确保构建成功

**特点：**

- ⚡ 在本地快速验证（无需等待 CI）
- 🎯 完全模拟 CI 环境的检查
- 💡 **推荐在每次提交前运行**

---

## 🌍 跨平台编译检查

### `check-cross.sh` - 全平台编译检查

检查代码在所有主要平台上的编译兼容性。

**使用方法：**

```bash
# 使用 Makefile（推荐）
make check-cross

# 或直接运行脚本
./scripts/check-cross.sh
```

**检查的平台：**

- ✅ Linux (x86_64-unknown-linux-gnu)
- ✅ Windows (x86_64-pc-windows-msvc)
- ✅ macOS Intel (x86_64-apple-darwin)
- ✅ macOS Apple Silicon (aarch64-apple-darwin)

**特点：**

- 🌍 全面覆盖所有目标平台
- 🔧 自动安装所有必要的编译目标
- 📊 显示详细的结果汇总

**重要说明：**

- ⚠️ **Tauri Linux 交叉编译限制**: Tauri 应用依赖 GTK/WebKit 等 Linux 系统库，在 macOS 上无法进行完整的交叉编译检查
- ⚠️ 只检查编译，不检查链接和运行时行为
- ⚠️ 首次运行会下载编译目标（可能需要几分钟）

---

## 🚀 典型工作流

### 场景 1: 日常开发

```bash
# 1. 修改代码
vim src-tauri/src/wallpaper_manager.rs

# 2. 提交前检查
make pre-commit

# 3. 如果通过，提交代码
git add -A
git commit -m "fix: resolve wallpaper setting issue"
git push
```

---

### 场景 2: 发布新版本

```bash
# 1. 确保所有检查通过
make pre-commit

# 2. 发布版本（会移除 SNAPSHOT 后缀、打 tag 并推送）
make release

# 3. GitHub Actions 自动构建并发布到 Releases

# 4. 创建下一个 SNAPSHOT 版本
make snapshot-patch
```

---

### 场景 3: 修复 CI 失败

```bash
# 1. CI 失败，查看错误信息

# 2. 本地复现问题
make pre-commit

# 3. 修改代码修复问题
vim src-tauri/src/wallpaper_manager.rs

# 4. 再次检查确认修复
make pre-commit

# 5. 确认通过后推送
git add -A
git commit -m "fix: resolve Clippy warnings"
git push
```

---

## 🔧 技术细节

### 工作原理

**代码质量检查：**

- 使用 `cargo fmt` 检查代码格式
- 使用 `cargo clippy` 进行静态代码分析
- 使用 `cargo test` 运行单元测试
- 使用 `tsc --noEmit` 进行 TypeScript 类型检查
- 使用 `eslint` 检查前端代码
- 使用 `prettier` 检查前端格式
- 使用 `vitest` 运行前端测试
- 使用 `vite build` 确保构建成功

**跨平台编译检查：**

- 使用 `rustup target add` 安装交叉编译目标
- 使用 `cargo check --target <platform>` 进行编译检查
- 不需要链接器：`cargo check` 只检查代码能否编译，不需要实际链接

---

### 优势

- ⚡ **快速**: 本地运行，无需等待 CI
- 🎯 **准确**: 与 CI 环境使用相同的检查
- 💻 **便捷**: 可以反复运行直到通过
- 🔄 **可靠**: 提前发现问题，避免 CI 失败

---

### 限制

- ⚠️ **Tauri 交叉编译限制**: Linux 构建依赖系统库，在 macOS 上无法完整检查
- ⚠️ 只检查编译，不检查链接
- ⚠️ 不会检查平台特定的运行时行为

---

## 📊 与 CI 的对比

| 特性         | 本地检查 (`check-commit`) | GitHub Actions CI  |
| ------------ | -------------------------- | ------------------ |
| 速度         | ⚡ 快 (秒级)               | 🐢 慢 (分钟级)     |
| 成本         | 免费                       | 有配额限制         |
| 反馈周期     | 即时                       | 需要推送等待       |
| 完整性       | 完整 CI 检查               | 完整 CI/CD 流程    |
| 跨平台构建   | 有限（依赖限制）           | 完整（原生构建）   |
| 建议用途     | 提交前验证                 | 正式发布前最终验证 |

---

## 💡 最佳实践

### 1. 每次提交前运行完整检查

```bash
make pre-commit
```

### 2. 配置 Git pre-push hook（可选）

防止推送未通过检查的代码：

```bash
# .git/hooks/pre-push
#!/bin/sh
echo "运行提交前检查..."
make pre-commit || exit 1
```

设置可执行权限：

```bash
chmod +x .git/hooks/pre-push
```

---

## 🐛 故障排除

### 问题: 脚本没有执行权限

**错误信息：**

```
Permission denied: ./scripts/check-commit.sh
```

**解决方案：**

```bash
chmod +x scripts/*.sh
```

---

### 问题: rustup 找不到

**错误信息：**

```
command not found: rustup
```

**解决方案：**

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

---

### 问题: 编译目标下载失败

**错误信息：**

```
error: toolchain 'stable-x86_64-unknown-linux-gnu' does not support target 'x86_64-unknown-linux-gnu'
```

**解决方案：**

```bash
# 手动安装编译目标
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-pc-windows-msvc
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin
```

---

### 问题: Clippy 检查失败

**错误信息：**

```
error: unused variable: `foo`
```

**解决方案：**

1. 根据 Clippy 提示修复代码问题
2. 如果是误报，可以在代码中添加 `#[allow(clippy::lint_name)]`
3. 查看 Clippy 文档了解具体 lint 规则

---

### 问题: 前端 ESLint 检查失败

**错误信息：**

```
error: 'foo' is assigned a value but never used
```

**解决方案：**

1. 根据 ESLint 提示修复代码问题
2. 自动修复：`pnpm run lint:fix` 或 `npm run lint:fix`
3. 如果是误报，可以添加 `// eslint-disable-next-line` 注释

---

## 📚 相关资源

- [Rust 交叉编译文档](https://rust-lang.github.io/rustup/cross-compilation.html)
- [cargo check 文档](https://doc.rust-lang.org/cargo/commands/cargo-check.html)
- [cargo clippy 文档](https://doc.rust-lang.org/clippy/)
- [Tauri 跨平台构建指南](https://tauri.app/v1/guides/building/cross-platform)
- [ESLint 文档](https://eslint.org/docs/latest/)
- [Prettier 文档](https://prettier.io/docs/en/index.html)
