// SPDX-FileCopyrightText: Copyright (C) 2024 Adaline Simonian
// SPDX-License-Identifier: GPL-3.0-or-later
//
// This file is part of gdvm.
//
// gdvm is free software: you can redistribute it and/or modify it under the
// terms of the GNU General Public License as published by the Free Software
// Foundation, either version 3 of the License, or (at your option) any later
// version.
//
// gdvm is distributed in the hope that it will be useful, but WITHOUT ANY
// WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR
// A PARTICULAR PURPOSE. See the GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along with
// this program. If not, see <https://www.gnu.org/licenses/>.

use anyhow::{Result, anyhow, bail};
use digest_io::IoWrapper;
use serde::Serialize;
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha512};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::{fs, io};

use super::v2;
use crate::date_utils::now_iso8601;
use crate::version_utils::Variant;

/// Current registry schema version produced by the authoring commands.
const SCHEMA_VERSION: u32 = 2;

/// Parameters for `add_build`.
pub struct AddBuild {
    pub version: String,
    pub variant: Option<String>,
    pub platform: String,
    /// Local archive to store and/or hash. Optional with `url`.
    pub file: Option<PathBuf>,
    /// Copy the archive into the registry tree and record a relative URL.
    pub store: bool,
    /// Absolute URL where the archive is hosted. Required if `store` is false.
    pub url: Option<String>,
    /// Provided SHA-512 in hex, in lieu of hashing the artifact.
    pub sha512: Option<String>,
    /// Provided archive size in bytes, in lieu of measuring the artifact.
    pub size: Option<u64>,
}

/// Parameters for `remove_build`.
pub struct RemoveBuild {
    pub version: String,
    pub variant: Option<String>,
    pub platform: Option<String>,
}

/// The result of validating a registry.
pub struct ValidationReport {
    pub errors: Vec<String>,
    pub checked: usize,
}

