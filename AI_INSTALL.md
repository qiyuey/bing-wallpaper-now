# AI Agent Install Guide / AI 安装指南

> Designed for AI coding/CLI assistants. Every command is copy-paste runnable.
> 本文档为 AI 编程/CLI 助手设计，所有命令均可直接复制执行。
>
> Human users → see [README.md](README.md) / [README.zh.md](README.zh.md).

## Project Identity

| Field | Value |
| --- | --- |
| Name | Bing Wallpaper Now |
| Repository | <https://github.com/qiyuey/bing-wallpaper-now> |
| WinGet ID | `Qiyuey.BingWallpaperNow` |
| Homebrew Cask | `qiyuey/tap/bing-wallpaper-now` |
| macOS Bundle ID | `top.qiyuey.wallpaper` |
| Latest version API | `https://api.github.com/repos/qiyuey/bing-wallpaper-now/releases/latest` |

## TL;DR — One-liners by platform

| Target | Command |
| --- | --- |
| macOS Apple Silicon | `brew install --cask qiyuey/tap/bing-wallpaper-now` |
| Windows (x64 / ARM64) | `winget install --exact --id Qiyuey.BingWallpaperNow` |
| Debian / Ubuntu | use the `.deb` direct-download block below |
| Fedora / RHEL | use the `.rpm` direct-download block below |
| Generic Linux | use the `.AppImage` direct-download block below |

If a package manager is unavailable on the target system, fall through to the direct-download path in the next sections.

## Package manager path (preferred)

### macOS via Homebrew

```bash
brew install --cask qiyuey/tap/bing-wallpaper-now
# verify
ls -d "/Applications/Bing Wallpaper Now.app"
```

### Windows via WinGet

```powershell
winget install --exact --id Qiyuey.BingWallpaperNow
# verify
winget list --id Qiyuey.BingWallpaperNow
```

WinGet auto-selects the matching architecture when both x64 and ARM64
manifests are published. To force one: `--architecture x64` or
`--architecture arm64`.

## Direct download path

All asset names follow these patterns (where `<TAG>` is the release tag, e.g. `1.4.4`):

| OS / Format | Filename pattern |
| --- | --- |
| macOS DMG | `Bing.Wallpaper.Now_<TAG>_aarch64.dmg` (Apple Silicon) |
| Windows MSI | `Bing.Wallpaper.Now_<TAG>_x64_zh-CN.msi` / `_arm64_zh-CN.msi` |
| Windows EXE | `Bing.Wallpaper.Now_<TAG>_x64-setup.exe` / `_arm64-setup.exe` |
| Debian/Ubuntu | `Bing.Wallpaper.Now_<TAG>_amd64.deb` / `_arm64.deb` |
| Fedora/RHEL | `Bing.Wallpaper.Now-<TAG>-1.x86_64.rpm` / `-1.aarch64.rpm` |
| AppImage | `Bing.Wallpaper.Now_<TAG>_amd64.AppImage` / `_aarch64.AppImage` |

Common preamble that resolves the latest version and the host architecture (POSIX shells; requires `curl` + `jq`):

```bash
LATEST=$(curl -fsSL https://api.github.com/repos/qiyuey/bing-wallpaper-now/releases/latest | jq -r .tag_name)
BASE="https://github.com/qiyuey/bing-wallpaper-now/releases/download/${LATEST}"
ARCH="$(uname -m)"
case "$ARCH" in
  x86_64)            DEB_ARCH=amd64;   RPM_ARCH=x86_64;   APPIMG_ARCH=amd64;    DMG_ARCH=unsupported ;;
  aarch64|arm64)     DEB_ARCH=arm64;   RPM_ARCH=aarch64;  APPIMG_ARCH=aarch64;  DMG_ARCH=aarch64 ;;
  *) echo "Unsupported arch: $(uname -m)"; exit 1 ;;
esac
```

### macOS DMG (fallback when Homebrew unavailable)

```bash
if [ "$DMG_ARCH" = "unsupported" ]; then
  echo "Current macOS releases require Apple Silicon"
  exit 1
fi
curl -fsSLO "${BASE}/Bing.Wallpaper.Now_${LATEST}_${DMG_ARCH}.dmg"
hdiutil attach "Bing.Wallpaper.Now_${LATEST}_${DMG_ARCH}.dmg"
cp -R "/Volumes/Bing Wallpaper Now/Bing Wallpaper Now.app" /Applications/
hdiutil detach "/Volumes/Bing Wallpaper Now"
sudo xattr -rd com.apple.quarantine "/Applications/Bing Wallpaper Now.app"
```

