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
    [string]$Version,

    [Parameter(Mandatory = $false)]
    [string]$SocialPost,

    [Parameter(Mandatory = $false)]
    [switch]$NoBluesky,

    [Parameter(Mandatory = $false)]
    [switch]$DryRun
)

# Ensure the current directory is the repository root.
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$RepoRoot = Split-Path -Parent $ScriptDir
Set-Location $RepoRoot

$SocialPostMaxLength = 280
$SocialPostFilePath = Join-Path $RepoRoot ".release-social-post.md"
$ReleaseUrl = "https://github.com/adalinesimonian/gdvm/releases/tag/v$Version"

$PostToBluesky = -not $NoBluesky
$IsDryRun = [bool]$DryRun
$HasBlockingIssues = $false

# Records a warning during dry run, otherwise exits.
function WarnOrExit {
    param([string]$Message)

    if ($IsDryRun) {
        Write-Host "DRY RUN warning: $Message" -ForegroundColor Yellow
        $script:HasBlockingIssues = $true
    }
    else {
        Exit-WithError $Message
    }
}

$GhCli = Get-Command gh -ErrorAction SilentlyContinue
if (-not $GhCli) {
    WarnOrExit "GitHub CLI (gh) is not installed. Install from https://cli.github.com/ and run 'gh auth login'."
}

if (-not $IsDryRun) {
    gh auth status 1>$null 2>$null
    if ($LASTEXITCODE -ne 0) {
        Exit-WithError "GitHub CLI (gh) is not authenticated. Run: gh auth login"
    }
}

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

# Retrieves the Bluesky post text from CLI input, environment, or a local file.
function Get-SocialPostContent {
    param(
        [string]$CliText,
        [string]$FilePath
    )

    $textSource = $CliText

    if ([string]::IsNullOrWhiteSpace($textSource)) {
        $EnvText = $env:GDVM_RELEASE_SOCIAL_POST
        if (-not [string]::IsNullOrWhiteSpace($EnvText)) {
            $textSource = $EnvText
        }
    }

    if ([string]::IsNullOrWhiteSpace($textSource) -and (Test-Path $FilePath)) {
        $fileContent = Get-Content $FilePath -Raw
        if (-not [string]::IsNullOrWhiteSpace($fileContent)) {
            $textSource = $fileContent
        }
    }

    if ([string]::IsNullOrWhiteSpace($textSource)) {
        return ""
    }

    return $textSource.Trim()
}

function Expand-SocialPostPlaceholders {
    param(
        [string]$Text,
        [string]$ReleaseUrl
    )

    if ([string]::IsNullOrWhiteSpace($Text)) {
        return ""
    }

    return $Text.Replace("{{RELEASE_URL}}", $ReleaseUrl)
}

function Assert-SocialPostLength {
    param(
        [string]$Text,
        [int]$MaxLength,
        [string]$FilePath
    )

    $length = $Text.Length
    if ($length -gt $MaxLength) {
        Exit-WithError "Bluesky post text is $length characters, exceeding the limit of $MaxLength. Please shorten it and rerun."
    }
}

# Validate version format.
if (-not (Test-SemanticVersion $Version)) {
    Exit-WithError "Version must be in the format Major.Minor.Patch (e.g., 1.2.3)"
}

Write-Host "Cutting release for version $Version..." -ForegroundColor Green

# Check that current branch is main.
$CurrentBranch = git rev-parse --abbrev-ref HEAD
if ($CurrentBranch -ne "main") {
    WarnOrExit "Current branch is '$CurrentBranch', but must be 'main'"
}

# Check for staged or unstaged changes, ignoring untracked files and CHANGELOG.md.
$StagedChanges = git diff --cached --name-only | Where-Object { $_ -ne "CHANGELOG.md" }
$UnstagedChanges = git diff --name-only | Where-Object { $_ -ne "CHANGELOG.md" }

if ($StagedChanges -or $UnstagedChanges) {
    WarnOrExit "There are staged or unstaged changes. Please commit or stash them first."
}

# Check that current branch is in sync with origin.
git fetch origin main
$LocalCommit = git rev-parse HEAD
$RemoteCommit = git rev-parse origin/main
$OriginalHead = $LocalCommit

