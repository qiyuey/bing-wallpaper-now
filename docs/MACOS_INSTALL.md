# macOS 安装和运行指南

## 问题：无法打开应用

如果在 macOS 上安装后出现以下问题之一：
- "应用已损坏，无法打开"
- "无法验证开发者"
- 需要运行 `xattr` 命令

这是因为应用**未经过 Apple 公证**。这是开源软件的常见情况。

## 解决方案

### 方法 1：命令行移除隔离属性（推荐）

打开终端（Terminal），运行以下命令：

```bash
xattr -rd com.apple.quarantine "/Applications/Bing Wallpaper Now.app"
```

**命令说明：**
- `xattr`: macOS 扩展属性管理工具
- `-r`: 递归处理应用包内的所有文件
- `-d`: 删除指定的扩展属性
- `com.apple.quarantine`: macOS Gatekeeper 的隔离标记

**为什么用这个命令？**
- ✅ 精准：只删除导致问题的隔离属性
- ✅ 安全：保留其他有用的文件元数据
- ✅ 标准：这是 macOS 开发社区的推荐做法

### 方法 2：系统设置中允许

1. 尝试打开应用
2. 看到"无法打开"对话框，点击"取消"
3. 打开"系统设置" → "隐私与安全性"
4. 找到"仍要打开"按钮，点击
5. 再次确认打开

### 方法 3：右键打开

1. 在 Finder 中找到应用
2. 按住 Control 键点击应用图标
3. 选择"打开"
4. 在弹出对话框中点击"打开"

## 为什么会这样？

macOS Gatekeeper 要求应用满足以下条件之一：
- ✅ 从 App Store 下载
- ✅ 经过 Apple 开发者签名和公证
- ❌ 开源软件（通常没有签名）

对开源项目进行 Apple 公证需要：
- 💰 Apple Developer 账号（$99/年）
- 📄 复杂的签名流程
- 🔒 需要在 CI 中存储证书

## 安全性

本项目是完全开源的，你可以：
- 📖 查看所有源代码
- 🔍 审计构建流程
- 🛠️ 自己编译应用

源代码：https://github.com/qiyuey/bing-wallpaper-now

## 我们的立场

我们选择不进行 Apple 公证，因为：
1. **透明度优先**：开源代码比签名更可信
2. **成本考虑**：$99/年对个人项目负担较大
3. **社区项目**：鼓励用户审查和参与

如果你愿意赞助 Apple Developer 账号，欢迎联系我们！

## 自己编译

如果你不信任预编译的应用，可以自己编译：

```bash
# 克隆仓库
git clone https://github.com/qiyuey/bing-wallpaper-now.git
cd bing-wallpaper-now

# 安装依赖
pnpm install

# 构建应用
pnpm run tauri build
```

编译后的应用位于 `src-tauri/target/release/bundle/`
