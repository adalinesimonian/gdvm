#!/usr/bin/env bash
set -euo pipefail

# Determine the directory where the script is located
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)

# Extract string names from Rust files
# shellcheck disable=SC2016
expectedKeys=$(find "$SCRIPT_DIR/../src" -type f -name '*.rs' -print0 | \
    xargs -0 perl -0777 -ne '
        while (/(?:i18n\.t(?:_args)?(?:_w)?\s*\(\s*|(?:[xe]?println_i18n|\bt(?:_w)?)!\s*\(\s*[^,\s]+,\s*)"([^"\\]*(?:\\.[^"\\]*)*)"/g) {
            print "$1\n";
        }
    ' | LC_ALL=C sort -u)

exitCode=0

# Iterate through each .ftl file in the i18n directory
for ftl in "$SCRIPT_DIR/../i18n/"*.ftl; do
    lang=$(basename "$ftl" .ftl)
    echo "Checking translations for language: ${lang}"

    # Extract keys from the .ftl file.
    ftlKeys=$(grep -oE '^[a-zA-Z0-9_-]+\s*=' "$ftl" | sed 's/\s*=$//' | LC_ALL=C sort -u)

    # Check for missing strings
    missing=$(comm -23 <(echo "$expectedKeys") <(echo "$ftlKeys"))

    if [[ -n "$missing" ]]; then
        exitCode=1
        echo "Missing translations in ${lang}:"
        while IFS= read -r key; do
            echo "  - $key"
        done <<< "$missing"
    else
        echo "All translations present for ${lang}."
    fi
    echo ""
done

exit $exitCode
