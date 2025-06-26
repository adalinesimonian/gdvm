#!/usr/bin/env pwsh
#requires -Version 7.0

<#
.SYNOPSIS
    Cuts a new release for gdvm.

.DESCRIPTION
    This script automates the release process for gdvm by:

    1. Validating the current git state.
    2. Updating version numbers.
    3. Updating the changelog.
    4. Creating a git commit and tag.
    5. Pushing to origin.

.PARAMETER Version
    The version to release in the format Major.Minor.Patch, for example 1.2.3

.EXAMPLE
    ./scripts/cut-release.ps1 1.2.3
#>

param(
    [Parameter(Mandatory = $true, Position = 0)]
    [string]$Version
)

# Ensure the current directory is the repository root.
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$RepoRoot = Split-Path -Parent $ScriptDir
Set-Location $RepoRoot

# Exits with an error message and exit code.
function Exit-WithError {
    param([string]$Message, [int]$ExitCode = 1)
    Write-Host "Error: $Message" -ForegroundColor Red
    exit $ExitCode
}

# Validates if a string is a valid semantic version.
function Test-SemanticVersion {
    param([string]$VersionString)
    return $VersionString -match '^\d+\.\d+\.\d+$'
}

# Compares two semantic versions.
function Compare-SemanticVersion {
    param([string]$Version1, [string]$Version2)

    $v1Parts = $Version1.Split('.') | ForEach-Object { [int]$_ }
    $v2Parts = $Version2.Split('.') | ForEach-Object { [int]$_ }

    for ($i = 0; $i -lt 3; $i++) {
        if ($v1Parts[$i] -gt $v2Parts[$i]) { return 1 }
        if ($v1Parts[$i] -lt $v2Parts[$i]) { return -1 }
    }
    return 0
}

# Validate version format.
if (-not (Test-SemanticVersion $Version)) {
    Exit-WithError "Version must be in the format Major.Minor.Patch (e.g., 1.2.3)"
}

Write-Host "Cutting release for version $Version..." -ForegroundColor Green

# Check that current branch is main.
$CurrentBranch = git rev-parse --abbrev-ref HEAD
if ($CurrentBranch -ne "main") {
    Exit-WithError "Current branch is '$CurrentBranch', but must be 'main'"
}

# Check for staged or unstaged changes, ignoring untracked files and CHANGELOG.md.
$StagedChanges = git diff --cached --name-only | Where-Object { $_ -ne "CHANGELOG.md" }
$UnstagedChanges = git diff --name-only | Where-Object { $_ -ne "CHANGELOG.md" }

if ($StagedChanges -or $UnstagedChanges) {
    Exit-WithError "There are staged or unstaged changes. Please commit or stash them first."
}

# Check that current branch is in sync with origin.
git fetch origin main
$LocalCommit = git rev-parse HEAD
$RemoteCommit = git rev-parse origin/main

if ($LocalCommit -ne $RemoteCommit) {
    Exit-WithError "Local branch is not in sync with origin/main. Please pull or push as needed."
}

# Check that CHANGELOG.md has content in the unreleased section.
$ChangelogContent = Get-Content "CHANGELOG.md" -Raw
$UnreleasedSection = [regex]::Match($ChangelogContent, '(?s)## Unreleased\s*\n(.*?)\n## ')
if (-not $UnreleasedSection.Success) {
    Exit-WithError "Could not find Unreleased section in CHANGELOG.md."
}

$UnreleasedContent = $UnreleasedSection.Groups[1].Value.Trim()
$NonEmptyLines = $UnreleasedContent -split '\n' | Where-Object { $_.Trim() -and $_ -notmatch '^\*\*Full Changelog\*\*:' }

if (-not $NonEmptyLines) {
    Exit-WithError "Unreleased section in CHANGELOG.md is empty."
}

# Check that new version is higher than current version.
$CargoTomlPath = "crates/gdvm/Cargo.toml"
$CargoContent = Get-Content $CargoTomlPath -Raw
$CurrentVersionMatch = [regex]::Match($CargoContent, 'version\s*=\s*"([^"]+)"')
if (-not $CurrentVersionMatch.Success) {
    Exit-WithError "Could not find version in '$CargoTomlPath'."
}

$CurrentVersion = $CurrentVersionMatch.Groups[1].Value
$VersionComparison = Compare-SemanticVersion $Version $CurrentVersion
$UpdateCargoFiles = $true

if ($VersionComparison -lt 0) {
    Exit-WithError "New version '$Version' cannot be lower than current version '$CurrentVersion'."
}
elseif ($VersionComparison -eq 0) {
    # Same version, check if it already exists in the changelog.
    $ExistingVersionPattern = "## v$([regex]::Escape($Version))\s"
    if ($ChangelogContent -match $ExistingVersionPattern) {
        Exit-WithError "Version '$Version' already exists in CHANGELOG.md. This version appears to have been released already."
    }

    Write-Host "Version '$Version' matches current Cargo.toml version. Assuming manual version bump - skipping Cargo file updates." -ForegroundColor Yellow
    $UpdateCargoFiles = $false
}
else {
    Write-Host "Version validation passed. Current: $CurrentVersion, New: $Version" -ForegroundColor Green
}

# Update CHANGELOG.md.
Write-Host "Updating CHANGELOG.md..." -ForegroundColor Yellow

