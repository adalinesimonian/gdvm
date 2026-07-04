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

registry_platform() {
    case "${GDVM_E2E_TARGET:-}" in
    *apple-darwin)
        echo "macos-universal"
        return
        ;;
    aarch64-*windows*)
        echo "windows-arm64"
        return
        ;;
    i686-*windows* | i586-*windows* | i386-*windows*)
        echo "windows-x86"
        return
        ;;
    x86_64-*windows*)
        echo "windows-x86_64"
        return
        ;;
    aarch64-*)
        echo "linux-arm64"
        return
        ;;
    i686-* | i586-* | i386-*)
        echo "linux-x86"
        return
        ;;
    x86_64-*)
        echo "linux-x86_64"
        return
        ;;
    esac

    local a
    case "$arch" in
    arm) a="arm64" ;;
    i686 | i586 | i386) a="x86" ;;
    *) a="x86_64" ;;
    esac
    case "$os" in
    macos) echo "macos-universal" ;;
    windows) echo "windows-$a" ;;
    *) echo "linux-$a" ;;
    esac
}

to_url_path() {
    if [[ "$os" == "windows" ]]; then
        cygpath -m "$1"
    else
        printf '%s' "$1"
    fi
}

export -f godot_shim link_target run_linked registry_platform to_url_path

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

_assert_emit() {
    printf '      %s\n' "$*" >&2
}

_assert_header() {
    local name="$1" msg="${2:-}"
    print_color red "  ✗ ${name}${msg:+: ${msg}}" >&2
}

_assert_kv() {
    local label="$1" value="$2"
    if [[ "$value" == *$'\n'* ]]; then
        _assert_emit "${label}:"
        printf '%s\n' "$value" | sed 's/^/        | /' >&2
    else
        _assert_emit "${label}: ${value}"
    fi
}

fail() {
    _assert_header "assertion" "${1:-explicit failure}"
    return 1
}

assert_eq() {
    local expected="$1" actual="$2" msg="${3:-}"
    [[ "$expected" == "$actual" ]] && return 0

    _assert_header "assert_eq" "$msg"
    if [[ "$expected" == *$'\n'* || "$actual" == *$'\n'* ]]; then
        _assert_emit "values differ (- expected, + actual):"
        diff <(printf '%s\n' "$expected") <(printf '%s\n' "$actual") |
            sed 's/^/        /' >&2 || true
    else
        _assert_kv "expected" "$expected"
        _assert_kv "actual" "$actual"
    fi
    return 1
}

assert_ne() {
    local unexpected="$1" actual="$2" msg="${3:-}"
    [[ "$unexpected" != "$actual" ]] && return 0

    _assert_header "assert_ne" "$msg"
    _assert_kv "both values are" "$actual"
    return 1
}

assert_contains() {
    local haystack="$1" needle="$2" msg="${3:-}"
    [[ "$haystack" == *"$needle"* ]] && return 0

    _assert_header "assert_contains" "$msg"
    _assert_kv "expected substring" "$needle"
    _assert_kv "actual text" "$haystack"
    return 1
}

assert_not_contains() {
    local haystack="$1" needle="$2" msg="${3:-}"
    [[ "$haystack" != *"$needle"* ]] && return 0

    _assert_header "assert_not_contains" "$msg"
    _assert_kv "unexpected substring" "$needle"
    _assert_kv "actual text" "$haystack"
    return 1
}

assert_matches() {
    local text="$1" pattern="$2" msg="${3:-}"
    if grep -Eq -- "$pattern" <<<"$text"; then
        return 0
    fi

    _assert_header "assert_matches" "$msg"
    _assert_kv "expected to match (ERE)" "$pattern"
    _assert_kv "actual text" "$text"
    return 1
}

assert_not_matches() {
    local text="$1" pattern="$2" msg="${3:-}"
    if ! grep -Eq -- "$pattern" <<<"$text"; then
        return 0
    fi

    _assert_header "assert_not_matches" "$msg"
    _assert_kv "unexpected match (ERE)" "$pattern"
    _assert_kv "actual text" "$text"
    return 1
}

assert_imatches() {
    local text="$1" pattern="$2" msg="${3:-}"
    if grep -Eqi -- "$pattern" <<<"$text"; then
        return 0
    fi

    _assert_header "assert_imatches" "$msg"
    _assert_kv "expected to match (ERE, case-insensitive)" "$pattern"
    _assert_kv "actual text" "$text"
    return 1
}

