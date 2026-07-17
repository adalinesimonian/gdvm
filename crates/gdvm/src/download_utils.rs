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

use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use futures_util::StreamExt;
use sha2::{Digest, Sha256, Sha512};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::ui::progress::Task;
use crate::ui::{self};
use crate::{t, terr};

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
        Err(terr!("error-insecure-url", url = url.to_string()))
    }
}

/// The maximum number of redirects to follow.
const MAX_REDIRECTS: usize = 10;

/// How long to wait when establishing a connection before giving up.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// How long a connection may sit without any data arriving before it is
/// considered stalled.
const READ_TIMEOUT: Duration = Duration::from_secs(30);

/// How many times to try a download before giving up.
const MAX_ATTEMPTS: u32 = 4;

/// Base delay for exponential backoff between retry attempts.
const RETRY_BASE_DELAY: Duration = Duration::from_millis(500);

/// The longest gdvm should wait between retry attempts.
const RETRY_MAX_DELAY: Duration = Duration::from_secs(15);

/// Get whether a response status should be retried.
fn is_retryable_status(status: reqwest::StatusCode) -> bool {
    status.is_server_error()
        || status == reqwest::StatusCode::REQUEST_TIMEOUT
        || status == reqwest::StatusCode::TOO_MANY_REQUESTS
}

/// Get the seconds from a `Retry-After` header, if one is present.
fn parse_retry_after(headers: &reqwest::header::HeaderMap) -> Option<Duration> {
    headers
        .get(reqwest::header::RETRY_AFTER)?
        .to_str()
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()
        .map(Duration::from_secs)
}

/// Get the delay, in milliseconds, for the next retry attempt.
fn retry_delay(attempt: u32, retry_after: Option<Duration>) -> Duration {
    if let Some(delay) = retry_after {
        return delay.min(RETRY_MAX_DELAY);
    }

    let exp = RETRY_BASE_DELAY.saturating_mul(1u32 << (attempt - 1).min(16));

    // Use jitter from 0.5 to 1.5 seconds to avoid hitting the server at the
    // same time as other clients.
    let jitter = 500 + fastrand::u32(0..=1000);
    (exp.saturating_mul(jitter) / 1000).min(RETRY_MAX_DELAY)
}

/// How a transfer attempt failed.
enum TransferError {
    /// The error is transient, i.e. it can be retried (like rate limits).
    Transient {
        error: anyhow::Error,
        retry_after: Option<Duration>,
    },
    /// The error is permanent, e.g. a 404 or something like it.
    Permanent(anyhow::Error),
}

/// Send a GET request with retries.
pub(crate) async fn get_retrying(
    client: &reqwest::Client,
    url: &str,
    request_timeout: Option<Duration>,
) -> Result<reqwest::Response> {
    let mut attempt = 1;
    loop {
        let mut request = client.get(url);
        if let Some(timeout) = request_timeout {
            request = request.timeout(timeout);
        }
        let outcome = match request.send().await {
            Ok(response) => {
                let status = response.status();
                if is_retryable_status(status) {
                    let retry_after = parse_retry_after(response.headers());
                    Err(TransferError::Transient {
                        error: terr!("error-download-failed", status = status.to_string()),
                        retry_after,
                    })
                } else {
                    Ok(response)
                }
            }
            Err(e) => Err(TransferError::Transient {
                error: e.into(),
                retry_after: None,
            }),
        };

        match outcome {
            Ok(response) => return Ok(response),
            Err(TransferError::Permanent(error)) => return Err(error),
            Err(TransferError::Transient { error, retry_after }) => {
                if attempt >= MAX_ATTEMPTS {
                    return Err(error);
                }
                tokio::time::sleep(retry_delay(attempt, retry_after)).await;
                attempt += 1;
            }
        }
    }
}

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

/// Digests for verifying a given download.
pub struct ExpectedDigests<'a> {
    /// The checksum.
    pub sha: &'a str,
    /// The expected size in bytes.
    pub size: Option<u64>,
}

