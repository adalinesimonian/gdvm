#!/usr/bin/env pwsh

$ErrorActionPreference = "Stop"

# Parse command line arguments manually to support --format syntax.
# Is this petty?
# Is this unnecessary?
# Does this fly in the face of PowerShell's design principles?
# Yes. Sue me!
# (Don't actually sue me, please, I'm stubborn but harmless.)
$Format = $false
for ($i = 0; $i -lt $args.Count; $i++) {
    if ($args[$i] -eq "--format") {
        $Format = $true
    } elseif ($args[$i] -match "^--") {
        Write-Error "Unknown option: $($args[$i])"
        exit 1
    }
}

# Get the directory of the script.
$ScriptPath = Split-Path -Parent $MyInvocation.MyCommand.Path
$I18nPath = Join-Path $ScriptPath ".." "crates" "gdvm" "i18n"

# Parses a Fluent file and preserves entries with their full content.
function Convert-FluentFile {
    param (
        [string]$FilePath
    )

    $entries = @()
    $currentEntry = @{
        Key = ""
        Lines = @()
        LeadingEmptyLines = 0
    }
    $inEntry = $false
    $emptyLineCount = 0

    $lines = Get-Content -Path $FilePath -Encoding UTF8

    for ($i = 0; $i -lt $lines.Count; $i++) {
        $line = $lines[$i]

        if ($line -match '^\s*$') {
            # Empty line.
            if ($inEntry) {
                # Check if the next non-empty line is another key or continuation.
                $nextKeyLine = $i + 1
                while ($nextKeyLine -lt $lines.Count -and $lines[$nextKeyLine] -match '^\s*$') {
                    $nextKeyLine++
                }

                if ($nextKeyLine -lt $lines.Count -and $lines[$nextKeyLine] -match '^([a-zA-Z][a-zA-Z0-9_-]*)\s*=') {
                    # Next non-empty line is a new key, so this empty line is between entries.
                    $emptyLineCount++
                } else {
                    # This empty line is within the current entry, i.e. a multi-line value.
                    $currentEntry.Lines += $line
                }
            } else {
                # Empty line between entries.
                $emptyLineCount++
            }
        } elseif ($line -match '^([a-zA-Z][a-zA-Z0-9_-]*)\s*=') {
            # Start of a new entry.
            if ($inEntry -and $currentEntry.Key) {
                # Save the previous entry.
                $entries += [PSCustomObject]$currentEntry
            }

            $currentEntry = @{
                Key = $Matches[1]
                Lines = @($line)
                LeadingEmptyLines = $emptyLineCount
            }
            $inEntry = $true
            $emptyLineCount = 0
        } elseif ($inEntry) {
            # Continuation of current entry in a multi-line value.
            $currentEntry.Lines += $line
        }
    }

    # Don't forget the last entry.
    if ($inEntry -and $currentEntry.Key) {
        $entries += [PSCustomObject]$currentEntry
    }

    return $entries
}

# Writes sorted entries to a file.
function Write-SortedFluentFile {
    param (
        [string]$FilePath,
        [array]$SortedEntries
    )

    $output = @()

    foreach ($entry in $SortedEntries) {
        for ($i = 0; $i -lt $entry.LeadingEmptyLines; $i++) {
            $output += ""
        }

        $output += $entry.Lines
    }

    $utf8NoBom = New-Object System.Text.UTF8Encoding $false
    [System.IO.File]::WriteAllLines($FilePath, $output, $utf8NoBom)
}

# Main script.
Write-Host "Checking Fluent translation files..."

# Parse the reference file.
$referenceFile = Join-Path $I18nPath "en-US.ftl"
if (-not (Test-Path $referenceFile)) {
    Write-Error "Reference file not found: $referenceFile"
    exit 1
}

Write-Host "Parsing reference file: en-US.ftl"
$referenceEntries = Convert-FluentFile -FilePath $referenceFile

# Create a lookup for the order and spacing.
$orderLookup = @{}
$spacingLookup = @{}
for ($i = 0; $i -lt $referenceEntries.Count; $i++) {
    $orderLookup[$referenceEntries[$i].Key] = $i
    $spacingLookup[$referenceEntries[$i].Key] = $referenceEntries[$i].LeadingEmptyLines
}

# Process all other .ftl files.
$ftlFiles = Get-ChildItem -Path $I18nPath -Filter "*.ftl" | Where-Object { $_.Name -ne "en-US.ftl" }

$hasChanges = $false
$exitCode = 0