assert_path_exists() {
    local path="$1" msg="${2:-}"
    [[ -e "$path" ]] && return 0

    _assert_header "assert_path_exists" "$msg"
    _assert_kv "expected path" "$path"
    _assert_kv "note" "path does not exist"
    return 1
}

assert_path_absent() {
    local path="$1" msg="${2:-}"
    [[ ! -e "$path" ]] && return 0

    _assert_header "assert_path_absent" "$msg"
    _assert_kv "unexpected path" "$path"
    if [[ -d "$path" ]]; then
        _assert_kv "note" "path exists and is a directory"
    elif [[ -f "$path" ]]; then
        _assert_kv "note" "path exists and is a regular file"
    else
        _assert_kv "note" "path exists"
    fi
    return 1
}

assert_file_exists() {
    local path="$1" msg="${2:-}"
    [[ -f "$path" ]] && return 0

    _assert_header "assert_file_exists" "$msg"
    _assert_kv "expected file" "$path"
    if [[ -e "$path" ]]; then
        _assert_kv "note" "path exists but is not a regular file"
    else
        _assert_kv "note" "path does not exist"
    fi
    return 1
}

assert_dir_exists() {
    local path="$1" msg="${2:-}"
    [[ -d "$path" ]] && return 0

    _assert_header "assert_dir_exists" "$msg"
    _assert_kv "expected directory" "$path"
    if [[ -e "$path" ]]; then
        _assert_kv "note" "path exists but is not a directory"
    else
        _assert_kv "note" "path does not exist"
    fi
    return 1
}

assert_succeeds() {
    [[ "${1:-}" == "--" ]] && shift

    local output status=0
    output="$("$@" 2>&1)" || status=$?
    [[ $status -eq 0 ]] && return 0

    _assert_header "assert_succeeds" ""
    _assert_kv "command" "$*"
    _assert_kv "exit status (wanted 0)" "$status"
    _assert_kv "output" "$output"
    return 1
}

assert_fails() {
    [[ "${1:-}" == "--" ]] && shift

    local output status=0
    output="$("$@" 2>&1)" || status=$?
    [[ $status -ne 0 ]] && return 0

    _assert_header "assert_fails" ""
    _assert_kv "command" "$*"
    _assert_kv "exit status (wanted non-0)" "$status"
    _assert_kv "output" "$output"
    return 1
}

assert_run_contains() {
    local needle="$1"
    shift
    [[ "${1:-}" == "--" ]] && shift

    local output status=0
    output="$("$@" 2>&1)" || status=$?
    if [[ $status -eq 0 && "$output" == *"$needle"* ]]; then
        return 0
    fi

    _assert_header "assert_run_contains" ""
    _assert_kv "command" "$*"
    _assert_kv "exit status (wanted 0)" "$status"
    _assert_kv "expected substring" "$needle"
    _assert_kv "actual output" "$output"
    return 1
}

assert_run_fails_with() {
    local needle="$1"
    shift
    [[ "${1:-}" == "--" ]] && shift

    local output status=0
    output="$("$@" 2>&1)" || status=$?
    if [[ $status -ne 0 && "$output" == *"$needle"* ]]; then
        return 0
    fi

    _assert_header "assert_run_fails_with" ""
    _assert_kv "command" "$*"
    _assert_kv "exit status (wanted non-zero)" "$status"
    _assert_kv "expected substring" "$needle"
    _assert_kv "actual output" "$output"
    return 1
}

export -f print_color
export -f _assert_emit _assert_header _assert_kv fail
export -f assert_eq assert_ne assert_contains assert_not_contains
export -f assert_matches assert_not_matches assert_imatches
export -f assert_path_exists assert_path_absent assert_file_exists assert_dir_exists
export -f assert_succeeds assert_fails assert_run_contains assert_run_fails_with

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
assert_run_contains 4.3-stable -- gdvm search --limit 0 --filter 4
TEST_SCRIPT

test "Refresh flag repopulates registry cache" <<'TEST_SCRIPT'
cache="$HOME/.gdvm/cache.json"
backup="$cache.bak.refresh"

restore_cache() {
    [[ -f "$backup" ]] && mv "$backup" "$cache"
}
trap restore_cache EXIT

if [[ -f "$cache" ]]; then
    cp "$cache" "$backup"
fi

