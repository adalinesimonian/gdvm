#!/usr/bin/env bash

# Check if pwsh is installed
if ! command -v pwsh &> /dev/null; then
    echo "PowerShell (pwsh) is not installed. Please install it to run this script."
    echo "See https://learn.microsoft.com/en-us/powershell/scripting/install/installing-powershell-on-linux"
    exit 1
fi

script_dir="$(dirname "$(realpath "$0")")"
pwsh -NoProfile -ExecutionPolicy Bypass -File "$script_dir/sort-i18n.ps1" "$@"