/// Download `url` into `dest`, resuming any partial download and verifying the
/// download against `expected`.
pub async fn download_verified(
    url: &str,
    dest: &Path,
    partial_path: &Path,
    meta_path: &Path,
    expected: ExpectedDigests<'_>,
    subject: &str,
) -> Result<std::fs::File> {
    // For local registries.
    let remote = !url.starts_with("file://");

    let read_prior = || -> Option<PriorPartial> {
        if !remote {
            return None;
        }
        let downloaded = std::fs::metadata(partial_path).ok()?.len();
        let validator = std::fs::read_to_string(meta_path).ok()?;
        let validator = validator.trim().to_string();
        (downloaded > 0 && !validator.is_empty()).then_some(PriorPartial {
            downloaded,
            validator,
        })
    };

    let mut resumed = read_prior().is_some();
    loop {
        let prior = read_prior();
        if let Some(prior) = &prior {
            let (value, unit) = crate::fs_utils::byte_display_args(prior.downloaded);
            ui::note(t!("download-resuming", value = value, unit = unit));
        } else {
            let _ = std::fs::remove_file(partial_path);
        }

        let mut file = tokio::fs::File::options()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(partial_path)
            .await?;
        let digests =
            download_to_file_resuming(url, &mut file, prior, remote.then_some(meta_path), subject)
                .await?;
        drop(file);

        match verify_digests(&digests, &expected, dest) {
            Ok(()) => break,
            Err(error) => {
                let _ = std::fs::remove_file(partial_path);
                let _ = std::fs::remove_file(meta_path);
                if !resumed {
                    return Err(error);
                }
                resumed = false;
                ui::warn(t!("warning-resume-verification-failed"));
            }
        }
    }

    let _ = std::fs::remove_file(meta_path);
    std::fs::rename(partial_path, dest)?;
    Ok(std::fs::File::open(dest)?)
}

/// Verify a download's digests and size.
fn verify_digests(
    digests: &DownloadDigests,
    expected: &ExpectedDigests<'_>,
    display_path: &Path,
) -> Result<()> {
    let sha_type = crate::hash_utils::ShaType::from_expected(expected.sha)?;
    let actual = match sha_type {
        crate::hash_utils::ShaType::Sha256 => &digests.sha256,
        crate::hash_utils::ShaType::Sha512 => &digests.sha512,
    };
    if !actual.eq_ignore_ascii_case(expected.sha) {
        return Err(crate::hash_utils::checksum_mismatch_error(display_path));
    }

    if let Some(expected_size) = expected.size
        && digests.size != expected_size
    {
        return Err(terr!(
            "error-size-mismatch",
            file = display_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            expected = expected_size,
            actual = digests.size
        ));
    }

    Ok(())
}

/// A leftover partial download.
pub struct PriorPartial {
    /// Bytes on disk.
    pub downloaded: u64,
    /// The validator from the HTTP request from when the download was
    /// interrupted.
    pub validator: String,
}

/// Download `url` to the `dest` file handle. `subject` is the text sent to the
/// progress indicator describing what's being downloaded.
pub async fn download_to_file(
    url: &str,
    dest: &mut tokio::fs::File,
    subject: &str,
) -> Result<DownloadDigests> {
    download_to_file_resuming(url, dest, None, None, subject).await
}