printf '%s' '{"gdvm":{"last_update_check":0,"new_version":null,"new_major_version":null},"registries":{}}' > "$cache"

cache_only_out="$(gdvm search --cache-only --limit 0 --filter 4 || true)"
assert_not_contains "$cache_only_out" 4.3-stable \
    "cache-only search unexpectedly found releases with empty cache"
assert_run_contains 4.3-stable -- gdvm search --refresh --limit 0 --filter 4

cache_contents="$(cat "$cache")"
assert_contains "$cache_contents" '"tag_name"' \
    "cache was not repopulated with releases"
TEST_SCRIPT

test "Install Godot 4.3" gdvm install 4.3

test "Install uses a custom install path from config" <<'TEST_SCRIPT'
custom_root="$HOME/.gdvm-e2e-installs"
custom_installs="$custom_root/custom_installs"
custom_cache="$custom_root/custom_cache"
rm -rf "$custom_root"
mkdir -p "$custom_installs" "$custom_cache"

gdvm config set install.path "$custom_installs"
gdvm config set cache.path "$custom_cache"

gdvm install 4.3 >/tmp/gdvm-custom-path.log 2>&1

assert_dir_exists "$custom_installs" "custom install directory was not created"
assert_dir_exists "$custom_cache" "custom cache directory was not created"

install_dir="$(find "$custom_installs" -type d -path '*/default/4.3-stable' | head -n 1)"
if [[ -z "$install_dir" ]]; then
    fail "Godot install was not found under the configured install path"
fi
assert_dir_exists "$install_dir" "Godot install directory was not created"

cat /tmp/gdvm-custom-path.log
TEST_SCRIPT

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
toml="$(cat gdvm.toml)"
assert_contains "$toml" 'version = "default:4.3-stable"' \
    "gdvm.toml missing the explicit-variant version line"

# .gdvmrc should exist with the old pre-refactor format.
gdvmrc="$(cat .gdvmrc)"
assert_contains "$gdvmrc" '4.3.0-stable' \
    ".gdvmrc missing the legacy version string"

assert_run_contains 4.3.stable.official -- gdvm run --console=true -- --version
TEST_SCRIPT

test "Godot shim points to the pinned version" <<'TEST_SCRIPT'
assert_run_contains 4.3.stable.official -- godot_shim --version
TEST_SCRIPT

test "Show without version resolves to pinned 4.3.0" <<'TEST_SCRIPT'
path="$(gdvm show)"
echo "gdvm show -> $path"

assert_run_contains 4.3.stable.official -- "$path" --version
TEST_SCRIPT

popd >/dev/null || exit

if [[ "$arch" == "arm" ]]; then
    test "Run Godot 4.4 explicitly (ARM)" <<'TEST_SCRIPT'
assert_run_contains 4.4.1.stable.official -- gdvm run 4.4 --console=true -- --version
TEST_SCRIPT
else
    test "Run Godot 3.5 explicitly (x86)" <<'TEST_SCRIPT'
assert_run_contains 3.5.3.stable.official -- gdvm run 3.5 --console=true -- --version
TEST_SCRIPT
fi

test "Run Godot 4.4 with C# support (deprecated --csharp flag)" <<'TEST_SCRIPT'
assert_run_contains 4.4.1.stable.mono.official -- gdvm run 4.4 --csharp --console=true -- --version
TEST_SCRIPT

test "Run Godot 4.4 with C# support" <<'TEST_SCRIPT'
assert_run_contains 4.4.1.stable.mono.official -- gdvm run csharp:4.4 --console=true -- --version
TEST_SCRIPT

test "default: variant specifier selects the standard build" <<'TEST_SCRIPT'
# 4.3 standard was installed earlier; default:4.3 must resolve to it, not Mono.
assert_run_contains 4.3.stable.official -- gdvm run default:4.3 --console=true -- --version
TEST_SCRIPT

test 'Running "stable" keyword resolves to latest stable' <<'TEST_SCRIPT'
assert_run_contains stable.official -- gdvm run stable --console=true -- --version
TEST_SCRIPT

test 'Running "latest" keyword resolves to latest stable' <<'TEST_SCRIPT'
assert_run_contains stable.official -- gdvm run latest --console=true -- --version
TEST_SCRIPT

test "Running C# build via csharp:stable" <<'TEST_SCRIPT'
assert_run_contains stable.mono.official -- gdvm run csharp:stable --console=true -- --version
TEST_SCRIPT

