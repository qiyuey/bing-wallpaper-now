# check-commit.ps1 - Windows 版本提交前本地 CI 检查
#
# 此脚本在提交前运行所有 CI 检查，确保代码能够通过 GitHub Actions CI
# 避免提交后 CI 失败的循环
#
# 使用方法:
#   .\scripts\check-commit.ps1

$ErrorActionPreference = "Stop"

# 检查计数器
$Script:ChecksPassed = 0
$Script:ChecksFailed = 0

# 打印函数
function Print-Separator {
    Write-Host ("━" * 60) -ForegroundColor Blue
}

function Print-Check {
    param([string]$Message)
    Write-Host ""
    Write-Host "🔍 $Message" -ForegroundColor Blue
    Print-Separator
}

function Print-Success {
    param([string]$Message)
    Write-Host "✅ $Message" -ForegroundColor Green
    $Script:ChecksPassed++
}

function Print-Error {
    param([string]$Message)
    Write-Host "❌ $Message" -ForegroundColor Red
    $Script:ChecksFailed++
}

function Print-Warning {
    param([string]$Message)
    Write-Host "⚠️  $Message" -ForegroundColor Yellow
}

# 检测包管理器
function Get-PackageManager {
    if (Get-Command pnpm -ErrorAction SilentlyContinue) {
        return "pnpm"
    } elseif (Get-Command npm -ErrorAction SilentlyContinue) {
        return "npm"
    } else {
        Print-Error "未找到 npm 或 pnpm"
        exit 1
    }
}

$PKG_MANAGER = Get-PackageManager

Write-Host "╔══════════════════════════════════════════════════════════════╗" -ForegroundColor Blue
Write-Host "║         Bing Wallpaper Now - 提交前检查 (Pre-Commit)        ║" -ForegroundColor Blue
Write-Host "╚══════════════════════════════════════════════════════════════╝" -ForegroundColor Blue
Write-Host ""
Write-Host "此脚本将运行所有 CI 检查，确保代码能够通过 GitHub Actions" -ForegroundColor Yellow
Write-Host "包管理器: $PKG_MANAGER" -ForegroundColor Yellow
Write-Host ""

# ============================================================================
# 1. Rust 代码格式检查
# ============================================================================
Print-Check "1/8 Rust 代码格式检查 (cargo fmt)"

try {
    cargo fmt --manifest-path src-tauri/Cargo.toml -- --check 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) {
        Print-Success "Rust 代码格式正确"
    } else {
        Print-Error "Rust 代码格式不正确，请运行: cargo fmt --manifest-path src-tauri/Cargo.toml"
    }
} catch {
    Print-Error "Rust 代码格式检查失败"
}

# ============================================================================
# 2. Rust Clippy 检查
# ============================================================================
Print-Check "2/8 Rust Clippy 检查 (cargo clippy)"

try {
    cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) {
        Print-Success "Clippy 检查通过"
    } else {
        Print-Error "Clippy 检查失败，请修复 Rust 代码问题"
    }
} catch {
    Print-Error "Clippy 检查失败"
}

# ============================================================================
# 3. Rust 测试
# ============================================================================
Print-Check "3/8 Rust 单元测试 (cargo test)"

try {
    cargo test --manifest-path src-tauri/Cargo.toml --quiet 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) {
        Print-Success "Rust 测试通过"
    } else {
        Print-Error "Rust 测试失败，请修复测试问题"
    }
} catch {
    Print-Error "Rust 测试失败"
}

# ============================================================================
# 4. TypeScript 类型检查
# ============================================================================
Print-Check "4/8 TypeScript 类型检查 (tsc)"

try {
    & $PKG_MANAGER run typecheck 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) {
        Print-Success "TypeScript 类型检查通过"
    } else {
        Print-Error "TypeScript 类型检查失败，请修复类型错误"
    }
} catch {
    Print-Error "TypeScript 类型检查失败"
}

# ============================================================================
# 5. ESLint 检查
# ============================================================================
Print-Check "5/8 ESLint 检查 (eslint)"

try {
    & $PKG_MANAGER run lint 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) {
        Print-Success "ESLint 检查通过"
    } else {
        Print-Error "ESLint 检查失败，请运行: $PKG_MANAGER run lint:fix"
    }
} catch {
    Print-Error "ESLint 检查失败"
}

# ============================================================================
# 6. Prettier 格式检查
# ============================================================================
Print-Check "6/8 Prettier 格式检查 (prettier)"

try {
    & $PKG_MANAGER run format:check 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) {
        Print-Success "Prettier 格式检查通过"
    } else {
        Print-Error "Prettier 格式检查失败，请运行: $PKG_MANAGER run format"
    }
} catch {
    Print-Error "Prettier 格式检查失败"
}

# ============================================================================
# 7. 前端测试
# ============================================================================
Print-Check "7/8 前端单元测试 (vitest)"

try {
    & $PKG_MANAGER run test:frontend 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) {
        Print-Success "前端测试通过"
    } else {
        Print-Error "前端测试失败，请修复测试问题"
    }
} catch {
    Print-Error "前端测试失败"
}

# ============================================================================
# 8. 前端构建
# ============================================================================
Print-Check "8/8 前端构建检查 (vite build)"

try {
    & $PKG_MANAGER run build 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) {
        Print-Success "前端构建成功"
    } else {
        Print-Error "前端构建失败，请修复构建问题"
    }
} catch {
    Print-Error "前端构建失败"
}

# ============================================================================
# 汇总结果
# ============================================================================
Write-Host ""
Print-Separator
Write-Host "检查汇总:" -ForegroundColor Cyan
Print-Separator

$TotalChecks = $Script:ChecksPassed + $Script:ChecksFailed
Write-Host "总检查项: $TotalChecks"
Write-Host "通过: $($Script:ChecksPassed)" -ForegroundColor Green
Write-Host "失败: $($Script:ChecksFailed)" -ForegroundColor Red

if ($Script:ChecksFailed -eq 0) {
    Write-Host ""
    Write-Host "✅ 所有检查通过！可以安全提交代码 🎉" -ForegroundColor Green
    Write-Host "建议使用以下命令提交:" -ForegroundColor Green
    Write-Host "  git add ." -ForegroundColor Blue
    Write-Host "  git commit" -ForegroundColor Blue
    Write-Host "  git push" -ForegroundColor Blue
    Write-Host ""
    exit 0
} else {
    Write-Host ""
    Write-Host "❌ 有 $($Script:ChecksFailed) 项检查失败，请修复后再提交" -ForegroundColor Red
    Write-Host "提示:" -ForegroundColor Yellow
    Write-Host "  - 格式问题: cargo fmt && $PKG_MANAGER run format" -ForegroundColor Blue
    Write-Host "  - Lint 问题: $PKG_MANAGER run lint:fix" -ForegroundColor Blue
    Write-Host "  - 类型问题: 根据 tsc 错误提示修复"
    Write-Host "  - 测试问题: 根据测试输出修复"
    Write-Host ""
    exit 1
}
