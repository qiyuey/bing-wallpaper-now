## 更新内容

${CHANGELOG}

---

## 快速安装

| 平台/架构           | 安装包                                                                                                                                                                                                                                                                                                                                                     |
| ------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Windows x64         | [MSI](https://github.com/${REPOSITORY}/releases/download/${VERSION}/Bing.Wallpaper.Now_${VERSION}_x64_zh-CN.msi)                                                                                                                                                                                                                                           |
| Windows arm64       | [MSI](https://github.com/${REPOSITORY}/releases/download/${VERSION}/Bing.Wallpaper.Now_${VERSION}_arm64_zh-CN.msi)                                                                                                                                                                                                                                         |
| macOS Apple Silicon | [arm64 dmg](https://github.com/${REPOSITORY}/releases/download/${VERSION}/Bing.Wallpaper.Now_${VERSION}_aarch64.dmg)                                                                                                                                                                                                                                       |

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

## 完整更新日志

${COMPARE_LINK}

---

感谢使用 Bing Wallpaper Now！如果你喜欢这个项目，欢迎在仓库加星支持。
