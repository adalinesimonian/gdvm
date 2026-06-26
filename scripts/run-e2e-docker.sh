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

set -euo pipefail

# Build the e2e image and run tests inside Docker.

IMAGE="${IMAGE:-gdvm-e2e}"
CACHE="${GDVM_E2E_CACHE-gdvm-e2e-cache}"

docker build -f Dockerfile.e2e -t "$IMAGE" .

run_args=(--rm -ti)

if [[ -n "${GITHUB_TOKEN:-}" ]]; then
    run_args+=("-e" "GITHUB_TOKEN=$GITHUB_TOKEN")
fi

if [[ -z "${GDVM_E2E_NO_CACHE:-}" && -n "$CACHE" ]]; then
    if [[ "$CACHE" == /* ]]; then
        mkdir -p "$CACHE"
    fi

    echo "Persisting e2e cache."

    run_args+=("-v" "$CACHE:/root/.gdvm/cache")
else
    echo "e2e cache disabled."
fi

docker run "${run_args[@]}" "$IMAGE"
