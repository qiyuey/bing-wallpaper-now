# version.ps1 - SNAPSHOT Version Management Script (Windows PowerShell)
#
# Version format: X.Y.Z or X.Y.Z-SNAPSHOT
#
# Usage:
#   .\scripts\version.ps1 snapshot-patch  # Create next patch SNAPSHOT (0.1.0 -> 0.1.1-SNAPSHOT)
#   .\scripts\version.ps1 snapshot-minor  # Create next minor SNAPSHOT (0.1.0 -> 0.2.0-SNAPSHOT)
#   .\scripts\version.ps1 snapshot-major  # Create next major SNAPSHOT (0.1.0 -> 1.0.0-SNAPSHOT)
#   .\scripts\version.ps1 release         # Release current SNAPSHOT version, create tag and push to remote (0.1.1-SNAPSHOT -> 0.1.1)
#
# Workflow:
#   1. After releasing 0.1.0, create 0.1.1-SNAPSHOT for development
#   2. When development is complete, run release to convert to 0.1.1 production version, create tag and push to remote
#   3. After release, create 0.1.2-SNAPSHOT again to continue development
#
# Rollback on Release Failure:
#   If CI build fails after release, you need to rollback:
#   1. Delete local tag: git tag -d vX.Y.Z
#   2. Delete remote tag: git push origin :refs/tags/vX.Y.Z
#   3. Revert commits:
#      - Only revert release commit: git reset --hard HEAD~1
#      - Also created SNAPSHOT: git reset --hard HEAD~2
#   4. Force push: git push origin main --force-with-lease
#   5. Fix issues and rerun: make release

$ErrorActionPreference = "Stop"

# File paths
$PACKAGE_JSON = "package.json"
$CARGO_TOML = "src-tauri/Cargo.toml"
$TAURI_CONF = "src-tauri/tauri.conf.json"

# Print functions
function Print-Info {
    param([string]$Message)
    Write-Host "ℹ $Message" -ForegroundColor Blue
}

function Print-Success {
    param([string]$Message)
    Write-Host "✓ $Message" -ForegroundColor Green
}

function Print-Warning {
    param([string]$Message)
    Write-Host "⚠ $Message" -ForegroundColor Yellow
}

function Print-Error {
    param([string]$Message)
    Write-Host "✗ $Message" -ForegroundColor Red
}

function Print-Header {
    param([string]$Message)
    Write-Host $Message -ForegroundColor Cyan
}

# Check if in Git repository
function Test-GitRepo {
    try {
        git rev-parse --git-dir 2>&1 | Out-Null
        return $true
    } catch {
        Print-Error "Not in a Git repository"
        exit 1
    }
}

# Check working directory status
function Test-WorkingDirectory {
    $status = git status -s
    if ($status) {
        Print-Warning "Working directory has uncommitted changes"
        Write-Host $status
        Write-Host ""
        $reply = Read-Host "Continue anyway? (y/N)"
        if ($reply -ne "y" -and $reply -ne "Y") {
            Print-Info "Cancelled"
            exit 0
        }
    }
}

# Get current version
function Get-CurrentVersion {
    $content = Get-Content $PACKAGE_JSON -Raw | ConvertFrom-Json
    return $content.version
}

# Check if is SNAPSHOT version
function Test-IsSnapshot {
    param([string]$Version)
    return $Version -match "-SNAPSHOT$"
}

# Remove SNAPSHOT suffix
function Remove-SnapshotSuffix {
    param([string]$Version)
    return $Version -replace "-SNAPSHOT$", ""
}

# Add SNAPSHOT suffix
function Add-SnapshotSuffix {
    param([string]$Version)
    return "$Version-SNAPSHOT"
}

# Split version number
function Split-Version {
    param([string]$Version)
    $baseVersion = Remove-SnapshotSuffix $Version
    $parts = $baseVersion -split '\.'
    return @{
        Major = [int]$parts[0]
        Minor = [int]$parts[1]
        Patch = [int]$parts[2]
    }
}

