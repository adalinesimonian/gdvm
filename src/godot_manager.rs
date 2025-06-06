use crate::config::Config;
use anyhow::{Result, anyhow};
#[cfg(target_family = "unix")]
use daemonize::Daemonize;
use directories::BaseDirs;
use i18n::I18n;
use indicatif::{ProgressBar, ProgressStyle};
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::Digest;
#[cfg(target_family = "unix")]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{env, fs};

use crate::download_utils::download_file;
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
struct GithubReleasesCache {
    /// Unix timestamp in seconds
    last_fetched: u64,
    releases: Vec<ReleaseCache>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GdvmCache {
    last_update_check: u64,
    new_version: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct FullCache {
    gdvm: GdvmCache,
    godot_github: GithubReleasesCache,
}

#[derive(Debug)]
pub enum InstallOutcome {
    Installed,
    AlreadyInstalled,
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
    i18n: &'a I18n,
}

#[allow(clippy::unimplemented)]
fn get_archive_name(version: &GodotVersionDeterminate, i18n: &I18n) -> String {
    let is_csharp = version.is_csharp == Some(true);
    let tag = version.to_remote_str();

    let platform = if cfg!(target_os = "windows") {
        if cfg!(target_arch = "x86_64") {
            if is_csharp { "win64" } else { "win64.exe" }
        } else if cfg!(target_arch = "x86") {
            if is_csharp { "win32" } else { "win32.exe" }
        } else if cfg!(target_arch = "aarch64") {
            if is_csharp {
                "windows_arm64"
            } else {
                "windows_arm64.exe"
            }
        } else {
            unimplemented!("{}", t_w!(i18n, "unsupported-architecture"))
        }
    } else if cfg!(target_os = "macos") {
        "macos.universal"
    } else if cfg!(target_os = "linux") {
        if cfg!(target_arch = "x86_64") {
            if is_csharp {
                "linux_x86_64"
            } else {
                "linux.x86_64"
            }
        } else if cfg!(target_arch = "x86") {
            if is_csharp {
                "linux_x86_32"
            } else {
                "linux.x86_32"
            }
        } else if cfg!(target_arch = "arm") {
            if is_csharp {
                "linux_arm32"
            } else {
                "linux.arm32"
            }
        } else if cfg!(target_arch = "aarch64") {
            if is_csharp {
                "linux_arm64"
            } else {
                "linux.arm64"
            }
        } else {
            unimplemented!("{}", t_w!(i18n, "unsupported-architecture"))
        }
    } else {
        unimplemented!("{}", t_w!(i18n, "unsupported-platform"))
    };

    if is_csharp {
        format!("Godot_v{}_mono_{}.zip", tag, platform)
    } else {
        format!("Godot_v{}_{}.zip", tag, platform)
    }
}

fn get_download_url(tag: &str, archive_name: &str) -> String {
    format!(
        "https://github.com/godotengine/godot-builds/releases/download/{}/{}",
        tag, archive_name
    )
}

fn verify_sha512(
    file_path: &Path,
    target_name: &str,
    release_tag: &str,
    i18n: &I18n,
) -> Result<()> {
    let sha_url = format!(
        "https://github.com/godotengine/godot-builds/releases/download/{}/SHA512-SUMS.txt",
        release_tag
    );
    let sum_file = file_path.with_extension("SHA512-SUMS");

    if download_file(&sha_url, &sum_file, i18n).is_err() {
        eprintln_i18n!(i18n, "warning-sha-sums-missing");
        return Ok(());
    }

    // Initialize indeterminate progress bar
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}")?);
    pb.enable_steady_tick(Duration::from_millis(100));
    pb.set_message(t_w!(i18n, "verifying-checksum"));

    let sums_content = fs::read_to_string(&sum_file)?;

    // Compute sha512 hash
    let mut hasher = sha2::Sha512::new();
    let mut f = fs::File::open(file_path)?;
    std::io::copy(&mut f, &mut hasher)?;
    let local_hash = format!("{:x}", hasher.finalize());

    // Check sums file
    for line in sums_content.lines() {
        if let Some((hash_part, file_part)) = line.split_once("  ") {
            if file_part.ends_with(&target_name) && hash_part == local_hash {
                fs::remove_file(sum_file)?;

                pb.finish_with_message(t_w!(i18n, "checksum-verified"));

                return Ok(());
            }
        }
    }

    pb.finish_and_clear();

    Err(anyhow!(t!(
        i18n,
        "error-checksum-mismatch",
        file = target_name,
    )))
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

fn copy_over_binary_if_different(source: &Path, dest: &Path) -> Result<()> {
    if let Ok(metadata) = fs::symlink_metadata(dest) {
        if metadata.file_type().is_symlink() {
            fs::remove_file(dest)?;
        }
    }
    if dest.exists() {
        let source_content = fs::read(source)?;
        let dest_content = fs::read(dest)?;
        if source_content != dest_content {
            fs::copy(source, dest)?;
        }
    } else {
        fs::copy(source, dest)?;
    }
    Ok(())
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

        let manager = GodotManager {
            base_path,
            install_path,
            cache_index_path,
            cache_path,
            client,
            i18n,
        };

        manager.ensure_godot_binaries()?;

        // Don't fail if update check fails, since it isn't critical
        manager.check_for_upgrades().ok();

        Ok(manager)
    }

    fn ensure_godot_binaries(&self) -> Result<()> {
        let bin_path = self.base_path.join("bin");
        let gdvm_exe_source = if cfg!(target_os = "windows") {
            bin_path.join("gdvm.exe")
        } else {
            bin_path.join("gdvm")
        };

        let targets: &[&str] = if cfg!(target_os = "windows") {
            &["godot.exe", "godot_console.exe"]
        } else {
            &["godot"]
        };

        for exe in targets.iter() {
            let exe_path = bin_path.join(exe);

            if std::env::current_exe().ok() != Some(exe_path.clone()) {
                if let Err(err) = copy_over_binary_if_different(&gdvm_exe_source, &exe_path) {
                    eprintln_i18n!(
                        self.i18n,
                        "error-ensure-godot-binaries-failed",
                        error = &err.to_string(),
                        path = &exe_path.to_string_lossy().to_string(),
                    );
                    return Err(err);
                }
            }
        }

        Ok(())
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

        let release_tag = gv.to_remote_str();
        let archive_name = get_archive_name(gv, self.i18n);
        let download_url = get_download_url(&release_tag, &archive_name);
        let cache_zip_path = self.cache_path.join(&archive_name);

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
            download_file(&download_url, &tmp_file, self.i18n)?;
            verify_sha512(&tmp_file, &archive_name, &release_tag, self.i18n)?;

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
                format!("token {}", token).parse()?,
            );
        }
        Ok(reqwest::blocking::ClientBuilder::new()
            .default_headers(headers)
            .user_agent("gdvm")
            .build()?)
    }

    /// Tries to query the GitHub API with a GET at a given URL. If it fails due
    /// to a rate-limit, it will return an error after printing a message.
    fn get_github_json(&self, url: &str) -> Result<serde_json::Value> {
        // Rate limits are 403s with a JSON object that has a "message" key that
        // starts with "API rate limit exceeded".

        let resp = self
            .client
            .get(url)
            .header("User-Agent", "gdvm")
            .timeout(Duration::from_secs(3))
            .send()?;

        if resp.status().is_success() {
            let json: serde_json::Value = resp.json()?;
            return Ok(json);
        }

        let status = resp.status();
        let json: serde_json::Value = resp.json()?;
        if status == reqwest::StatusCode::FORBIDDEN {
            if let Some(message) = json.get("message").and_then(|m| m.as_str()) {
                if message.starts_with("API rate limit exceeded") {
                    return Err(anyhow!(t_w!(self.i18n, "error-github-rate-limit")));
                }
            }
        }

        let error_message = json.get("message").and_then(|m| m.as_str());

        Err(anyhow!(t_w!(
            self.i18n,
            "error-github-api",
            error = error_message,
        )))
    }

    /// Update the cache by fetching from GitHub and updating `last_fetched`
    fn update_cache(&self, cache: &mut GithubReleasesCache) -> Result<()> {
        let mut page = 1;
        let page_size = if cache.releases.is_empty() { 100 } else { 10 };

        let mut new_releases = Vec::new();
        let mut seen_existing_release = false;

        // Initialize indeterminate progress bar
        let pb = ProgressBar::new_spinner();
        pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}")?);
        pb.enable_steady_tick(Duration::from_millis(100));
        pb.set_message(t_w!(self.i18n, "fetching-releases"));

        while !seen_existing_release {
            let releases = self.get_github_json(&format!(
                "https://api.github.com/repos/godotengine/godot-builds/releases?per_page={}&page={}",
                page_size, page
            ))?;

            if let Some(arr) = releases.as_array() {
                if arr.is_empty() {
                    break;
                }
                for r in arr {
                    if let Some(id) = r.get("id").and_then(|i| i.as_u64()) {
                        if let Some(tag_name) = r.get("tag_name").and_then(|t| t.as_str()) {
                            if cache.releases.iter().any(|c| c.id == id) {
                                seen_existing_release = true;
                                continue;
                            }
                            new_releases.push(ReleaseCache {
                                id,
                                tag_name: tag_name.to_string(),
                            });
                        }
                    }
                }

                if arr.len() < page_size {
                    break;
                }
            } else {
                break;
            }

            page += 1;
        }

        pb.finish_with_message(t_w!(self.i18n, "releases-fetched"));

        cache.releases.extend(new_releases);
        cache.last_fetched = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| anyhow!("System time before UNIX EPOCH"))?
            .as_secs();

        self.save_github_cache(cache)?;

        Ok(())
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
                        },
                        godot_github: GithubReleasesCache {
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
                },
                godot_github: GithubReleasesCache {
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
    fn load_cache(&self) -> Result<GithubReleasesCache> {
        let full = self.load_full_cache()?;
        Ok(full.godot_github)
    }

    fn save_github_cache(&self, cache: &GithubReleasesCache) -> Result<()> {
        let mut full = self.load_full_cache()?;
        full.godot_github = cache.clone();
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

    pub fn check_for_upgrades(&self) -> Result<()> {
        // Load or initialize gdvm cache
        let gdvm_cache = self.load_gdvm_cache()?;

        // Check for updates
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| anyhow!("System time before UNIX EPOCH"))?
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

            if let Err(err) = {
                let release = self.get_github_json(
                    "https://api.github.com/repos/adalinesimonian/gdvm/releases/latest",
                )?;

                if let Some(tag_name) = release.get("tag_name").and_then(|t| t.as_str()) {
                    let latest_version = Version::parse(tag_name.trim_start_matches('v'))?;
                    let current_version = Version::parse(env!("CARGO_PKG_VERSION"))?;
                    if latest_version > current_version {
                        new_version = Some(tag_name.to_string());
                    }
                }

                Ok(())
            } {
                progress.finish_and_clear();
                return Err(err);
            }

            progress.finish_and_clear();

            if let Some(new_version) = new_version {
                eprint!("\x1b[1;32m"); // Bold and green
                eprintln_i18n!(self.i18n, "upgrade-available", version = &new_version);
                eprint!("\x1b[0m"); // Reset
                eprintln!();

                self.save_gdvm_cache(&GdvmCache {
                    last_update_check: now,
                    new_version: Some(new_version),
                })?;
            } else {
                self.save_gdvm_cache(&GdvmCache {
                    last_update_check: now,
                    new_version: None,
                })?;
            }
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
                    self.save_gdvm_cache(&GdvmCache {
                        last_update_check: now,
                        new_version: None,
                    })?;
                }
            } else {
                self.save_gdvm_cache(&GdvmCache {
                    last_update_check: now,
                    new_version: None,
                })?;
            }
        }

        Ok(())
    }

    pub fn upgrade(&self) -> Result<()> {
        println_i18n!(self.i18n, "upgrade-starting");
        println_i18n!(self.i18n, "upgrade-downloading-latest");

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

        // Set download URL based on architecture
        let repo_url = "https://github.com/adalinesimonian/gdvm";
        let latest_url = format!("{}/releases/latest/download", repo_url);
        #[cfg(target_os = "windows")]
        let file = format!("gdvm-{}.exe", arch);
        #[cfg(not(target_os = "windows"))]
        let file = format!("gdvm-{}", arch);
        let bin_url = format!("{}/{}", latest_url, file);
        let out_file = install_dir.join("gdvm.new");

        // Download the new binary
        if let Err(err) = download_file(&bin_url, &out_file, self.i18n) {
            eprintln_i18n!(self.i18n, "upgrade-download-failed");
            return Err(err);
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
        })?;

        self.ensure_godot_binaries()?;

        println_i18n!(self.i18n, "upgrade-complete");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_download_url_and_archive_name() {
        let i18n = I18n::new(80).unwrap();
        let gv = GodotVersion::from_remote_str("4.1-stable", Some(false))
            .unwrap()
            .to_determinate();
        let name = super::get_archive_name(&gv, &i18n);
        #[cfg(target_os = "linux")]
        assert_eq!(name, "Godot_v4.1-stable_linux.x86_64.zip");
        #[cfg(target_os = "windows")]
        assert_eq!(name, "Godot_v4.1-stable_win64.exe.zip");
        #[cfg(target_os = "macos")]
        assert_eq!(name, "Godot_v4.1-stable_macos.universal.zip");
        let url = super::get_download_url(&gv.to_remote_str(), &name);
        assert!(url.ends_with(&name));
    }
}