test "Install with csharp: variant specifier" <<'TEST_SCRIPT'
gdvm install csharp:4.3

list_out="$(gdvm list)"
assert_matches "$list_out" '4\.3.*csharp' \
    "csharp 4.3 not shown in gdvm list after install"
TEST_SCRIPT

test "Show with csharp: variant specifier returns existing file" <<'TEST_SCRIPT'
path="$(gdvm show csharp:4.3.0)"
echo "gdvm show csharp:4.3.0 -> $path"

assert_path_exists "$path" "resolved csharp build path does not exist"

assert_run_contains 4.3.stable.mono.official -- "$path" --version
TEST_SCRIPT

test "Pin with csharp: variant specifier and verify version" <<'TEST_SCRIPT'
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT
cd "$tmpdir"

gdvm pin csharp:4.3.0

# gdvm.toml should carry the csharp variant prefix.
toml="$(cat gdvm.toml)"
assert_contains "$toml" 'version = "csharp:4.3-stable"' \
    "gdvm.toml missing the csharp variant prefix"

# .gdvmrc should have the old pre-refactor format.
gdvmrc="$(cat .gdvmrc)"
assert_contains "$gdvmrc" '4.3.0-stable-csharp' \
    ".gdvmrc missing the legacy csharp version string"

assert_run_contains 4.3.stable.mono.official -- gdvm run --console=true -- --version
TEST_SCRIPT

test "Use with csharp: variant specifier sets default" <<'TEST_SCRIPT'
gdvm use csharp:4.3.0

path="$(gdvm show)"
assert_run_contains 4.3.stable.mono.official -- "$path" --version
TEST_SCRIPT

test "Pin with --no-legacy skips .gdvmrc" <<'TEST_SCRIPT'
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT
cd "$tmpdir"

gdvm pin 4.3.0 --no-legacy

toml="$(cat gdvm.toml)"
assert_contains "$toml" 'version = "default:4.3-stable"' \
    "gdvm.toml missing the default variant version line"

assert_path_absent .gdvmrc ".gdvmrc should not exist with --no-legacy"

assert_run_contains 4.3.stable.official -- gdvm run --console=true -- --version
TEST_SCRIPT

test "gdvm.toml takes precedence over .gdvmrc" <<'TEST_SCRIPT'
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT
cd "$tmpdir"

printf '[godot]\nversion = "4.3-stable"\n' > gdvm.toml
printf '4.4-stable' > .gdvmrc

assert_run_contains 4.3.stable.official -- gdvm run --console=true -- --version
TEST_SCRIPT

test "Falls back to .gdvmrc when no gdvm.toml" <<'TEST_SCRIPT'
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT
cd "$tmpdir"

printf '4.3.0-stable' > .gdvmrc

assert_run_contains 4.3.stable.official -- gdvm run --console=true -- --version
TEST_SCRIPT

test "Link with csharp: variant specifier creates working file" <<'TEST_SCRIPT'
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT
cd "$tmpdir"

target="$(link_target ./godot-csharp-link)"
gdvm link csharp:4.3.0 "$target"

assert_path_exists "$target" "link target $target was not created"

assert_run_contains 4.3.stable.mono.official -- run_linked ./godot-csharp-link --version
TEST_SCRIPT

test "Remove with csharp: variant specifier" <<'TEST_SCRIPT'
gdvm remove csharp:4.3.0 --yes

list_out="$(gdvm list)"
assert_not_matches "$list_out" '4\.3.*csharp' \
    "csharp 4.3 should have been removed but still appears in list"
TEST_SCRIPT

test 'Install with "stable" keyword' <<'TEST_SCRIPT'
gdvm install stable

assert_run_contains stable.official -- gdvm run stable --console=true -- --version
TEST_SCRIPT

test 'Install with "latest" keyword' <<'TEST_SCRIPT'
gdvm install latest

assert_run_contains stable.official -- gdvm run latest --console=true -- --version
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

assert_contains "$output" "ENV_VALUE=hello_from_dotenv" \
    "expected env var value not found in Godot output"
TEST_SCRIPT

test "Link 4.3.0 to a custom path and run it" <<'TEST_SCRIPT'
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT
cd "$tmpdir"

target="$(link_target ./godot-link)"
gdvm link 4.3.0 "$target"

assert_path_exists "$target" "link target $target was not created"

assert_run_contains 4.3.stable.official -- run_linked ./godot-link --version
TEST_SCRIPT

