#!/usr/bin/env pwsh
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