# Extract the unreleased content, excluding the "Full Changelog" line.
$UnreleasedLines = $UnreleasedContent -split '\n' | Where-Object { $_ -notmatch '^\*\*Full Changelog\*\*:' }
$UnreleasedText = ($UnreleasedLines | Where-Object { $_.Trim() }) -join "`n"

# Create new version section.
$NewVersionSection = @"
## v$Version

$UnreleasedText

**Full Changelog**: https://github.com/adalinesimonian/gdvm/compare/v$CurrentVersion...v$Version
"@

# Create new unreleased section.
$NewUnreleasedSection = @"
## Unreleased

**Full Changelog**: https://github.com/adalinesimonian/gdvm/compare/v$Version...main
"@

# Create a stash to save the current state before making changes.
Write-Host "Saving current git state..." -ForegroundColor Yellow
$StashName = "gdvm-release-script-backup-$(Get-Date -Format 'yyyyMMdd-HHmmss')"
$StashResult = git stash push -u -m "$StashName" 2>&1
$HasStash = $LASTEXITCODE -eq 0 -and $StashResult -notmatch "No local changes to save"

if ($UpdateCargoFiles) {
    # Update Cargo.toml with the new version.
    $NewCargoContent = $CargoContent -replace 'version\s*=\s*"[^"]+"', "version = `"$Version`""
    Set-Content -Path $CargoTomlPath -Value $NewCargoContent -NoNewline

    # Update Cargo.lock.
    Write-Host "Updating Cargo.lock..." -ForegroundColor Yellow
    $CargoUpdateResult = cargo update -p gdvm 2>&1
    if ($LASTEXITCODE -ne 0) {
        Exit-WithError "Failed to update Cargo.lock: $CargoUpdateResult"
    }
}

# Replace the changelog content.
$NewChangelogContent = $ChangelogContent -replace '(?s)## Unreleased.*?(?=\n## )', "$NewUnreleasedSection`n`n$NewVersionSection`n"
Set-Content -Path "CHANGELOG.md" -Value $NewChangelogContent -NoNewline

# Stage changes and show diff.
if ($UpdateCargoFiles) {
    git add $CargoTomlPath "Cargo.lock" "CHANGELOG.md"
} else {
    git add "CHANGELOG.md"
}

Write-Host "`nHere are the changes that will be committed:" -ForegroundColor Yellow
git diff --cached

# Prompt user for confirmation.
Write-Host "`nDo you want to proceed with the release? (y/N): " -ForegroundColor Yellow -NoNewline
$UserInput = Read-Host

if ($UserInput -notmatch '^[Yy]$') {
    Write-Host "Release cancelled. Restoring previous state..." -ForegroundColor Yellow

    # Reset the staged changes.
    if ($UpdateCargoFiles) {
        git reset HEAD $CargoTomlPath "Cargo.lock" "CHANGELOG.md" 2>&1 | Out-Null
        git checkout HEAD -- $CargoTomlPath "Cargo.lock" "CHANGELOG.md" 2>&1 | Out-Null
    } else {
        git reset HEAD "CHANGELOG.md" 2>&1 | Out-Null
        git checkout HEAD -- "CHANGELOG.md" 2>&1 | Out-Null
    }

    # Restore the stash if one was created.
    if ($HasStash) {
        git stash pop 2>&1 | Out-Null
    }

    exit 0
}

# Create commit and push.
$CommitMessage = "chore: bump gdvm version to $Version"
Write-Host "Creating commit: '$CommitMessage'" -ForegroundColor Green

git commit -m $CommitMessage
if ($LASTEXITCODE -ne 0) {
    Exit-WithError "Failed to create commit."
}

# Delete the stash if it was created.
if ($HasStash) {
    Write-Host "Deleting stash created by the release script..." -ForegroundColor Yellow

    git stash drop -q "$StashName"
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Failed to delete stash '$StashName'. It may still exist." -ForegroundColor Yellow
    } else {
        Write-Host "Stash '$StashName' deleted successfully." -ForegroundColor Green
    }
}

Write-Host "Pushing commit to origin/main..." -ForegroundColor Green
git push origin main
if ($LASTEXITCODE -ne 0) {
    Exit-WithError "Failed to push commit to origin/main."
}

# Create and push version tag.
$TagName = "v$Version"
Write-Host "Creating tag: $TagName" -ForegroundColor Green

git tag $TagName
if ($LASTEXITCODE -ne 0) {
    Exit-WithError "Failed to create tag '$TagName'."
}

Write-Host "Pushing tag to origin..." -ForegroundColor Green
git push origin $TagName
if ($LASTEXITCODE -ne 0) {
    Exit-WithError "Failed to push tag '$TagName' to origin."
}

Write-Host "`nRelease $Version created successfully! 🎉" -ForegroundColor Green
Write-Host "- Commit: $CommitMessage" -ForegroundColor Gray
Write-Host "- Tag: $TagName" -ForegroundColor Gray
Write-Host "- Changelog updated with new version section." -ForegroundColor Gray
Write-Host "See CI for build and release artifacts:" -ForegroundColor Gray
Write-Host "  https://github.com/adalinesimonian/gdvm/actions/workflows/build-and-test.yml" -ForegroundColor Gray
