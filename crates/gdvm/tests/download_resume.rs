// SPDX-FileCopyrightText: Copyright (C) 2026 Adaline Simonian
// SPDX-License-Identifier: GPL-3.0-or-later
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;

use gdvm::download_utils::{
    ExpectedDigests, PriorPartial, download_to_file, download_to_file_resuming, download_verified,
};
use sha2::{Digest, Sha256, Sha512};

/// A scripted response for an incoming connection.
enum Script {
    /// A 200 but with a response that's cut off.
    Truncated { send: usize, resumable: bool },
    /// A 206 with a remainder of the file.
    Range,
    /// 200 with the complete body.
    Full,
    /// A bare status response.
    Status(u16),
}

/// The request lines observed by the server.
type Requests = Arc<Mutex<Vec<String>>>;

/// Serve the scripted responses for `content` on a loopback port, one
/// connection per script entry, and record each request's headers.
fn serve(content: Vec<u8>, scripts: Vec<Script>) -> (String, Requests) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let requests: Requests = Arc::new(Mutex::new(Vec::new()));
    let seen = Arc::clone(&requests);

    thread::spawn(move || {
        for script in scripts {
            let (mut stream, _) = listener.accept().unwrap();
            let request = read_request(&mut stream);

            seen.lock().unwrap().push(request.clone());

            match script {
                Script::Truncated { send, resumable } => {
                    let extra = if resumable {
                        "Accept-Ranges: bytes\r\nETag: \"v1\"\r\n"
                    } else {
                        ""
                    };
                    let header = format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n{extra}Connection: close\r\n\r\n",
                        content.len()
                    );
                    stream.write_all(header.as_bytes()).unwrap();
                    stream.write_all(&content[..send]).unwrap();
                    // Whoops, dropped the connection while writing the body...
                }
                Script::Range => {
                    let from = request
                        .lines()
                        .find_map(|l| l.strip_prefix("range: bytes="))
                        .and_then(|r| r.strip_suffix('-'))
                        .and_then(|n| n.parse::<usize>().ok())
                        .expect("client did not send a Range header");
                    let header = format!(
                        "HTTP/1.1 206 Partial Content\r\nContent-Length: {}\r\nContent-Range: bytes {}-{}/{}\r\nConnection: close\r\n\r\n",
                        content.len() - from,
                        from,
                        content.len() - 1,
                        content.len()
                    );

                    stream.write_all(header.as_bytes()).unwrap();
                    stream.write_all(&content[from..]).unwrap();
                }
                Script::Full => {
                    let header = format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        content.len()
                    );
                    stream.write_all(header.as_bytes()).unwrap();
                    stream.write_all(&content).unwrap();
                }
                Script::Status(code) => {
                    let header = format!(
                        "HTTP/1.1 {code} X\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                    );
                    stream.write_all(header.as_bytes()).unwrap();
                }
            }
        }
    });

    (format!("http://127.0.0.1:{port}/file.bin"), requests)
}

/// Read one HTTP request.
fn read_request(stream: &mut std::net::TcpStream) -> String {
    let mut buf = Vec::new();
    let mut byte = [0u8; 1];
    while !buf.ends_with(b"\r\n\r\n") {
        if stream.read(&mut byte).unwrap() == 0 {
            break;
        }
        buf.push(byte[0]);
    }
    String::from_utf8_lossy(&buf).to_lowercase()
}

fn test_content() -> Vec<u8> {
    (0..300000u32).flat_map(|i| i.to_le_bytes()).collect()
}

fn allow_loopback_http() {
    unsafe { std::env::set_var("GDVM_ALLOW_INSECURE_URLS", "1") };
}

async fn download(url: &str) -> anyhow::Result<(gdvm::download_utils::DownloadDigests, Vec<u8>)> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("out.bin");
    let mut file = tokio::fs::File::options()
        .create_new(true)
        .read(true)
        .write(true)
        .open(&path)
        .await?;
    let digests = download_to_file(url, &mut file, "test").await?;
    Ok((digests, std::fs::read(&path)?))
}

#[tokio::test]
async fn resumes_after_connection_drop() {
    allow_loopback_http();
    let content = test_content();
    let half = content.len() / 2;
    let (url, requests) = serve(
        content.clone(),
        vec![
            Script::Truncated {
                send: half,
                resumable: true,
            },
            Script::Range,
        ],
    );

    let (digests, bytes) = download(&url).await.unwrap();

    assert_eq!(bytes, content);
    assert_eq!(digests.size, content.len() as u64);
    assert_eq!(
        digests.sha256,
        gdvm::hash_utils::to_hex(&Sha256::digest(&content)),
        "digests must cover the resumed file exactly"
    );

    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 2);
    assert!(!requests[0].contains("range:"));
    assert!(requests[1].contains(&format!("range: bytes={half}-")));
    assert!(
        requests[1].contains("if-range: \"v1\""),
        "resume must be validated so a changed file is never spliced"
    );
}

