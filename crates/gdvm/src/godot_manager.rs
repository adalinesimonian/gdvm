use crate::config::Config;
use crate::registry::{Registry, ReleaseMetadata};
use anyhow::{Result, anyhow};
#[cfg(target_family = "unix")]
use daemonize::Daemonize;
use directories::BaseDirs;
use i18n::I18n;
use indicatif::{ProgressBar, ProgressStyle};
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256, Sha512};
#[cfg(target_family = "unix")]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{env, fs};

use crate::download_utils::download_file;
use crate::migrations;
use crate::version_utils;
use crate::version_utils::GodotVersionDeterminate;
use crate::zip_utils;
use crate::{eprintln_i18n, println_i18n};
use crate::{i18n, project_version_detector, t, t_w};

use version_utils::{GodotVersion, GodotVersionDeterminateVecExt};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ReleaseCache {
    id: u64,
    tag_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct RegistryReleasesCache {
    /// Unix timestamp in seconds
    last_fetched: u64,
    releases: Vec<ReleaseCache>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GdvmCache {
    last_update_check: u64,
    new_version: Option<String>,
    new_major_version: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct FullCache {
    /// Cache for GDVM metadata
    gdvm: GdvmCache,
    /// Cache for Godot releases
    godot_registry: RegistryReleasesCache,
}

#[derive(Debug)]
pub enum InstallOutcome {
    Installed,
    AlreadyInstalled,
}

#[derive(Debug)]
enum GithubJsonError {
    Network(reqwest::Error),
    Api(anyhow::Error),
    Other(anyhow::Error),
}

impl std::fmt::Display for GithubJsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Network(e) => write!(f, "{e}"),
            Self::Api(e) | Self::Other(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for GithubJsonError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Network(e) => Some(e),
            Self::Api(e) | Self::Other(e) => Some(e.root_cause()),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum ShaType {
    Sha256,
    Sha512,
}

impl ShaType {
    /// Attempts to detect the SHA type based on the expected hash length.
    /// SHA256 produces 64 hex characters, SHA512 produces 128 hex characters.
    fn from_hash_length(hash: &str) -> Option<Self> {
        match hash.len() {
            64 => Some(ShaType::Sha256),
            128 => Some(ShaType::Sha512),
            _ => None,
        }
    }
}

/// GodotManager is a struct that manages the installation and running of Godot versions.
pub struct GodotManager<'a> {
    /// Path to GDVM base directory
    base_path: PathBuf,
    /// Path to directory to store installed Godot versions
    install_path: PathBuf,
    /// Path to cache.json
    cache_index_path: PathBuf,
    /// Path to directory to store cached zip files
    cache_path: PathBuf,
    /// Client for GitHub API requests
    client: reqwest::blocking::Client,
    /// Registry instance for fetching Godot releases
    registry: Registry,
    i18n: &'a I18n,
}

/// Verifies the SHA of a file against an expected hash.
fn verify_sha(file_path: &Path, expected: &str, i18n: &I18n) -> Result<()> {
    let sha_type = ShaType::from_hash_length(expected).ok_or_else(|| {
        anyhow!(t_w!(
            i18n,
            "error-invalid-sha-length",
            length = expected.len()
        ))
    })?;

    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}")?);
    pb.enable_steady_tick(Duration::from_millis(100));
    pb.set_message(t_w!(i18n, "verifying-checksum"));

    let mut f = fs::File::open(file_path)?;

    let local_hash = match sha_type {
        ShaType::Sha256 => {
            let mut hasher = Sha256::new();
            std::io::copy(&mut f, &mut hasher)?;
            format!("{:x}", hasher.finalize())
        }
        ShaType::Sha512 => {
            let mut hasher = Sha512::new();
            std::io::copy(&mut f, &mut hasher)?;
            format!("{:x}", hasher.finalize())
        }
    };

    if local_hash == expected.to_lowercase() {
        pb.finish_with_message(t_w!(i18n, "checksum-verified"));
        Ok(())
    } else {
        pb.finish_and_clear();
        Err(anyhow!(t!(
            i18n,
            "error-checksum-mismatch",
            file = file_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        )))
    }
}

/// Searches for the Godot executable within the given directory.
///
/// ## Arguments
///
/// - `version_dir` - A reference to the directory path where the search is performed.
/// - `console` - A boolean indicating whether to search for the console version (relevant for Windows).
///
/// ## Returns
///
/// - `Ok(Some(PathBuf))` containing the path to the Godot executable if found.
/// - `Ok(None)` if no executable is found.
/// - `Err(io::Error)` if there is an error reading the directory.
#[allow(unused_variables)]
pub fn find_godot_executable(version_dir: &Path, console: bool) -> Result<Option<PathBuf>> {
    // Collect all entries (files/folders) under version_dir
    let entries: Vec<_> = fs::read_dir(version_dir)?
        .filter_map(|entry| entry.ok())
        .collect();

    #[cfg(target_os = "windows")]
    {
        // If console is requested, try to find a "_console" executable first
        if console {
            let console_candidate = entries.iter().find_map(|entry| {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with("_console.exe") {
                    Some(entry.path())
                } else {
                    None
                }
            });

            // If found, return it; otherwise fall back to any .exe
            if console_candidate.is_some() {
                return Ok(console_candidate);
            }
        }

        // Either console was false, or no _console exe was found. Look for any .exe.
        let exe_candidate = entries.iter().find_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".exe") {
                Some(entry.path())
            } else {
                None
            }
        });

        Ok(exe_candidate)
    }

    #[cfg(target_os = "macos")]
    {
        // On macOS, look for Godot.app or Godot_mono.app
        let app_candidate = entries.iter().find_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            if name == "Godot.app" || name == "Godot_mono.app" {
                Some(entry.path())
            } else {
                None
            }
        });

        Ok(app_candidate)
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        // For Linux or other Unix-likes
        // Look for a few known suffixes or naming patterns
        let unix_candidate = entries.iter().find_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("Godot_v")
                || name.ends_with(".x86_64")
                || name.ends_with(".x86_32")
                || name.ends_with(".arm64")
            {
                Some(entry.path())
            } else {
                None
            }
        });

        Ok(unix_candidate)
    }
}

