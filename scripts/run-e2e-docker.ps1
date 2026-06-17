#!/usr/bin/env pwsh
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

$ErrorActionPreference = "Stop"

# Build the e2e image and run tests inside Docker.

$image = if ($env:IMAGE) { $env:IMAGE } else { "gdvm-e2e" }

docker build -f Dockerfile.e2e -t $image .

$envVars = @()
if ($env:GITHUB_TOKEN) {
    $envVars += "-e"
    $envVars += "GITHUB_TOKEN=$($env:GITHUB_TOKEN)"
}
docker run --rm -ti $envVars $image