# Calculate next version
function Get-NextVersion {
    param(
        [string]$Current,
        [string]$BumpType
    )

    $baseVersion = Remove-SnapshotSuffix $Current
    $parts = Split-Version $baseVersion

    switch ($BumpType) {
        "patch" {
            $parts.Patch++
        }
        "minor" {
            $parts.Minor++
            $parts.Patch = 0
        }
        "major" {
            $parts.Major++
            $parts.Minor = 0
            $parts.Patch = 0
        }
        default {
            Print-Error "Invalid version type: $BumpType"
            exit 1
        }
    }

    return "$($parts.Major).$($parts.Minor).$($parts.Patch)"
}

# Validate version format (MSI compatibility)
function Test-VersionFormat {
    param([string]$Version)

    # MSI requirement: pre-release identifier must be numeric only (e.g. 1.0.0 or 1.0.0-123)
    # Cannot contain letter suffixes (e.g. 1.0.0-alpha, 1.0.0-SNAPSHOT)
    if ($Version -match "-[^0-9]") {
        Print-Error "Version '$Version' contains non-numeric pre-release identifier"
        Print-Error "MSI build requires pre-release identifiers to be numeric only (e.g. 1.0.0 or 1.0.0-123)"
        Print-Error "Current version contains letter suffix, which will cause Windows MSI build failure"
        return $false
    }
    return $true
}