if ($LocalCommit -ne $RemoteCommit) {
    WarnOrExit "Local branch is not in sync with origin/main. Please pull or push as needed."
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
$CurrentVersionMatch = [regex]::Match($CargoContent, '(?s)\[package\](?:(?!\[).)*?version\s*=\s*"([^"]+)"')
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

$SocialPostRaw = Get-SocialPostContent -CliText $SocialPost -FilePath $SocialPostFilePath
$SocialPostContent = Expand-SocialPostPlaceholders -Text $SocialPostRaw -ReleaseUrl $ReleaseUrl
if (-not [string]::IsNullOrWhiteSpace($SocialPostContent)) {
    Assert-SocialPostLength -Text $SocialPostContent -MaxLength $SocialPostMaxLength -FilePath $SocialPostFilePath
}

# Update CHANGELOG.md.
Write-Host "Updating CHANGELOG.md..." -ForegroundColor Yellow

# Extract the unreleased content, excluding the "Full Changelog" line.
$UnreleasedLines = $UnreleasedContent -split '\n' | Where-Object { $_ -notmatch '^\*\*Full Changelog\*\*:' }
$UnreleasedText = ($UnreleasedLines | Where-Object { $_.Trim() }) -join "`n"

# Find the previous version from the changelog.
$PreviousVersionMatch = [regex]::Match($ChangelogContent, '## v(\d+\.\d+\.\d+)\s')
if ($PreviousVersionMatch.Success) {
    $PreviousVersion = $PreviousVersionMatch.Groups[1].Value
}
elseif ($UpdateCargoFiles) {
    # Use the version from Cargo.toml if no previous version found.
    $PreviousVersion = $CurrentVersion
}
else {
    # Should never happen.
    Exit-WithError "Could not determine previous version from CHANGELOG.md or Cargo.toml."
}

# Create new version section.
$NewVersionSection = @"
## v$Version

$UnreleasedText

**Full Changelog**: https://github.com/adalinesimonian/gdvm/compare/v$PreviousVersion...v$Version
"@

# Create new unreleased section.
$NewUnreleasedSection = @"
## Unreleased

**Full Changelog**: https://github.com/adalinesimonian/gdvm/compare/v$Version...main
"@

if ($IsDryRun) {
    Write-Host "" -ForegroundColor Gray
    Write-Host "DRY RUN: No changes will be made. Planned actions:" -ForegroundColor Green
    Write-Host "- Would update CHANGELOG.md: move Unreleased to v$Version and create new Unreleased section." -ForegroundColor Gray
    if ($UpdateCargoFiles) {
        Write-Host "- Would bump version in crates/gdvm/Cargo.toml to $Version and update Cargo.lock." -ForegroundColor Gray
    }
    else {
        Write-Host "- Version already set, would leave Cargo files unchanged." -ForegroundColor Gray
    }
    if (-not [string]::IsNullOrWhiteSpace($SocialPostContent)) {
        Write-Host "- Would send social post (length $($SocialPostContent.Length)): $SocialPostContent" -ForegroundColor Gray
    }
    else {
        Write-Host "- No social post provided, would skip Bluesky post body." -ForegroundColor Gray
    }

    $CommitMessage = "chore: bump gdvm version to $Version"
    $TagName = "v$Version"
    $PostFlag = if ($PostToBluesky) { "true" } else { "false" }

    Write-Host "- Would create commit: $CommitMessage" -ForegroundColor Gray
    Write-Host "- Would create annotated tag: $TagName" -ForegroundColor Gray
    Write-Host "- Would push commit to origin/main and push tag $TagName" -ForegroundColor Gray
    Write-Host "- Would trigger workflow: gh workflow run release.yml --ref $TagName -f release_tag=$TagName -f social_post=... -f post_to_bsky=$PostFlag" -ForegroundColor Gray
    Write-Host "  social_post preview: $SocialPostContent" -ForegroundColor Gray

    if ($HasBlockingIssues) {
        Write-Host "`nDry run completed with warnings above. Resolve them before running for real." -ForegroundColor Yellow
    }
    else {
        Write-Host "`nDry run completed with no blocking issues detected." -ForegroundColor Green
    }

    exit 0
}

# Create a stash to save the current state before making changes.
Write-Host "Saving current git state..." -ForegroundColor Yellow
$StashName = "gdvm-release-script-backup-$(Get-Date -Format 'yyyyMMdd-HHmmss')"
$StashResult = git stash push -u -m "$StashName" 2>&1
$HasStash = $LASTEXITCODE -eq 0 -and $StashResult -notmatch "No local changes to save"

if ($UpdateCargoFiles) {
    # Update Cargo.toml with the new version.
    $PackageVersionRegex = '(?s)(\[package\](?:(?!\[).)*?version\s*=\s*")[^"]+(")'
    $NewCargoContent = $CargoContent -replace $PackageVersionRegex, ('${1}' + $Version + '${2}')
    Set-Content -Path $CargoTomlPath -Value $NewCargoContent -NoNewline

    # Update Cargo.lock.
    Write-Host "Updating Cargo.lock..." -ForegroundColor Yellow
    $CargoUpdateResult = cargo update -p gdvm 2>&1
    if ($LASTEXITCODE -ne 0) {
        Exit-WithError "Failed to update Cargo.lock: $CargoUpdateResult"
    }
}

# Replace the changelog content.
$NewChangelogContent = $ChangelogContent -replace '(?s)## Unreleased\s*\n.*?(?=\n## v|\z)', "$NewUnreleasedSection`n`n$NewVersionSection`n"
Set-Content -Path "CHANGELOG.md" -Value $NewChangelogContent -NoNewline

Write-Host "Formatting CHANGELOG.md with Prettier..." -ForegroundColor Yellow

$YarnExists = Get-Command yarn -ErrorAction SilentlyContinue
if ($YarnExists) {
    $PrettierResult = yarn dlx prettier --write CHANGELOG.md 2>&1
    $PrettierExitCode = $LASTEXITCODE
}
else {
    $PrettierResult = npx prettier --write CHANGELOG.md 2>&1
    $PrettierExitCode = $LASTEXITCODE
}

if ($PrettierExitCode -ne 0) {
    Exit-WithError "Failed to format CHANGELOG.md with Prettier: $PrettierResult"
}

# Stage changes and show diff.
if ($UpdateCargoFiles) {
    git add $CargoTomlPath "Cargo.lock" "CHANGELOG.md"
}
else {
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
    }
    else {
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

Write-Host "`nFinal verification before pushing and triggering the release." -ForegroundColor Yellow
Write-Host "Type ""yes"" to continue. Any other input cancels and restores the repository state: " -ForegroundColor Yellow -NoNewline
$FinalConfirmation = Read-Host

if ($FinalConfirmation -ne "yes") {
    Write-Host "Release cancelled. Reverting repository to its previous state..." -ForegroundColor Yellow

    git reset --hard $OriginalHead 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) {
        Exit-WithError "Failed to restore repository to original state."
    }

    if ($HasStash) {
        git stash pop 2>&1 | Out-Null
        if ($LASTEXITCODE -ne 0) {
            Exit-WithError "Failed to restore saved stash state."
        }
    }

    exit 0
}

# Delete the stash if it was created.
if ($HasStash) {
    Write-Host "Deleting stash created by the release script..." -ForegroundColor Yellow

    git stash drop -q "$StashName"
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Failed to delete stash '$StashName'. It may still exist." -ForegroundColor Yellow
    }
    else {
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

git tag -a $TagName -m "gdvm $Version"
if ($LASTEXITCODE -ne 0) {
    Exit-WithError "Failed to create tag '$TagName'."
}

Write-Host "Pushing tag to origin..." -ForegroundColor Green
git push origin $TagName
if ($LASTEXITCODE -ne 0) {
    Exit-WithError "Failed to push tag '$TagName' to origin."
}

# Trigger release workflow via GitHub Actions using gh CLI.
$PostFlag = if ($PostToBluesky) { "true" } else { "false" }

Write-Host "Triggering release workflow on $TagName..." -ForegroundColor Green

gh workflow run release.yml `
    --ref $TagName `
    -f release_tag=$TagName `
    -f social_post="$SocialPostContent" `
    -f post_to_bsky=$PostFlag

if ($LASTEXITCODE -ne 0) {
    Exit-WithError "Failed to trigger release workflow."
}

Write-Host "`nRelease workflow started for $Version! 🎉" -ForegroundColor Green
Write-Host "- Commit: $CommitMessage" -ForegroundColor Gray
Write-Host "- Tag: $TagName" -ForegroundColor Gray
Write-Host "- Changelog updated with new version section." -ForegroundColor Gray
Write-Host "See CI for build and release artifacts:" -ForegroundColor Gray
Write-Host "  https://github.com/adalinesimonian/gdvm/actions/workflows/release.yml" -ForegroundColor Gray
