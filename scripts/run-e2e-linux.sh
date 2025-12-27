#!/usr/bin/env bash
set -euo pipefail

# This script mirrors the end-to-end tests from the CI workflow.

if [[ -z "${GITHUB_TOKEN:-}" && -f /run/secrets/github_token ]]; then
    token_from_file="$(cat /run/secrets/github_token)"

    if [[ -n "$token_from_file" ]]; then
        echo "Using GitHub token from /run/secrets/github_token"
        export GITHUB_TOKEN="$token_from_file"
    else
        echo "Running without a GitHub token"
    fi
fi

gdvm --version

if [[ -z "${GITHUB_TOKEN:-}" ]]; then
    echo "Warning: GITHUB_TOKEN is not set. Some tests may fail due to rate limiting."
else
    echo "Configuring gdvm with provided GitHub token."
    gdvm config set github.token "$GITHUB_TOKEN"
fi

# Allow individual test bodies to fail without killing the whole suite.
set +e

workdir="$(mktemp -d)"
trap 'rm -rf "$workdir"' EXIT

test_index=1
tests_failed=0
declare -a test_numbers=()
declare -a test_descriptions=()
declare -a test_results=()
is_tty=$(tty >/dev/null 2>&1 && echo true || echo false)

test() {
    local desc="$1"
    local cmd=("${@:2}")

    echo ""
    echo "[$test_index] Running test: $desc"

    local status="failed"

    if ((${#cmd[@]} > 0)); then
        if "${cmd[@]}"; then
            status="passed"
        fi
    else
        local script
        script="$(cat)"

        if bash -euo pipefail -c "$script"; then
            status="passed"
        fi
    fi

    test_numbers+=("$test_index")
    test_descriptions+=("$desc")
    test_results+=("$status")

    if [[ "$status" == "passed" ]]; then
        if [[ "$is_tty" == "true" ]]; then
            echo -e "\033[0;32m[$test_index] Test passed: $desc\033[0m"
        else
            echo "[$test_index] Test passed: $desc"
        fi
    else
        if [[ "$is_tty" == "true" ]]; then
            echo -e "\033[0;31m[$test_index] Test failed: $desc\033[0m"
        else
            echo "[$test_index] Test failed: $desc"
        fi
        ((tests_failed++))
    fi

    ((test_index++))
}

summarize_tests() {
    local total_tests=${#test_results[@]}
    local passed_tests=$((total_tests - tests_failed))

    echo

    if [[ "$is_tty" == "true" ]]; then
        if ((passed_tests > 0)); then
            passed_msg="\033[0;32m$passed_tests passed\033[0m"
        else
            passed_msg="$passed_tests passed"
        fi
        if ((tests_failed > 0)); then
            failed_msg="\033[0;31m$tests_failed failed\033[0m"
        else
            failed_msg="$tests_failed failed"
        fi
        echo -e "Test summary: $passed_msg, $failed_msg, $total_tests total"
    else
        echo "Test summary: $passed_tests passed, $tests_failed failed, $total_tests total"
    fi

    if ((tests_failed > 0)); then
        echo "Failed tests:"
        for i in "${!test_results[@]}"; do
            if [[ "${test_results[$i]}" == "failed" ]]; then
                if [[ "$is_tty" == "true" ]]; then
                    echo -e "\033[0;31m - [${test_numbers[$i]}] ${test_descriptions[$i]}\033[0m"
                else
                    echo " - [${test_numbers[$i]}] ${test_descriptions[$i]}"
                fi
            fi
        done
        exit 1
    fi

    echo "E2E workflow completed successfully."
}

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

arch="$(uname -m)"
if [[ "$arch" == "aarch64" || "$arch" == "arm64" ]]; then
    test "Install Godot 4.4.0 on ARM" <<'TEST_SCRIPT'
gdvm install 4.4.0
gdvm use 4.4.0
TEST_SCRIPT
else
    test "Install Godot 3.6.2 on x86_64" <<'TEST_SCRIPT'
gdvm install 3.6.2
gdvm use 3.6.2
TEST_SCRIPT
fi

pushd "$workdir" >/dev/null

test "Pin Godot 4.3.0 and verify version" <<'TEST_SCRIPT'
gdvm pin 4.3.0
gdvm run --console=true -- --version | grep 4.3.stable.official
TEST_SCRIPT

test "Verify godot alias points to pinned version" <<'TEST_SCRIPT'
command -v godot >/dev/null 2>&1
godot --version | grep 4.3.stable.official
TEST_SCRIPT

test "Show without version resolves to pinned 4.3.0" <<'TEST_SCRIPT'
path="$(gdvm show)"
echo "gdvm show -> $path"
"$path" --version | grep 4.3.stable.official
TEST_SCRIPT

popd >/dev/null

if [[ "$arch" == "aarch64" || "$arch" == "arm64" ]]; then
    test "Verify Godot 4.4 on ARM" <<'TEST_SCRIPT'
gdvm run 4.4 --console=true -- --version | grep 4.4.1.stable.official
TEST_SCRIPT
else
    test "Verify Godot 3.5 on x86_64" <<'TEST_SCRIPT'
gdvm run 3.5 --console=true -- --version | grep 3.5.3.stable.official
TEST_SCRIPT
fi

test "Verify Godot 4.4 with C# support" <<'TEST_SCRIPT'
gdvm run 4.4 --csharp --console=true -- --version | grep 4.4.1.stable.mono.official
TEST_SCRIPT

test "Verify Godot stable version" <<'TEST_SCRIPT'
gdvm run stable --console=true -- --version | grep stable.official
TEST_SCRIPT

test "Link 4.3.0 to custom path and run it" <<'TEST_SCRIPT'
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT
cd "$tmpdir"
gdvm link 4.3.0 ./godot-link
if [[ ! -e ./godot-link ]]; then
    echo "Link target ./godot-link was not created"
    exit 1
fi
./godot-link --version | grep 4.3.stable.official
TEST_SCRIPT

test "Copy 4.3.0 to custom path with --copy" <<'TEST_SCRIPT'
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT
cd "$tmpdir"
gdvm link 4.3.0 ./godot-copy --copy
if [[ ! -e ./godot-copy ]]; then
    echo "Copy target ./godot-copy was not created"
    exit 1
fi
./godot-copy --version | grep 4.3.stable.official
TEST_SCRIPT

summarize_tests