impl ValidationReport {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Scaffold a new registry directory with a manifest and an empty index.
pub fn init(dir: &Path, name: Option<&str>) -> Result<String> {
    let manifest_path = dir.join("registry.json");
    if manifest_path.exists() {
        bail!("registry already initialized at {}", dir.display());
    }

    let name = name.map(|s| s.to_string()).unwrap_or_else(|| dir_name(dir));

    let manifest = v2::Manifest {
        schema: SCHEMA_VERSION,
        name: Some(name.clone()),
        description: None,
        updated_at: Some(now_iso8601()),
    };
    let index = v2::Index {
        schema: SCHEMA_VERSION,
        releases: Vec::new(),
    };

    write_json(&manifest_path, &manifest)?;
    write_json(&dir.join("index.json"), &index)?;

    Ok(name)
}

/// Add or replace a single platform binary for a version.
pub fn add_build(dir: &Path, args: &AddBuild) -> Result<()> {
    require_registry(dir)?;
    validate_segment(&args.version, "version")?;
    if let Some(variant) = &args.variant {
        validate_segment(variant, "variant")?;
    }
    validate_segment(&args.platform, "platform")?;

    let variant_key = Variant::from_option(args.variant.as_deref())
        .as_str()
        .to_string();

    let (url, sha512, size) = if args.store {
        let file = args
            .file
            .as_deref()
            .ok_or_else(|| anyhow!("--store requires a local --file"))?;
        if !file.is_file() {
            bail!("archive not found: {}", file.display());
        }
        let (sha512, size) = resolve_integrity(file, args)?;

        let rel = stored_binary_path(&args.version, &args.platform, &variant_key);
        let dest = dir.join(&rel);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(file, &dest)?;
        (rel, sha512, size)
    } else {
        let url = args
            .url
            .clone()
            .ok_or_else(|| anyhow!("either --store or --url must be provided"))?;
        let (sha512, size) = match (&args.sha512, args.size) {
            (Some(sha512), Some(size)) => (sha512.clone(), size),
            _ => {
                let file = args.file.as_deref().ok_or_else(|| {
                    anyhow!("--url requires either a local --file or explicit --sha512 and --size")
                })?;
                if !file.is_file() {
                    bail!("archive not found: {}", file.display());
                }
                resolve_integrity(file, args)?
            }
        };
        (url, sha512, size)
    };

    let rel_path = release_rel_path(&args.version);
    let release_path = dir.join(&rel_path);
    let mut release: v2::ReleaseMetadata =
        read_json(&release_path)?.unwrap_or_else(|| v2::ReleaseMetadata {
            schema: SCHEMA_VERSION,
            updated_at: None,
            version: args.version.clone(),
            variants: BTreeMap::new(),
        });
    release.schema = SCHEMA_VERSION;
    release.version = args.version.clone();
    release.variants.entry(variant_key).or_default().insert(
        args.platform.clone(),
        v2::BinaryInfo {
            sha512,
            size: Some(size),
            urls: vec![url],
        },
    );
    release.updated_at = Some(now_iso8601());
    write_json(&release_path, &release)?;

    upsert_index_entry(dir, &args.version, summarize(&release), rel_path)?;
    touch_manifest(dir)?;

    Ok(())
}

/// Remove a binary, a whole variant, or an entire version, then reconcile the index.
pub fn remove_build(dir: &Path, args: &RemoveBuild) -> Result<()> {
    require_registry(dir)?;

    let rel_path = release_rel_path(&args.version);
    let release_path = dir.join(&rel_path);
    let mut release: v2::ReleaseMetadata =
        read_json(&release_path)?.ok_or_else(|| anyhow!("no such version: {}", args.version))?;

    match (&args.variant, &args.platform) {
        (Some(variant), Some(platform)) => {
            let variant_key = Variant::from_option(Some(variant)).as_str().to_string();
            let removed = release
                .variants
                .get_mut(&variant_key)
                .is_some_and(|platforms| platforms.remove(platform).is_some());
            if !removed {
                bail!("no such platform {platform} for variant {variant_key}");
            }
            if release
                .variants
                .get(&variant_key)
                .is_some_and(|p| p.is_empty())
            {
                release.variants.remove(&variant_key);
            }
        }
        (Some(variant), None) => {
            let variant_key = Variant::from_option(Some(variant)).as_str().to_string();
            if release.variants.remove(&variant_key).is_none() {
                bail!("no such variant: {variant_key}");
            }
        }
        (None, _) => {
            release.variants.clear();
        }
    }

    let mut index = load_index(dir)?;
    if release.variants.is_empty() {
        let _ = fs::remove_file(&release_path);
        index.releases.retain(|r| r.version != args.version);
    } else {
        release.updated_at = Some(now_iso8601());
        write_json(&release_path, &release)?;
        let summary = summarize(&release);
        if let Some(entry) = index
            .releases
            .iter_mut()
            .find(|r| r.version == args.version)
        {
            entry.variants = summary;
            entry.path = rel_path;
        }
    }
    write_index(dir, &mut index)?;
    touch_manifest(dir)?;

    Ok(())
}

/// Validate a registry directory.
pub fn validate(dir: &Path) -> Result<ValidationReport> {
    let mut errors = Vec::new();
    let mut checked = 0usize;

    let manifest: Option<v2::Manifest> = read_json(&dir.join("registry.json"))?;
    match manifest {
        None => errors.push("missing registry.json".to_string()),
        Some(m) => {
            if m.schema != SCHEMA_VERSION {
                errors.push(format!(
                    "registry.json declares unsupported schema {} (expected {SCHEMA_VERSION})",
                    m.schema
                ));
            }
            if m.name.as_deref().unwrap_or("").is_empty() {
                errors.push("registry.json is missing a name".to_string());
            }
            if m.updated_at.as_deref().unwrap_or("").is_empty() {
                errors.push("registry.json is missing updated_at".to_string());
            }
        }
    }

    let index: Option<v2::Index> = read_json(&dir.join("index.json"))?;
    let Some(index) = index else {
        errors.push("missing index.json".to_string());
        return Ok(ValidationReport { errors, checked });
    };
    if index.schema != SCHEMA_VERSION {
        errors.push(format!(
            "index.json declares unsupported schema {} (expected {SCHEMA_VERSION})",
            index.schema
        ));
    }

    let actual_order: Vec<&str> = index.releases.iter().map(|r| r.version.as_str()).collect();
    let mut expected_order = actual_order.clone();
    expected_order.sort_by(|a, b| crate::version_utils::cmp_versions_newest_first(a, b));
    if actual_order != expected_order {
        errors.push("index.json releases are not ordered newest to oldest".to_string());
    }

    for entry in &index.releases {
        let release_path = dir.join(&entry.path);
        let release: Option<v2::ReleaseMetadata> = match read_json(&release_path) {
            Ok(r) => r,
            Err(e) => {
                errors.push(format!("{}: failed to parse ({e})", entry.path));
                continue;
            }
        };
        let Some(release) = release else {
            errors.push(format!("{}: release file missing", entry.path));
            continue;
        };

        if release.schema != SCHEMA_VERSION {
            errors.push(format!(
                "{}: unsupported schema {} (expected {SCHEMA_VERSION})",
                entry.path, release.schema
            ));
        }
        if release.version != entry.version {
            errors.push(format!(
                "{}: version '{}' does not match index '{}'",
                entry.path, release.version, entry.version
            ));
        }
        if release.updated_at.as_deref().unwrap_or("").is_empty() {
            errors.push(format!("{}: missing updated_at", entry.path));
        }

        if summarize(&release) != entry.variants {
            errors.push(format!(
                "{}: index variant/platform summary does not match release file",
                entry.version
            ));
        }

        for (variant, platforms) in &release.variants {
            for (platform, bin) in platforms {
                checked += 1;
                let where_ = format!("{} [{variant}/{platform}]", entry.version);
                if bin.urls.is_empty() {
                    errors.push(format!("{where_}: no urls"));
                    continue;
                }
                for url in &bin.urls {
                    if is_absolute_url(url) {
                        continue; // Remote artifact, cannot verify locally.
                    }
                    let artifact = dir.join(url);
                    if !artifact.is_file() {
                        errors.push(format!("{where_}: missing file {url}"));
                        continue;
                    }
                    match sha512_file(&artifact) {
                        Ok(actual) if actual != bin.sha512 => {
                            errors.push(format!("{where_}: sha512 mismatch for {url}"));
                        }
                        Ok(_) => {}
                        Err(e) => errors.push(format!("{where_}: {e}")),
                    }
                    match bin.size {
                        None => errors.push(format!("{where_}: missing size for local file {url}")),
                        Some(declared) => {
                            if let Ok(meta) = fs::metadata(&artifact)
                                && meta.len() != declared
                            {
                                errors.push(format!(
                                    "{where_}: size mismatch for {url} (declared {declared}, actual {})",
                                    meta.len()
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(ValidationReport { errors, checked })
}

fn require_registry(dir: &Path) -> Result<()> {
    if !dir.join("registry.json").is_file() {
        bail!("not a registry (no registry.json): {}", dir.display());
    }
    Ok(())
}

fn load_index(dir: &Path) -> Result<v2::Index> {
    Ok(
        read_json(&dir.join("index.json"))?.unwrap_or_else(|| v2::Index {
            schema: SCHEMA_VERSION,
            releases: Vec::new(),
        }),
    )
}

/// Order index releases, newest first.
fn sort_index(index: &mut v2::Index) {
    index
        .releases
        .sort_by(|a, b| crate::version_utils::cmp_versions_newest_first(&a.version, &b.version));
}

/// Sort the index newest first and write it to `index.json`.
fn write_index(dir: &Path, index: &mut v2::Index) -> Result<()> {
    index.schema = SCHEMA_VERSION;
    sort_index(index);
    write_json(&dir.join("index.json"), index)
}

/// Bump the registry manifest's `updated_at` to the current time.
fn touch_manifest(dir: &Path) -> Result<()> {
    let path = dir.join("registry.json");
    let mut manifest: v2::Manifest =
        read_json(&path)?.ok_or_else(|| anyhow!("missing registry.json"))?;
    manifest.schema = SCHEMA_VERSION;
    manifest.updated_at = Some(now_iso8601());
    write_json(&path, &manifest)
}

/// Insert or replace the index entry for a version.
fn upsert_index_entry(
    dir: &Path,
    version: &str,
    summary: BTreeMap<String, Vec<String>>,
    rel_path: String,
) -> Result<()> {
    let mut index = load_index(dir)?;
    if let Some(entry) = index.releases.iter_mut().find(|r| r.version == version) {
        entry.variants = summary;
        entry.path = rel_path;
    } else {
        index.releases.push(v2::IndexRelease {
            version: version.to_string(),
            variants: summary,
            path: rel_path,
        });
    }
    write_index(dir, &mut index)
}

/// Build a summary of the variant and platform keys for a release.
fn summarize(release: &v2::ReleaseMetadata) -> BTreeMap<String, Vec<String>> {
    release
        .variants
        .iter()
        .map(|(variant, platforms)| {
            let mut keys: Vec<String> = platforms.keys().cloned().collect();
            keys.sort();
            (variant.clone(), keys)
        })
        .collect()
}

fn release_rel_path(version: &str) -> String {
    format!("releases/{}.json", slug(version))
}

fn stored_binary_path(version: &str, platform: &str, variant_key: &str) -> String {
    let leaf = if variant_key == Variant::DEFAULT {
        platform.to_string()
    } else {
        format!("{platform}-{variant_key}")
    };
    format!("binaries/{}/{leaf}.zip", slug(version))
}

/// Make a version tag safe to use as a single path segment.
fn slug(version: &str) -> String {
    version
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '+') {
                c
            } else {
                '-'
            }
        })
        .collect()
}

fn validate_segment(value: &str, what: &str) -> Result<()> {
    if value.is_empty() || value.contains('/') || value.contains('\\') || value.contains("..") {
        bail!("invalid {what}: {value}");
    }
    Ok(())
}

fn is_absolute_url(url: &str) -> bool {
    url.starts_with("http://") || url.starts_with("https://") || url.starts_with("file://")
}

fn dir_name(dir: &Path) -> String {
    dir.file_name()
        .map(|s| s.to_string_lossy().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "registry".to_string())
}

fn sha512_file(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = IoWrapper(Sha512::new());
    io::copy(&mut file, &mut hasher)?;
    let digest = hasher.0.finalize();
    Ok(digest.iter().map(|b| format!("{b:02x}")).collect())
}

/// Hash a local archive, returning its SHA-512 in hex and size in bytes.
pub fn hash_file(path: &Path) -> Result<(String, u64)> {
    let sha512 = sha512_file(path)?;
    let size = fs::metadata(path)?.len();
    Ok((sha512, size))
}

/// Resolve `(sha512, size)` for a local archive.
fn resolve_integrity(file: &Path, args: &AddBuild) -> Result<(String, u64)> {
    let (sha512, size) = hash_file(file)?;
    Ok((
        args.sha512.clone().unwrap_or(sha512),
        args.size.unwrap_or(size),
    ))
}

fn read_json<T: DeserializeOwned>(path: &Path) -> Result<Option<T>> {
    if !path.exists() {
        return Ok(None);
    }
    let data = fs::read_to_string(path)?;
    Ok(Some(serde_json::from_str(&data)?))
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut json = serde_json::to_string_pretty(value)?;
    json.push('\n');
    fs::write(path, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write_archive(dir: &Path, name: &str, contents: &[u8]) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, contents).unwrap();
        path
    }

    #[test]
    fn init_creates_manifest_and_empty_index() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("reg");
        let name = init(&dir, Some("my-builds")).unwrap();
        assert_eq!(name, "my-builds");

        let manifest: v2::Manifest = read_json(&dir.join("registry.json")).unwrap().unwrap();
        assert_eq!(manifest.schema, 2);
        assert_eq!(manifest.name.as_deref(), Some("my-builds"));
        assert!(manifest.updated_at.is_some());

        let index: v2::Index = read_json(&dir.join("index.json")).unwrap().unwrap();
        assert!(index.releases.is_empty());

        assert!(init(&dir, None).is_err());
    }

    #[test]
    fn add_build_store_then_validate_roundtrips() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("reg");
        init(&dir, Some("r")).unwrap();

        let archive = write_archive(tmp.path(), "godot.zip", b"fake-archive-bytes");

        add_build(
            &dir,
            &AddBuild {
                version: "4.4-stable".to_string(),
                variant: None,
                platform: "linux-x86_64".to_string(),
                file: Some(archive.clone()),
                store: true,
                url: None,
                sha512: None,
                size: None,
            },
        )
        .unwrap();
        add_build(
            &dir,
            &AddBuild {
                version: "4.4-stable".to_string(),
                variant: Some("csharp".to_string()),
                platform: "linux-x86_64".to_string(),
                file: Some(archive.clone()),
                store: true,
                url: None,
                sha512: None,
                size: None,
            },
        )
        .unwrap();

        assert!(
            dir.join("binaries/4.4-stable/linux-x86_64.zip").is_file(),
            "default variant archive should be stored"
        );
        assert!(
            dir.join("binaries/4.4-stable/linux-x86_64-csharp.zip")
                .is_file(),
            "csharp archive should carry the variant suffix"
        );

        let index: v2::Index = read_json(&dir.join("index.json")).unwrap().unwrap();
        assert_eq!(index.releases.len(), 1);
        let entry = &index.releases[0];
        assert!(entry.variants.contains_key("default"));
        assert!(entry.variants.contains_key("csharp"));

        let report = validate(&dir).unwrap();
        assert!(report.is_valid(), "unexpected errors: {:?}", report.errors);
        assert_eq!(report.checked, 2);
    }

    #[test]
    fn validate_detects_sha_mismatch() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("reg");
        init(&dir, Some("r")).unwrap();
        let archive = write_archive(tmp.path(), "godot.zip", b"original");
        add_build(
            &dir,
            &AddBuild {
                version: "4.4-stable".to_string(),
                variant: None,
                platform: "linux-x86_64".to_string(),
                file: Some(archive),
                store: true,
                url: None,
                sha512: None,
                size: None,
            },
        )
        .unwrap();

        fs::write(
            dir.join("binaries/4.4-stable/linux-x86_64.zip"),
            b"tampered",
        )
        .unwrap();

        let report = validate(&dir).unwrap();
        assert!(!report.is_valid());
        assert!(
            report.errors.iter().any(|e| e.contains("sha512 mismatch")),
            "expected a sha512 mismatch, got: {:?}",
            report.errors
        );
    }

    #[test]
    fn remove_build_prunes_and_reconciles_index() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("reg");
        init(&dir, Some("r")).unwrap();
        let archive = write_archive(tmp.path(), "godot.zip", b"bytes");

        for variant in [None, Some("csharp".to_string())] {
            add_build(
                &dir,
                &AddBuild {
                    version: "4.4-stable".to_string(),
                    variant,
                    platform: "linux-x86_64".to_string(),
                    file: Some(archive.clone()),
                    store: true,
                    url: None,
                    sha512: None,
                    size: None,
                },
            )
            .unwrap();
        }

        remove_build(
            &dir,
            &RemoveBuild {
                version: "4.4-stable".to_string(),
                variant: Some("csharp".to_string()),
                platform: None,
            },
        )
        .unwrap();
        let index: v2::Index = read_json(&dir.join("index.json")).unwrap().unwrap();
        assert_eq!(index.releases.len(), 1);
        assert!(!index.releases[0].variants.contains_key("csharp"));
        assert!(index.releases[0].variants.contains_key("default"));

        remove_build(
            &dir,
            &RemoveBuild {
                version: "4.4-stable".to_string(),
                variant: None,
                platform: None,
            },
        )
        .unwrap();
        let index: v2::Index = read_json(&dir.join("index.json")).unwrap().unwrap();
        assert!(index.releases.is_empty());
        assert!(!dir.join("releases/4.4-stable.json").exists());
    }

    #[test]
    fn add_build_url_mode_records_absolute_url() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("reg");
        init(&dir, Some("r")).unwrap();
        let archive = write_archive(tmp.path(), "godot.zip", b"bytes");

        add_build(
            &dir,
            &AddBuild {
                version: "4.4-stable".to_string(),
                variant: None,
                platform: "linux-x86_64".to_string(),
                file: Some(archive),
                store: false,
                url: Some("https://cdn.example.com/godot.zip".to_string()),
                sha512: None,
                size: None,
            },
        )
        .unwrap();

        let release: v2::ReleaseMetadata = read_json(&dir.join("releases/4.4-stable.json"))
            .unwrap()
            .unwrap();
        let bin = &release.variants["default"]["linux-x86_64"];
        assert_eq!(
            bin.urls,
            vec!["https://cdn.example.com/godot.zip".to_string()]
        );

        let report = validate(&dir).unwrap();
        assert!(report.is_valid(), "unexpected errors: {:?}", report.errors);
    }

    #[test]
    fn add_build_rejects_path_traversal() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("reg");
        init(&dir, Some("r")).unwrap();
        let archive = write_archive(tmp.path(), "godot.zip", b"bytes");

        for (version, variant, platform) in [
            ("../evil", None, "linux-x86_64"),
            ("4.4-stable", Some("../evil".to_string()), "linux-x86_64"),
            ("4.4-stable", None, "../evil"),
        ] {
            let err = add_build(
                &dir,
                &AddBuild {
                    version: version.to_string(),
                    variant,
                    platform: platform.to_string(),
                    file: Some(archive.clone()),
                    store: true,
                    url: None,
                    sha512: None,
                    size: None,
                },
            );
            assert!(err.is_err(), "expected rejection for traversal input");
        }
    }

