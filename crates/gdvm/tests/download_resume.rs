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

use gdvm::download_utils::download_to_file;
use sha2::{Digest, Sha256};

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