impl<'a> GodotManager<'a> {
    /// Create a new GodotManager instance and set up the installation and cache paths
    pub fn new(i18n: &'a I18n) -> Result<Self> {
        // For cross-platform user directory management:
        let base_dirs = BaseDirs::new().ok_or(anyhow!(t_w!(i18n, "error-find-user-dirs")))?;
        let base_path = base_dirs.home_dir().join(".gdvm");
        let install_path = base_path.join("installs");
        let cache_index_path = base_path.join("cache.json");
        let cache_path = base_path.join("cache");

        fs::create_dir_all(&install_path)?;
        fs::create_dir_all(&cache_path)?;

        let client = GodotManager::get_github_client(i18n)?;
        let registry = Registry::new()?;

        let manager = GodotManager {
            base_path,
            install_path,
            cache_index_path,
            cache_path,
            client,
            registry,
            i18n,
        };

        migrations::run_migrations(&manager.base_path, i18n)?;

        // Don't fail if update check fails, since it isn't critical
        manager.check_for_upgrades().ok();

        Ok(manager)
    }

    /// Gets the path to the GodotManager's base directory
    /// (e.g. `~/.gdvm` on Unix-like systems)
    pub fn get_base_path(&self) -> &Path {
        &self.base_path
    }

    /// Install a specified Godot version
    ///
    /// - `force`: If true, reinstall the version even if it's already installed.
    /// - `redownload`: If true, ignore cached zip files and download fresh ones.
    pub fn install(
        &self,
        gv: &GodotVersionDeterminate,
        force: bool,
        redownload: bool,
    ) -> Result<InstallOutcome> {
        let install_str = gv.to_install_str();
        let version_path = self.install_path.join(install_str);

        if version_path.exists() {
            if force {
                self.remove(gv)?;
                eprintln_i18n!(
                    self.i18n,
                    "force-reinstalling-version",
                    version = gv.to_display_str(),
                );
            } else {
                return Ok(InstallOutcome::AlreadyInstalled);
            }
        }

        if !gv.is_stable() {
            eprintln_i18n!(self.i18n, "warning-prerelease", branch = &gv.release_type);
        }

        fs::create_dir_all(&self.cache_path)?;

        let meta = self.get_release_metadata(gv)?;
        let is_csharp = gv.is_csharp.unwrap_or(false);
        let platform_key = if cfg!(target_os = "windows") {
            if is_csharp {
                "windows-csharp"
            } else {
                "windows"
            }
        } else if cfg!(target_os = "macos") {
            if is_csharp { "macos-csharp" } else { "macos" }
        } else if cfg!(target_os = "linux") {
            if is_csharp { "linux-csharp" } else { "linux" }
        } else {
            return Err(anyhow!(t_w!(self.i18n, "unsupported-platform")));
        };

        let arch_key = if cfg!(target_arch = "x86_64") {
            "x86_64"
        } else if cfg!(target_arch = "x86") {
            "x86"
        } else if cfg!(target_arch = "aarch64") {
            "arm64"
        } else {
            return Err(anyhow!(t_w!(self.i18n, "unsupported-architecture")));
        };

        let platform_map = meta
            .binaries
            .get(platform_key)
            .ok_or_else(|| anyhow!(t_w!(self.i18n, "unsupported-platform")))?;

        let arch_choice = if cfg!(target_os = "macos") && platform_map.contains_key("universal") {
            "universal"
        } else {
            arch_key
        };

        let binary = platform_map
            .get(arch_choice)
            .ok_or_else(|| anyhow!(t_w!(self.i18n, "unsupported-architecture")))?;

        let download_url = binary
            .urls
            .first()
            .ok_or_else(|| anyhow!(t_w!(self.i18n, "error-file-not-found")))?;
        let archive_name = download_url.split('/').next_back().unwrap_or("godot.zip");
        let cache_zip_path = self.cache_path.join(archive_name);

        if !redownload && cache_zip_path.exists() {
            eprintln_i18n!(self.i18n, "using-cached-zip");
        } else {
            if redownload && cache_zip_path.exists() {
                eprintln_i18n!(self.i18n, "force-redownload", version = gv.to_display_str());
            }

            let tmp_file = self
                .install_path
                .join(format!("{}.zip", gv.to_install_str()));

            // Download the archive
            download_file(download_url, &tmp_file, self.i18n)?;
            verify_sha(&tmp_file, &binary.sha512, self.i18n)?;

            // Move the verified zip to cache_dir
            fs::rename(&tmp_file, &cache_zip_path)?;
            eprintln_i18n!(self.i18n, "cached-zip-stored");
        }

        fs::create_dir_all(&version_path)?;

        // Extract from cache_zip_path
        zip_utils::extract_zip(&cache_zip_path, &version_path, self.i18n)?;

        Ok(InstallOutcome::Installed)
    }