foreach ($file in $ftlFiles) {
    Write-Host "Processing: $($file.Name)"

    # Parse the file.
    $entries = Convert-FluentFile -FilePath $file.FullName

    # Create a lookup for easy access.
    $entryLookup = @{}
    foreach ($entry in $entries) {
        $entryLookup[$entry.Key] = $entry
    }

    # Sort entries according to reference file order.
    $sortedEntries = @()
    $missingKeys = @()

    foreach ($refEntry in $referenceEntries) {
        if ($entryLookup.ContainsKey($refEntry.Key)) {
            $entry = $entryLookup[$refEntry.Key]
            # Keep the original entry but update spacing to match reference.
            $sortedEntry = [PSCustomObject]@{
                Key = $entry.Key
                Lines = $entry.Lines
                LeadingEmptyLines = $refEntry.LeadingEmptyLines
            }
            $sortedEntries += $sortedEntry
        } else {
            $missingKeys += $refEntry.Key
        }
    }

    # Check for keys that exist in translation but not in reference.
    $extraKeys = @()
    foreach ($entry in $entries) {
        if (-not $orderLookup.ContainsKey($entry.Key)) {
            $extraKeys += $entry
        }
    }

    # Check if extra keys need formatting before we modify them.
    $extraKeysNeedFormatting = $false
    if ($extraKeys.Count -gt 0) {
        foreach ($extraKey in $extraKeys) {
            if ($extraKey.LeadingEmptyLines -ne 1) {
                $extraKeysNeedFormatting = $true
                break
            }
        }
    }

    # Append extra keys at the end with appropriate spacing.
    if ($extraKeys.Count -gt 0) {
        foreach ($extraKey in $extraKeys) {
            # Extra keys should always have exactly 1 leading empty line for consistency.
            $extraKey.LeadingEmptyLines = 1
            $sortedEntries += $extraKey
        }
    }

    # Report missing keys.
    if ($missingKeys.Count -gt 0) {
        Write-Warning "$($file.Name) is missing $($missingKeys.Count) keys: $($missingKeys -join ', ')"
    }

    # Report extra keys.
    if ($extraKeys.Count -gt 0) {
        Write-Warning "$($file.Name) has $($extraKeys.Count) extra keys: $($extraKeys.Key -join ', ')"
    }

    # Check if the file needs changes by comparing current vs expected content.
    # Reference keys are compoared for order and spacing, and extra keys are
    # checked for spacing only.

    # Build what the content should look like with reference keys.
    $expectedReferenceContent = @()
    foreach ($entry in $sortedEntries) {
        if ($orderLookup.ContainsKey($entry.Key)) {
            # Add leading empty lines.
            for ($i = 0; $i -lt $entry.LeadingEmptyLines; $i++) {
                $expectedReferenceContent += ""
            }
            # Add the entry lines.
            $expectedReferenceContent += $entry.Lines
        }
    }

    # Extract reference keys from current file for comparison.
    $currentReferenceContent = @()
    foreach ($refEntry in $referenceEntries) {
        if ($entryLookup.ContainsKey($refEntry.Key)) {
            $currentEntry = $entryLookup[$refEntry.Key]
            # Add leading empty lines as they currently are.
            for ($i = 0; $i -lt $currentEntry.LeadingEmptyLines; $i++) {
                $currentReferenceContent += ""
            }
            # Add the entry lines.
            $currentReferenceContent += $currentEntry.Lines
        }
    }

    # Compare reference key portions.
    $needsChanges = $false
    if ($currentReferenceContent.Count -ne $expectedReferenceContent.Count) {
        $needsChanges = $true
    } else {
        for ($i = 0; $i -lt $currentReferenceContent.Count; $i++) {
            if ($currentReferenceContent[$i] -ne $expectedReferenceContent[$i]) {
                $needsChanges = $true
                break
            }
        }
    }

    # Check if extra keys need formatting.
    if (-not $needsChanges -and $extraKeysNeedFormatting) {
        $needsChanges = $true
    }

    if ($needsChanges) {
        $hasChanges = $true
        if ($Format) {
            Write-SortedFluentFile -FilePath $file.FullName -SortedEntries $sortedEntries
            Write-Host "  ✓ Formatted and saved"
        } else {
            Write-Host "  ✗ File is not properly sorted" -ForegroundColor Red
            $exitCode = 1
        }
    } else {
        Write-Host "  ✓ File is properly sorted" -ForegroundColor Green
    }
}

if ($Format) {
    if ($hasChanges) {
        Write-Host "`nFormatting complete! Files have been updated."
    } else {
        Write-Host "`nAll files were already properly sorted."
    }
} else {
    if ($exitCode -eq 0) {
        Write-Host "`nAll files are properly sorted!"
    } else {
        Write-Host "`nSome files are not properly sorted. Run with --format to fix them." -ForegroundColor Red
    }
}

exit $exitCode
