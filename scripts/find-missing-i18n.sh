#!/usr/bin/env bash

# Determine the directory where the script is located
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)

# Extract string names from Rust files
strings=$(grep -rhoP 'i18n\.t\s*\(\s*"[^"]*"' "$SCRIPT_DIR/../src" | sed 's/i18n\.t\s*(\s*"\([^"]*\)".*/\1/' | sort | uniq)

exitCode=0

# Iterate through each .ftl file in the i18n directory
for ftl in "$SCRIPT_DIR/../i18n/"*.ftl; do
    lang=$(basename "$ftl" .ftl)
    echo "Checking translations for language: ${lang}"

    # Extract keys from the .ftl file
    keys=$(grep -E '^[a-zA-Z0-9_-]+ *= ' "$ftl" | cut -d '=' -f1 | tr -d ' ')

    missing=()
    # Check for missing strings
    for string in $strings; do
        if ! grep -q "^${string}[[:space:]]*=" "$ftl"; then
            missing+=("$string")
        fi
    done

    # Print missing translations
    if [ ${#missing[@]} -ne 0 ]; then
        exitCode=1
        echo "Missing translations in ${lang}:"
        for m in "${missing[@]}"; do
            echo "  - $m"
        done
    else
        echo "All translations present for ${lang}."
    fi
    echo ""
done

exit $exitCode
