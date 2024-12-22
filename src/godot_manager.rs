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
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::i18n;
use crate::version_utils;
use crate::zip_utils;

use version_utils::{GodotBranch, GodotVersion};

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
    install_path: PathBuf,
    /// Path to cache.json
    cache_path: PathBuf,
    /// Path to directory to store cached zip files
    cache_dir: PathBuf,
    i18n: &'a I18n,
}

fn get_archive_name(tag: &str, is_csharp: bool, i18n: &I18n) -> String {
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
    download_file(&sha_url, &sum_file, &i18n)?;

    // Initialize indeterminate progress bar
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}")?);
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
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
        let install_path = base_dirs.home_dir().join(".gdvm");
        fs::create_dir_all(&install_path)?;
        let cache_path = install_path.join("cache.json");
        let cache_dir = install_path.join("cache");
        fs::create_dir_all(&cache_dir)?;
        Ok(GodotManager {
            install_path,
            cache_path,
            cache_dir,
            i18n,
        })
    }

    /// Install a specified Godot version
    ///
    /// - `force`: If true, reinstall the version even if it's already installed.
    /// - `redownload`: If true, ignore cached zip files and download fresh ones.
    pub fn install(
        &self,
        gv: &GodotVersion,
        force: bool,
        redownload: bool,
    ) -> Result<InstallOutcome> {
        let version_path = self.install_path.join(format!(
            "{}-{}{}",
            gv.version,
            match gv.branch {
                GodotBranch::Stable => "stable".to_string(),
                GodotBranch::PreRelease(ref pr) => pr.clone(),
            }
            .as_str(),
            if gv.is_csharp { "-csharp" } else { "" }
        ));

        if version_path.exists() {
            if force {
                self.remove(&gv)?;
                println!(
                    "{}",
                    self.i18n.t_args(
                        "force-reinstalling-version",
                        &[("version", FluentValue::from(gv.to_string()))]
                    )
                );
            } else {
                return Ok(InstallOutcome::AlreadyInstalled);
            }
        }

        fs::create_dir_all(&version_path)?;

        if let GodotBranch::PreRelease(pr) = &gv.branch {
            println!(
                "{}",
                self.i18n
                    .t_args("warning-prerelease", &[("branch", FluentValue::from(pr))])
            );
        }

        let release_tag = version_utils::build_release_tag(&gv.version, &gv.branch);
        let archive_name = get_archive_name(&release_tag, gv.is_csharp, self.i18n);
        let download_url = get_download_url(&release_tag, &archive_name);
        let cache_zip_path = self.cache_dir.join(&archive_name);

        if !redownload && cache_zip_path.exists() {
            println!("{}", self.i18n.t("using-cached-zip"));
        } else {
            if redownload && cache_zip_path.exists() {
                println!(
                    "{}",
                    self.i18n.t_args(
                        "force-redownload",
                        &[("version", FluentValue::from(gv.to_string()))]
                    )
                );
            }

            let tmp_file = self.install_path.join(format!("{}.zip", gv.version));

            // Download the archive
            download_file(&download_url, &tmp_file, &self.i18n)?;
            verify_sha512(&tmp_file, &archive_name, &release_tag, &self.i18n)?;

            // Move the verified zip to cache_dir
            fs::rename(&tmp_file, &cache_zip_path)?;
            println!("{}", self.i18n.t("cached-zip-stored"));
        }

        // Extract from cache_zip_path
        zip_utils::extract_zip(&cache_zip_path, &version_path, self.i18n)?;

        Ok(InstallOutcome::Installed)
    }

    /// List all installed Godot versions
    pub fn list_installed(&self) -> Result<Vec<String>> {
        let mut versions = vec![];
        for entry in fs::read_dir(&self.install_path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                versions.push(entry.file_name().to_string_lossy().to_string());
            }
        }
        version_utils::sort_releases(&mut versions);
        Ok(versions)
    }

    /// Remove a specified Godot version
    pub fn remove(&self, gv: &GodotVersion) -> Result<()> {
        let path = self.install_path.join(format!(
            "{}-{}{}",
            gv.version,
            match gv.branch {
                GodotBranch::Stable => "stable".to_string(),
                GodotBranch::PreRelease(ref pr) => pr.clone(),
            },
            if gv.is_csharp { "-csharp" } else { "" }
        ));
        if path.exists() {
            // If this version is the default, unset it
            if let Some(def) = self.get_default()? {
                if def == path.file_name().unwrap().to_string_lossy() {
                    self.unset_default()?;
                }
            }
            fs::remove_dir_all(path)?;
            Ok(())
        } else {
            Err(anyhow!(self.i18n.t("version-not-found")))
        }
    }

    /// Run a specified Godot version
    pub fn run(&self, version: &str, console: bool) -> Result<()> {
        let version_dir = self.install_path.join(version);
        if !version_dir.exists() {
            return Err(anyhow!(self.i18n.t_args(
                "version-not-found",
                &[(
                    "version",
                    FluentValue::from(version_utils::friendly_installed_version(version))
                )]
            )));
        }

        // Enumerate the version directory to find a Godot binary or app
        let godot_executable = find_godot_executable(&version_dir, console)?;

        let path = godot_executable.ok_or_else(|| {
            anyhow!(self.i18n.t_args(
                "godot-executable-not-found",
                &[(
                    "version",
                    FluentValue::from(version_utils::friendly_installed_version(version))
                )]
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
                        &[(
                            "version",
                            FluentValue::from(version_utils::friendly_installed_version(version))
                        )]
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
        filter: Option<&str>,
        use_cache_only: bool,
    ) -> Result<Vec<String>> {
        let cache_duration = Duration::from_secs(48 * 3600); // 48 hours

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
        let mut filtered_releases: Vec<String> = cache
            .releases
            .iter()
            .filter(|r| filter.map_or(true, |f| r.tag_name.starts_with(f)))
            .map(|r| r.tag_name.clone())
            .collect();

        // If no releases found and not using cache only, try fetching if we haven't already
        if filtered_releases.is_empty() && !use_cache_only && !is_time_to_refresh_index {
            self.update_cache(&mut cache)?;
            filtered_releases = cache
                .releases
                .iter()
                .filter(|r| filter.map_or(true, |f| r.tag_name.starts_with(f)))
                .map(|r| r.tag_name.clone())
                .collect();
        }

        // Sort releases
        version_utils::sort_releases(&mut filtered_releases);

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
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
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
        if self.cache_path.exists() {
            let data = fs::read_to_string(&self.cache_path)?;
            match serde_json::from_str::<GithubReleasesCache>(&data) {
                Ok(cache) => Ok(cache),
                Err(_) => {
                    // Log a warning about the corrupted or unexpected cache format
                    println!("{}", self.i18n.t("warning-cache-metadata-reset"));

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
        fs::write(&self.cache_path, data)?;
        Ok(())
    }

    /// Clears the release cache by deleting the cache file and all cached zip files
    pub fn clear_cache(&self) -> Result<()> {
        if self.cache_path.exists() {
            fs::remove_file(&self.cache_path)?;
            println!("{}", self.i18n.t("cache-metadata-removed"));
        } else {
            println!("{}", self.i18n.t("no-cache-metadata-found"));
        }

        if self.cache_dir.exists() {
            for entry in fs::read_dir(&self.cache_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    fs::remove_file(&path)?;
                }
            }
            println!("{}", self.i18n.t("cache-files-removed"));
        } else {
            println!("{}", self.i18n.t("no-cache-files-found"));
        }
        Ok(())
    }

    /// Fetch the latest stable Godot version
    pub fn get_latest_stable_version(&self) -> Result<String> {
        let releases = self.fetch_available_releases(Some("stable"), false)?;
        // Assuming releases are sorted latest first
        releases
            .iter()
            .find(|r| r.contains("stable"))
            .cloned()
            .ok_or_else(|| anyhow!(self.i18n.t("error-no-stable-releases-found")))
    }

    /// Resolve the Godot version from a string, for an installed version
    /// Returns a list of possible versions. If the input is ambiguous, the list
    /// will have more than one element. Otherwise, it will have one element,
    /// unless of course the version is not found, in which case the list will
    /// be empty.
    /// Accepts full and partial versions.
    pub fn resolve_installed_version(&self, version: &str, csharp: bool) -> Result<Vec<String>> {
        let installed = self.list_installed()?;
        let mut resolved = vec![];
        for v in installed {
            if v.starts_with(version) {
                if csharp {
                    if v.ends_with("-csharp") {
                        resolved.push(v);
                    }
                } else {
                    if !v.ends_with("-csharp") {
                        resolved.push(v);
                    }
                }
            }
        }
        Ok(resolved)
    }

    /// Resolve the Godot version from a string, for an available version
    /// Returns a single version, whichever is the latest that matches the
    /// input.
    /// Accepts full and partial versions.
    pub fn resolve_available_version(&self, version: &str, use_cache_only: bool) -> Option<String> {
        let releases = self
            .fetch_available_releases(Some(version), use_cache_only)
            .ok()?;
        releases.iter().find(|r| r.starts_with(version)).cloned()
    }

    pub fn set_default(&self, version_str: &str) -> Result<()> {
        // Check if the version exists
        let version_path = self.install_path.join(version_str);
        if !version_path.exists() {
            return Err(anyhow!(self.i18n.t("version-not-found")));
        }

        // Write version to .gdvm/default
        let default_path = self.install_path.join("default");
        fs::write(&default_path, version_str)?;

        // Create directory symlink .gdvm/bin/current_godot -> .gdvm/<version_str>/
        let symlink_dir = self.install_path.join("bin").join("current_godot");
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
        let default_file = self.install_path.join("default");
        if default_file.exists() {
            fs::remove_file(default_file)?;
        }

        let symlink_dir = self.install_path.join("bin").join("current_godot");
        if symlink_dir.exists() {
            fs::remove_dir_all(symlink_dir)?;
        }

        let symlink_path = self.install_path.join("bin").join("godot");
        if symlink_path.exists() {
            fs::remove_file(symlink_path)?;
        }

        #[cfg(target_family = "windows")]
        {
            let symlink_path = self.install_path.join("bin").join("godot_console");
            if symlink_path.exists() {
                fs::remove_file(symlink_path)?;
            }
        }

        Ok(())
    }

    pub fn get_default(&self) -> Result<Option<String>> {
        let default_file = self.install_path.join("default");
        if default_file.exists() {
            let contents = fs::read_to_string(&default_file)?;
            Ok(Some(contents.trim().to_string()))
        } else {
            Ok(None)
        }
    }
}

fn download_file(url: &str, dest: &Path, i18n: &I18n) -> Result<()> {
    // Print downloading URL message
    println!(
        "{}",
        i18n.t_args(
            "operation-downloading-url",
            &[("url", FluentValue::from(url))]
        )
    );

    let response = reqwest::blocking::get(url)?;
    let total_size = response
        .content_length()
        .ok_or_else(|| anyhow!("Failed to get content length"))?;
    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
            .progress_chars("#>-"),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    let mut file = fs::File::create(dest)?;
    let mut downloaded: u64 = 0;
    let mut buffer = [0; 8192]; // 8 KB buffer
    let mut reader = response;

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break; // Download complete
        }
        file.write_all(&buffer[..bytes_read])?;
        downloaded += bytes_read as u64;
        pb.set_position(downloaded);
    }
    pb.finish_with_message(i18n.t("operation-download-complete"));
    Ok(())
}
