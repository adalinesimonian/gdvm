#!/usr/bin/env pwsh
# SPDX-FileCopyrightText: Copyright (C) 2024 Adaline Simonian
# SPDX-License-Identifier: GPL-3.0-or-later
#
# This file is part of gdvm.
#
# gdvm is free software: you can redistribute it and/or modify it under the
# terms of the GNU General Public License as published by the Free Software
# Foundation, either version 3 of the License, or (at your option) any later
# version.
#
# gdvm is distributed in the hope that it will be useful, but WITHOUT ANY
# WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
# A PARTICULAR PURPOSE. See the GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License along with
# this program. If not, see <https://www.gnu.org/licenses/>.

# Rules:
#
# missing-translation  error    A key or key.attr used from Rust code is missing
#                               in the bundle.
#
# undefined-term       error    A { -term } referenced in this file is missing
#                               in the bundle.
#
# undefined-reference  error    A { message } or { message.attr } referenced in
#                               this file is missing.
#
# attribute-mismatch   error    A message's attributes differ from the fallback
#                               locale (en-US).
#
# not-formatted        error    The file is not formatted correctly, e.g. keys
#                               are not sorted.
#
# unused-translation   warning  A message or attribute defined in the fallback
#                               locale (en-US) is not used in Rust code nor
#                               referenced by another message.
#
# missing-message      warning  A message exists in the fallback locale but is
#                               missing in this locale.
#
# extra-message        warning  A message exists in this locale but not in the
#                               fallback locale (en-US).

$ErrorActionPreference = "Stop"

function Show-Usage {
    Write-Host @"
Usage: lint-i18n.ps1 [OPTIONS]

Lints the Fluent translation bundles.

Options:
  --fix       Fix all formatting issues and remove unused and extra messages.
  -h, --help  Show this help message and exit.
"@
}

# Parse command line arguments manually to support --long-option syntax.
# Is this petty?
# Is this unnecessary?
# Does this fly in the face of PowerShell's design principles?
# Yes. Sue me!
# (Don't actually sue me, please, I'm stubborn but harmless.)
$Fix = $false
for ($i = 0; $i -lt $args.Count; $i++) {
    if ($args[$i] -eq "--fix") {
        $Fix = $true
    }
    elseif ($args[$i] -eq "--help" -or $args[$i] -eq "-h") {
        Show-Usage
        exit 0
    }
    else {
        [Console]::Error.WriteLine("Unknown option: $($args[$i]). Try --help.")
        exit 2
    }
}

# Get the directory of the script.
$ScriptPath = Split-Path -Parent $MyInvocation.MyCommand.Path
$I18nPath = Join-Path $ScriptPath ".." "crates" "gdvm" "i18n"
$SrcPath = Join-Path $ScriptPath ".." "crates" "gdvm" "src"
$script:RepoRoot = (Resolve-Path (Join-Path $ScriptPath "..")).Path
$script:IsGitHub = $env:GITHUB_ACTIONS -eq "true"

# Gets a file's path relative to the repository root.
function Get-RepoRelativePath {
    param ([string]$FullPath)

    return ([System.IO.Path]::GetRelativePath($script:RepoRoot, $FullPath)) -replace '\\', '/'
}

# Escape a value for use in a GitHub Actions workflow command.
function ConvertTo-GitHubData {
    param ([string]$Value)

    return $Value -replace '%', '%25' -replace "`r", '%0D' -replace "`n", '%0A'
}

# Escape a value for use in a property of a GitHub Actions workflow command.
function ConvertTo-GitHubProperty {
    param ([string]$Value)

    return (ConvertTo-GitHubData $Value) -replace ':', '%3A' -replace ',', '%2C'
}

# Get the column number of the first non-whitespace character in a line.
function Get-KeyColumn {
    param ([string]$Text)

    return ([regex]::Match($Text, '^\s*').Length + 1)
}