    /// List all installed Godot versions
    pub fn list_installed(&self) -> Result<Vec<GodotVersionDeterminate>> {
        let mut versions = vec![];
        for entry in fs::read_dir(&self.install_path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                if let Ok(gv) = GodotVersion::from_install_str(&name) {
                    versions.push(gv.to_determinate())
                }
            }
        }
        versions.sort_by_version();
        Ok(versions)
    }

    /// Remove a specified Godot version
    pub fn remove(&self, gv: &GodotVersionDeterminate) -> Result<()> {
        let path = self.install_path.join(gv.to_install_str());

        if path.exists() {
            // If this version is the default, unset it
            if let Some(def) = self.get_default()? {
                if def.to_install_str() == path.file_name().unwrap().to_string_lossy() {
                    self.unset_default()?;
                }
            }
            fs::remove_dir_all(path)?;
            Ok(())
        } else {
            Err(anyhow!(t_w!(self.i18n, "error-version-not-found")))
        }
    }

    /// Run a specified Godot version
    pub fn run(
        &self,
        gv: &GodotVersionDeterminate,
        console: bool,
        godot_args: &[String],
    ) -> Result<()> {
        let version_dir = self.install_path.join(gv.to_install_str());
        if !version_dir.exists() {
            return Err(anyhow!(t!(
                self.i18n,
                "error-version-not-found",
                version = &gv.to_display_str(),
            )));
        }

        // Enumerate the version directory to find a Godot binary or app
        let godot_executable = find_godot_executable(&version_dir, console)?;

        let path = godot_executable.ok_or_else(|| {
            anyhow!(t!(
                self.i18n,
                "godot-executable-not-found",
                version = &gv.to_display_str(),
            ))
        })?;

        // Special handling for macOS .app bundles
        #[cfg(target_os = "macos")]
        {
            if path.extension().and_then(|ext| ext.to_str()) == Some("app") {
                let inner_path = path.join("Contents/MacOS/Godot");
                if !inner_path.exists() {
                    return Err(anyhow!(t!(
                        self.i18n,
                        "godot-executable-not-found",
                        version = &gv.to_display_str(),
                    )));
                }
                if console {
                    // On macOS, running attached is the default behavior
                    std::process::Command::new(inner_path)
                        .args(godot_args)
                        .spawn()?;
                } else {
                    // Detached process
                    std::process::Command::new(inner_path)
                        .args(godot_args)
                        .spawn()?;
                }
                return Ok(());
            }
        }

        if console {
            // Run the process attached to the terminal and wait for it to exit
            std::process::Command::new(&path)
                .args(godot_args)
                .stdin(std::process::Stdio::inherit())
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .status()?;
        } else {
            // Detached process configuration
            #[cfg(target_family = "unix")]
            {
                Daemonize::new().start().map_err(|e| {
                    anyhow!(t!(self.i18n, "error-starting-godot", error = e.to_string(),))
                })?;
                std::process::Command::new(&path).args(godot_args).spawn()?;
            }

            #[cfg(target_family = "windows")]
            {
                use std::os::windows::process::CommandExt;
                use winapi::um::winbase::DETACHED_PROCESS;
                std::process::Command::new(&path)
                    .args(godot_args)
                    .creation_flags(DETACHED_PROCESS)
                    .spawn()?;
            }
        }

        Ok(())
    }

    /// Fetch available releases from GitHub with caching
    pub fn fetch_available_releases(
        &self,
        filter: &Option<GodotVersion>,
        use_cache_only: bool,
    ) -> Result<Vec<GodotVersionDeterminate>> {
        let cache_duration = Duration::from_secs(48 * 3600); // 48 hours
        let filter = filter.as_ref();

        let mut cache = self.load_cache()?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| anyhow!("System time before UNIX EPOCH"))?
            .as_secs();

        let cache_age = now - cache.last_fetched;
        let is_time_to_refresh_index = cache_age > cache_duration.as_secs();

        if is_time_to_refresh_index && !use_cache_only {
            // Fetch from GitHub and update cache.
            if let Err(error) = self.update_cache(&mut cache) {
                if cache.releases.is_empty() {
                    eprintln_i18n!(
                        self.i18n,
                        "error-fetching-releases",
                        error = error.to_string()
                    );
                    return Err(error);
                } else {
                    // If we have cached releases, just reference them.
                    eprintln_i18n!(
                        self.i18n,
                        "warning-fetching-releases-using-cache",
                        error = error.to_string()
                    );
                }
            }
        }

        // Filter releases
        let mut filtered_releases: Vec<GodotVersionDeterminate> = cache
            .releases
            .iter()
            .filter_map(|r| GodotVersion::from_remote_str(&r.tag_name, None).ok())
            .map(|gv| gv.to_determinate())
            .filter(|r| filter.is_none_or(|f| f.matches(r)))
            .collect();

        // If no releases found and not using cache only, try fetching if we haven't already
        if filtered_releases.is_empty() && !use_cache_only && !is_time_to_refresh_index {
            // self.update_cache(&mut cache)?;
            filtered_releases = cache
                .releases
                .iter()
                .filter_map(|r| GodotVersion::from_remote_str(&r.tag_name, None).ok())
                .map(|gv| gv.to_determinate())
                .filter(|r| filter.is_none_or(|f| f.matches(r)))
                .collect();
        }

        // Sort releases
        filtered_releases.sort_by_version();

        Ok(filtered_releases)
    }

    /// Gets a reqwest client with the GitHub token if available
    fn get_github_client(i18n: &I18n) -> Result<reqwest::blocking::Client> {
        let token = env::var("GITHUB_TOKEN").ok().or_else(|| {
            let config = Config::load(i18n).ok()?;
            config.github_token
        });
        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(token) = token {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                format!("token {token}").parse()?,
            );
        }
        Ok(reqwest::blocking::ClientBuilder::new()
            .default_headers(headers)
            .user_agent("gdvm")
            .build()?)
    }

    /// Tries to query the GitHub API with a GET at a given URL. If it fails due
    /// to a rate-limit, it will return an error after printing a message.
    fn get_github_json(&self, url: &str) -> Result<serde_json::Value, GithubJsonError> {
        // Rate limits are 403s with a JSON object that has a "message" key that
        // starts with "API rate limit exceeded".

        let resp = self
            .client
            .get(url)
            .timeout(Duration::from_secs(3))
            .send()
            .map_err(GithubJsonError::Network)?;

        if resp.status().is_success() {
            let json: serde_json::Value =
                resp.json().map_err(|e| GithubJsonError::Other(e.into()))?;
            return Ok(json);
        }

        let status = resp.status();
        let json: serde_json::Value = resp.json().map_err(|e| GithubJsonError::Other(e.into()))?;
        if status == reqwest::StatusCode::FORBIDDEN {
            if let Some(message) = json.get("message").and_then(|m| m.as_str()) {
                if message.starts_with("API rate limit exceeded") {
                    return Err(GithubJsonError::Api(anyhow!(t_w!(
                        self.i18n,
                        "error-github-rate-limit"
                    ))));
                }
            }
        }

        let error_message = json.get("message").and_then(|m| m.as_str());

        Err(GithubJsonError::Api(anyhow!(t_w!(
            self.i18n,
            "error-github-api",
            error = error_message,
        ))))
    }

    /// Update the cache by fetching from the registry and updating `last_fetched`
    fn update_cache(&self, cache: &mut RegistryReleasesCache) -> Result<()> {
        let pb = ProgressBar::new_spinner();
        pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}")?);
        pb.enable_steady_tick(Duration::from_millis(100));
        pb.set_message(t_w!(self.i18n, "fetching-releases"));

        let index = self.registry.fetch_index()?;

        pb.finish_with_message(t_w!(self.i18n, "releases-fetched"));

        cache.releases = index
            .into_iter()
            .map(|r| ReleaseCache {
                id: r.id,
                tag_name: r.name,
            })
            .collect();
        cache.last_fetched = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| anyhow!("System time before UNIX EPOCH"))?
            .as_secs();

        self.save_registry_cache(cache)?;

        Ok(())
    }

    fn get_release_metadata(&self, gv: &GodotVersionDeterminate) -> Result<ReleaseMetadata> {
        let tag = gv.to_remote_str();
        let mut cache = self.load_cache()?;
        if let Some(entry) = cache.releases.iter().find(|r| r.tag_name == tag) {
            return self.registry.fetch_release(entry.id, &entry.tag_name);
        }

        self.update_cache(&mut cache)?;
        if let Some(entry) = cache.releases.iter().find(|r| r.tag_name == tag) {
            return self.registry.fetch_release(entry.id, &entry.tag_name);
        }

        Err(anyhow!(t_w!(self.i18n, "error-version-not-found")))
    }

    fn load_full_cache(&self) -> Result<FullCache> {
        if self.cache_index_path.exists() {
            let data = fs::read_to_string(&self.cache_index_path)?;
            match serde_json::from_str::<FullCache>(&data) {
                Ok(full) => Ok(full),
                Err(_) => {
                    // Overwrite with a default FullCache if corrupted
                    let empty_full = FullCache {
                        gdvm: GdvmCache {
                            last_update_check: 0,
                            new_version: None,
                            new_major_version: None,
                        },
                        godot_registry: RegistryReleasesCache {
                            last_fetched: 0,
                            releases: vec![],
                        },
                    };
                    self.save_full_cache(&empty_full)?;
                    Ok(empty_full)
                }
            }
        } else {
            Ok(FullCache {
                gdvm: GdvmCache {
                    last_update_check: 0,
                    new_version: None,
                    new_major_version: None,
                },
                godot_registry: RegistryReleasesCache {
                    last_fetched: 0,
                    releases: vec![],
                },
            })
        }
    }

    fn save_full_cache(&self, full: &FullCache) -> Result<()> {
        let data = serde_json::to_string(full)?;
        fs::write(&self.cache_index_path, data)?;
        Ok(())
    }

    // Load the release cache from cache.json
    fn load_cache(&self) -> Result<RegistryReleasesCache> {
        let full = self.load_full_cache()?;
        Ok(full.godot_registry)
    }

    fn save_registry_cache(&self, cache: &RegistryReleasesCache) -> Result<()> {
        let mut full = self.load_full_cache()?;
        full.godot_registry = cache.clone();
        self.save_full_cache(&full)
    }

    fn load_gdvm_cache(&self) -> Result<GdvmCache> {
        let full = self.load_full_cache()?;
        Ok(full.gdvm)
    }

    fn save_gdvm_cache(&self, cache: &GdvmCache) -> Result<()> {
        let mut full = self.load_full_cache()?;
        full.gdvm = cache.clone();
        self.save_full_cache(&full)
    }

    /// Clears the release cache by deleting the cache file and all cached zip files
    pub fn clear_cache(&self) -> Result<()> {
        if self.cache_index_path.exists() {
            fs::remove_file(&self.cache_index_path)?;
            println_i18n!(self.i18n, "cache-metadata-removed");
        } else {
            println_i18n!(self.i18n, "no-cache-metadata-found");
        }

        if self.cache_path.exists() {
            for entry in fs::read_dir(&self.cache_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    fs::remove_file(&path)?;
                }
            }
            println_i18n!(self.i18n, "cache-files-removed");
        } else {
            println_i18n!(self.i18n, "no-cache-files-found");
        }
        Ok(())
    }

    /// Fetch the latest stable Godot version
    pub fn get_latest_stable_version(&self) -> Result<GodotVersionDeterminate> {
        let stable_version = GodotVersion {
            release_type: Some("stable".to_string()),
            ..Default::default()
        };
        let releases = self.fetch_available_releases(&Some(stable_version), false)?;
        // Assuming releases are sorted latest first
        releases
            .iter()
            .find(|r| r.release_type == "stable")
            .cloned()
            .ok_or_else(|| anyhow!(t_w!(self.i18n, "error-no-stable-releases-found")))
    }

    /// Resolve the Godot version from a string, for an installed version
    /// Returns a list of possible versions. If the input is ambiguous, the list
    /// will have more than one element. Otherwise, it will have one element,
    /// unless of course the version is not found, in which case the list will
    /// be empty.
    /// Accepts full and partial versions.
    pub fn resolve_installed_version<T>(&self, gv: &T) -> Result<Vec<GodotVersionDeterminate>>
    where
        T: Into<GodotVersion> + Clone,
    {
        let gv: GodotVersion = gv.clone().into();
        let installed = self.list_installed()?;
        let mut matches: Vec<GodotVersionDeterminate> =
            installed.into_iter().filter(|v| gv.matches(v)).collect();

        matches.sort_by_version();
        Ok(matches)
    }

    /// Resolve the Godot version from a string, for an available version
    /// Returns a single version, whichever is the latest that matches the
    /// input.
    /// Accepts full and partial versions.
    pub fn resolve_available_version<T>(
        &self,
        gv: &T,
        use_cache_only: bool,
    ) -> Option<GodotVersionDeterminate>
    where
        T: Into<GodotVersion> + Clone,
    {
        let gv: GodotVersion = gv.clone().into();

        let available_releases = self
            .fetch_available_releases(&Some(gv), use_cache_only)
            .ok()?;

        // If some releases were stable, prefer the latest stable release

        let latest_stable = available_releases
            .iter()
            .find(|r| r.release_type == "stable")
            .cloned();

        if let Some(latest_stable) = latest_stable {
            return Some(latest_stable);
        }

        // If no stable releases were found, return the latest release

        available_releases.into_iter().next()
    }

    pub fn set_default(&self, gv: &GodotVersionDeterminate) -> Result<()> {
        // Check if the version exists
        let version_str = gv.to_install_str();
        let version_path = self.install_path.join(&version_str);
        if !version_path.exists() {
            return Err(anyhow!(t_w!(self.i18n, "error-version-not-found")));
        }

        // Write version to .gdvm/default
        let default_path = self.base_path.join("default");
        fs::write(&default_path, &version_str)?;

        // Create directory symlink .gdvm/bin/current_godot -> .gdvm/<version_str>/
        let symlink_dir = self.base_path.join("bin").join("current_godot");
        let target_dir = self.install_path.join(version_str);

        // Make sure bin directory exists
        fs::create_dir_all(symlink_dir.parent().unwrap())?;

        if symlink_dir.exists() {
            fs::remove_dir_all(&symlink_dir)?;
        }
        #[cfg(target_family = "unix")]
        std::os::unix::fs::symlink(&target_dir, &symlink_dir)?;
        #[cfg(target_family = "windows")]
        if let Err(e) = std::os::windows::fs::symlink_dir(&target_dir, &symlink_dir) {
            if e.raw_os_error() == Some(1314) {
                return Err(anyhow!(t_w!(self.i18n, "error-create-symlink-windows")));
            }
            return Err(anyhow!(e));
        }

        Ok(())
    }

    pub fn unset_default(&self) -> Result<()> {
        // Remove default file and symlink
        let default_file = self.base_path.join("default");
        if default_file.exists() {
            fs::remove_file(default_file)?;
        }

        let symlink_dir = self.base_path.join("bin").join("current_godot");
        if symlink_dir.exists() {
            fs::remove_dir_all(symlink_dir)?;
        }

        Ok(())
    }

    pub fn get_default(&self) -> Result<Option<GodotVersionDeterminate>> {
        let default_file = self.base_path.join("default");
        if default_file.exists() {
            let contents = fs::read_to_string(&default_file)?;
            Ok(Some(
                GodotVersion::from_install_str(contents.trim())?.to_determinate(),
            ))
        } else {
            Ok(None)
        }
    }

    /// Recursively search upward for .gdvmrc, return the pinned version if found
    pub fn get_pinned_version(&self) -> Option<GodotVersion> {
        let mut current = std::env::current_dir().ok()?;
        loop {
            let candidate = current.join(".gdvmrc");
            if candidate.is_file() {
                if let Ok(contents) = fs::read_to_string(&candidate) {
                    return GodotVersion::from_install_str(contents.trim()).ok();
                }
            }
            if !current.pop() {
                break;
            }
        }
        None
    }

    /// Try to determine the version to use based on the current Godot project
    pub fn determine_version<P: AsRef<Path>>(&self, path: Option<P>) -> Option<GodotVersion> {
        let current_dir = match path {
            Some(p) => p.as_ref().to_path_buf(),
            None => std::env::current_dir().ok()?,
        };

        project_version_detector::detect_godot_version_in_path(self.i18n, &current_dir)
    }

    /// Pin a version to .gdvmrc in the current directory
    pub fn pin_version(&self, gv: &GodotVersionDeterminate) -> Result<()> {
        let path = std::env::current_dir()?;
        let file = path.join(".gdvmrc");
        fs::write(&file, gv.to_pinned_str())?;
        Ok(())
    }

    pub fn auto_install_version<T>(&self, gv: &T) -> Result<GodotVersionDeterminate>
    where
        T: Into<GodotVersion> + Clone,
    {
        let gv: GodotVersion = gv.clone().into();

        // If user requested "stable", resolve it similarly to install logic
        let mut actual_version: GodotVersionDeterminate = if gv.is_stable() && gv.major.is_none() {
            self.get_latest_stable_version()?
        } else if gv.is_incomplete() {
            self.resolve_available_version(&gv, false)
                .ok_or_else(|| anyhow!(t_w!(self.i18n, "error-version-not-found")))?
        } else {
            gv.clone().into()
        };

        actual_version.is_csharp = gv.is_csharp;

        // Check if version is installed, if not, install
        if !self.is_version_installed(&actual_version)? {
            eprintln_i18n!(
                self.i18n,
                "auto-installing-version",
                version = &actual_version.to_display_str(),
            );
            self.install(&actual_version, false, false)?;
        }
        Ok(actual_version)
    }

    fn is_version_installed<T>(&self, gv: &T) -> Result<bool>
    where
        T: Into<GodotVersion> + Clone,
    {
        let gv: GodotVersion = gv.clone().into();

        let installed_versions = self.list_installed()?;
        Ok(installed_versions.iter().any(|v| gv.matches(v)))
    }

    /// Find the latest stable release matching a semver requirement from a list of GitHub releases
    fn find_latest_stable_release(
        &self,
        releases: &serde_json::Value,
        version_req: &str,
    ) -> Result<Option<String>> {
        let req = VersionReq::parse(version_req)
            .map_err(|e| anyhow!("Invalid version requirement '{}': {}", version_req, e))?; // Should never fail.

        let releases_array = releases
            .as_array()
            .ok_or_else(|| anyhow!("Expected releases to be an array"))?; // Should never fail.

        let mut matching_versions = Vec::new();

        for release in releases_array {
            // Skip drafts and prereleases.
            if release
                .get("draft")
                .and_then(|d| d.as_bool())
                .unwrap_or(false)
                || release
                    .get("prerelease")
                    .and_then(|p| p.as_bool())
                    .unwrap_or(false)
            {
                continue;
            }

            let tag_name = release
                .get("tag_name")
                .and_then(|t| t.as_str())
                .ok_or_else(|| anyhow!("Release missing tag_name"))?; // Should never fail.

            // Parse version, expecting "vM.m.p" format.
            if let Ok(version) = Version::parse(tag_name.trim_start_matches('v')) {
                // Only consider stable versions that match the requirement.
                if version.pre.is_empty() && req.matches(&version) {
                    matching_versions.push((version, tag_name.to_string()));
                }
            }
        }

        // Sort by version, newest first, and return the tag name of the latest match.
        matching_versions.sort_by(|a, b| b.0.cmp(&a.0));
        Ok(matching_versions.first().map(|(_, tag)| tag.clone()))
    }

    pub fn check_for_upgrades(&self) -> Result<()> {
        // Load or initialize gdvm cache
        let gdvm_cache = self.load_gdvm_cache()?;

        // Check for updates
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| anyhow!("System time before UNIX EPOCH"))? // Should never fail.
            .as_secs();
        let cache_duration = Duration::from_secs(48 * 3600); // 48 hours
        let cache_age = now - gdvm_cache.last_update_check;

        if cache_age > cache_duration.as_secs() {
            let progress = ProgressBar::new_spinner();
            progress
                .set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}")?);
            progress.enable_steady_tick(Duration::from_millis(100));
            progress.set_message(t_w!(self.i18n, "checking-updates"));

            let mut new_version = None;
            let mut new_major_version = None;

            let releases = match self
                .get_github_json("https://api.github.com/repos/adalinesimonian/gdvm/releases")
            {
                Ok(json) => json,
                Err(e) => {
                    progress.finish_and_clear();
                    if matches!(e, GithubJsonError::Api(_)) {
                        eprintln!("{e}");
                    } else {
                        self.save_gdvm_cache(&GdvmCache {
                            last_update_check: now,
                            new_version: None,
                            new_major_version: None,
                        })?;
                    }
                    return Err(e.into());
                }
            };

            // Get current version and determine major version for upgrade compatibility.
            let current_version = Version::parse(env!("CARGO_PKG_VERSION"))?;
            let major_version = current_version.major;

            // Check for updates within current major version.
            let version_req = format!("^{major_version}");
            if let Some(latest_stable_tag) =
                self.find_latest_stable_release(&releases, &version_req)?
            {
                let latest_version = Version::parse(latest_stable_tag.trim_start_matches('v'))?;
                if latest_version > current_version {
                    new_version = Some(latest_stable_tag);
                }
            }

            // Check for updates across all versions.
            if let Some(latest_major_tag) = self.find_latest_stable_release(&releases, "*")? {
                let latest_major_version =
                    Version::parse(latest_major_tag.trim_start_matches('v'))?;
                if latest_major_version > current_version {
                    // Only set new_major_version if it's different from new_version.
                    if new_version.as_ref() != Some(&latest_major_tag) {
                        new_major_version = Some(latest_major_tag);
                    }
                }
            }

            progress.finish_and_clear();

            // Display appropriate message based on available updates.
            if let (Some(minor_ver), Some(major_ver)) = (&new_version, &new_major_version) {
                eprint!("\x1b[1;32m"); // Bold and green
                eprintln_i18n!(
                    self.i18n,
                    "upgrade-available-both",
                    minor_version = minor_ver,
                    major_version = major_ver
                );
                eprint!("\x1b[0m"); // Reset
                eprintln!();
            } else if let Some(new_ver) = &new_version {
                eprint!("\x1b[1;32m"); // Bold and green
                eprintln_i18n!(self.i18n, "upgrade-available", version = new_ver);
                eprint!("\x1b[0m"); // Reset
                eprintln!();
            } else if let Some(major_ver) = &new_major_version {
                eprint!("\x1b[1;32m"); // Bold and green
                eprintln_i18n!(self.i18n, "upgrade-available-major", version = major_ver);
                eprint!("\x1b[0m"); // Reset
                eprintln!();
            }

            self.save_gdvm_cache(&GdvmCache {
                last_update_check: now,
                new_version,
                new_major_version,
            })?;
        } else if let Some(new_version) = &gdvm_cache.new_version {
            if let Ok(new_version) = Version::parse(new_version.trim_start_matches('v')) {
                let current_version = Version::parse(env!("CARGO_PKG_VERSION"))?;
                if new_version > current_version {
                    eprint!("\x1b[1;32m"); // Bold and green
                    eprintln_i18n!(
                        self.i18n,
                        "upgrade-available",
                        version = new_version.to_string(),
                    );
                    eprint!("\x1b[0m"); // Reset
                    eprintln!();
                } else {
                    // Check cached versions.
                    let current_version = Version::parse(env!("CARGO_PKG_VERSION"))?;
                    let mut should_clear_cache = false;

                    // Parse cached versions.
                    let cached_minor = gdvm_cache
                        .new_version
                        .as_ref()
                        .and_then(|v| Version::parse(v.trim_start_matches('v')).ok());
                    let cached_major = gdvm_cache
                        .new_major_version
                        .as_ref()
                        .and_then(|v| Version::parse(v.trim_start_matches('v')).ok());

                    // Check if cached versions are still newer than current.
                    let valid_minor = cached_minor
                        .as_ref()
                        .map(|v| v > &current_version)
                        .unwrap_or(false);
                    let valid_major = cached_major
                        .as_ref()
                        .map(|v| v > &current_version)
                        .unwrap_or(false);

                    if valid_minor && valid_major && cached_minor != cached_major {
                        eprint!("\x1b[1;32m"); // Bold and green
                        eprintln_i18n!(
                            self.i18n,
                            "upgrade-available-both",
                            minor_version = gdvm_cache.new_version.as_ref().unwrap(),
                            major_version = gdvm_cache.new_major_version.as_ref().unwrap()
                        );
                        eprint!("\x1b[0m"); // Reset
                        eprintln!();
                    } else if valid_minor {
                        eprint!("\x1b[1;32m"); // Bold and green
                        eprintln_i18n!(
                            self.i18n,
                            "upgrade-available",
                            version = gdvm_cache.new_version.as_ref().unwrap()
                        );
                        eprint!("\x1b[0m"); // Reset
                        eprintln!();
                    } else if valid_major {
                        eprint!("\x1b[1;32m"); // Bold and green
                        eprintln_i18n!(
                            self.i18n,
                            "upgrade-available-major",
                            version = gdvm_cache.new_major_version.as_ref().unwrap()
                        );
                        eprint!("\x1b[0m"); // Reset
                        eprintln!();
                    } else {
                        should_clear_cache = true;
                    }

                    if should_clear_cache {
                        self.save_gdvm_cache(&GdvmCache {
                            last_update_check: now,
                            new_version: None,
                            new_major_version: None,
                        })?;
                    }
                }
            } else {
                self.save_gdvm_cache(&GdvmCache {
                    last_update_check: now,
                    new_version: None,
                    new_major_version: None,
                })?;
            }
        }

        Ok(())
    }

    pub fn upgrade(&self, allow_major: bool) -> Result<()> {
        println_i18n!(self.i18n, "upgrade-starting");
        println_i18n!(self.i18n, "upgrade-downloading-latest");

        let current_version = Version::parse(env!("CARGO_PKG_VERSION"))?;

        let releases =
            self.get_github_json("https://api.github.com/repos/adalinesimonian/gdvm/releases")?;

        // Determine version requirement based on allow_major flag.
        let version_req = if allow_major {
            "*".to_string() // Allow any version.
        } else {
            format!("^{}", current_version.major) // Stay within current major version.
        };

        let latest_stable_tag = self
            .find_latest_stable_release(&releases, &version_req)?
            .ok_or_else(|| {
                if allow_major {
                    anyhow!("No stable releases found")
                } else {
                    anyhow!("No stable {}.x.x releases found", current_version.major)
                }
            })?;

        // Check if upgrade is necessary.
        let latest_version = Version::parse(latest_stable_tag.trim_start_matches('v'))?;
        let current_version = Version::parse(env!("CARGO_PKG_VERSION"))?;

        match latest_version.cmp(&current_version) {
            std::cmp::Ordering::Equal => {
                println_i18n!(
                    self.i18n,
                    "upgrade-not-needed",
                    version = latest_version.to_string()
                );
                return Ok(());
            }
            std::cmp::Ordering::Less => {
                println_i18n!(
                    self.i18n,
                    "upgrade-current-version-newer",
                    current = current_version.to_string(),
                    latest = latest_version.to_string()
                );
                return Ok(());
            }
            std::cmp::Ordering::Greater => {}
        }

        // Define install directory
        let install_dir = self.get_base_path().join("bin");
        std::fs::create_dir_all(&install_dir)
            .map_err(|_| anyhow!(t_w!(self.i18n, "upgrade-install-dir-failed")))?;

        // Detect architecture
        let arch = if cfg!(target_os = "windows") {
            if cfg!(target_arch = "aarch64") {
                "aarch64-pc-windows-msvc"
            } else if cfg!(target_arch = "x86_64") {
                "x86_64-pc-windows-msvc"
            } else if cfg!(target_arch = "x86") {
                "i686-pc-windows-msvc"
            } else {
                return Err(anyhow!(t_w!(self.i18n, "unsupported-architecture")));
            }
        } else if cfg!(target_os = "linux") {
            if cfg!(target_arch = "aarch64") {
                "aarch64-unknown-linux-gnu"
            } else if cfg!(target_arch = "x86_64") {
                "x86_64-unknown-linux-gnu"
            } else if cfg!(target_arch = "x86") {
                "i686-unknown-linux-gnu"
            } else {
                return Err(anyhow!(t_w!(self.i18n, "unsupported-architecture")));
            }
        } else if cfg!(target_os = "macos") {
            if cfg!(target_arch = "aarch64") {
                "aarch64-apple-darwin"
            } else if cfg!(target_arch = "x86_64") {
                "x86_64-apple-darwin"
            } else {
                return Err(anyhow!(t_w!(self.i18n, "unsupported-architecture")));
            }
        } else {
            return Err(anyhow!(t_w!(self.i18n, "unsupported-platform")));
        };

        // Set download URL based on architecture.
        let repo_url = "https://github.com/adalinesimonian/gdvm";
        let release_url = format!("{repo_url}/releases/download/{latest_stable_tag}");
        #[cfg(target_os = "windows")]
        let file = format!("gdvm-{arch}.exe");
        #[cfg(not(target_os = "windows"))]
        let file = format!("gdvm-{arch}");
        let bin_url = format!("{release_url}/{file}");
        let out_file = install_dir.join("gdvm.new");

        // Download the new binary.
        if let Err(err) = download_file(&bin_url, &out_file, self.i18n) {
            eprintln_i18n!(self.i18n, "upgrade-download-failed");
            return Err(err);
        }

        // Find the specific release to get the digest.
        let mut found_digest = None;
        if let Some(releases_array) = releases.as_array() {
            for release in releases_array {
                if release.get("tag_name").and_then(|t| t.as_str()) == Some(&latest_stable_tag) {
                    if let Some(assets) = release.get("assets").and_then(|a| a.as_array()) {
                        found_digest = assets.iter().find_map(|asset| {
                            let name = asset.get("name").and_then(|n| n.as_str());
                            let digest = asset.get("digest").and_then(|d| d.as_str());

                            if name == Some(&file) {
                                digest
                                    .and_then(|d| d.strip_prefix("sha256:"))
                                    .map(|d| d.to_string())
                            } else {
                                None
                            }
                        });
                    }
                    break;
                }
            }
        }

        if let Some(digest) = found_digest {
            if let Err(e) = verify_sha(&out_file, &digest, self.i18n) {
                let _ = std::fs::remove_file(&out_file);
                return Err(e);
            }
        } else {
            eprintln_i18n!(self.i18n, "warning-sha-sums-missing");
        }

        #[cfg(target_family = "unix")]
        {
            // Make the new binary executable
            let mut perms = out_file.metadata()?.permissions();
            perms.set_mode(perms.mode() | 0o111);
            std::fs::set_permissions(&out_file, perms)?;
        }

        // Rename current executable to .bak and replace it with the new file
        let current_exe = std::env::current_exe()?;
        let backup_exe = current_exe.with_extension("bak");

        std::fs::rename(&current_exe, &backup_exe)
            .map_err(|_| anyhow!(t_w!(self.i18n, "upgrade-rename-failed")))?;
        std::fs::rename(&out_file, &current_exe)
            .map_err(|_| anyhow!(t_w!(self.i18n, "upgrade-replace-failed")))?;

        // Update gdvm cache
        self.save_gdvm_cache(&GdvmCache {
            last_update_check: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|_| anyhow!("System time before UNIX EPOCH"))?
                .as_secs(),
            new_version: None,
            new_major_version: None,
        })?;

        migrations::run_migrations(&self.base_path, self.i18n)?;

        println_i18n!(self.i18n, "upgrade-complete");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_find_latest_stable_release() {
        let i18n = I18n::new(100).unwrap();
        let manager = GodotManager::new(&i18n).unwrap();

        // Mock release data based on GitHub API response.
        let releases = json!([
            {
                "tag_name": "v1.0.0",
                "draft": false,
                "prerelease": false
            },
            {
                "tag_name": "v0.9.0",
                "draft": false,
                "prerelease": false
            },
            {
                "tag_name": "v0.8.1",
                "draft": false,
                "prerelease": false
            },
            {
                "tag_name": "v0.8.0",
                "draft": false,
                "prerelease": false
            },
            {
                "tag_name": "v0.7.0",
                "draft": false,
                "prerelease": false
            },
            {
                "tag_name": "v0.8.0-beta.1",
                "draft": false,
                "prerelease": true
            },
            {
                "tag_name": "v0.9.0-rc.1",
                "draft": false,
                "prerelease": false
            }
        ]);

        let result = manager.find_latest_stable_release(&releases, "^0").unwrap();

        assert_eq!(result, Some("v0.9.0".to_string()));

        let result = manager
            .find_latest_stable_release(&releases, "^0.8")
            .unwrap();

        assert_eq!(result, Some("v0.8.1".to_string()));

        let result = manager
            .find_latest_stable_release(&releases, "=0.8.0")
            .unwrap();

        assert_eq!(result, Some("v0.8.0".to_string()));
    }

    #[test]
    fn test_find_latest_stable_release_no_matches() {
        let i18n = I18n::new(100).unwrap();
        let manager = GodotManager::new(&i18n).unwrap();

        // Mock releases with no stable 0.x.x versions.
        let releases = json!([
            {
                "tag_name": "v1.0.0",
                "draft": false,
                "prerelease": false
            },
            {
                "tag_name": "v1.1.0",
                "draft": false,
                "prerelease": false
            },
            {
                "tag_name": "v0.8.0-beta.1",
                "draft": false,
                "prerelease": true
            }
        ]);

        let result = manager.find_latest_stable_release(&releases, "^0").unwrap();

        assert_eq!(result, None);
    }

    #[test]
    fn test_find_latest_stable_release_skips_drafts() {
        let i18n = I18n::new(100).unwrap();
        let manager = GodotManager::new(&i18n).unwrap();

        // Mock releases with drafts.
        let releases = json!([
            {
                "tag_name": "v0.9.0",
                "draft": true,
                "prerelease": false
            },
            {
                "tag_name": "v0.8.0",
                "draft": false,
                "prerelease": false
            }
        ]);

        let result = manager.find_latest_stable_release(&releases, "^0").unwrap();

        assert_eq!(result, Some("v0.8.0".to_string()));
    }

    #[test]
    fn test_find_latest_stable_release_invalid_requirement() {
        let i18n = I18n::new(100).unwrap();
        let manager = GodotManager::new(&i18n).unwrap();

        let releases = json!([]);

        let result = manager.find_latest_stable_release(&releases, "invalid-version-req");

        assert!(result.is_err());
    }
}