# Update all version files
function Update-VersionFiles {
    param([string]$NewVersion)

    # Validate version format
    if (-not (Test-VersionFormat $NewVersion)) {
        exit 1
    }

    Print-Info "Updating $PACKAGE_JSON..."
    $packageJson = Get-Content $PACKAGE_JSON -Raw | ConvertFrom-Json
    $packageJson.version = $NewVersion
    $packageJson | ConvertTo-Json -Depth 100 | Set-Content $PACKAGE_JSON -Encoding UTF8

    Print-Info "Updating $CARGO_TOML..."
    $cargoContent = Get-Content $CARGO_TOML -Raw
    $cargoContent = $cargoContent -replace 'version = ".*"', "version = `"$NewVersion`""
    $cargoContent | Set-Content $CARGO_TOML -NoNewline -Encoding UTF8

    Print-Info "Updating $TAURI_CONF..."
    $tauriConf = Get-Content $TAURI_CONF -Raw | ConvertFrom-Json
    $tauriConf.version = $NewVersion
    $tauriConf | ConvertTo-Json -Depth 100 | Set-Content $TAURI_CONF -Encoding UTF8

    Print-Info "Updating Cargo.lock..."
    cargo update -p bing-wallpaper-now --manifest-path src-tauri/Cargo.toml --quiet 2>&1 | Out-Null

    Print-Success "Version files updated to $NewVersion"
}

# Create SNAPSHOT version
function New-SnapshotVersion {
    param([string]$BumpType)

    $current = Get-CurrentVersion

    if (Test-IsSnapshot $current) {
        Print-Warning "Current version is already SNAPSHOT: $current"
        $base = Remove-SnapshotSuffix $current
        Print-Info "Will create new SNAPSHOT based on $base"
    }

    $baseVersion = Remove-SnapshotSuffix $current
    $nextVersion = Get-NextVersion $baseVersion $BumpType
    $snapshotVersion = Add-SnapshotSuffix $nextVersion

    Print-Header "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    Print-Header "  Create SNAPSHOT Version"
    Print-Header "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    Write-Host ""
    Print-Info "Current version: $current"
    Print-Info "New version:     $snapshotVersion"
    Write-Host ""

    $reply = Read-Host "Confirm creating SNAPSHOT version? (y/N)"
    if ($reply -ne "y" -and $reply -ne "Y") {
        Print-Info "Cancelled"
        exit 0
    }

    Update-VersionFiles $snapshotVersion

    git add $PACKAGE_JSON $CARGO_TOML $TAURI_CONF "src-tauri/Cargo.lock"
    git commit -m "chore(version): bump to $snapshotVersion"

    Print-Success "Created SNAPSHOT version: $snapshotVersion"
    Print-Info "Ready to start developing new features!"
}

# Validate CHANGELOG is updated
function Test-Changelog {
    param([string]$Version)

    if (-not (Test-Path "CHANGELOG.md")) {
        Print-Error "CHANGELOG.md file not found"
        return $false
    }

    $changelogContent = Get-Content "CHANGELOG.md" -Raw
    if ($changelogContent -notmatch "\#\# \[$Version\]") {
        Print-Error "Version [$Version] not found in CHANGELOG.md"
        Print-Info "Please add the following content to CHANGELOG.md first:"
        Write-Host ""
        Write-Host "  ## [$Version]"
        Write-Host ""
        Write-Host "  ### Added/Changed/Fixed"
        Write-Host "  - Your changelog notes..."
        Write-Host ""
        Print-Info "Then rerun make release"
        return $false
    }

    Print-Success "CHANGELOG.md validation passed"
    return $true
}

# Run pre-release quality checks
function Invoke-PreReleaseChecks {
    Print-Header "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    Print-Header "  Running Pre-release Quality Checks"
    Print-Header "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    Write-Host ""

    # Check if make command exists or use PowerShell scripts
    $hasMake = Get-Command make -ErrorAction SilentlyContinue

    if (-not $hasMake) {
        Print-Warning "make command not found, will use PowerShell scripts for checks"
        if (Test-Path "scripts\check-commit.ps1") {
            Print-Info "Running code formatting checks, linting and tests..."
            try {
                & ".\scripts\check-commit.ps1"
                if ($LASTEXITCODE -ne 0) {
                    Print-Error "Quality checks failed"
                    Print-Info "Please fix the above issues and rerun make release"
                    exit 1
                }
            } catch {
                Print-Error "Quality checks failed: $_"
                exit 1
            }
        } else {
            Print-Warning "scripts\check-commit.ps1 not found, skipping quality checks"
            Print-Warning "Recommended to run manually: .\scripts\check-commit.ps1"
            Write-Host ""
            $reply = Read-Host "Continue with release? (y/N)"
            if ($reply -ne "y" -and $reply -ne "Y") {
                Print-Info "Cancelled"
                exit 0
            }
            return
        }
    } else {
        Print-Info "Running code formatting checks, linting and tests..."
        try {
            make pre-commit
            if ($LASTEXITCODE -ne 0) {
                Print-Error "Quality checks failed"
                Print-Info "Please fix the above issues and rerun make release"
                exit 1
            }
        } catch {
            Print-Error "Quality checks failed: $_"
            exit 1
        }
    }

    Print-Success "All quality checks passed"
    Write-Host ""
}

# Release version (pushes to remote by default)
function Publish-Release {
    $current = Get-CurrentVersion

    if (-not (Test-IsSnapshot $current)) {
        Print-Error "Current version is not SNAPSHOT: $current"
        Print-Info "Can only release from SNAPSHOT version"
        exit 1
    }

    $releaseVersion = Remove-SnapshotSuffix $current
    $tag = "v$releaseVersion"

    Print-Header "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    Print-Header "  Release Production Version"
    Print-Header "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    Write-Host ""
    Print-Info "SNAPSHOT version: $current"
    Print-Info "Release version:  $releaseVersion"
    Print-Info "Git Tag:          $tag"
    Write-Host ""

    # Validate CHANGELOG
    if (-not (Test-Changelog $releaseVersion)) {
        exit 1
    }
    Write-Host ""

    # Run pre-release checks
    Invoke-PreReleaseChecks

    $reply = Read-Host "Confirm releasing version? (y/N)"
    if ($reply -ne "y" -and $reply -ne "Y") {
        Print-Info "Cancelled"
        exit 0
    }

    # Update version (remove SNAPSHOT)
    Update-VersionFiles $releaseVersion

    # Commit and tag
    git add $PACKAGE_JSON $CARGO_TOML $TAURI_CONF "src-tauri/Cargo.lock"
    git commit -m "chore(release): $releaseVersion"
    git tag -a $tag -m "Release $releaseVersion"

    Print-Success "Created release version: $releaseVersion"
    Print-Success "Created Git tag: $tag"

    Write-Host ""
    $reply = Read-Host "Push to remote immediately? (Y/n)"
    $pushed = $false
    if ($reply -ne "n" -and $reply -ne "N") {
        Print-Info "Pushing to remote..."
        git push origin main
        git push origin $tag
        Print-Success "Pushed to remote, CI will start building"
        $pushed = $true
        Write-Host ""
        Print-Info "GitHub Actions will automatically build and publish to Releases"
    } else {
        Print-Info "Skipped push, manually push later:"
        Write-Host "  git push origin main && git push origin $tag"
    }

    # Ask if create next SNAPSHOT version
    Write-Host ""
    $reply = Read-Host "Create next patch SNAPSHOT version? (y/N)"
    if ($reply -eq "y" -or $reply -eq "Y") {
        Write-Host ""
        Print-Info "Creating next SNAPSHOT version..."

        $nextVersion = Get-NextVersion $releaseVersion "patch"
        $snapshotVersion = Add-SnapshotSuffix $nextVersion

        Update-VersionFiles $snapshotVersion

        git add $PACKAGE_JSON $CARGO_TOML $TAURI_CONF "src-tauri/Cargo.lock"
        git commit -m "chore(version): bump to $snapshotVersion"

        Print-Success "Created SNAPSHOT version: $snapshotVersion"

        if ($pushed) {
            Write-Host ""
            $reply = Read-Host "Push SNAPSHOT version to remote? (y/N)"
            if ($reply -eq "y" -or $reply -eq "Y") {
                git push origin main
                Print-Success "Pushed SNAPSHOT version to remote"
            } else {
                Print-Info "Push manually later: git push origin main"
            }
        }

        Write-Host ""
        Print-Success "Ready to start developing new features!"
    } else {
        Write-Host ""
        Print-Info "Create SNAPSHOT version manually later: make snapshot-patch"
    }
}

# Show current version information
function Show-VersionInfo {
    $current = Get-CurrentVersion

    Print-Header "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    Print-Header "  Version Information"
    Print-Header "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    Write-Host ""
    Print-Info "Current version: $current"

    if (Test-IsSnapshot $current) {
        $release = Remove-SnapshotSuffix $current
        Print-Info "Type:            SNAPSHOT (development version)"
        Print-Info "Release version: $release (when released)"
    } else {
        Print-Info "Type:            Release (production version)"
        Print-Warning "Recommend creating next SNAPSHOT version to continue development"
    }

    Write-Host ""
    Print-Info "Recent Git tags:"
    git tag --sort=-v:refname | Select-Object -First 3
    Write-Host ""
}

# Main function
function Main {
    param([string]$Command)

    Test-GitRepo

    if (-not $Command) {
        Show-VersionInfo
        Write-Host ""
        Print-Info "Usage:"
        Write-Host "  .\scripts\version.ps1 snapshot-patch  # Create next patch SNAPSHOT"
        Write-Host "  .\scripts\version.ps1 snapshot-minor  # Create next minor SNAPSHOT"
        Write-Host "  .\scripts\version.ps1 snapshot-major  # Create next major SNAPSHOT"
        Write-Host "  .\scripts\version.ps1 release         # Release current SNAPSHOT version, create tag and push to remote"
        Write-Host ""
        exit 0
    }

    Test-WorkingDirectory

    switch ($Command) {
        "snapshot-patch" {
            New-SnapshotVersion "patch"
        }
        "snapshot-minor" {
            New-SnapshotVersion "minor"
        }
        "snapshot-major" {
            New-SnapshotVersion "major"
        }
        "release" {
            Publish-Release
        }
        "info" {
            Show-VersionInfo
        }
        default {
            Print-Error "Unknown command: $Command"
            Print-Info "Usage: .\scripts\version.ps1 <snapshot-patch|snapshot-minor|snapshot-major|release|info>"
            exit 1
        }
    }
}

# Execute main function
Main $args[0]
