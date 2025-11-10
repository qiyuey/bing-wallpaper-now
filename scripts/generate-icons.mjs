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
 */

import { execSync } from 'child_process';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const rootDir = join(__dirname, '..');
const iconsDir = join(rootDir, 'src-tauri', 'icons');
const svgPath = join(iconsDir, 'icon.svg');

/**
 * 使用 Tauri CLI 生成主要图标（icns, ico, png）
 */
function generateMainIcons() {
  console.log('使用 Tauri CLI 生成主要图标...');
  
  try {
    execSync(`pnpm tauri icon "${svgPath}" -o "${iconsDir}"`, {
      stdio: 'inherit',
      cwd: rootDir
    });
    console.log('✓ 已使用 Tauri CLI 生成主要图标');
  } catch (error) {
    throw new Error(`Tauri CLI 图标生成失败: ${error.message}`);
  }
}

/**
 * 主函数
 */
function main() {
  console.log('开始生成图标...\n');
  
  try {
    // 使用 Tauri CLI 生成主要图标（icns, ico, png）
    generateMainIcons();
    
    console.log('\n✓ 所有图标生成完成！');
  } catch (error) {
    console.error('\n✗ 生成图标时出错:', error);
    process.exit(1);
  }
}

main();