# Parses a Fluent file and preserves entries with their full content.
function Convert-FluentFile {
    param (
        [string]$FilePath
    )

    $preamble = @()
    $entries = @()
    $currentEntry = $null
    $currentAttribute = $null
    $emptyLineCount = 0

    $lines = Get-Content -Path $FilePath -Encoding UTF8

    for ($lineIndex = 0; $lineIndex -lt $lines.Count; $lineIndex++) {
        $line = $lines[$lineIndex]

        if ($line -match '^(-?[a-zA-Z][a-zA-Z0-9_-]*)\s*=') {
            # Start of a new entry.
            if ($null -ne $currentEntry) {
                $entries += [PSCustomObject]$currentEntry
            }

            $currentEntry = @{
                Key               = $Matches[1]
                IsTerm            = $Matches[1].StartsWith('-')
                Line              = $lineIndex + 1
                ValueLines        = @($line)
                Attributes        = @()
                LeadingEmptyLines = $emptyLineCount
            }
            $currentAttribute = $null
            $emptyLineCount = 0
        }
        elseif ($null -eq $currentEntry) {
            if ($line -match '^\s*$') {
                $emptyLineCount++
            }
            else {
                for ($i = 0; $i -lt $emptyLineCount; $i++) {
                    $preamble += ""
                }
                $emptyLineCount = 0
                $preamble += $line
            }
        }
        elseif ($line -match '^\s+\.([a-zA-Z][a-zA-Z0-9_-]*)\s*=') {
            # An attribute of the current entry.
            $currentAttribute = @{
                Name  = $Matches[1]
                Line  = $lineIndex + 1
                Lines = @($line)
            }
            $currentEntry.Attributes += [PSCustomObject]$currentAttribute
            $currentAttribute = $currentEntry.Attributes[-1]
        }
        elseif ($line -match '^\s*$') {
            # Empty line.
            $nextLine = $lineIndex + 1
            while ($nextLine -lt $lines.Count -and $lines[$nextLine] -match '^\s*$') {
                $nextLine++
            }

            if ($nextLine -ge $lines.Count -or $lines[$nextLine] -match '^(-?[a-zA-Z][a-zA-Z0-9_-]*)\s*=') {
                # Next non-empty line is a new key, so this empty line is between entries.
                $emptyLineCount++
            }
            elseif ($null -ne $currentAttribute) {
                $currentAttribute.Lines += $line
            }
            else {
                $currentEntry.ValueLines += $line
            }
        }
        else {
            # Continuation of current entry in a multi-line value.
            if ($null -ne $currentAttribute) {
                $currentAttribute.Lines += $line
            }
            else {
                $currentEntry.ValueLines += $line
            }
        }
    }

    # Don't forget the last entry.
    if ($null -ne $currentEntry) {
        $entries += [PSCustomObject]$currentEntry
    }

    return [PSCustomObject]@{
        Preamble = $preamble
        Entries  = $entries
    }
}

# Gets the lines of an entry.
function Get-EntryLines {
    param ($Entry)

    $lines = @() + $Entry.ValueLines
    foreach ($attribute in ($Entry.Attributes | Sort-Object -Property Name)) {
        $lines += $attribute.Lines
    }
    return $lines
}

# Get the properly ordered and formatted content of a Fluent bundle.
function Get-FormattedContent {
    param (
        [array]$Preamble,
        [array]$Terms,
        [array]$MessageOrder,
        [hashtable]$MessageLookup
    )

    $content = @() + $Preamble

    $firstTerm = $true

    foreach ($term in ($Terms | Sort-Object -Property Key)) {
        $leading = if ($firstTerm) { 1 } else { 0 }

        for ($i = 0; $i -lt $leading; $i++) {
            $content += ""
        }

        $content += Get-EntryLines -Entry $term
        $firstTerm = $false
    }

    foreach ($slot in $MessageOrder) {
        if (-not $MessageLookup.ContainsKey($slot.Key)) {
            continue
        }

        for ($i = 0; $i -lt $slot.LeadingEmptyLines; $i++) {
            $content += ""
        }

        $content += Get-EntryLines -Entry $MessageLookup[$slot.Key]
    }

    return $content
}

# Get an entry's attribute names, sorted.
function Get-AttributeNames {
    param ($Entry)

    return @($Entry.Attributes | ForEach-Object { $_.Name } | Sort-Object)
}

# Format a list of attribute names for display, or "none" when empty.
function Format-AttributeList {
    param ([array]$Names)

    if ($Names.Count -gt 0) { return $Names -join ', ' }
    return 'none'
}

# Test whether two arrays of lines are identical.
function Test-ContentMatches {
    param ([array]$Expected, [array]$Actual)

    if ($Expected.Count -ne $Actual.Count) {
        return $false
    }

    for ($i = 0; $i -lt $Expected.Count; $i++) {
        if ($Expected[$i] -ne $Actual[$i]) {
            return $false
        }
    }

    return $true
}