    fn add_stored(dir: &Path, tmp: &Path, version: &str) {
        let archive = write_archive(tmp, &format!("{}.zip", slug(version)), b"bytes");
        add_build(
            dir,
            &AddBuild {
                version: version.to_string(),
                variant: None,
                platform: "linux-x86_64".to_string(),
                file: Some(archive),
                store: true,
                url: None,
                sha512: None,
                size: None,
            },
        )
        .unwrap();
    }

    #[test]
    fn add_build_orders_index_newest_first() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("reg");
        init(&dir, Some("r")).unwrap();

        for v in ["4.3-stable", "4.4-stable", "4.4-rc1", "4.5-dev1"] {
            add_stored(&dir, tmp.path(), v);
        }

        let index: v2::Index = read_json(&dir.join("index.json")).unwrap().unwrap();
        let order: Vec<&str> = index.releases.iter().map(|r| r.version.as_str()).collect();
        assert_eq!(order, ["4.5-dev1", "4.4-stable", "4.4-rc1", "4.3-stable"]);

        assert!(validate(&dir).unwrap().is_valid());
    }

    #[test]
    fn add_build_sets_release_updated_at_and_touches_manifest() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("reg");
        init(&dir, Some("r")).unwrap();

        let manifest_before: v2::Manifest = read_json(&dir.join("registry.json")).unwrap().unwrap();

        add_stored(&dir, tmp.path(), "4.4-stable");

        let release: v2::ReleaseMetadata = read_json(&dir.join("releases/4.4-stable.json"))
            .unwrap()
            .unwrap();
        assert!(
            release.updated_at.as_deref().is_some_and(|s| !s.is_empty()),
            "release file should carry updated_at"
        );

        let manifest_after: v2::Manifest = read_json(&dir.join("registry.json")).unwrap().unwrap();
        assert!(
            manifest_after.updated_at.is_some(),
            "manifest should keep an updated_at after publishing"
        );
        let _ = manifest_before;
    }

    #[test]
    fn add_build_url_explicit_metadata_records_without_file() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("reg");
        init(&dir, Some("r")).unwrap();

        add_build(
            &dir,
            &AddBuild {
                version: "4.4-stable".to_string(),
                variant: None,
                platform: "linux-x86_64".to_string(),
                file: None,
                store: false,
                url: Some("https://cdn.example.com/godot.zip".to_string()),
                sha512: Some("ab".repeat(64)),
                size: Some(4096),
            },
        )
        .unwrap();

        let release: v2::ReleaseMetadata = read_json(&dir.join("releases/4.4-stable.json"))
            .unwrap()
            .unwrap();
        let bin = &release.variants["default"]["linux-x86_64"];
        assert_eq!(bin.sha512, "ab".repeat(64));
        assert_eq!(bin.size, Some(4096));
        assert_eq!(
            bin.urls,
            vec!["https://cdn.example.com/godot.zip".to_string()]
        );
    }

    #[test]
    fn add_build_url_without_source_or_metadata_errors() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("reg");
        init(&dir, Some("r")).unwrap();

        let err = add_build(
            &dir,
            &AddBuild {
                version: "4.4-stable".to_string(),
                variant: None,
                platform: "linux-x86_64".to_string(),
                file: None,
                store: false,
                url: Some("https://cdn.example.com/godot.zip".to_string()),
                sha512: None,
                size: None,
            },
        );
        assert!(
            err.is_err(),
            "url mode without a file or explicit metadata must be rejected at this layer"
        );
    }

    #[test]
    fn validate_flags_unordered_index_and_missing_updated_at() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("reg");
        init(&dir, Some("r")).unwrap();
        add_stored(&dir, tmp.path(), "4.3-stable");
        add_stored(&dir, tmp.path(), "4.5-stable");

        let mut index: v2::Index = read_json(&dir.join("index.json")).unwrap().unwrap();
        index.releases.sort_by(|a, b| a.version.cmp(&b.version));
        write_json(&dir.join("index.json"), &index).unwrap();

        let mut release: v2::ReleaseMetadata = read_json(&dir.join("releases/4.5-stable.json"))
            .unwrap()
            .unwrap();
        release.updated_at = None;
        write_json(&dir.join("releases/4.5-stable.json"), &release).unwrap();

        let report = validate(&dir).unwrap();
        assert!(!report.is_valid());
        assert!(
            report.errors.iter().any(|e| e.contains("not ordered")),
            "expected an ordering error, got: {:?}",
            report.errors
        );
        assert!(
            report.errors.iter().any(|e| e.contains("updated_at")),
            "expected a missing updated_at error, got: {:?}",
            report.errors
        );
    }
}
