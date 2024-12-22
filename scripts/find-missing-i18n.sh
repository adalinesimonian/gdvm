#!/usr/bin/env bash

# Determine the directory where the script is located
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)

# Extract string names from Rust files
# shellcheck disable=SC2016
strings=$(find "$SCRIPT_DIR/../src" -type f -name '*.rs' -print0 | \
    xargs -0 perl -0777 -ne '
        while (/i18n\.t(?:_args)?\s*\(\s*"([^"\\]*(?:\\.[^"\\]*)*)"/g) {
            print "$1\n";
        }
    ' | sort | uniq)

exitCode=0

# Iterate through each .ftl file in the i18n directory
for ftl in "$SCRIPT_DIR/../i18n/"*.ftl; do
    lang=$(basename "$ftl" .ftl)
    echo "Checking translations for language: ${lang}"

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
