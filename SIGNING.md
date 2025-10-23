# macOS Code Signing Setup

本文档说明如何设置 GitHub Actions 的 macOS 代码签名，以便自动签名应用并避免用户在打开应用时遇到 Gatekeeper 警告。

## 方案说明

我们使用 **ad-hoc 签名**（免费方案），不需要付费的 Apple Developer 账号。这种方式虽然不能进行公证（notarization），但可以让应用被 Gatekeeper 识别，减少用户打开应用时的麻烦。

## 前提条件

- 一台 macOS 电脑
- 一个 Apple ID（免费）
- Xcode 或 Xcode Command Line Tools

## 步骤 1：创建自签名证书

在你的 Mac 上打开"钥匙串访问"（Keychain Access）应用：

1. 打开"钥匙串访问" -> "证书助理" -> "创建证书"
2. 填写以下信息：
   - **名称**: `Bing Wallpaper Now Developer` （可以自定义）
   - **身份类型**: 选择 "自签名根证书"
   - **证书类型**: 选择 "代码签名"
3. 点击"创建"，然后"继续"直到完成

## 步骤 2：导出证书

1. 在"钥匙串访问"中找到刚创建的证书
2. 右键点击证书，选择"导出"
3. 选择文件格式为 `.p12` (Personal Information Exchange)
4. 设置一个密码（记住这个密码，后面会用到）
5. 保存文件，例如保存为 `certificate.p12`

## 步骤 3：将证书转换为 Base64

在终端运行以下命令：

```bash
base64 -i certificate.p12 | pbcopy
```

这会将证书的 Base64 编码复制到剪贴板。

## 步骤 4：在 GitHub 中添加 Secrets

前往你的 GitHub 仓库：

1. 进入 **Settings** -> **Secrets and variables** -> **Actions**
2. 点击 **New repository secret**，添加以下三个 secrets：

### Secret 1: APPLE_CERTIFICATE
- **Name**: `APPLE_CERTIFICATE`
- **Value**: 粘贴步骤 3 中复制的 Base64 字符串

### Secret 2: APPLE_CERTIFICATE_PASSWORD
- **Name**: `APPLE_CERTIFICATE_PASSWORD`
- **Value**: 步骤 2 中设置的证书密码

### Secret 3: KEYCHAIN_PASSWORD
- **Name**: `KEYCHAIN_PASSWORD`
- **Value**: 一个随机密码（用于 CI 中创建临时钥匙串，例如：`github-actions-temp-keychain-password`）

## 步骤 5：更新 tauri.conf.json（已完成）

在 `src-tauri/tauri.conf.json` 中设置签名标识为 `"-"`（ad-hoc 签名）：

```json
{
  "bundle": {
    "macOS": {
      "signingIdentity": "-"
    }
  }
}
```

✅ 这一步已经在配置文件中完成。

## 步骤 6：触发构建

推送代码或创建新的 tag，GitHub Actions 会自动：

1. 导入证书到临时钥匙串
2. 使用证书签名应用
3. 上传签名后的应用到 Release
4. 清理临时钥匙串

## 验证签名

下载构建好的 `.dmg` 文件后，可以在终端验证签名：

```bash
codesign -dv --verbose=4 "/Applications/Bing Wallpaper Now.app"
```

应该会看到类似以下输出：

```
Identifier=top.qiyuey.wallpaper
Format=app bundle with Mach-O universal (arm64 x86_64)
CodeDirectory v=...
Signature=adhoc
```

`Signature=adhoc` 表示这是一个 ad-hoc 签名。

## 注意事项

### Ad-hoc 签名的限制

- ❌ **不能进行公证**（notarization）- 需要付费的 Apple Developer 账号
- ❌ **不能通过 App Store 分发**
- ✅ **可以减少 Gatekeeper 警告** - 用户首次打开时仍需要右键点击"打开"
- ✅ **完全免费**

### 升级到付费签名

如果你想要更好的用户体验（无需右键打开），可以：

1. 注册 Apple Developer 账号（$99/年）
2. 在 Apple Developer Portal 创建证书和 Provisioning Profile
3. 更新 `signingIdentity` 为你的团队 ID
4. 添加公证步骤到 CI

## 故障排查

### 问题：构建失败，提示找不到证书

**解决方案**：
- 检查 GitHub Secrets 是否正确设置
- 确保 `APPLE_CERTIFICATE` 是正确的 Base64 编码
- 确保 `APPLE_CERTIFICATE_PASSWORD` 与导出证书时设置的密码一致

### 问题：签名后仍然被 Gatekeeper 阻止

**解决方案**：
- Ad-hoc 签名不能完全绕过 Gatekeeper
- 用户首次打开时需要右键点击应用，选择"打开"
- 或者运行：`xattr -rd com.apple.quarantine "/Applications/Bing Wallpaper Now.app"`

## 参考资料

- [Apple Code Signing Guide](https://developer.apple.com/support/code-signing/)
- [Tauri Signing Documentation](https://v2.tauri.app/reference/config/#bundleconfig.macos)
- [GitHub Actions: Encrypted Secrets](https://docs.github.com/en/actions/security-guides/encrypted-secrets)
