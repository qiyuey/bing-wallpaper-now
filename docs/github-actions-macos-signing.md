# GitHub Actions macOS 代码签名配置指南

## 概述

GitHub Actions 默认不知道你的开发者证书。需要在 GitHub Secrets 中配置证书，CI 才能使用 `tauri.conf.json` 中指定的 `signingIdentity` 进行代码签名。

## 当前配置

- **签名身份**: `Bing Wallpaper Now Developer`（在 `tauri.conf.json` 中配置）
- **Hardened Runtime**: 已禁用（`hardenedRuntime: false`）
- **Entitlements**: 无（`entitlements: null`）

## 设置步骤

### 1. 导出证书为 .p12 文件

**重要**：`APPLE_CERTIFICATE_PASSWORD` 是在导出 `.p12` 文件时设置的密码，不是证书本身的密码。

#### 方法一：使用 Keychain Access（推荐，图形界面）

1. 打开 **Keychain Access**（钥匙串访问）应用
2. 在左侧选择 **login** 密钥链
3. 在搜索框输入 `Bing Wallpaper Now Developer`
4. 找到证书后，**右键点击** -> 选择 **Export "Bing Wallpaper Now Developer"...**
5. 选择保存位置（建议桌面），文件名例如 `certificate.p12`
6. 文件格式选择 **Personal Information Exchange (.p12)**
7. 点击 **Save** 后，会弹出密码设置对话框
8. **设置一个密码**（这就是 `APPLE_CERTIFICATE_PASSWORD`，记住它！）
9. 确认密码

#### 方法二：使用命令行（推荐，明确设置密码）

```bash
# 方法 A：交互式导出（会提示输入密钥链密码和设置 .p12 密码）
security export -k ~/Library/Keychains/login.keychain-db \
  -t identities -f pkcs12 \
  -o ~/Desktop/certificate.p12 \
  "Bing Wallpaper Now Developer"

# 方法 B：直接设置密码（推荐，避免交互）
security export -k ~/Library/Keychains/login.keychain-db \
  -t identities -f pkcs12 -P "你设置的密码" \
  -o ~/Desktop/certificate.p12 \
  "Bing Wallpaper Now Developer"
```

**注意**：
- 如果使用 `-P` 参数，密码会直接设置（这就是 `APPLE_CERTIFICATE_PASSWORD`）
- 如果不使用 `-P` 参数，macOS 可能会使用密钥链密码或要求交互输入
- **建议使用方法 B**，明确设置密码，避免混淆

### 2. 将证书转换为 Base64

```bash
# 将 .p12 文件转换为 Base64 字符串
base64 -i ~/Desktop/certificate.p12 -o ~/Desktop/certificate_base64.txt

# 或者直接输出到终端（复制完整内容，包括所有行）
base64 -i ~/Desktop/certificate.p12 | pbcopy  # macOS 自动复制到剪贴板
# 或者
base64 -i ~/Desktop/certificate.p12  # 手动复制输出内容
```

**注意**：Base64 字符串可能很长，确保复制完整内容。

### 3. 配置 GitHub Secrets

在 GitHub 仓库中：

1. 进入 **Settings** -> **Secrets and variables** -> **Actions**
2. 点击 **New repository secret**
3. 添加以下三个 Secrets：

   - **`APPLE_CERTIFICATE`**
     - Value: 步骤 2 中生成的 Base64 字符串（完整内容）
   
   - **`APPLE_CERTIFICATE_PASSWORD`**
     - Value: **导出 .p12 文件时设置的密码**（步骤 1 中设置的密码）
     - ⚠️ 这不是证书本身的密码，而是导出文件时设置的密码
   
   - **`KEYCHAIN_PASSWORD`**
     - Value: 用于创建临时密钥链的密码
     - **长度要求**：至少 8 个字符（建议 16-32 个字符）
     - **建议**：使用随机生成的字符串，例如：
       ```bash
       # 生成 32 位随机密码
       openssl rand -base64 24
       # 或使用 Python
       python3 -c "import secrets; print(secrets.token_urlsafe(24))"
       ```
     - **注意**：这只是临时密钥链的密码，与证书密码无关，可以设置为任何安全的随机字符串

### 4. 验证配置

配置完成后，下次推送标签触发构建时：

- ✅ **如果所有 Secrets 都已配置**：证书会被导入，应用会被签名，构建成功
- ❌ **如果缺少任何 Secret**：macOS 构建会失败，错误信息会明确指出缺少哪个 Secret

**验证方法**：
1. 确保三个 Secrets 都已添加
2. 推送一个新标签触发构建
3. 检查 GitHub Actions 日志，应该看到 "✅ All required secrets are configured" 和 "✅ Certificate imported successfully"

## 工作原理

GitHub Actions 工作流中的 `Import macOS signing certificate` 步骤会：

1. **验证必需的 Secrets**：检查 `APPLE_CERTIFICATE`、`APPLE_CERTIFICATE_PASSWORD` 和 `KEYCHAIN_PASSWORD` 是否都已配置
2. 如果缺少任何 Secret，**构建会失败**（不会跳过签名）
3. 从 `APPLE_CERTIFICATE` Secret 读取 Base64 编码的证书
4. 解码并保存为 `.p12` 文件
5. 创建临时密钥链
6. 导入证书到密钥链
7. Tauri 构建时会自动使用 `tauri.conf.json` 中指定的 `signingIdentity` 进行签名

## ⚠️ 重要提示

- **强制要求**：所有三个 Secrets（`APPLE_CERTIFICATE`、`APPLE_CERTIFICATE_PASSWORD`、`KEYCHAIN_PASSWORD`）都是**必需的**
- **构建失败**：如果缺少任何 Secret，macOS 构建会失败，不会继续构建未签名的应用
- **不上架 App Store**：当前配置已禁用 Hardened Runtime，适合直接分发
- **证书有效期**：确保证书未过期，过期后需要更新 Secret
- **安全性**：证书和密码存储在 GitHub Secrets 中，只有仓库管理员可以访问
- **免费开发者账号**：可以使用免费 Apple Developer 账号创建的证书

## 故障排除

### 构建失败：缺少必需的 Secret

**错误信息**：
```
❌ Error: APPLE_CERTIFICATE secret is not configured
```

**解决方法**：
1. 进入 GitHub 仓库的 Settings -> Secrets and variables -> Actions
2. 添加缺失的 Secret
3. 重新触发构建

### 构建失败：证书导入失败

**错误信息**：
```
❌ Error: Failed to import certificate
```

**检查清单**：
1. `APPLE_CERTIFICATE` 是否为有效的 Base64 编码的 .p12 文件
2. `APPLE_CERTIFICATE_PASSWORD` 是否与导出证书时设置的密码完全匹配
3. `tauri.conf.json` 中的 `signingIdentity` 是否与证书名称完全匹配
4. 证书是否已过期（检查 Apple Developer 账号）

### 构建失败：证书解码失败

**错误信息**：
```
❌ Error: Failed to decode certificate
```

**解决方法**：
1. 重新导出证书为 .p12 文件
2. 使用 `base64 -i certificate.p12` 重新生成 Base64 字符串
3. 确保复制完整的 Base64 字符串（包括所有行）
4. 更新 `APPLE_CERTIFICATE` Secret

