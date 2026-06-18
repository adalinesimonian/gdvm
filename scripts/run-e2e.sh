#!/usr/bin/env bash
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

set -uo pipefail

is_github="${GITHUB_ACTIONS:-false}"

detect_os() {
    if [[ -n "${RUNNER_OS:-}" ]]; then
        case "$RUNNER_OS" in
        Windows) echo windows ;;
        macOS) echo macos ;;
        *) echo linux ;;
        esac
        return
    fi
    case "$(uname -s)" in
    MINGW* | MSYS* | CYGWIN* | Windows_NT) echo windows ;;
    Darwin) echo macos ;;
    *) echo linux ;;
    esac
}

detect_arch() {
    if [[ -n "${RUNNER_ARCH:-}" ]]; then
        case "$RUNNER_ARCH" in
        ARM64 | ARM) echo arm ;;
        *) echo x86 ;;
        esac
        return
    fi
    case "$(uname -m)" in
    aarch64 | arm64) echo arm ;;
    *) echo x86 ;;
    esac
}

os="$(detect_os)"
arch="$(detect_arch)"
export os arch

if [[ -t 1 || "$is_github" == "true" ]]; then
    USE_COLOR=true
else
    USE_COLOR=false
fi
export USE_COLOR

echo "Running e2e suite on $os $arch"

godot_shim() {
    if [[ "$os" == "windows" ]]; then
        godot_console "$@"
    else
        godot "$@"
    fi
}

link_target() {
    local base="$1"
    case "$os" in
    windows) echo "${base}.exe" ;;
    macos) echo "${base}.app" ;;
    *) echo "${base}" ;;
    esac
}

run_linked() {
    local base="$1"
    shift
    case "$os" in
    windows) "${base}_console.exe" "$@" ;;
    macos) "${base}.app/Contents/MacOS/Godot" "$@" ;;
    *) "${base}" "$@" ;;
    esac
}

export -f godot_shim link_target run_linked

if [[ -z "${GITHUB_TOKEN:-}" && -f /run/secrets/github_token ]]; then
    token_from_file="$(cat /run/secrets/github_token)"
    if [[ -n "$token_from_file" ]]; then
        echo "Using GitHub token from /run/secrets/github_token"
        export GITHUB_TOKEN="$token_from_file"
    fi
fi

gdvm --version

if [[ -z "${GITHUB_TOKEN:-}" ]]; then
    echo "Warning: GITHUB_TOKEN is not set. Some tests may fail due to rate limiting."
else
    echo "Configuring gdvm with provided GitHub token."
    gdvm config set github.token "$GITHUB_TOKEN"
fi

test_index=1
tests_failed=0
declare -a test_numbers=()
declare -a test_descriptions=()
declare -a test_results=()

print_color() {
    local color="$1"
    shift
    if [[ "$USE_COLOR" == "true" ]]; then
        case "$color" in
        green) printf '\033[0;32m%s\033[0m\n' "$*" ;;
        red) printf '\033[0;31m%s\033[0m\n' "$*" ;;
        *) printf '%s\n' "$*" ;;
        esac
    else
        printf '%s\n' "$*"
    fi
}

