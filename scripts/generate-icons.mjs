#!/usr/bin/env node
/**
 * 图标生成脚本
 * 从 SVG 源文件生成 macOS、Windows、Linux 所需的图标文件
 * 
 * 使用方法：
 *   node scripts/generate-icons.mjs
 * 
 * 依赖：
 *   - @tauri-apps/cli: 用于生成主要图标（icns, ico, png）
 * 
 * 说明：
 *   - macOS 使用 icon.svg（带 0.75 缩放）
 *   - Windows 使用 icon-windows.svg（无缩放，完整大小）
 */

import { execSync } from 'child_process';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
import { existsSync, copyFileSync, unlinkSync } from 'fs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const rootDir = join(__dirname, '..');
const iconsDir = join(rootDir, 'src-tauri', 'icons');
const svgPath = join(iconsDir, 'icon.svg');
const svgWindowsPath = join(iconsDir, 'icon-windows.svg');

/**
 * 使用 Tauri CLI 生成 Windows 图标（.ico）
 */
function generateWindowsIcons() {
  console.log('生成 Windows 图标（使用完整大小 SVG）...');
  
  if (!existsSync(svgWindowsPath)) {
    throw new Error(`Windows SVG 文件不存在: ${svgWindowsPath}`);
  }
  
  try {
    execSync(`pnpm tauri icon "${svgWindowsPath}" -o "${iconsDir}"`, {
      stdio: 'inherit',
      cwd: rootDir
    });
    
    // 从生成的 128x128.png 创建 Windows 托盘图标
    const png128Path = join(iconsDir, '128x128.png');
    const trayIconWindowsPath = join(iconsDir, 'tray-icon-windows.png');
    if (existsSync(png128Path)) {
      copyFileSync(png128Path, trayIconWindowsPath);
      console.log('✓ 已生成 Windows 托盘图标 (tray-icon-windows.png)');
    }
    
    console.log('✓ 已生成 Windows 图标 (.ico)');
  } catch (error) {
    throw new Error(`Windows 图标生成失败: ${error.message}`);
  }
}

/**
 * 使用 Tauri CLI 生成 macOS 图标（.icns）
 */
function generateMacOSIcons() {
  console.log('生成 macOS 图标（使用缩放 SVG）...');
  
  if (!existsSync(svgPath)) {
    throw new Error(`macOS SVG 文件不存在: ${svgPath}`);
  }
  
  try {
    execSync(`pnpm tauri icon "${svgPath}" -o "${iconsDir}"`, {
      stdio: 'inherit',
      cwd: rootDir
    });
    console.log('✓ 已生成 macOS 图标 (.icns)');
  } catch (error) {
    throw new Error(`macOS 图标生成失败: ${error.message}`);
  }
}

/**
 * 主函数
 */
function main() {
  console.log('开始生成图标...\n');
  
  const icoPath = join(iconsDir, 'icon.ico');
  const icoBackupPath = join(iconsDir, 'icon.ico.backup');
  
  try {
    // 先生成 Windows 图标（.ico）
    generateWindowsIcons();
    
    // 备份 Windows 的 .ico 文件
    if (existsSync(icoPath)) {
      copyFileSync(icoPath, icoBackupPath);
      console.log('✓ 已备份 Windows 图标文件');
    }
    
    // 再生成 macOS 图标（.icns）
    // `tauri icon` 不支持只生成特定格式，每次都会覆盖所有输出文件，
    // 因此需要先备份 Windows .ico 再恢复。
    generateMacOSIcons();
    
    // 恢复 Windows 的 .ico 文件
    if (existsSync(icoBackupPath)) {
      copyFileSync(icoBackupPath, icoPath);
      unlinkSync(icoBackupPath);
      console.log('✓ 已恢复 Windows 图标文件');
    }
    
    console.log('\n✓ 所有图标生成完成！');
    console.log('  - Windows: icon.ico (完整大小)');
    console.log('  - Windows 托盘: tray-icon-windows.png (完整大小)');
    console.log('  - macOS: icon.icns (0.75 缩放)');
    console.log('  - macOS 托盘: tray-icon-macos@2x.png (保持不变)');
    console.log('  - PNG 文件: 使用 macOS 版本生成');
  } catch (error) {
    // 如果出错，尝试恢复备份
    if (existsSync(icoBackupPath) && !existsSync(icoPath)) {
      copyFileSync(icoBackupPath, icoPath);
      unlinkSync(icoBackupPath);
    }
    console.error('\n✗ 生成图标时出错:', error);
    process.exit(1);
  }
}

main();

