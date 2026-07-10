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
import {
  existsSync,
  copyFileSync,
  readFileSync,
  unlinkSync,
  writeFileSync,
} from "fs";
import { PNG } from "pngjs";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const rootDir = join(__dirname, "..");
const iconsDir = join(rootDir, "src-tauri", "icons");
const publicDir = join(rootDir, "public");
const svgPath = join(iconsDir, "icon.svg");
const traySvgPath = join(iconsDir, "tray-icon.svg");

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
 * 从单色 SVG 母版生成 macOS 菜单栏模板图标。
 */
function generateMacOSTrayIcons() {
  console.log("生成 macOS 菜单栏模板图标...");

  if (!existsSync(traySvgPath)) {
    throw new Error(`托盘 SVG 源文件不存在: ${traySvgPath}`);
  }

  try {
    execSync(
      `pnpm tauri icon "${traySvgPath}" -o "${iconsDir}" --png "22,44"`,
      {
        stdio: "inherit",
        cwd: rootDir,
      },
    );

    const outputs = [
      ["22x22.png", "tray-icon-macos.png"],
      ["44x44.png", "tray-icon-macos@2x.png"],
    ];

    for (const [generatedName, targetName] of outputs) {
      const generatedPath = join(iconsDir, generatedName);
      if (!existsSync(generatedPath)) {
        throw new Error(`缺少 Tauri 生成的 macOS 菜单栏图标: ${generatedPath}`);
      }
      copyFileSync(generatedPath, join(iconsDir, targetName));
      unlinkSync(generatedPath);
    }

    console.log("✓ 已生成 macOS 菜单栏模板图标 (1x, 2x)");
  } catch (error) {
    throw new Error(`macOS 菜单栏图标生成失败: ${error.message}`);
  }
}

/**
 * 从同一单色 SVG 母版生成 Windows 深浅主题托盘图标。
 */
function generateWindowsTrayIcons() {
  console.log("生成 Windows 深浅主题托盘图标...");

  if (!existsSync(traySvgPath)) {
    throw new Error(`托盘 SVG 源文件不存在: ${traySvgPath}`);
  }

  const generatedPath = join(iconsDir, "48x48.png");
  const lightThemePath = join(iconsDir, "tray-icon-windows-light.png");
  const darkThemePath = join(iconsDir, "tray-icon-windows-dark.png");

  try {
    execSync(`pnpm tauri icon "${traySvgPath}" -o "${iconsDir}" --png "48"`, {
      stdio: "inherit",
      cwd: rootDir,
    });

    if (!existsSync(generatedPath)) {
      throw new Error(`缺少 Tauri 生成的 Windows 托盘图标: ${generatedPath}`);
    }

    copyFileSync(generatedPath, lightThemePath);

    const darkThemeIcon = PNG.sync.read(readFileSync(generatedPath));
    for (let i = 0; i < darkThemeIcon.data.length; i += 4) {
      darkThemeIcon.data[i] = 255;
      darkThemeIcon.data[i + 1] = 255;
      darkThemeIcon.data[i + 2] = 255;
    }
    writeFileSync(darkThemePath, PNG.sync.write(darkThemeIcon));
    unlinkSync(generatedPath);

    console.log("✓ 已生成 Windows 深浅主题托盘图标 (48x48)");
  } catch (error) {
    if (existsSync(generatedPath)) {
      unlinkSync(generatedPath);
    }
    throw new Error(`Windows 托盘图标生成失败: ${error.message}`);
  }
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
    generateMacOSTrayIcons();
    generateWindowsTrayIcons();
    syncFrontendIcons();

    console.log("\n✓ 所有图标生成完成！");
    console.log("  - Source: icon.svg");
    console.log("  - macOS: icon.icns");
    console.log("  - Windows: icon.ico");
    console.log(
      "  - Windows tray: tray-icon-windows-light.png, tray-icon-windows-dark.png",
    );
    console.log("  - macOS tray: tray-icon-macos.png, tray-icon-macos@2x.png");
    console.log("  - Frontend: public/icon.png, public/app-icon.png");
  } catch (error) {
    console.error("\n✗ 生成图标时出错:", error);
    process.exit(1);
  }
}

main();