# Gets the symbols defined and referenced in a Fluent bundle.
function Get-FileSymbols {
    param ($Parsed)

    $messages = @($Parsed.Entries | Where-Object { -not $_.IsTerm })
    $terms = @($Parsed.Entries | Where-Object { $_.IsTerm })
    $defined = @{}

    foreach ($entry in $messages) {
        $defined[$entry.Key] = $true
        foreach ($attribute in $entry.Attributes) {
            $defined["$($entry.Key).$($attribute.Name)"] = $true
        }
    }

    $definedTerms = @{}

    foreach ($term in $terms) {
        $definedTerms[$term.Key] = $true
    }

    $referencedTerms = @{}
    $referencedMessages = @{}

    foreach ($entry in $Parsed.Entries) {
        $segments = @()

        for ($j = 0; $j -lt $entry.ValueLines.Count; $j++) {
            $segments += [PSCustomObject]@{ LineNo = $entry.Line + $j; Text = $entry.ValueLines[$j] }
        }

        foreach ($attribute in $entry.Attributes) {
            for ($k = 0; $k -lt $attribute.Lines.Count; $k++) {
                $segments += [PSCustomObject]@{ LineNo = $attribute.Line + $k; Text = $attribute.Lines[$k] }
            }
        }

        foreach ($segment in $segments) {
            foreach ($m in [regex]::Matches($segment.Text, '\{\s*(-[a-zA-Z][a-zA-Z0-9_-]*)')) {
                $name = $m.Groups[1].Value

                if (-not $referencedTerms.ContainsKey($name)) {
                    $referencedTerms[$name] = @{ Line = $segment.LineNo; Column = $m.Groups[1].Index + 1 }
                }
            }

            foreach ($m in [regex]::Matches($segment.Text, '\{\s*([a-z][a-zA-Z0-9_-]*(?:\.[a-zA-Z][a-zA-Z0-9_-]*)?)\s*\}')) {
                $name = $m.Groups[1].Value

                if (-not $referencedMessages.ContainsKey($name)) {
                    $referencedMessages[$name] = @{ Line = $segment.LineNo; Column = $m.Groups[1].Index + 1 }
                }
            }
        }
    }

    return [PSCustomObject]@{
        Messages           = $messages
        Terms              = $terms
        Defined            = $defined
        DefinedTerms       = $definedTerms
        ReferencedTerms    = $referencedTerms
        ReferencedMessages = $referencedMessages
    }
}

# Get all the translation keys used in the source code.
function Get-CodeKeys {
    $keys = @()
    $rsFiles = Get-ChildItem -Path $SrcPath -Recurse -Filter *.rs
    $pattern = '(?:i18n\.t(?:_args)?(?:_w)?\s*\(\s*|(?:[xe]?println_i18n|\bt(?:_w)?)!\s*\(\s*)"([^"\\]*(?:\\.[^"\\]*)*)"'
    $attrPattern = '\bt_attr!\s*\(\s*"([^"\\]*(?:\\.[^"\\]*)*)"\s*,\s*"([^"\\]*(?:\\.[^"\\]*)*)"'

    foreach ($file in $rsFiles) {
        $content = Get-Content -Path $file.FullName -Raw

        foreach ($match in [regex]::Matches($content, $pattern, [System.Text.RegularExpressions.RegexOptions]::Singleline)) {
            $keys += $match.Groups[1].Value
        }

        foreach ($match in [regex]::Matches($content, $attrPattern, [System.Text.RegularExpressions.RegexOptions]::Singleline)) {
            $keys += $match.Groups[1].Value
            $keys += "$($match.Groups[1].Value).$($match.Groups[2].Value)"
        }
    }

    return @($keys | Sort-Object -Unique)
}

$script:errorCount = 0
$script:warningCount = 0
$script:fixedCount = 0

function Write-Issue {
    param (
        [string]$Path,
        [int]$Line = 1,
        [int]$Column = 1,
        [ValidateSet("error", "warning", "fixed")][string]$Severity,
        [string]$Rule,
        [string]$Message
    )

    if ($Line -lt 1) { $Line = 1 }
    if ($Column -lt 1) { $Column = 1 }

    switch ($Severity) {
        "error" { $script:errorCount++; $color = "Red" }
        "warning" { $script:warningCount++; $color = "Yellow" }
        "fixed" { $script:fixedCount++; $color = "Green" }
    }

    Write-Host "${Path}:${Line}:${Column}: ${Severity}[$Rule]: $Message" -ForegroundColor $color

    if ($script:IsGitHub) {
        $command = switch ($Severity) {
            "error" { "error" }
            "warning" { "warning" }
            "fixed" { "notice" }
        }

        $properties = "file=$(ConvertTo-GitHubProperty $Path),line=$Line,col=$Column,title=$(ConvertTo-GitHubProperty "i18n: $Rule")"
        Write-Host "::${command} ${properties}::$(ConvertTo-GitHubData $Message)"
    }
}

$referenceFile = Join-Path $I18nPath "en-US.ftl"

