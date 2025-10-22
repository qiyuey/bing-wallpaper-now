# check-commit.ps1 - Windows Pre-Commit Local CI Checks
#
# This script runs all CI checks before commit to ensure code passes GitHub Actions CI
# Avoids the cycle of failed CI after commit
#
# Usage:
#   .\scripts\check-commit.ps1

$ErrorActionPreference = "Stop"

# Check counters
$Script:ChecksPassed = 0
$Script:ChecksFailed = 0

# Print functions
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

# Detect package manager
function Get-PackageManager {
    if (Get-Command pnpm -ErrorAction SilentlyContinue) {
        return "pnpm"
    } elseif (Get-Command npm -ErrorAction SilentlyContinue) {
        return "npm"
    } else {
        Print-Error "npm or pnpm not found"
        exit 1
    }
}

$PKG_MANAGER = Get-PackageManager

Write-Host "╔══════════════════════════════════════════════════════════════╗" -ForegroundColor Blue
Write-Host "║        Bing Wallpaper Now - Pre-Commit Checks               ║" -ForegroundColor Blue
Write-Host "╚══════════════════════════════════════════════════════════════╝" -ForegroundColor Blue
Write-Host ""
Write-Host "This script runs all CI checks to ensure code passes GitHub Actions" -ForegroundColor Yellow
Write-Host "Package Manager: $PKG_MANAGER" -ForegroundColor Yellow
Write-Host ""

# ============================================================================
# 1. Rust Code Format Check
# ============================================================================
Print-Check "1/8 Rust Code Format Check (cargo fmt)"

try {
    cargo fmt --manifest-path src-tauri/Cargo.toml -- --check 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) {
        Print-Success "Rust code format correct"
    } else {
        Print-Error "Rust code format incorrect, please run: cargo fmt --manifest-path src-tauri/Cargo.toml"
    }
} catch {
    Print-Error "Rust code format check failed"
}

# ============================================================================
# 2. Rust Clippy Check
# ============================================================================
Print-Check "2/8 Rust Clippy Check (cargo clippy)"

try {
    cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) {
        Print-Success "Clippy check passed"
    } else {
        Print-Error "Clippy check failed, please fix Rust code issues"
    }
} catch {
    Print-Error "Clippy check failed"
}

# ============================================================================
# 3. Rust Tests
# ============================================================================
Print-Check "3/8 Rust Unit Tests (cargo test)"

try {
    cargo test --manifest-path src-tauri/Cargo.toml --quiet 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) {
        Print-Success "Rust tests passed"
    } else {
        Print-Error "Rust tests failed, please fix test issues"
    }
} catch {
    Print-Error "Rust tests failed"
}

# ============================================================================
# 4. TypeScript Type Check
# ============================================================================
Print-Check "4/8 TypeScript Type Check (tsc)"

try {
    & $PKG_MANAGER run typecheck 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) {
        Print-Success "TypeScript type check passed"
    } else {
        Print-Error "TypeScript type check failed, please fix type errors"
    }
} catch {
    Print-Error "TypeScript type check failed"
}

# ============================================================================
# 5. ESLint Check
# ============================================================================
Print-Check "5/8 ESLint Check (eslint)"

try {
    & $PKG_MANAGER run lint 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) {
        Print-Success "ESLint check passed"
    } else {
        Print-Error "ESLint check failed, please run: $PKG_MANAGER run lint:fix"
    }
} catch {
    Print-Error "ESLint check failed"
}

# ============================================================================
# 6. Prettier Format Check
# ============================================================================
Print-Check "6/8 Prettier Format Check (prettier)"

try {
    & $PKG_MANAGER run format:check 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) {
        Print-Success "Prettier format check passed"
    } else {
        Print-Error "Prettier format check failed, please run: $PKG_MANAGER run format"
    }
} catch {
    Print-Error "Prettier format check failed"
}

# ============================================================================
# 7. Frontend Tests
# ============================================================================
Print-Check "7/8 Frontend Unit Tests (vitest)"

try {
    & $PKG_MANAGER run test:frontend 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) {
        Print-Success "Frontend tests passed"
    } else {
        Print-Error "Frontend tests failed, please fix test issues"
    }
} catch {
    Print-Error "Frontend tests failed"
}

# ============================================================================
# 8. Frontend Build
# ============================================================================
Print-Check "8/8 Frontend Build Check (vite build)"

try {
    & $PKG_MANAGER run build 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) {
        Print-Success "Frontend build succeeded"
    } else {
        Print-Error "Frontend build failed, please fix build issues"
    }
} catch {
    Print-Error "Frontend build failed"
}

# ============================================================================
# Summary Results
# ============================================================================
Write-Host ""
Print-Separator
Write-Host "Check Summary:" -ForegroundColor Cyan
Print-Separator

$TotalChecks = $Script:ChecksPassed + $Script:ChecksFailed
Write-Host "Total Checks: $TotalChecks"
Write-Host "Passed: $($Script:ChecksPassed)" -ForegroundColor Green
Write-Host "Failed: $($Script:ChecksFailed)" -ForegroundColor Red

if ($Script:ChecksFailed -eq 0) {
    Write-Host ""
    Write-Host "✅ All checks passed! Safe to commit 🎉" -ForegroundColor Green
    Write-Host "Suggested commit commands:" -ForegroundColor Green
    Write-Host "  git add ." -ForegroundColor Blue
    Write-Host "  git commit" -ForegroundColor Blue
    Write-Host "  git push" -ForegroundColor Blue
    Write-Host ""
    exit 0
} else {
    Write-Host ""
    Write-Host "❌ $($Script:ChecksFailed) check(s) failed, please fix before commit" -ForegroundColor Red
    Write-Host "Tips:" -ForegroundColor Yellow
    Write-Host "  - Format issues: cargo fmt && $PKG_MANAGER run format" -ForegroundColor Blue
    Write-Host "  - Lint issues: $PKG_MANAGER run lint:fix" -ForegroundColor Blue
    Write-Host "  - Type issues: Fix according to tsc error messages"
    Write-Host "  - Test issues: Fix according to test output"
    Write-Host ""
    exit 1
}
