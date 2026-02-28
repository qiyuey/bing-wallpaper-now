<!--
Release Body Template (GitHub Actions envsubst)
Use envsubst to replace ${VARIABLES} before creating the release.

Expected environment variables to be exported in the workflow before running envsubst:
  VERSION        -> Current tag, e.g. 0.2.3
  REPOSITORY     -> GitHub repository in owner/name form
  CHANGELOG      -> Extracted markdown changelog content
  PREVIOUS_TAG   -> Previous semantic version tag (may be empty)
  COMPARE_LINK   -> Link to compare view or commit history fallback

Example in workflow (bash):
  export VERSION="${{ github.ref_name }}"
  export REPOSITORY="${{ github.repository }}"
  export CHANGELOG="$( ...extract logic... )"
  export PREVIOUS_TAG="$( ...previous tag... )"
  if [ -n "$PREVIOUS_TAG" ]; then
    export COMPARE_LINK="[${PREVIOUS_TAG}...${VERSION}](https://github.com/${REPOSITORY}/compare/${PREVIOUS_TAG}...${VERSION})"
  else
    export COMPARE_LINK="[查看提交历史](https://github.com/${REPOSITORY}/commits/${VERSION})"
  fi
  envsubst < .github/release_body_template.md > release_body.md

Then pass release_body.md as body file to the release action (softprops/action-gh-release supports body_path).
-->

## ✨ 更新内容

${CHANGELOG}

---

## 📦 快速安装

| 平台/架构 | 安装包 |
|-----------|--------|
| Windows x64 | [msi](https://github.com/${REPOSITORY}/releases/download/${VERSION}/Bing.Wallpaper.Now_${VERSION}_x64_zh-CN.msi) / [exe](https://github.com/${REPOSITORY}/releases/download/${VERSION}/Bing.Wallpaper.Now_${VERSION}_x64-setup.exe) |
| Windows arm64 | [msi](https://github.com/${REPOSITORY}/releases/download/${VERSION}/Bing.Wallpaper.Now_${VERSION}_arm64_zh-CN.msi) / [exe](https://github.com/${REPOSITORY}/releases/download/${VERSION}/Bing.Wallpaper.Now_${VERSION}_arm64-setup.exe) |
| macOS Apple Silicon | [arm64 dmg](https://github.com/${REPOSITORY}/releases/download/${VERSION}/Bing.Wallpaper.Now_${VERSION}_aarch64.dmg) |
| macOS Intel | [x64 dmg](https://github.com/${REPOSITORY}/releases/download/${VERSION}/Bing.Wallpaper.Now_${VERSION}_x64.dmg) |
| Linux x64 | [deb](https://github.com/${REPOSITORY}/releases/download/${VERSION}/Bing.Wallpaper.Now_${VERSION}_amd64.deb) / [rpm](https://github.com/${REPOSITORY}/releases/download/${VERSION}/Bing.Wallpaper.Now-${VERSION}-1.x86_64.rpm) / [AppImage](https://github.com/${REPOSITORY}/releases/download/${VERSION}/Bing.Wallpaper.Now_${VERSION}_amd64.AppImage) |
| Linux arm64 | [deb](https://github.com/${REPOSITORY}/releases/download/${VERSION}/Bing.Wallpaper.Now_${VERSION}_arm64.deb) / [rpm](https://github.com/${REPOSITORY}/releases/download/${VERSION}/Bing.Wallpaper.Now-${VERSION}-1.aarch64.rpm) / [AppImage](https://github.com/${REPOSITORY}/releases/download/${VERSION}/Bing.Wallpaper.Now_${VERSION}_aarch64.AppImage) |

---

### macOS 通过 Homebrew 安装

```bash
brew tap qiyuey/tap
brew install --cask bing-wallpaper-now
```

### Windows 通过 WinGet 安装

```bash
winget install Qiyuey.BingWallpaperNow
```

### macOS 安装方法

若出现"应用已损坏"或"无法打开"，在终端执行（需要管理员权限时可在前面加 sudo）：
```bash
xattr -rd com.apple.quarantine "/Applications/Bing Wallpaper Now.app"
```

---

## 📄 完整更新日志

${COMPARE_LINK}

---

感谢使用 Bing Wallpaper Now！如果你喜欢这个项目，欢迎在仓库加星支持。😊

<!-- End of template -->