if (-not (Test-Path $referenceFile)) {
    [Console]::Error.WriteLine("Reference file not found: $referenceFile")
    exit 2
}

$codeKeys = Get-CodeKeys
$reference = Convert-FluentFile -FilePath $referenceFile
$referenceSymbols = Get-FileSymbols -Parsed $reference
$referenceOrder = @()
$attributeLookup = @{}
$refEntryByKey = @{}

foreach ($message in $referenceSymbols.Messages) {
    $referenceOrder += [PSCustomObject]@{
        Key               = $message.Key
        LeadingEmptyLines = $message.LeadingEmptyLines
    }

    $attributeLookup[$message.Key] = Get-AttributeNames $message
    $refEntryByKey[$message.Key] = $message
}

$usedKeys = @{}

foreach ($key in $codeKeys) {
    $usedKeys[$key] = $true
}

foreach ($ref in $referenceSymbols.ReferencedMessages.Keys) {
    $usedKeys[$ref] = $true
}

$unusedKeys = @{}

foreach ($key in $referenceSymbols.Defined.Keys) {
    if (-not $usedKeys.ContainsKey($key)) {
        $unusedKeys[$key] = $true
    }
}

# Only get rid of keys if the entire message, including all its attributes, is
# unused. If a message is used but some of its attributes are not, only remove
# the unused attributes.
$removedMessageKeys = @{}
$removedAttributes = @{}

foreach ($message in $referenceSymbols.Messages) {
    $attrNames = @($message.Attributes | ForEach-Object { $_.Name })
    $unusedAttrNames = @($attrNames | Where-Object { $unusedKeys.ContainsKey("$($message.Key).$_") })

    if ($unusedKeys.ContainsKey($message.Key) -and $unusedAttrNames.Count -eq $attrNames.Count) {
        $removedMessageKeys[$message.Key] = $true
    }
    elseif ($unusedAttrNames.Count -gt 0) {
        $removedAttributes[$message.Key] = $unusedAttrNames
    }
}

$ftlFiles = Get-ChildItem -Path $I18nPath -Filter "*.ftl" | Sort-Object -Property Name

