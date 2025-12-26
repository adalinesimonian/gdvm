#!/usr/bin/env bash
set -euo pipefail

# Build the e2e image and run tests inside Docker.

IMAGE="${IMAGE:-gdvm-e2e}"

docker build -f Dockerfile.e2e -t "$IMAGE" .

env_vars=()
if [[ -n "${GITHUB_TOKEN:-}" ]]; then
    env_vars+=("-e" "GITHUB_TOKEN=$GITHUB_TOKEN")
fi

docker run --rm -ti "${env_vars[@]}" "$IMAGE"