### Debian / Ubuntu (.deb)

```bash
curl -fsSLO "${BASE}/Bing.Wallpaper.Now_${LATEST}_${DEB_ARCH}.deb"
sudo apt install -y "./Bing.Wallpaper.Now_${LATEST}_${DEB_ARCH}.deb"
```

### Fedora / RHEL (.rpm)

```bash
curl -fsSLO "${BASE}/Bing.Wallpaper.Now-${LATEST}-1.${RPM_ARCH}.rpm"
sudo dnf install -y "./Bing.Wallpaper.Now-${LATEST}-1.${RPM_ARCH}.rpm"
```

### Generic Linux (.AppImage)

```bash
curl -fsSLO "${BASE}/Bing.Wallpaper.Now_${LATEST}_${APPIMG_ARCH}.AppImage"
chmod +x "Bing.Wallpaper.Now_${LATEST}_${APPIMG_ARCH}.AppImage"
./"Bing.Wallpaper.Now_${LATEST}_${APPIMG_ARCH}.AppImage"
```

If AppImage fails with a FUSE error on Ubuntu 22.04+:

```bash
sudo apt install -y libfuse2
# or extract & run without FUSE:
./Bing.Wallpaper.Now_*.AppImage --appimage-extract
./squashfs-root/AppRun
```

## Verify install

| Target | Verify command | Expected |
| --- | --- | --- |
| macOS | `ls -d "/Applications/Bing Wallpaper Now.app"` | path printed |
| Windows | `winget list --id Qiyuey.BingWallpaperNow` | row with version |
| Debian/Ubuntu | `dpkg -l \| grep -i bing-wallpaper-now` | non-empty row |
| Fedora/RHEL | `rpm -q bing-wallpaper-now` | version string |
| AppImage | binary launches; no installer registry entry | n/a |

## Uninstall

| Target | Command |
| --- | --- |
| macOS (Homebrew) | `brew uninstall --cask bing-wallpaper-now` |
| macOS (manual) | `rm -rf "/Applications/Bing Wallpaper Now.app"` |
| Windows | `winget uninstall --id Qiyuey.BingWallpaperNow` |
| Debian/Ubuntu | `sudo apt remove -y bing-wallpaper-now` |
| Fedora/RHEL | `sudo dnf remove -y bing-wallpaper-now` |
| AppImage | delete the `.AppImage` file |

User data (settings, runtime state, downloaded wallpapers) is **not** removed by uninstall. Locations:

| OS | Settings & runtime state | Wallpapers (default) | Logs |
| --- | --- | --- | --- |
| macOS | `~/Library/Application Support/top.qiyuey.wallpaper/` | `~/Pictures/Bing Wallpaper Now/` | `~/Library/Logs/top.qiyuey.wallpaper/` |
| Windows | `%APPDATA%\top.qiyuey.wallpaper\` | `%USERPROFILE%\Pictures\Bing Wallpaper Now\` | `%LOCALAPPDATA%\top.qiyuey.wallpaper\logs\` |
| Linux | `~/.config/top.qiyuey.wallpaper/` | `~/Pictures/Bing Wallpaper Now/` | `~/.local/share/top.qiyuey.wallpaper/logs/` |

## Common pitfalls

- **macOS "App is damaged or cannot be opened"** — Gatekeeper quarantine. Fix:
  `sudo xattr -rd com.apple.quarantine "/Applications/Bing Wallpaper Now.app"`
- **Windows: wrong architecture installed** — verify host arch with PowerShell
  `$env:PROCESSOR_ARCHITECTURE` (returns `ARM64` on ARM), then re-install with
  `--architecture <x64|arm64>`.
- **Linux AppImage: FUSE not available** — install `libfuse2`, or extract with
  `--appimage-extract` (see direct-download section).
- **No Dock / Taskbar icon after launch** — by design. The app runs in the
  system tray / menu bar. Click the tray icon to open the main window.

## After install — first run expectations

- The app runs as a tray-only / menu-bar app (no Dock or Taskbar icon).
- First launch downloads today's Bing UHD wallpaper (1–3 MB) and applies it to all displays.
- Open the main window via the tray icon → Settings.
- Auto-update checks on startup and every 1 hour; failed days enter catchup mode (15 / 30 / 60 minute backoff).