/// Resume a download from a prior partial, if one exists. Otherwise, download
/// from scratch.
pub async fn download_to_file_resuming(
    url: &str,
    dest: &mut tokio::fs::File,
    prior: Option<PriorPartial>,
    validator_sink: Option<&Path>,
    subject: &str,
) -> Result<DownloadDigests> {
    ensure_url_scheme_allowed(url)?;

    // Copy from local file if the registry is on disk.
    if let Some(path) = url.strip_prefix("file://") {
        let mut hasher = StreamHasher::new();
        let src_path = Path::new(path);
        if !src_path.is_file() {
            return Err(terr!("error-file-not-found"));
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
        return Ok(hasher.finish());
    }

    let client = http_client()?;
    let mut state = TransferState {
        validator_sink: validator_sink.map(Path::to_path_buf),
        ..TransferState::default()
    };
    if let Some(prior) = prior {
        state.downloaded = prior.downloaded;
        state.validator = Some(prior.validator);
        state.range_supported = true;
    }
    let mut attempt = 1;

    loop {
        match transfer_attempt(&client, url, dest, &mut state, subject).await {
            Ok(digests) => break Ok(digests),
            Err(TransferError::Permanent(error)) => break Err(error),
            Err(TransferError::Transient { error, retry_after }) => {
                if attempt >= MAX_ATTEMPTS {
                    break Err(error);
                }
                state.println(t!(
                    "download-retrying",
                    attempt = attempt,
                    max = MAX_ATTEMPTS - 1
                ));
                tokio::time::sleep(retry_delay(attempt, retry_after)).await;
                attempt += 1;
            }
        }
    }
}

/// Transfer state for a download, across all retries.
#[derive(Default)]
struct TransferState {
    /// Verified bytes written to disk.
    downloaded: u64,
    /// `ETag` or `Last-Modified` header from the server, if any, used to
    /// validate that a resumed range request is still valid.
    validator: Option<String>,
    /// Whether or not the server supports range requests.
    range_supported: bool,
    /// The total size of the file, if known.
    total: Option<u64>,
    /// The progress indicator.
    task: Option<Task>,
    /// Where to store the validator when the server provides it.
    validator_sink: Option<std::path::PathBuf>,
}

impl TransferState {
    /// Check if the transfer can be resumed.
    fn can_resume(&self) -> bool {
        self.downloaded > 0 && self.range_supported && self.validator.is_some()
    }

    /// Print a line, keeping it clear of the indicator if one exists.
    fn println(&self, message: String) {
        match &self.task {
            Some(task) => task.print_above(message),
            None => ui::with_suspended(|| eprintln!("{message}")),
        }
    }
}

/// Make one attempt to make the transfer.
async fn transfer_attempt(
    client: &reqwest::Client,
    url: &str,
    dest: &mut tokio::fs::File,
    state: &mut TransferState,
    subject: &str,
) -> Result<DownloadDigests, TransferError> {
    use tokio::io::AsyncSeekExt;

    let resuming = state.can_resume();
    if !resuming {
        // Restart from scratch.
        reset_dest(dest, state)
            .await
            .map_err(TransferError::Permanent)?;
    }

    let mut request = client.get(url);
    if resuming {
        request = request
            .header(
                reqwest::header::RANGE,
                format!("bytes={}-", state.downloaded),
            )
            .header(
                reqwest::header::IF_RANGE,
                state.validator.clone().unwrap_or_default(),
            );
    }

    let response = request.send().await.map_err(|e| TransferError::Transient {
        error: e.into(),
        retry_after: None,
    })?;

    match response.status() {
        reqwest::StatusCode::OK => {
            reset_dest(dest, state)
                .await
                .map_err(TransferError::Permanent)?;
            state.total = response.content_length();
            state.range_supported = response
                .headers()
                .get(reqwest::header::ACCEPT_RANGES)
                .and_then(|v| v.to_str().ok())
                .is_some_and(|v| v.eq_ignore_ascii_case("bytes"));
            state.validator = response
                .headers()
                .get(reqwest::header::ETAG)
                .or_else(|| response.headers().get(reqwest::header::LAST_MODIFIED))
                .and_then(|v| v.to_str().ok())
                .map(str::to_string);

            if let Some(sink) = &state.validator_sink {
                // If this fails, that's fine, next download will just start
                // from scratch.
                let _ = std::fs::write(sink, state.validator.as_deref().unwrap_or_default());
            }
        }
        reqwest::StatusCode::PARTIAL_CONTENT => {
            let content_range = response
                .headers()
                .get(reqwest::header::CONTENT_RANGE)
                .and_then(|v| v.to_str().ok())
                .and_then(parse_content_range);

            let starts_at_prefix = content_range.map(|(start, _)| start) == Some(state.downloaded);
            if !starts_at_prefix {
                state.range_supported = false;
                return Err(TransferError::Transient {
                    error: terr!(
                        "error-download-failed",
                        status = response.status().to_string()
                    ),
                    retry_after: None,
                });
            }

            state.total = content_range
                .and_then(|(_, total)| total)
                .or_else(|| {
                    response
                        .content_length()
                        .map(|remaining| state.downloaded + remaining)
                })
                .or(state.total);
        }
        reqwest::StatusCode::NOT_FOUND => {
            return Err(TransferError::Permanent(terr!("error-file-not-found")));
        }
        status if is_retryable_status(status) => {
            let retry_after = parse_retry_after(response.headers());

            return Err(TransferError::Transient {
                error: terr!("error-download-failed", status = status.to_string()),
                retry_after,
            });
        }
        status => {
            return Err(TransferError::Permanent(terr!(
                "error-download-failed",
                status = status.to_string()
            )));
        }
    }

    let mut hasher = rehash_prefix(dest, state.downloaded)
        .await
        .map_err(TransferError::Permanent)?;

    update_progress(state, subject);

    let mut stream = response.bytes_stream();
    loop {
        match stream.next().await {
            Some(Ok(chunk)) => {
                dest.write_all(&chunk)
                    .await
                    .map_err(|e| TransferError::Permanent(e.into()))?;
                hasher.update(&chunk);
                if let Some(task) = &state.task
                    && state.total.is_some()
                {
                    task.set_position(hasher.size);
                }
            }
            Some(Err(e)) => {
                dest.flush()
                    .await
                    .map_err(|e| TransferError::Permanent(e.into()))?;
                state.downloaded = hasher.size;
                return Err(TransferError::Transient {
                    error: e.into(),
                    retry_after: None,
                });
            }
            None => break,
        }
    }

    dest.flush()
        .await
        .map_err(|e| TransferError::Permanent(e.into()))?;

    if let Some(total) = state.total
        && hasher.size < total
    {
        state.downloaded = hasher.size;
        return Err(TransferError::Transient {
            error: terr!(
                "error-size-mismatch",
                expected = total,
                actual = hasher.size
            ),
            retry_after: None,
        });
    }

    let _ = dest.seek(std::io::SeekFrom::End(0)).await;
    Ok(hasher.finish())
}

/// Reset the downloaded file to empty and reset the transfer state.
async fn reset_dest(dest: &mut tokio::fs::File, state: &mut TransferState) -> Result<()> {
    use tokio::io::AsyncSeekExt;
    dest.set_len(0).await?;
    dest.seek(std::io::SeekFrom::Start(0)).await?;
    state.downloaded = 0;
    Ok(())
}

/// Hash the first `len` bytes of the downloaded file, so that it can be used to
/// resume a download and continue hashing from that point.
async fn rehash_prefix(dest: &mut tokio::fs::File, len: u64) -> Result<StreamHasher> {
    use tokio::io::AsyncSeekExt;

    let mut hasher = StreamHasher::new();
    dest.set_len(len).await?;
    dest.seek(std::io::SeekFrom::Start(0)).await?;

    let mut remaining = len;
    let mut buffer = vec![0u8; 64 * 1024];
    while remaining > 0 {
        let want = remaining.min(buffer.len() as u64) as usize;
        let read = dest.read(&mut buffer[..want]).await?;
        if read == 0 {
            return Err(terr!(
                "error-size-mismatch",
                expected = len,
                actual = len - remaining
            ));
        }
        hasher.update(&buffer[..read]);
        remaining -= read as u64;
    }

    dest.seek(std::io::SeekFrom::Start(len)).await?;
    Ok(hasher)
}

/// Parse a `Content-Range` header into a `(start, total)` tuple.
fn parse_content_range(value: &str) -> Option<(u64, Option<u64>)> {
    let value = value.strip_prefix("bytes ")?;
    let (range, total) = value.split_once('/')?;
    let start = range.split('-').next()?.parse::<u64>().ok()?;
    let total = match total.trim() {
        "*" => None,
        total => Some(total.parse::<u64>().ok()?),
    };
    Some((start, total))
}

/// Update the progress bar. Creates one if it hasn't been made yet.
fn update_progress(state: &mut TransferState, subject: &str) {
    match &state.task {
        Some(task) => {
            if let Some(total) = state.total {
                task.set_length(total);
            }
            task.set_position(state.downloaded);
        }
        None => {
            let task = ui::progress::transfer(t!("status-downloading"), subject, state.total);
            task.set_position(state.downloaded);
            state.task = Some(task);
        }
    }
}

/// The maximum size accepted for responses from the registry, in bytes.
pub const MAX_METADATA_RESPONSE_SIZE: u64 = 64 * 1024 * 1024;

/// Read a response body as text. Refuses bodies larger than `max_bytes`.
pub async fn response_text_limited(response: reqwest::Response, max_bytes: u64) -> Result<String> {
    let url = response.url().to_string();

    let too_large = || {
        terr!(
            "error-response-too-large",
            url = url.clone(),
            limit = max_bytes
        )
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
        terr!(
            "error-response-not-utf8",
            url = url.clone(),
            error = e.to_string()
        )
    })
}