foreach ($file in $ftlFiles) {
    $isReference = $file.Name -eq "en-US.ftl"

    if ($isReference) {
        $parsed = $reference
        $symbols = $referenceSymbols
    }
    else {
        $parsed = Convert-FluentFile -FilePath $file.FullName
        $symbols = Get-FileSymbols -Parsed $parsed
    }

    $fileRel = Get-RepoRelativePath $file.FullName

    foreach ($key in $codeKeys) {
        if (-not $symbols.Defined.ContainsKey($key)) {
            Write-Issue $fileRel 1 1 "error" "missing-translation" "$key is used in code but is not defined here"
        }
    }

    if ($isReference) {
        foreach ($key in ($unusedKeys.Keys | Sort-Object)) {
            $issueLine = 1
            $issueColumn = 1
            $removable = $false

            if ($removedMessageKeys.ContainsKey($key)) {
                $removable = $true

                if ($refEntryByKey.ContainsKey($key)) {
                    $issueLine = $refEntryByKey[$key].Line
                }
            }
            elseif ($key.Contains('.')) {
                $dot = $key.LastIndexOf('.')
                $messageKey = $key.Substring(0, $dot)
                $attrName = $key.Substring($dot + 1)

                if ($removedAttributes.ContainsKey($messageKey) -and ($removedAttributes[$messageKey] -contains $attrName)) {
                    $removable = $true
                }

                if ($refEntryByKey.ContainsKey($messageKey)) {
                    $entry = $refEntryByKey[$messageKey]
                    $attribute = $entry.Attributes | Where-Object { $_.Name -eq $attrName } | Select-Object -First 1

                    if ($attribute) {
                        $issueLine = $attribute.Line
                        $issueColumn = Get-KeyColumn $attribute.Lines[0]
                    }
                    else {
                        $issueLine = $entry.Line
                    }
                }
            }
            elseif ($refEntryByKey.ContainsKey($key)) {
                $issueLine = $refEntryByKey[$key].Line
            }

            if ($Fix -and $removable) {
                Write-Issue $fileRel $issueLine $issueColumn "fixed" "unused-translation" "removed $key"
            }
            else {
                Write-Issue $fileRel $issueLine $issueColumn "warning" "unused-translation" "$key is defined here but is not used in code"
            }
        }
    }

    foreach ($term in ($symbols.ReferencedTerms.Keys | Sort-Object)) {
        if (-not $symbols.DefinedTerms.ContainsKey($term)) {
            $location = $symbols.ReferencedTerms[$term]
            Write-Issue $fileRel $location.Line $location.Column "error" "undefined-term" "$term is referenced but not defined in this file"
        }
    }

    foreach ($ref in ($symbols.ReferencedMessages.Keys | Sort-Object)) {
        if (-not $symbols.Defined.ContainsKey($ref)) {
            $location = $symbols.ReferencedMessages[$ref]
            Write-Issue $fileRel $location.Line $location.Column "error" "undefined-reference" "$ref is referenced but not defined in this file"
        }
    }

    $messageLookup = @{}

    foreach ($message in $symbols.Messages) {
        $messageLookup[$message.Key] = $message
    }

    $messageOrder = $referenceOrder
    $extraMessages = @()

    if (-not $isReference) {
        foreach ($slot in $referenceOrder) {
            if ($messageLookup.ContainsKey($slot.Key)) {
                $refAttributes = $attributeLookup[$slot.Key]
                $entryAttributes = Get-AttributeNames $messageLookup[$slot.Key]

                if (($refAttributes -join ',') -ne ($entryAttributes -join ',')) {
                    $expected = Format-AttributeList $refAttributes
                    $found = Format-AttributeList $entryAttributes

                    Write-Issue $fileRel $messageLookup[$slot.Key].Line 1 "error" "attribute-mismatch" "$($slot.Key) (expected attributes: $expected; found: $found)"
                }
            }
            else {
                Write-Issue $fileRel 1 1 "warning" "missing-message" "Missing translation key $($slot.Key)"
            }
        }

        $extraMessages = @($symbols.Messages | Where-Object { -not ($referenceOrder.Key -contains $_.Key) })

        foreach ($extra in $extraMessages) {
            if ($Fix) {
                Write-Issue $fileRel $extra.Line 1 "fixed" "extra-message" "removed $($extra.Key)"
            }
            else {
                Write-Issue $fileRel $extra.Line 1 "warning" "extra-message" "$($extra.Key) does not exist in the reference"
            }

            $messageOrder = @($messageOrder) + [PSCustomObject]@{
                Key               = $extra.Key
                LeadingEmptyLines = 1
            }
        }
    }

    $expectedContent = Get-FormattedContent -Preamble $parsed.Preamble -Terms $symbols.Terms -MessageOrder $messageOrder -MessageLookup $messageLookup
    $currentContent = @(Get-Content -Path $file.FullName -Encoding UTF8)

    while ($currentContent.Count -gt 0 -and $currentContent[-1] -match '^\s*$') {
        $currentContent = $currentContent[0..($currentContent.Count - 2)]
    }

    $needsFormatting = -not (Test-ContentMatches $currentContent $expectedContent)

    if ($Fix) {
        $extraKeys = @{}

        foreach ($extra in $extraMessages) {
            $extraKeys[$extra.Key] = $true
        }

        $fixedOrder = @($messageOrder | Where-Object {
                -not $removedMessageKeys.ContainsKey($_.Key) -and -not $extraKeys.ContainsKey($_.Key)
            })

        foreach ($entry in $messageLookup.Values) {
            if ($removedAttributes.ContainsKey($entry.Key)) {
                $entry.Attributes = @($entry.Attributes | Where-Object { -not ($removedAttributes[$entry.Key] -contains $_.Name) })
            }
        }

        $fixedContent = Get-FormattedContent -Preamble $parsed.Preamble -Terms $symbols.Terms -MessageOrder $fixedOrder -MessageLookup $messageLookup

        if (-not (Test-ContentMatches $currentContent $fixedContent)) {
            $utf8NoBom = New-Object System.Text.UTF8Encoding $false
            [System.IO.File]::WriteAllLines($file.FullName, $fixedContent, $utf8NoBom)
        }

        if ($needsFormatting) {
            Write-Issue $fileRel 1 1 "fixed" "not-formatted" "formatted the bundle"
        }
    }
    elseif ($needsFormatting) {
        Write-Issue $fileRel 1 1 "error" "not-formatted" "the bundle is not formatted correctly. Run with --fix to fix it."
    }
}

$summary = @()

if ($script:errorCount -gt 0) {
    $summary += "$($script:errorCount) error$(if ($script:errorCount -ne 1) { 's' })"
}

if ($script:warningCount -gt 0) {
    $summary += "$($script:warningCount) warning$(if ($script:warningCount -ne 1) { 's' })"
}

if ($script:fixedCount -gt 0) {
    $summary += "$($script:fixedCount) fixed"
}

if ($summary.Count -gt 0) {
    Write-Host "`n$($summary -join ', ')."
}

if ($script:errorCount -gt 0) {
    exit 1
}

Write-Host "All Fluent bundles are clean."
exit 0
