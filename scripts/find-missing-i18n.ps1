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

# Determine the directory where the script is located
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition

# Extract string names from Rust files

# Get all .rs files recursively
$rsFiles = Get-ChildItem -Path "$ScriptDir\..\crates\gdvm\src" -Recurse -Filter *.rs

# Initialize an empty array to store keys
$keys = @()

# Define the regex pattern with single-line and multi-line options
$pattern = '(?:i18n\.t(?:_args)?(?:_w)?\s*\(\s*|(?:[xe]?println_i18n|\bt(?:_w)?)!\s*\(\s*)"([^"\\]*(?:\\.[^"\\]*)*)"'

foreach ($file in $rsFiles) {
    # Read the entire file content as a single string
    $content = Get-Content -Path $file.FullName -Raw

    # Use [regex] with Singleline option to allow . to match newlines
    $regexMatches = [regex]::Matches($content, $pattern, [System.Text.RegularExpressions.RegexOptions]::Singleline)

    foreach ($match in $regexMatches) {
        # Add the captured key to the array
        $keys += $match.Groups[1].Value
    }
}

# Remove duplicates and sort
$uniqueKeys = $keys | Sort-Object -Unique

$script:exitCode = 0

# Iterate through each .ftl file in the i18n directory
Get-ChildItem -Path "$ScriptDir/../crates/gdvm/i18n" -Filter *.ftl | ForEach-Object {
    $ftl = $_.FullName
    $lang = $_.BaseName
    Write-Output "Checking translations for language: ${lang}"

    # Extract keys from the .ftl file
    $keys = Get-Content $ftl |
    Select-String -Pattern '^[a-zA-Z0-9_-]+\s*=' |
    ForEach-Object { ($_ -split '=')[0].Trim() }

    $missing = @()
    # Check for missing strings
    foreach ($string in $uniqueKeys) {
        if (-not ($keys -contains $string)) {
            $missing += $string
        }
    }

    # Print missing translations
    if ($missing.Count -gt 0) {
        $script:exitCode = 1
        Write-Output "Missing translations in ${lang}:"
        foreach ($m in $missing) {
            Write-Output "  - ${m}"
        }
    }
    else {
        Write-Output "All translations present for ${lang}."
    }
    Write-Output ""
}

exit $script:exitCode
