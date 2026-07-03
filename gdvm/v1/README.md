# gdvm release index (v1)

https://registry.gdvm.io/gdvm/v1/

This directory contains the v1 gdvm release index, which lists available versions of [gdvm](https://github.com/adalinesimonian/gdvm) itself.

Whenever a gdvm version is released, the [update-gdvm-releases.mts](update-gdvm-releases.mts) script is run by the [release workflow](https://github.com/adalinesimonian/gdvm/blob/main/.github/workflows/release.yml) to update the release index with the data for the new version.

## Structure

[`releases.json`](releases.json) lists all known gdvm releases from newest to oldest:

```json
{
  "schema": 1,
  "releases": [
    {
      "version": "0.12.1",
      "tag": "v0.12.1",
      "prerelease": false,
      "binaries": {
        "aarch64-apple-darwin": {
          "filename": "gdvm-aarch64-apple-darwin",
          "size": 10330512,
          "urls": [
            "https://github.com/adalinesimonian/gdvm/releases/download/v0.12.1/gdvm-aarch64-apple-darwin"
          ],
          "sha256": "..."
        },
        "x86_64-unknown-linux-gnu": {
          "filename": "gdvm-x86_64-unknown-linux-gnu",
          "size": 11454232,
          "urls": [
            "https://github.com/adalinesimonian/gdvm/releases/download/v0.12.1/gdvm-x86_64-unknown-linux-gnu"
          ],
          "sha256": "..."
        }
      }
    },
    ...
  ]
}
```

Each release entry includes:

- `version`: the release version
- `tag`: the Git tag
- `prerelease`: whether this is a pre-release or not
- `binaries`: a map of Rust targets to binary data

Each binary entry includes:

- `filename`: the asset filename
- `size`: the asset size in bytes
- `urls`: one or more download URLs
- `sha256`: SHA-256 checksum of the binary
- `provenance` (optional): build provenance data, if available

### Provenance

When available, each binary's `provenance` object contains:

- `attestation_url`: path to a [JSONL](https://jsonlines.org/) file relative to this directory, e.g. `v0.12.1/shahash.jsonl`

Each line in that file is a [Sigstore](https://www.sigstore.dev/) attestation bundle fetched from the [GitHub Artifact Attestations API](https://docs.github.com/en/actions/security-for-github-actions/using-artifact-attestations/using-artifact-attestations-to-establish-provenance-for-builds). The bundle can be verified with the [`gh attestation verify`](https://cli.github.com/manual/gh_attestation_verify) command or other tools that support Sigstore.
