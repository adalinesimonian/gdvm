#!/usr/bin/env pwsh
#requires -Version 7.0

<#
.SYNOPSIS
    Extracts the last release section from CHANGELOG.md.

.DESCRIPTION
    This script reads the CHANGELOG.md file and extracts the content of the most
    recent release section (the first version section after "Unreleased"). This
    is used to populate the release description in GitHub.

.EXAMPLE
    ./scripts/get-release-description.ps1
#>

# Get the repository root.
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$RepoRoot = Split-Path -Parent $ScriptDir

$ChangelogPath = "$RepoRoot/CHANGELOG.md"

# Exits with an error message and exit code.
function Exit-WithError {
    param([string]$Message, [int]$ExitCode = 1)
    Write-Error $Message
    exit $ExitCode
}

# Check if changelog file exists.
if (-not (Test-Path $ChangelogPath)) {
    Exit-WithError "Changelog file not found at: $ChangelogPath"
}

# Read the changelog content.
try {
    $ChangelogContent = Get-Content $ChangelogPath -Raw
} catch {
    Exit-WithError "Failed to read changelog file: $($_.Exception.Message)"
}

# Use regex to find the last release section. Matches from the first version
# header after "Unreleased" until the next version header.
$Pattern = '(?s)## Unreleased.*?\n\n(## v[^\n]+.*?)(?=\n## v|\n## [^v]|\Z)'
$Match = [regex]::Match($ChangelogContent, $Pattern)

if (-not $Match.Success) {
    Exit-WithError "Could not find a release section in the changelog"
}

# Extract the last release content.
$LastReleaseContent = $Match.Groups[1].Value.Trim()

if ([string]::IsNullOrWhiteSpace($LastReleaseContent)) {
    Exit-WithError "Last release section appears to be empty"
}

# Remove the version heading line and any blank lines after it.
$ContentLines = $LastReleaseContent -split '\r?\n'
$FilteredLines = @()
$SkipBlankLines = $true

# Start from index 1 to skip the version heading.
for ($i = 1; $i -lt $ContentLines.Length; $i++) {
    $Line = $ContentLines[$i]

    # Skip blank lines immediately after the version heading.
    if ($SkipBlankLines -and [string]::IsNullOrWhiteSpace($Line)) {
        continue
    }

    $SkipBlankLines = $false
    $FilteredLines += $Line
}

# Join the filtered content.
$FilteredContent = $FilteredLines -join "`n"

# Increase heading priority by reducing the number of # symbols by 1.
# ### becomes ##, #### becomes ###, etc.
$AdjustedContent = $FilteredContent -replace '(?m)^(#{3,})', { $_.Groups[1].Value.Substring(1) }

# Output the adjusted content.
Write-Output $AdjustedContent.Trim()
