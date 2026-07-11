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

use anyhow::{Result, anyhow};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use sha2::{Digest, Sha256, Sha512};
use std::{path::Path, time::Duration};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{eprintln_i18n, t};

/// Allows opting into unencrypted HTTP for fetches.
pub const ALLOW_INSECURE_URLS_ENV_VAR: &str = "GDVM_ALLOW_INSECURE_URLS";

/// Check if the URL scheme is allowed.
pub fn url_scheme_allowed(url: &str) -> bool {
    url_scheme_allowed_with(url, std::env::var_os(ALLOW_INSECURE_URLS_ENV_VAR).is_some())
}

/// Check if the URL scheme is allowed. Allows setting whether or not insecure
/// URLs are allowed.
fn url_scheme_allowed_with(url: &str, allow_insecure: bool) -> bool {
    url.starts_with("https://")
        || url.starts_with("file://")
        || (allow_insecure && url.starts_with("http://"))
}

/// Error if the URL scheme is not allowed. Returns `Ok(())` otherwise.
pub fn ensure_url_scheme_allowed(url: &str) -> Result<()> {
    if url_scheme_allowed(url) {
        Ok(())
    } else {
        Err(anyhow!(t!("error-insecure-url", url = url.to_string())))
    }
}

/// The maximum number of redirects to follow.
const MAX_REDIRECTS: usize = 10;

/// How long to wait when establishing a connection before giving up.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// How long a connection may sit without any data arriving before it is
/// considered stalled.
const READ_TIMEOUT: Duration = Duration::from_secs(30);

/// Get a reusable HTTP client.
pub fn http_client() -> Result<reqwest::Client> {
    let allow_insecure = std::env::var_os(ALLOW_INSECURE_URLS_ENV_VAR).is_some();
    let too_many_redirects = t!("error-too-many-redirects");
    let insecure_redirect = t!("error-insecure-redirect");

    let policy = reqwest::redirect::Policy::custom(move |attempt| {
        if attempt.previous().len() >= MAX_REDIRECTS {
            return attempt.error(too_many_redirects.clone());
        }
        if !allow_insecure
            && attempt.url().scheme() == "http"
            && attempt.previous().iter().any(|url| url.scheme() == "https")
        {
            return attempt.error(insecure_redirect.clone());
        }
        attempt.follow()
    });

    Ok(reqwest::ClientBuilder::new()
        .user_agent("gdvm")
        .redirect(policy)
        .connect_timeout(CONNECT_TIMEOUT)
        .read_timeout(READ_TIMEOUT)
        .build()?)
}

/// Digests and size of a completed download.
#[derive(Debug, Clone)]
pub struct DownloadDigests {
    /// SHA 256 sum of the download.
    pub sha256: String,
    /// SHA 512 sum of the download.
    pub sha512: String,
    /// Total number of bytes written.
    pub size: u64,
}

/// Incremental hasher used while streaming a download to disk.
struct StreamHasher {
    sha256: Sha256,
    sha512: Sha512,
    size: u64,
}

impl StreamHasher {
    fn new() -> Self {
        Self {
            sha256: Sha256::new(),
            sha512: Sha512::new(),
            size: 0,
        }
    }

    fn update(&mut self, chunk: &[u8]) {
        self.sha256.update(chunk);
        self.sha512.update(chunk);
        self.size += chunk.len() as u64;
    }

    fn finish(self) -> DownloadDigests {
        DownloadDigests {
            sha256: crate::hash_utils::to_hex(&self.sha256.finalize()),
            sha512: crate::hash_utils::to_hex(&self.sha512.finalize()),
            size: self.size,
        }
    }
}