test "Copy 4.3.0 to a custom path with --copy" <<'TEST_SCRIPT'
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT
cd "$tmpdir"

target="$(link_target ./godot-copy)"
gdvm link 4.3.0 "$target" --copy

assert_path_exists "$target" "copy target $target was not created"

assert_run_contains 4.3.stable.official -- run_linked ./godot-copy --version
TEST_SCRIPT

reg_state="$workdir/registry-under-test"

test "Publish a custom file: registry from a cached Godot archive" <<TEST_SCRIPT
set -euo pipefail

reg="\$(mktemp -d)/myreg"
echo "\$reg" > "$reg_state"
platform="\$(registry_platform)"

src_zip="\$(gdvm cache-path 4.3-stable)"
echo "Publishing \$src_zip as \$platform into \$reg"

gdvm registry init "\$reg" --name "E2E Builds"
gdvm registry add-build "\$reg" --version 4.3-stable --platform "\$platform" --file "\$src_zip" --store
gdvm registry validate "\$reg"

assert_file_exists "\$reg/binaries/4.3-stable/\$platform.zip" \
    "published binary archive was not created"
TEST_SCRIPT

test "Add the registry and see it listed" <<TEST_SCRIPT
set -euo pipefail

reg="\$(cat "$reg_state")"
gdvm registry add e2ereg "file://\$(to_url_path "\$reg")"

list_out="\$(gdvm registry list)"
assert_contains "\$list_out" e2ereg "registry e2ereg not listed after add"
TEST_SCRIPT

test "Declining the trust prompt aborts the install" <<'TEST_SCRIPT'
set -uo pipefail

status=0
printf 'no\n' | gdvm install e2ereg/4.3 >/dev/null 2>&1 || status=$?
assert_ne 0 "$status" "install should have aborted when trust was declined"

list_out="$(gdvm list)"
assert_not_contains "$list_out" e2ereg \
    "nothing should be installed after declining trust"
TEST_SCRIPT

test "Confirming trust installs from the custom registry" <<'TEST_SCRIPT'
set -euo pipefail

out="$(printf 'yes\n' | gdvm install e2ereg/4.3 2>&1)"
echo "$out"

assert_imatches "$out" 'custom registry' "trust warning was not shown"

assert_dir_exists "$HOME/.gdvm/installs/registry.gdvm.io-7999f4302078c203/default/4.3-stable" \
    "official install dir missing"
TEST_SCRIPT

test "Run a build from the custom registry" <<'TEST_SCRIPT'
set -euo pipefail

assert_run_contains 4.3.stable.official -- gdvm run e2ereg/4.3 --console=true -- --version
TEST_SCRIPT

test "Custom registry version shows in list with the registry prefix" <<'TEST_SCRIPT'
set -euo pipefail

list_out="$(gdvm list)"
assert_imatches "$list_out" 'e2ereg/4.3' \
    "custom registry version not shown with the registry prefix"
TEST_SCRIPT

test "Project gdvm.toml can define and use a registry without machine config" <<TEST_SCRIPT
set -euo pipefail

platform="\$(registry_platform)"
src_zip="\$(gdvm cache-path 4.3-stable)"

preg="\$(mktemp -d)/projreg"
gdvm registry init "\$preg" --name "Project Reg"
gdvm registry add-build "\$preg" --version 4.3-stable --platform "\$platform" --file "\$src_zip" --store

projdir="\$(mktemp -d)"
printf '[godot]\nversion = "projreg/4.3-stable"\n\n[registries.projreg]\nurl = "file://%s"\n' "\$(to_url_path "\$preg")" > "\$projdir/gdvm.toml"
cd "\$projdir"

reg_list="\$(gdvm registry list)"
assert_contains "\$reg_list" projreg "project-defined registry not listed"

out="\$(printf 'yes\n' | gdvm run --console=true -- --version 2>&1)"
echo "\$out"

assert_imatches "\$out" 'custom registry' \
    "trust prompt not shown for project-defined registry"
assert_contains "\$out" 4.3.stable.official \
    "project-defined registry did not run the expected build"

list_out="\$(gdvm list)"
assert_imatches "\$list_out" 'projreg/4.3' \
    "project registry build not shown in list after install"
TEST_SCRIPT

test "Project gdvm.toml that shadows a machine registry warns about the conflict" <<TEST_SCRIPT
set -euo pipefail