test() {
    local desc="$1"
    local cmd=("${@:2}")

    if [[ "$is_github" == "true" ]]; then
        echo "::group::[$test_index] $desc"
    else
        echo ""
        echo "[$test_index] Running test: $desc"
    fi

    local status="failed"
    if ((${#cmd[@]} > 0)); then
        if "${cmd[@]}"; then status="passed"; fi
    else
        local script
        script="$(cat)"
        if bash -euo pipefail -c "$script"; then status="passed"; fi
    fi

    test_numbers+=("$test_index")
    test_descriptions+=("$desc")
    test_results+=("$status")

    if [[ "$status" == "passed" ]]; then
        print_color green "[$test_index] Test passed: $desc"
    else
        print_color red "[$test_index] Test failed: $desc"
        ((tests_failed++))
    fi

    if [[ "$is_github" == "true" ]]; then
        echo "::endgroup::"
        if [[ "$status" == "failed" ]]; then
            echo "::error title=E2E test failed::[$test_index] $desc"
        fi
    fi

    ((test_index++))
}

summarize_tests() {
    local total=${#test_results[@]}
    local passed=$((total - tests_failed))
    local i

    echo
    if ((tests_failed > 0)); then
        print_color red "Test summary: $passed passed, $tests_failed failed, $total total"
    else
        print_color green "Test summary: $passed passed, $tests_failed failed, $total total"
    fi

    if [[ "$is_github" == "true" && -n "${GITHUB_STEP_SUMMARY:-}" ]]; then
        {
            echo "### End-to-end tests ($os/$arch)"
            echo ""
            echo "**$passed passed, $tests_failed failed, $total total**"
            echo ""
            echo "| # | Test | Result |"
            echo "| --: | --- | :--: |"
            for i in "${!test_results[@]}"; do
                local icon="✅"
                [[ "${test_results[$i]}" == "failed" ]] && icon="❌"
                echo "| ${test_numbers[$i]} | ${test_descriptions[$i]} | $icon |"
            done
        } >>"$GITHUB_STEP_SUMMARY"
    fi

    if ((tests_failed > 0)); then
        echo "Failed tests:"
        for i in "${!test_results[@]}"; do
            if [[ "${test_results[$i]}" == "failed" ]]; then
                print_color red " - [${test_numbers[$i]}] ${test_descriptions[$i]}"
            fi
        done
        exit 1
    fi

    echo "E2E workflow completed successfully."
}

workdir="$(mktemp -d)"
trap 'rm -rf "$workdir"' EXIT

test "Search for 4.x releases" <<'TEST_SCRIPT'
gdvm search --filter 4 | grep 4.3-stable
TEST_SCRIPT

test "Refresh flag repopulates registry cache" <<'TEST_SCRIPT'
cache="$HOME/.gdvm/cache.json"
backup="$cache.bak.refresh"

if [[ -f "$cache" ]]; then
    cp "$cache" "$backup"
fi

printf '%s' '{"gdvm":{"last_update_check":0,"new_version":null,"new_major_version":null},"godot_registry":{"last_fetched":0,"releases":[]},"release_capabilities":{"last_fetched":0,"entries":[]}}' > "$cache"

if gdvm search --cache-only --filter 4 | grep -q 4.3-stable; then
    echo "cache-only search unexpectedly found releases with empty cache"
    [[ -f "$backup" ]] && mv "$backup" "$cache"
    exit 1
fi

gdvm search --refresh --filter 4 | grep 4.3-stable

if ! grep -q '"tag_name"' "$cache"; then
    echo "cache was not repopulated with releases"
    cat "$cache"
    [[ -f "$backup" ]] && mv "$backup" "$cache"
    exit 1
fi

[[ -f "$backup" ]] && mv "$backup" "$cache"
TEST_SCRIPT

test "Install Godot 4.3" gdvm install 4.3

# ARM lacks Godot 3.x builds, so the ARM matrix uses 4.4 instead of 3.x.
if [[ "$arch" == "arm" ]]; then
    test "Install and use Godot 4.4.0 (ARM)" <<'TEST_SCRIPT'
gdvm install 4.4.0
gdvm use 4.4.0
TEST_SCRIPT
else
    test "Install and use Godot 3.6.2 (x86)" <<'TEST_SCRIPT'
gdvm install 3.6.2
gdvm use 3.6.2
TEST_SCRIPT
fi

pushd "$workdir" >/dev/null || exit

test "Pin Godot 4.3.0 and verify gdvm.toml and .gdvmrc are created" <<'TEST_SCRIPT'
gdvm pin 4.3.0
# gdvm.toml should exist with the new explicit-variant format.
cat gdvm.toml | grep 'version = "default:4.3-stable"'
# .gdvmrc should exist with the old pre-refactor format.
cat .gdvmrc | grep '4.3.0-stable'
gdvm run --console=true -- --version | grep 4.3.stable.official
TEST_SCRIPT

test "Godot shim points to the pinned version" <<'TEST_SCRIPT'
godot_shim --version | grep 4.3.stable.official
TEST_SCRIPT

test "Show without version resolves to pinned 4.3.0" <<'TEST_SCRIPT'
path="$(gdvm show)"
echo "gdvm show -> $path"
"$path" --version | grep 4.3.stable.official
TEST_SCRIPT

popd >/dev/null || exit

if [[ "$arch" == "arm" ]]; then
    test "Run Godot 4.4 explicitly (ARM)" <<'TEST_SCRIPT'
gdvm run 4.4 --console=true -- --version | grep 4.4.1.stable.official
TEST_SCRIPT
else
    test "Run Godot 3.5 explicitly (x86)" <<'TEST_SCRIPT'
gdvm run 3.5 --console=true -- --version | grep 3.5.3.stable.official
TEST_SCRIPT
fi

test "Run Godot 4.4 with C# support (deprecated --csharp flag)" <<'TEST_SCRIPT'
gdvm run 4.4 --csharp --console=true -- --version | grep 4.4.1.stable.mono.official
TEST_SCRIPT

test "Run Godot 4.4 with C# support" <<'TEST_SCRIPT'
gdvm run csharp:4.4 --console=true -- --version | grep 4.4.1.stable.mono.official
TEST_SCRIPT

test "default: variant specifier selects the standard build" <<'TEST_SCRIPT'
# 4.3 standard was installed earlier; default:4.3 must resolve to it, not Mono.
gdvm run default:4.3 --console=true -- --version | grep 4.3.stable.official
TEST_SCRIPT

test 'Running "stable" keyword resolves to latest stable' <<'TEST_SCRIPT'
gdvm run stable --console=true -- --version | grep stable.official
TEST_SCRIPT

test 'Running "latest" keyword resolves to latest stable' <<'TEST_SCRIPT'
gdvm run latest --console=true -- --version | grep stable.official
TEST_SCRIPT

test "Running C# build via csharp:stable" <<'TEST_SCRIPT'
gdvm run csharp:stable --console=true -- --version | grep stable.mono.official
TEST_SCRIPT

test "Install with csharp: variant specifier" <<'TEST_SCRIPT'
gdvm install csharp:4.3
gdvm list | grep -E '4\.3.*csharp'
TEST_SCRIPT

test "Show with csharp: variant specifier returns existing file" <<'TEST_SCRIPT'
path="$(gdvm show csharp:4.3.0)"
echo "gdvm show csharp:4.3.0 -> $path"
if [[ ! -e "$path" ]]; then
    echo "Resolved path does not exist: $path"
    exit 1
fi
"$path" --version | grep 4.3.stable.mono.official
TEST_SCRIPT

test "Pin with csharp: variant specifier and verify version" <<'TEST_SCRIPT'
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT
cd "$tmpdir"
gdvm pin csharp:4.3.0
# gdvm.toml should carry the csharp variant prefix.
cat gdvm.toml | grep 'version = "csharp:4.3-stable"'
# .gdvmrc should have the old pre-refactor format.
cat .gdvmrc | grep '4.3.0-stable-csharp'
gdvm run --console=true -- --version | grep 4.3.stable.mono.official
TEST_SCRIPT

test "Use with csharp: variant specifier sets default" <<'TEST_SCRIPT'
gdvm use csharp:4.3.0
path="$(gdvm show)"
"$path" --version | grep 4.3.stable.mono.official
TEST_SCRIPT

test "Pin with --no-legacy skips .gdvmrc" <<'TEST_SCRIPT'
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT
cd "$tmpdir"
gdvm pin 4.3.0 --no-legacy
cat gdvm.toml | grep 'version = "default:4.3-stable"'
if [[ -f .gdvmrc ]]; then
    echo ".gdvmrc should not exist with --no-legacy"
    exit 1
fi
gdvm run --console=true -- --version | grep 4.3.stable.official
TEST_SCRIPT

test "gdvm.toml takes precedence over .gdvmrc" <<'TEST_SCRIPT'
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT
cd "$tmpdir"
printf '[godot]\nversion = "4.3-stable"\n' > gdvm.toml
printf '4.4-stable' > .gdvmrc
gdvm run --console=true -- --version | grep 4.3.stable.official
TEST_SCRIPT

test "Falls back to .gdvmrc when no gdvm.toml" <<'TEST_SCRIPT'
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT
cd "$tmpdir"
printf '4.3.0-stable' > .gdvmrc
gdvm run --console=true -- --version | grep 4.3.stable.official
TEST_SCRIPT

test "Link with csharp: variant specifier creates working file" <<'TEST_SCRIPT'
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT
cd "$tmpdir"
target="$(link_target ./godot-csharp-link)"
gdvm link csharp:4.3.0 "$target"
if [[ ! -e "$target" ]]; then
    echo "Link target $target was not created"
    exit 1
fi
run_linked ./godot-csharp-link --version | grep 4.3.stable.mono.official
TEST_SCRIPT

test "Remove with csharp: variant specifier" <<'TEST_SCRIPT'
gdvm remove csharp:4.3.0 --yes
if gdvm list | grep -qE '4\.3.*csharp'; then
    echo "Version should have been removed but still appears in list"
    exit 1
fi
TEST_SCRIPT

test 'Install with "stable" keyword' <<'TEST_SCRIPT'
gdvm install stable
gdvm run stable --console=true -- --version | grep stable.official
TEST_SCRIPT

test 'Install with "latest" keyword' <<'TEST_SCRIPT'
gdvm install latest
gdvm run latest --console=true -- --version | grep stable.official
TEST_SCRIPT

test ".env file variables are loaded by gdvm and passed to Godot" <<'TEST_SCRIPT'
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT
cd "$tmpdir"
gdvm pin 4.3.0

cat > check_env.gd << 'GDSCRIPT'
extends SceneTree

func _initialize():
    print("ENV_VALUE=" + OS.get_environment("GDVM_E2E_TEST_VAR"))
    quit(0)
GDSCRIPT

printf 'GDVM_E2E_TEST_VAR=hello_from_dotenv\n' > .env

output="$(gdvm run --console=true -- --headless --script check_env.gd 2>&1)"
echo "Script output: $output"

if ! echo "$output" | grep -q "ENV_VALUE=hello_from_dotenv"; then
    echo "Expected env var value not found in Godot output"
    exit 1
fi
TEST_SCRIPT

test "Link 4.3.0 to a custom path and run it" <<'TEST_SCRIPT'
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT
cd "$tmpdir"
target="$(link_target ./godot-link)"
gdvm link 4.3.0 "$target"
if [[ ! -e "$target" ]]; then
    echo "Link target $target was not created"
    exit 1
fi
run_linked ./godot-link --version | grep 4.3.stable.official
TEST_SCRIPT

test "Copy 4.3.0 to a custom path with --copy" <<'TEST_SCRIPT'
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT
cd "$tmpdir"
target="$(link_target ./godot-copy)"
gdvm link 4.3.0 "$target" --copy
if [[ ! -e "$target" ]]; then
    echo "Copy target $target was not created"
    exit 1
fi
run_linked ./godot-copy --version | grep 4.3.stable.official
TEST_SCRIPT

summarize_tests
