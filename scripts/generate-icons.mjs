#!/usr/bin/env node
/**
 * 图标生成脚本
 * 使用 Tauri CLI 从单一 SVG 源文件生成各平台图标。
 *
 * 使用方法：
 *   node scripts/generate-icons.mjs
 *
 * 依赖：
 *   - @tauri-apps/cli
 */

import { execSync } from "child_process";
import { join, dirname } from "path";
import { fileURLToPath } from "url";
import { existsSync, copyFileSync } from "fs";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const rootDir = join(__dirname, "..");
const iconsDir = join(rootDir, "src-tauri", "icons");
const publicDir = join(rootDir, "public");
const svgPath = join(iconsDir, "icon.svg");

/**
 * 使用 Tauri CLI 生成标准图标集。
 */
function generateIcons() {
  console.log("使用 Tauri CLI 生成标准图标集...");

  if (!existsSync(svgPath)) {
    throw new Error(`SVG 源文件不存在: ${svgPath}`);
  }

  try {
    execSync(`pnpm tauri icon "${svgPath}" -o "${iconsDir}"`, {
      stdio: "inherit",
      cwd: rootDir,
    });
    console.log("✓ 已生成标准图标集");
  } catch (error) {
    throw new Error(`图标生成失败: ${error.message}`);
  }
}

/**
 * 从标准图标集中同步项目自定义的 Windows 托盘图标。
 */
function syncWindowsTrayIcon() {
  const png128Path = join(iconsDir, "128x128.png");
  const trayIconWindowsPath = join(iconsDir, "tray-icon-windows.png");

  if (!existsSync(png128Path)) {
    throw new Error(`无法同步 Windows 托盘图标，缺少: ${png128Path}`);
  }

  copyFileSync(png128Path, trayIconWindowsPath);
  console.log("✓ 已同步 Windows 托盘图标 (tray-icon-windows.png)");
}

/**
 * 同步前端静态图标，确保浏览器预览与桌面应用图标一致。
 */
function syncFrontendIcons() {
  const png128Path = join(iconsDir, "128x128.png");
  const png512Path = join(iconsDir, "icon.png");
  const publicIconPath = join(publicDir, "icon.png");
  const publicAppIconPath = join(publicDir, "app-icon.png");

  if (!existsSync(png128Path) || !existsSync(png512Path)) {
    throw new Error("无法同步前端图标，缺少 Tauri 生成的 PNG 产物");
  }

  copyFileSync(png128Path, publicIconPath);
  copyFileSync(png512Path, publicAppIconPath);
  console.log("✓ 已同步前端静态图标 (public/icon.png, public/app-icon.png)");
}

/**
 * 主函数
 */
function main() {
  console.log("开始生成图标...\n");

  try {
    generateIcons();
    syncWindowsTrayIcon();
    syncFrontendIcons();

    console.log("\n✓ 所有图标生成完成！");
    console.log("  - Source: icon.svg");
    console.log("  - macOS: icon.icns");
    console.log("  - Windows: icon.ico");
    console.log("  - Windows tray: tray-icon-windows.png");
    console.log("  - Frontend: public/icon.png, public/app-icon.png");
  } catch (error) {
    console.error("\n✗ 生成图标时出错:", error);
    process.exit(1);
  }
}

main();