machine_reg="\$(mktemp -d)/dup-machine"
proj_reg="\$(mktemp -d)/dup-project"

gdvm registry init "\$machine_reg" --name "Machine Dup" >/dev/null
gdvm registry init "\$proj_reg" --name "Project Dup" >/dev/null
gdvm registry add dup "file://\$(to_url_path "\$machine_reg")"

projdir="\$(mktemp -d)"
printf '[registries.dup]\nurl = "file://%s"\n' "\$(to_url_path "\$proj_reg")" > "\$projdir/gdvm.toml"
cd "\$projdir"

out="\$(gdvm registry list 2>&1)"
echo "\$out"

assert_imatches "\$out" 'takes precedence' "conflict override warning not shown"
assert_contains "\$out" "\$(to_url_path "\$machine_reg")" "warning missing the machine URL"
assert_contains "\$out" "\$(to_url_path "\$proj_reg")" "warning missing the project URL"

clean_list="\$(gdvm registry list 2>/dev/null)"
assert_contains "\$clean_list" "\$(to_url_path "\$proj_reg")" "project registry should take precedence"

cd /
gdvm registry remove dup
TEST_SCRIPT

test "A malformed project gdvm.toml warns instead of being silently ignored" <<TEST_SCRIPT
set -euo pipefail

projdir="\$(mktemp -d)"
printf 'this is { not valid toml\n' > "\$projdir/gdvm.toml"
cd "\$projdir"

out="\$(gdvm registry list 2>&1)"
echo "\$out"

assert_imatches "\$out" 'could not be parsed' "malformed gdvm.toml was not reported"

proj_base="\$(basename "\$projdir")"
# gdvm prints the offending path in native form (backslashes on Windows); compare
# against a forward-slash tail after normalizing separators.
assert_contains "\${out//\\\\//}" "\$proj_base/gdvm.toml" "warning missing the offending path"

cd /
gdvm registry list >/dev/null
TEST_SCRIPT

test "remove-build prunes the version and validate stays green" <<TEST_SCRIPT
set -euo pipefail

reg="\$(cat "$reg_state")"

gdvm registry remove-build "\$reg" --version 4.3-stable
gdvm registry validate "\$reg"

index_contents="\$(cat "\$reg/index.json")"
assert_not_contains "\$index_contents" '4.3-stable' \
    "index still references the removed build"
TEST_SCRIPT

test "Remove the custom registry" <<'TEST_SCRIPT'
set -euo pipefail

gdvm registry remove e2ereg

list_out="$(gdvm registry list)"
assert_not_contains "$list_out" e2ereg "registry should have been removed"
TEST_SCRIPT

test "prune --dry-run does not change installed versions" <<'TEST_SCRIPT'
set -euo pipefail

before="$(gdvm list)"
gdvm prune --dry-run >/dev/null
after="$(gdvm list)"
assert_eq "$before" "$after" "dry run must not change installed versions"
TEST_SCRIPT

test "prune never removes the default install" <<'TEST_SCRIPT'
set -euo pipefail

gdvm install 4.3.0 >/dev/null
gdvm use 4.3.0

gdvm prune --all --force

list_out="$(gdvm list)"
assert_imatches "$list_out" '4\.3' "default install must survive prune --all --force"
assert_run_contains 4.3.stable.official -- gdvm run --console=true -- --version

gdvm use unset
TEST_SCRIPT

test "prune --all keeps a linked install and removes others" <<'TEST_SCRIPT'
set -euo pipefail

gdvm install 4.3.0 >/dev/null

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT
target="$(link_target "$tmpdir/godot-link")"
gdvm link 4.3.0 "$target"
gdvm install 4.4.0 >/dev/null

gdvm prune --all

list_out="$(gdvm list)"
assert_imatches "$list_out" '4\.3' "linked 4.3 must survive prune --all"
assert_not_matches "$list_out" '4\.4\.0' \
    "unlinked 4.4.0 should be removed by prune --all"
TEST_SCRIPT

test "prune --all --force removes even a linked install" <<'TEST_SCRIPT'
set -euo pipefail

gdvm install 4.3.0 >/dev/null

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT
target="$(link_target "$tmpdir/godot-link")"
gdvm link 4.3.0 "$target"

gdvm prune --all --force

list_out="$(gdvm list)"
assert_not_matches "$list_out" '4\.3' \
    "prune --all --force must remove even a linked install"
TEST_SCRIPT

summarize_tests
