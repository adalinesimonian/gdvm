use anyhow::{anyhow, Result};
#[cfg(target_family = "unix")]
use daemonize::Daemonize;
use directories::BaseDirs;
use fluent_bundle::FluentValue;
use i18n::I18n;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::download_utils::download_file;
use crate::i18n;
use crate::println_i18n;
use crate::version_utils;
use crate::version_utils::GodotVersionDeterminate;
use crate::zip_utils;

use version_utils::{GodotVersion, GodotVersionDeterminateVecExt};

#[derive(Serialize, Deserialize, Debug)]
struct ReleaseCache {
    id: u64,
    tag_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct GithubReleasesCache {
    /// Unix timestamp in seconds
    last_fetched: u64,
    releases: Vec<ReleaseCache>,
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
    i18n: &'a I18n,
}

fn get_archive_name(version: &GodotVersionDeterminate, i18n: &I18n) -> String {
    let is_csharp = version.is_csharp == Some(true);
    let tag = version.to_remote_str();

    let platform = if cfg!(target_os = "windows") {
        if cfg!(target_arch = "x86_64") {
            if is_csharp {
                "win64"
            } else {
                "win64.exe"
            }
        } else if cfg!(target_arch = "x86") {
            if is_csharp {
                "win32"
            } else {
                "win32.exe"
            }
        } else if cfg!(target_arch = "aarch64") {
            if is_csharp {
                "windows_arm64"
            } else {
                "windows_arm64.exe"
            }
        } else {
            unimplemented!("{}", i18n.t("unsupported-architecture"))
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
            unimplemented!("{}", i18n.t("unsupported-architecture"))
        }
    } else {
        unimplemented!("{}", i18n.t("unsupported-platform"))
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

    if let Err(_) = download_file(&sha_url, &sum_file, &i18n) {
        println_i18n!(i18n, "warning-sha-sums-missing");
        return Ok(());
    }

    // Initialize indeterminate progress bar
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}")?);
    pb.enable_steady_tick(Duration::from_millis(100));
    pb.set_message(i18n.t("verifying-checksum"));

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

                pb.finish_with_message(i18n.t("checksum-verified"));

                return Ok(());
            }
        }
    }

    pb.finish_and_clear();

    Err(anyhow!(i18n.t_args(
        "error-checksum-mismatch",
        &[("file", FluentValue::from(target_name))]
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
fn find_godot_executable(version_dir: &Path, console: bool) -> Result<Option<PathBuf>> {
    // Read the entries in the specified directory
    let godot_executable = fs::read_dir(version_dir)?
        .filter_map(|entry| entry.ok()) // Filter out entries that resulted in an error
        .find(|entry| {
            // Get the file name
            let name = entry.file_name().to_string_lossy().to_string();

            // Determine if the current entry is the Godot executable based on the OS
            #[cfg(target_os = "windows")]
            let is_godot_executable = if console {
                // For console builds on Windows
                name.ends_with("_console.exe")
            } else {
                // For regular builds on Windows
                name.ends_with(".exe")
            };

            #[cfg(target_os = "macos")]
            let is_godot_executable = name == "Godot.app" || name == "Godot_mono.app";

            #[cfg(not(any(target_os = "windows", target_os = "macos")))]
            let is_godot_executable =
                // For Linux or other Unix-like systems
                (name.starts_with("Godot_v") && name.ends_with(".exe")) ||
                name.ends_with(".x86_64") ||
                name.ends_with(".x86_32") ||
                name.ends_with(".arm64");

            is_godot_executable
        })
        .map(|entry| entry.path()); // Extract the path of the found executable

    Ok(godot_executable)
}

impl<'a> GodotManager<'a> {
    /// Create a new GodotManager instance and set up the installation and cache paths
    pub fn new(i18n: &'a I18n) -> Result<Self> {
        // For cross-platform user directory management:
        let base_dirs = BaseDirs::new().ok_or(anyhow!(i18n.t("error-find-user-dirs")))?;
        let base_path = base_dirs.home_dir().join(".gdvm");
        let install_path = base_path.join("installs");
        let cache_index_path = base_path.join("cache.json");
        let cache_path = base_path.join("cache");

        fs::create_dir_all(&install_path)?;
        fs::create_dir_all(&cache_path)?;

        Ok(GodotManager {
            base_path,
            install_path,
            cache_index_path,
            cache_path,
            i18n,
        })
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
                self.remove(&gv)?;
                println_i18n!(
                    self.i18n,
                    "force-reinstalling-version",
                    [("version", gv.to_display_str())]
                );
            } else {
                return Ok(InstallOutcome::AlreadyInstalled);
            }
        }

        if !gv.is_stable() {
            println_i18n!(
                self.i18n,
                "warning-prerelease",
                [("branch", &gv.release_type)]
            );
        }

        fs::create_dir_all(&self.cache_path)?;

        let release_tag = gv.to_remote_str();
        let archive_name = get_archive_name(&gv, self.i18n);
        let download_url = get_download_url(&release_tag, &archive_name);
        let cache_zip_path = self.cache_path.join(&archive_name);

        if !redownload && cache_zip_path.exists() {
            println_i18n!(self.i18n, "using-cached-zip");
        } else {
            if redownload && cache_zip_path.exists() {
                println_i18n!(
                    self.i18n,
                    "force-redownload",
                    [("version", gv.to_display_str())]
                );
            }

            let tmp_file = self
                .install_path
                .join(format!("{}.zip", gv.to_install_str()));

            // Download the archive
            download_file(&download_url, &tmp_file, &self.i18n)?;
            verify_sha512(&tmp_file, &archive_name, &release_tag, &self.i18n)?;

            // Move the verified zip to cache_dir
            fs::rename(&tmp_file, &cache_zip_path)?;
            println_i18n!(self.i18n, "cached-zip-stored");
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
                match GodotVersion::from_install_str(&name) {
                    Ok(gv) => versions.push(gv.to_determinate()),
                    Err(_) => (), // Ignore invalid version directories
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
            Err(anyhow!(self.i18n.t("error-version-not-found")))
        }
    }

    /// Run a specified Godot version
    pub fn run(&self, gv: &GodotVersionDeterminate, console: bool) -> Result<()> {
        let version_dir = self.install_path.join(gv.to_install_str());
        if !version_dir.exists() {
            return Err(anyhow!(self.i18n.t_args(
                "error-version-not-found",
                &[("version", FluentValue::from(&gv.to_display_str()))]
            )));
        }

        // Enumerate the version directory to find a Godot binary or app
        let godot_executable = find_godot_executable(&version_dir, console)?;

        let path = godot_executable.ok_or_else(|| {
            anyhow!(self.i18n.t_args(
                "godot-executable-not-found",
                &[("version", FluentValue::from(&gv.to_display_str()))]
            ))
        })?;

        // Special handling for macOS .app bundles
        #[cfg(target_os = "macos")]
        {
            if path.extension().and_then(|ext| ext.to_str()) == Some("app") {
                let inner_path = path.join("Contents/MacOS/Godot");
                if !inner_path.exists() {
                    return Err(anyhow!(self.i18n.t_args(
                        "godot-executable-not-found",
                        &[("version", FluentValue::from(&gv.to_display_str()))]
                    )));
                }
                if console {
                    // On macOS, running attached is the default behavior
                    std::process::Command::new(inner_path).spawn()?;
                } else {
                    // Detached process
                    std::process::Command::new(inner_path).spawn()?;
                }
                return Ok(());
            }
        }

        if console {
            // Run the process attached to the terminal and wait for it to exit
            std::process::Command::new(&path)
                .stdin(std::process::Stdio::inherit())
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .status()?;
        } else {
            // Detached process configuration
            #[cfg(target_family = "unix")]
            {
                Daemonize::new().start().map_err(|e| {
                    anyhow!(self.i18n.t_args(
                        "error-starting-godot",
                        &[("error", FluentValue::from(e.to_string()))]
                    ))
                })?;
                std::process::Command::new(&path).spawn()?;
            }

            #[cfg(target_family = "windows")]
            {
                use std::os::windows::process::CommandExt;
                use winapi::um::winbase::DETACHED_PROCESS;
                std::process::Command::new(&path)
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
            // Fetch from GitHub and update cache
            self.update_cache(&mut cache)?;
        }

        // Filter releases
        let mut filtered_releases: Vec<GodotVersionDeterminate> = cache
            .releases
            .iter()
            .filter_map(|r| GodotVersion::from_remote_str(&r.tag_name, None).ok())
            .map(|gv| gv.to_determinate())
            .filter(|r| filter.map_or(true, |f| f.matches(r)))
            .collect();

        // If no releases found and not using cache only, try fetching if we haven't already
        if filtered_releases.is_empty() && !use_cache_only && !is_time_to_refresh_index {
            // self.update_cache(&mut cache)?;
            filtered_releases = cache
                .releases
                .iter()
                .filter_map(|r| GodotVersion::from_remote_str(&r.tag_name, None).ok())
                .map(|gv| gv.to_determinate())
                .filter(|r| filter.map_or(true, |f| f.matches(r)))
                .collect();
        }

        // Sort releases
        filtered_releases.sort_by_version();

        Ok(filtered_releases)
    }

    /// Update the cache by fetching from GitHub and updating `last_fetched`
    fn update_cache(&self, cache: &mut GithubReleasesCache) -> Result<()> {
        let client = reqwest::blocking::Client::new();
        let mut page = 1;
        let page_size = if cache.releases.is_empty() { 100 } else { 10 };

        let mut new_releases = Vec::new();
        let mut seen_existing_release = false;

        // Initialize indeterminate progress bar
        let pb = ProgressBar::new_spinner();
        pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}")?);
        pb.enable_steady_tick(Duration::from_millis(100));
        pb.set_message(self.i18n.t("fetching-releases"));

        while !seen_existing_release {
            let url = format!(
                "https://api.github.com/repos/godotengine/godot-builds/releases?per_page={}&page={}",
                page_size, page
            );
            let resp = client
                .get(&url)
                .header("User-Agent", "gdvm")
                .send()?
                .error_for_status()?;
            let releases: serde_json::Value = serde_json::from_str(&resp.text()?)?;

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

        pb.finish_with_message(self.i18n.t("releases-fetched"));

        cache.releases = new_releases;
        cache.last_fetched = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| anyhow!("System time before UNIX EPOCH"))?
            .as_secs();

        self.save_cache(cache)?;

        Ok(())
    }

    // Load the release cache from cache.json
    fn load_cache(&self) -> Result<GithubReleasesCache> {
        if self.cache_index_path.exists() {
            let data = fs::read_to_string(&self.cache_index_path)?;
            match serde_json::from_str::<GithubReleasesCache>(&data) {
                Ok(cache) => Ok(cache),
                Err(_) => {
                    // Log a warning about the corrupted or unexpected cache format
                    println_i18n!(self.i18n, "warning-cache-metadata-reset");

                    // Overwrite the cache with an empty cache
                    let empty_cache = GithubReleasesCache {
                        last_fetched: 0,
                        releases: vec![],
                    };
                    self.save_cache(&empty_cache)?;

                    Ok(empty_cache)
                }
            }
        } else {
            // Initialize empty cache
            Ok(GithubReleasesCache {
                last_fetched: 0,
                releases: vec![],
            })
        }
    }

    fn save_cache(&self, cache: &GithubReleasesCache) -> Result<()> {
        let data = serde_json::to_string(cache)?;
        fs::write(&self.cache_index_path, data)?;
        Ok(())
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
            .ok_or_else(|| anyhow!(self.i18n.t("error-no-stable-releases-found")))
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
            return Err(anyhow!(self.i18n.t("error-version-not-found")));
        }

        // Write version to .gdvm/default
        let default_path = self.base_path.join("default");
        fs::write(&default_path, &version_str)?;

        // Create directory symlink .gdvm/bin/current_godot -> .gdvm/<version_str>/
        let symlink_dir = self.base_path.join("bin").join("current_godot");
        let target_dir = self.install_path.join(version_str);

        // Make sure bin directory exists
        fs::create_dir_all(&symlink_dir.parent().unwrap())?;

        if symlink_dir.exists() {
            fs::remove_dir_all(&symlink_dir)?;
        }
        #[cfg(target_family = "unix")]
        std::os::unix::fs::symlink(&target_dir, &symlink_dir)?;
        #[cfg(target_family = "windows")]
        std::os::windows::fs::symlink_dir(&target_dir, &symlink_dir).map_err(|e| anyhow!(e))?;

        // Create symlink for godot executable
        // .gdvm/bin/godot -> .gdvm/<version_str>/godot-executable-name(.exe)

        if let Some(godot_executable) = find_godot_executable(&version_path, false)? {
            let symlink_path = symlink_dir.join("godot");
            if symlink_path.exists() {
                fs::remove_file(&symlink_path)?;
            }
            #[cfg(target_family = "unix")]
            std::os::unix::fs::symlink(&godot_executable, &symlink_path)?;
            #[cfg(target_family = "windows")]
            std::os::windows::fs::symlink_file(&godot_executable, &symlink_path)
                .map_err(|e| anyhow!(e))?;
        }

        // (on Windows) .gdvm/bin/godot_console -> .gdvm/<version_str>/godot-executable-name_console.exe
        #[cfg(target_family = "windows")]
        {
            if let Some(godot_executable) = find_godot_executable(&version_path, true)? {
                let symlink_path = symlink_dir.join("godot_console");
                if symlink_path.exists() {
                    fs::remove_file(&symlink_path)?;
                }
                std::os::windows::fs::symlink_file(&godot_executable, &symlink_path)
                    .map_err(|e| anyhow!(e))?;
            }
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

        let symlink_path = self.base_path.join("bin").join("godot");
        if symlink_path.exists() {
            fs::remove_file(symlink_path)?;
        }

        #[cfg(target_family = "windows")]
        {
            let symlink_path = self.base_path.join("bin").join("godot_console");
            if symlink_path.exists() {
                fs::remove_file(symlink_path)?;
            }
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

    /// Pin a version to .gdvmrc in the current directory
    pub fn pin_version(&self, gv: &GodotVersionDeterminate) -> Result<()> {
        let path = std::env::current_dir()?;
        let file = path.join(".gdvmrc");
        fs::write(&file, gv.to_install_str())?;
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
                .ok_or_else(|| anyhow!(self.i18n.t("error-version-not-found")))?
        } else {
            gv.clone().into()
        };

        actual_version.is_csharp = gv.is_csharp;

        // Check if version is installed, if not, install
        if !self.is_version_installed(&actual_version)? {
            println_i18n!(
                self.i18n,
                "auto-installing-version",
                [("version", &actual_version.to_display_str())]
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
}