#[tokio::test]
async fn restarts_from_scratch_without_range_support() {
    allow_loopback_http();
    let content = test_content();
    let (url, requests) = serve(
        content.clone(),
        vec![
            Script::Truncated {
                send: content.len() / 3,
                resumable: false,
            },
            Script::Full,
        ],
    );

    let (digests, bytes) = download(&url).await.unwrap();

    assert_eq!(bytes, content);
    assert_eq!(
        digests.sha256,
        gdvm::hash_utils::to_hex(&Sha256::digest(&content))
    );

    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 2);
    assert!(
        !requests[1].contains("range:"),
        "no Range request without server support"
    );
}

#[tokio::test]
async fn retries_transient_server_errors() {
    allow_loopback_http();
    let content = test_content();
    let (url, requests) = serve(content.clone(), vec![Script::Status(503), Script::Full]);

    let (digests, bytes) = download(&url).await.unwrap();

    assert_eq!(bytes, content);
    assert_eq!(
        digests.sha256,
        gdvm::hash_utils::to_hex(&Sha256::digest(&content))
    );
    assert_eq!(requests.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn permanent_errors_do_not_retry() {
    allow_loopback_http();
    let (url, requests) = serve(test_content(), vec![Script::Status(404)]);

    assert!(download(&url).await.is_err());
    assert_eq!(
        requests.lock().unwrap().len(),
        1,
        "a 404 will not change; retrying it wastes time and requests"
    );
}

/// Download `content`, resuming it from `prefix_len` bytes.
async fn download_resuming(
    url: &str,
    content: &[u8],
    prefix_len: usize,
    validator: &str,
) -> anyhow::Result<(
    gdvm::download_utils::DownloadDigests,
    Vec<u8>,
    Option<String>,
)> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("out.bin");
    let meta = dir.path().join("out.bin.meta");
    std::fs::write(&path, &content[..prefix_len])?;

    let mut file = tokio::fs::File::options()
        .read(true)
        .write(true)
        .open(&path)
        .await?;
    let digests = download_to_file_resuming(
        url,
        &mut file,
        Some(PriorPartial {
            downloaded: prefix_len as u64,
            validator: validator.to_string(),
        }),
        Some(&meta),
        "test",
    )
    .await?;
    let persisted = std::fs::read_to_string(&meta).ok();
    Ok((digests, std::fs::read(&path)?, persisted))
}

#[tokio::test]
async fn resumes_a_previous_runs_partial_download() {
    allow_loopback_http();
    let content = test_content();
    let half = content.len() / 2;
    let (url, requests) = serve(content.clone(), vec![Script::Range]);

    let (digests, bytes, _) = download_resuming(&url, &content, half, "\"v1\"")
        .await
        .unwrap();

    assert_eq!(bytes, content);
    assert_eq!(
        digests.sha256,
        gdvm::hash_utils::to_hex(&Sha256::digest(&content)),
        "digests must cover prior bytes and resumed bytes together"
    );

    let first = &requests.lock().unwrap()[0];
    assert!(
        first.contains(&format!("range: bytes={half}-")),
        "the request must resume from the prior offset: {first}"
    );
    assert!(
        first.contains("if-range: \"v1\""),
        "the request must carry the persisted validator: {first}"
    );
}

#[tokio::test]
async fn restarts_when_the_server_rejects_the_resume() {
    allow_loopback_http();
    let content = test_content();
    let half = content.len() / 2;
    let (url, _) = serve(content.clone(), vec![Script::Full]);

    let stale: Vec<u8> = vec![0xAB; half];
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("out.bin");
    let meta = dir.path().join("out.bin.meta");
    std::fs::write(&path, &stale).unwrap();

    let mut file = tokio::fs::File::options()
        .read(true)
        .write(true)
        .open(&path)
        .await
        .unwrap();
    let digests = download_to_file_resuming(
        url.as_str(),
        &mut file,
        Some(PriorPartial {
            downloaded: half as u64,
            validator: "\"old\"".to_string(),
        }),
        Some(&meta),
        "test",
    )
    .await
    .unwrap();

    assert_eq!(std::fs::read(&path).unwrap(), content);
    assert_eq!(
        digests.sha256,
        gdvm::hash_utils::to_hex(&Sha256::digest(&content)),
        "a rejected resume must restart cleanly from zero"
    );
}

#[tokio::test]
async fn persists_the_validator_for_the_next_run() {
    allow_loopback_http();
    let content = test_content();
    let full_len = content.len();
    let (url, _) = serve(
        content.clone(),
        vec![Script::Truncated {
            send: full_len,
            resumable: true,
        }],
    );

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("out.bin");
    let meta = dir.path().join("out.bin.meta");
    let mut file = tokio::fs::File::options()
        .create_new(true)
        .read(true)
        .write(true)
        .open(&path)
        .await
        .unwrap();

    download_to_file_resuming(url.as_str(), &mut file, None, Some(&meta), "test")
        .await
        .unwrap();

    assert_eq!(
        std::fs::read_to_string(&meta).unwrap(),
        "\"v1\"",
        "the validator must be persisted as soon as the transfer starts"
    );
}

#[tokio::test]
async fn corrupted_prior_bytes_yield_a_mismatching_digest() {
    allow_loopback_http();
    let content = test_content();
    let half = content.len() / 2;
    let (url, _) = serve(content.clone(), vec![Script::Range]);

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("out.bin");
    let meta = dir.path().join("out.bin.meta");
    let mut corrupted = content[..half].to_vec();
    corrupted[0] ^= 0xFF;
    std::fs::write(&path, &corrupted).unwrap();

    let mut file = tokio::fs::File::options()
        .read(true)
        .write(true)
        .open(&path)
        .await
        .unwrap();
    let digests = download_to_file_resuming(
        url.as_str(),
        &mut file,
        Some(PriorPartial {
            downloaded: half as u64,
            validator: "\"v1\"".to_string(),
        }),
        Some(&meta),
        "test",
    )
    .await
    .unwrap();

    assert_ne!(
        digests.sha256,
        gdvm::hash_utils::to_hex(&Sha256::digest(&content)),
        "the digest must expose the corruption for the caller to catch"
    );
}

fn sha512_hex(content: &[u8]) -> String {
    gdvm::hash_utils::to_hex(&Sha512::digest(content))
}

#[tokio::test]
async fn verified_download_lands_only_at_the_final_path() {
    allow_loopback_http();
    let content = test_content();
    let (url, _) = serve(content.clone(), vec![Script::Full]);

    let dir = tempfile::tempdir().unwrap();
    let final_path = dir.path().join("artifact.zip");
    let partial = dir.path().join(".partial-artifact.zip");
    let meta = dir.path().join(".partial-artifact.zip.meta");

    let sha = sha512_hex(&content);
    download_verified(
        url.as_str(),
        &final_path,
        &partial,
        &meta,
        ExpectedDigests {
            sha: &sha,
            size: Some(content.len() as u64),
        },
        "test",
    )
    .await
    .unwrap();

    assert_eq!(std::fs::read(&final_path).unwrap(), content);
    assert!(!partial.exists(), "the partial must be renamed away");
    assert!(!meta.exists(), "the sidecar must be cleaned up");
}

#[tokio::test]
async fn corrupted_prior_bytes_trigger_one_fresh_redownload() {
    allow_loopback_http();
    let content = test_content();
    let half = content.len() / 2;
    let (url, requests) = serve(content.clone(), vec![Script::Range, Script::Full]);

    let dir = tempfile::tempdir().unwrap();
    let final_path = dir.path().join("artifact.zip");
    let partial = dir.path().join(".partial-artifact.zip");
    let meta = dir.path().join(".partial-artifact.zip.meta");

    let mut corrupted = content[..half].to_vec();
    corrupted[0] ^= 0xFF;
    std::fs::write(&partial, &corrupted).unwrap();
    std::fs::write(&meta, "\"v1\"").unwrap();

    let sha = sha512_hex(&content);
    download_verified(
        url.as_str(),
        &final_path,
        &partial,
        &meta,
        ExpectedDigests {
            sha: &sha,
            size: None,
        },
        "test",
    )
    .await
    .unwrap();

    assert_eq!(
        std::fs::read(&final_path).unwrap(),
        content,
        "the fresh retry must produce the correct file"
    );
    assert!(!partial.exists());
    assert!(!meta.exists());

    let requests = requests.lock().unwrap();
    assert!(
        requests[0].contains(&format!("range: bytes={half}-")),
        "the first attempt must have tried to resume: {}",
        requests[0]
    );
    assert!(
        !requests[1].contains("range: bytes="),
        "the retry must download from scratch: {}",
        requests[1]
    );
}

#[tokio::test]
async fn a_fresh_download_with_a_wrong_digest_is_an_error() {
    allow_loopback_http();
    let content = test_content();
    let (url, _) = serve(content.clone(), vec![Script::Full]);

    let dir = tempfile::tempdir().unwrap();
    let final_path = dir.path().join("artifact.zip");
    let partial = dir.path().join(".partial-artifact.zip");
    let meta = dir.path().join(".partial-artifact.zip.meta");

    let wrong = sha512_hex(b"different content entirely");
    let err = download_verified(
        url.as_str(),
        &final_path,
        &partial,
        &meta,
        ExpectedDigests {
            sha: &wrong,
            size: None,
        },
        "test",
    )
    .await
    .expect_err("a fresh mismatch is a registry problem, not a retry");

    let message = format!("{err}");
    assert!(message.contains("artifact.zip"), "{message}");
    assert!(!final_path.exists(), "nothing may land at the final path");
    assert!(!partial.exists(), "the failed download must not linger");
}