/// Download `url` to path at `dest`.
pub async fn download_file(url: &str, dest: &Path) -> Result<DownloadDigests> {
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(dest)
        .await?;

    let subject = dest
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();

    match download_to_file(url, &mut file, &subject).await {
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

    #[test]
    fn retryable_statuses() {
        assert!(is_retryable_status(
            reqwest::StatusCode::INTERNAL_SERVER_ERROR
        ));
        assert!(is_retryable_status(
            reqwest::StatusCode::SERVICE_UNAVAILABLE
        ));
        assert!(is_retryable_status(reqwest::StatusCode::TOO_MANY_REQUESTS));
        assert!(is_retryable_status(reqwest::StatusCode::REQUEST_TIMEOUT));
        assert!(!is_retryable_status(reqwest::StatusCode::NOT_FOUND));
        assert!(!is_retryable_status(reqwest::StatusCode::FORBIDDEN));
        assert!(!is_retryable_status(reqwest::StatusCode::OK));
    }

    #[test]
    fn retry_delay_backs_off_and_caps() {
        for attempt in 1..=10 {
            let delay = retry_delay(attempt, None);
            assert!(delay >= RETRY_BASE_DELAY / 2);
            assert!(delay <= RETRY_MAX_DELAY);
        }
        assert_eq!(
            retry_delay(1, Some(Duration::from_secs(2))),
            Duration::from_secs(2)
        );
        assert_eq!(
            retry_delay(1, Some(Duration::from_secs(600))),
            RETRY_MAX_DELAY
        );
    }

    #[test]
    fn retry_after_parses_delay_seconds_only() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(reqwest::header::RETRY_AFTER, "3".parse().unwrap());
        assert_eq!(parse_retry_after(&headers), Some(Duration::from_secs(3)));

        headers.insert(
            reqwest::header::RETRY_AFTER,
            "Fri, 21 Jul 2000 06:00:00 GMT".parse().unwrap(),
        );
        assert_eq!(parse_retry_after(&headers), None);

        assert_eq!(parse_retry_after(&reqwest::header::HeaderMap::new()), None);
    }

    #[test]
    fn content_range_parses_start_and_total() {
        assert_eq!(
            parse_content_range("bytes 100-199/500"),
            Some((100, Some(500)))
        );
        assert_eq!(parse_content_range("bytes 0-67/68"), Some((0, Some(68))));
        assert_eq!(parse_content_range("bytes 100-199/*"), Some((100, None)));
        assert_eq!(parse_content_range("bytes 100-199"), None);
        assert_eq!(parse_content_range("items 100-199/500"), None);
        assert_eq!(parse_content_range("bytes x-199/500"), None);
        assert_eq!(parse_content_range("bytes 100-199/x"), None);
    }

    #[test]
    fn resumed_total_prefers_header_then_length() {
        let downloaded = 100u64;
        let from_header = Some((downloaded, Some(500u64)));
        let content_length = Some(400u64);

        let total = from_header
            .and_then(|(_, total)| total)
            .or_else(|| content_length.map(|remaining| downloaded + remaining));
        assert_eq!(total, Some(500));

        let total_without_header = None::<(u64, Option<u64>)>
            .and_then(|(_, total)| total)
            .or_else(|| content_length.map(|remaining| downloaded + remaining));
        assert_eq!(total_without_header, Some(500));
    }

    #[tokio::test]
    async fn rehash_prefix_matches_single_pass_hash() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("data");
        let content: Vec<u8> = (0..200000u32).flat_map(|i| i.to_le_bytes()).collect();
        let (prefix, rest) = content.split_at(300001);

        let mut file = tokio::fs::File::options()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(&path)
            .await?;
        file.write_all(prefix).await?;
        // Nothing past the prefix should be used.
        file.write_all(b"garbage").await?;
        file.flush().await?;

        let mut hasher = rehash_prefix(&mut file, prefix.len() as u64).await?;
        file.write_all(rest).await?;
        hasher.update(rest);
        let resumed = hasher.finish();

        let mut single = StreamHasher::new();
        single.update(&content);
        let expected = single.finish();

        assert_eq!(resumed.sha256, expected.sha256);
        assert_eq!(resumed.sha512, expected.sha512);
        assert_eq!(resumed.size, expected.size);
        assert_eq!(tokio::fs::read(&path).await?, content);
        Ok(())
    }
}
