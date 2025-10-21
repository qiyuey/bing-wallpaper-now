# Scripts 目录

本目录包含项目开发和构建相关的脚本工具。

---

## 📋 目录

- [版本管理](#-版本管理)
- [代码质量检查](#-代码质量检查)
- [跨平台编译检查](#-跨平台编译检查)
- [典型工作流](#-典型工作流)
- [技术细节](#-技术细节)
- [故障排除](#-故障排除)

---

## 🏷️ 版本管理

### `version.sh` - 自动版本管理

快速升级版本号并创建 Git 标签，类似 `npm version` 命令。

**使用方法：**

```bash
# 使用 Makefile（推荐）
make version-patch    # 0.1.0 -> 0.1.1
make version-minor    # 0.1.0 -> 0.2.0
make version-major    # 0.1.0 -> 1.0.0

# 或直接使用脚本
./scripts/version.sh patch
./scripts/version.sh minor
./scripts/version.sh major
./scripts/version.sh 1.2.3  # 指定版本
```

**自动完成的操作：**

- ✅ 更新 `package.json` 版本号
- ✅ 更新 `src-tauri/Cargo.toml` 版本号
- ✅ 更新 `src-tauri/tauri.conf.json` 版本号
- ✅ 更新 `Cargo.lock`
- ✅ Git 提交（`chore(release): x.y.z`）
- ✅ 创建 Git 标签（`vx.y.z`）
- ✅ 可选：推送到远程

详细文档：[VERSION_MANAGEMENT.md](../docs/VERSION_MANAGEMENT.md)

---

## 🔍 代码质量检查

### `check-quality.sh` - 快速质量检查 ⚡ (推荐)

在本地快速验证代码质量，包括 Clippy、格式检查和单元测试。

**使用方法：**

```bash
# 方法 1: 使用 Makefile（推荐）
make check-quality

# 方法 2: 直接运行脚本
./scripts/check-quality.sh
```

**包含的检查：**

- ✅ **Rust Clippy**: 捕获常见错误和代码异味
- ✅ **Rust 格式检查**: 确保代码风格一致
- ✅ **单元测试**: 验证核心功能
- ✅ **前端 ESLint**: 检查 TypeScript/React 代码质量
- ✅ **前端格式检查**: Prettier 格式验证

**特点：**

- ⚡ 速度快（不涉及交叉编译）
- 🎯 最接近 CI 环境的检查
- 💡 **推荐在每次提交前运行**

---

## 🌍 跨平台编译检查

### `check-all-platforms.sh` - 全平台编译检查

检查代码在所有主要平台上的编译兼容性。

**使用方法：**

```bash
# 方法 1: 使用 Makefile（推荐）
make check-cross

# 方法 2: 直接运行脚本
./scripts/check-all-platforms.sh
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

### `make check-ci` - 模拟 CI 环境 🤖

模拟 GitHub Actions CI 检查流程。

**使用方法：**

```bash
make check-ci
```

**包含的检查：**

1. 所有代码质量检查（Clippy、格式、Lint、测试）
2. 跨平台编译兼容性验证

**推荐用法：** 在提交和推送前运行此命令

---

## 🚀 典型工作流

### 场景 1: 日常开发（快速检查）

适用于小改动、bug 修复、代码优化等。

```bash
# 1. 修改代码
vim src-tauri/src/wallpaper_manager.rs

# 2. 快速代码质量检查
make check-quality

# 3. 如果通过，提交代码
git add -A
git commit -m "fix: resolve wallpaper setting issue"
git push
```

---

### 场景 2: 重要修改（全面检查）

适用于新功能、架构调整、依赖升级等。

```bash
# 1. 修改代码
vim src-tauri/src/wallpaper_manager.rs

# 2. 完整的质量和跨平台检查
make check-ci

# 3. 如果通过，提交代码
git add -A
git commit -m "feat: add new wallpaper download feature"
git push
```

---

### 场景 3: 修复 CI 失败

当 GitHub Actions CI 检查失败时，在本地快速复现和修复。

```bash
# 1. CI 失败，查看错误信息（例如 Clippy 错误）

# 2. 本地复现问题
make check-quality

# 3. 修改代码修复问题
vim src-tauri/src/wallpaper_manager.rs

# 4. 再次检查确认修复
make check-quality

# 5. 确认通过后推送
git add -A
git commit -m "fix: resolve Clippy warnings"
git push
```

---

### 场景 4: 发布新版本

完整的版本发布流程。

```bash
# 1. 确保所有检查通过
make check-ci

# 2. 升级版本号
make version-patch    # 或 version-minor / version-major

# 3. 推送到远程（触发 CI 构建和发布）
git push --follow-tags
```

---

## 🔧 技术细节

### 工作原理

**代码质量检查：**

- 使用 `cargo clippy` 进行静态代码分析
- 使用 `cargo fmt` 检查代码格式
- 使用 `cargo test` 运行单元测试
- 使用 `eslint` 检查前端代码
- 使用 `prettier` 检查前端格式

**跨平台编译检查：**

- 使用 `rustup target add` 安装交叉编译目标
- 使用 `cargo check --target <platform>` 进行编译检查
- 不需要链接器：`cargo check` 只检查代码能否编译，不需要实际链接

---

### 优势

- ⚡ **快速**: `cargo check` 比完整编译快得多
- 🎯 **准确**: 与 CI 环境使用相同的编译目标
- 💻 **本地**: 无需等待 CI 运行
- 🔄 **可重复**: 可以反复运行直到通过

---

### 限制

- ⚠️ **Tauri 交叉编译限制**: Linux 构建依赖系统库，在 macOS 上无法完整检查
- ⚠️ 只检查编译，不检查链接
- ⚠️ 不会检查平台特定的运行时行为
- ⚠️ 无法检测某些链接器错误

---

### 实际建议

对于 Tauri 项目，推荐的本地检查流程：

1. **日常开发**: 使用 `make check-quality` 快速验证
2. **重要修改**: 使用 `make check-ci` 全面检查
3. **最终验证**: 依赖 GitHub Actions 进行完整的 CI/CD 流程

---

### 首次运行注意事项

首次运行 `check-cross` 会自动下载编译目标，需要一些时间：

```bash
$ make check-cross
📦 安装 x86_64-unknown-linux-gnu...
info: downloading component 'rust-std' for 'x86_64-unknown-linux-gnu'
info: installing component 'rust-std' for 'x86_64-unknown-linux-gnu'
...
```

后续运行会很快。`check-quality` 无需下载额外依赖，始终快速。

---

## 📊 与 CI 的对比

| 特性         | 本地检查 (`check-quality`) | GitHub Actions CI  |
| ------------ | -------------------------- | ------------------ |
| 速度         | ⚡ 快 (秒级)               | 🐢 慢 (分钟级)     |
| 成本         | 免费                       | 有配额限制         |
| 反馈周期     | 即时                       | 需要推送等待       |
| 完整性       | 编译检查 + 质量检查        | 完整 CI/CD 流程    |
| 跨平台构建   | 有限（依赖限制）           | 完整（原生构建）   |
| 建议用途     | 开发时快速验证             | 正式发布前最终验证 |

---

## 💡 最佳实践

### 1. 每次修改代码后运行质量检查

```bash
make check-quality
```

### 2. 提交前运行 CI 模拟

```bash
make check-ci
```

### 3. 重大重构时运行全平台检查

```bash
make check-cross
```

### 4. 配置 Git pre-push hook（可选）

防止推送未通过检查的代码：

```bash
# .git/hooks/pre-push
#!/bin/sh
echo "运行代码质量检查..."
make check-quality || exit 1
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
Permission denied: ./scripts/check-quality.sh
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
2. 自动修复：`npm run lint:fix`
3. 如果是误报，可以添加 `// eslint-disable-next-line` 注释

---

## 📚 相关资源

- [Rust 交叉编译文档](https://rust-lang.github.io/rustup/cross-compilation.html)
- [cargo check 文档](https://doc.rust-lang.org/cargo/commands/cargo-check.html)
- [cargo clippy 文档](https://doc.rust-lang.org/clippy/)
- [Tauri 跨平台构建指南](https://tauri.app/v1/guides/building/cross-platform)
- [ESLint 文档](https://eslint.org/docs/latest/)
- [Prettier 文档](https://prettier.io/docs/en/index.html)
