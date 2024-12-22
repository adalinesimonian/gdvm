# Determine the directory where the script is located
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition

# Extract string names from Rust files
$strings = Get-ChildItem -Path "$ScriptDir/../src" -Recurse -Filter *.rs |
    Select-String -Pattern 'i18n\.t\s*\(\s*"([^"]*)"' -AllMatches |
    ForEach-Object { $_.Matches } |
    ForEach-Object { $_.Groups[1].Value } |
    Sort-Object -Unique

$exitCode = 0

# Iterate through each .ftl file in the i18n directory
Get-ChildItem -Path "$ScriptDir/../i18n" -Filter *.ftl | ForEach-Object {
    $ftl = $_.FullName
    $lang = $_.BaseName
    Write-Output "Checking translations for language: ${lang}"

    # Extract keys from the .ftl file
    $keys = Get-Content $ftl |
        Select-String -Pattern '^[a-zA-Z0-9_-]+\s*=' |
        ForEach-Object { ($_ -split '=')[0].Trim() }

    $missing = @()
    # Check for missing strings
    foreach ($string in $strings) {
        if (-not ($keys -contains $string)) {
            $missing += $string
        }
    }

    # Print missing translations
    if ($missing.Count -gt 0) {
        $exitCode = 1
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

exit $exitCode