/// Download `url` to the `dest` file handle.
pub async fn download_to_file(url: &str, dest: &mut tokio::fs::File) -> Result<DownloadDigests> {
    ensure_url_scheme_allowed(url)?;

    // Print downloading URL message
    eprintln_i18n!("operation-downloading-url", url = url);

    let mut hasher = StreamHasher::new();

    // Copy from local file if the registry is on disk.
    if let Some(path) = url.strip_prefix("file://") {
        let src_path = Path::new(path);
        if !src_path.is_file() {
            return Err(anyhow!(t!("error-file-not-found")));
        }
        let mut src = tokio::fs::File::open(src_path).await?;
        let mut buffer = vec![0u8; 64 * 1024];
        loop {
            let read = src.read(&mut buffer).await?;
            if read == 0 {
                break;
            }
            dest.write_all(&buffer[..read]).await?;
            hasher.update(&buffer[..read]);
        }
        dest.flush().await?;
        eprintln_i18n!("operation-download-complete");
        return Ok(hasher.finish());
    }

    let client = http_client()?;
    let response = client.get(url).send().await?;

    match response.status() {
        reqwest::StatusCode::OK => {
            let total_size = response.content_length();
            let pb = if let Some(size) = total_size {
                let pb = ProgressBar::new(size);
                pb.set_style(
                    ProgressStyle::default_bar()
                        .template(
                            "{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})",
                        )?
                        .progress_chars("#>-"),
                );
                pb
            } else {
                crate::progress_utils::spinner(t!("operation-downloading-url", url = url))?
            };

            pb.enable_steady_tick(Duration::from_millis(100));

            let mut stream = response.bytes_stream();

            while let Some(chunk) = stream.next().await {
                let chunk = chunk?;
                dest.write_all(&chunk).await?;
                hasher.update(&chunk);
                if total_size.is_some() {
                    pb.set_position(hasher.size);
                }
            }

            dest.flush().await?;

            pb.finish_with_message(t!("operation-download-complete"));
        }
        reqwest::StatusCode::NOT_FOUND => {
            return Err(anyhow!(t!("error-file-not-found")));
        }
        status => {
            return Err(anyhow!(t!(
                "error-download-failed",
                status = status.to_string(),
            )));
        }
    }

    Ok(hasher.finish())
}

/// The maximum size accepted for responses from the registry, in bytes.
pub const MAX_METADATA_RESPONSE_SIZE: u64 = 64 * 1024 * 1024;

/// Read a response body as text. Refuses bodies larger than `max_bytes`.
pub async fn response_text_limited(response: reqwest::Response, max_bytes: u64) -> Result<String> {
    let url = response.url().to_string();

    let too_large = || {
        anyhow!(t!(
            "error-response-too-large",
            url = url.clone(),
            limit = max_bytes
        ))
    };

    if let Some(len) = response.content_length()
        && len > max_bytes
    {
        return Err(too_large());
    }

    let mut buf: Vec<u8> = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if buf.len() as u64 + chunk.len() as u64 > max_bytes {
            return Err(too_large());
        }
        buf.extend_from_slice(&chunk);
    }

    String::from_utf8(buf).map_err(|e| {
        anyhow!(t!(
            "error-response-not-utf8",
            url = url.clone(),
            error = e.to_string()
        ))
    })
}

/// Download `url` to path at `dest`.
pub async fn download_file(url: &str, dest: &Path) -> Result<DownloadDigests> {
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(dest)
        .await?;

    match download_to_file(url, &mut file).await {
        Ok(digests) => Ok(digests),
        Err(err) => {
            drop(file);
            let _ = tokio::fs::remove_file(dest).await;
            Err(err)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheme_policy_allows_https_and_file_unconditionally() {
        for allow_insecure in [false, true] {
            assert!(url_scheme_allowed_with(
                "https://example.com/x",
                allow_insecure
            ));
            assert!(url_scheme_allowed_with("file:///tmp/x", allow_insecure));
        }
    }

    #[test]
    fn scheme_policy_gates_http_behind_override() {
        assert!(!url_scheme_allowed_with("http://example.com/x", false));
        assert!(url_scheme_allowed_with("http://example.com/x", true));
    }

    #[test]
    fn scheme_policy_rejects_other_schemes() {
        for allow_insecure in [false, true] {
            assert!(!url_scheme_allowed_with(
                "ftp://example.com/x",
                allow_insecure
            ));
            assert!(!url_scheme_allowed_with("example.com/x", allow_insecure));
        }
    }
}
