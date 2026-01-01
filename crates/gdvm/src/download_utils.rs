use anyhow::{Result, anyhow};
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::{path::Path, time::Duration};
use tokio::{fs, io::AsyncWriteExt};

use crate::{eprintln_i18n, i18n::I18n, t};

pub async fn download_file(url: &str, dest: &Path, i18n: &I18n) -> Result<()> {
    // Print downloading URL message
    eprintln_i18n!(i18n, "operation-downloading-url", url = url);

    let client = reqwest::ClientBuilder::new().user_agent("gdvm").build()?;
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
                let pb = ProgressBar::new_spinner();
                pb.set_style(ProgressStyle::default_spinner().template("{spinner:.green} {msg}")?);
                pb.set_message(t!(i18n, "operation-downloading-url", url = url));
                pb
            };

            pb.enable_steady_tick(Duration::from_millis(100));

            let mut file = fs::File::create(dest).await?;
            let mut downloaded: u64 = 0;
            let mut stream = response.bytes_stream();

            while let Some(chunk) = stream.next().await {
                let chunk = chunk?;
                file.write_all(&chunk).await?;
                downloaded += chunk.len() as u64;
                if total_size.is_some() {
                    pb.set_position(downloaded);
                }
            }

            pb.finish_with_message(t!(i18n, "operation-download-complete"));
        }
        reqwest::StatusCode::NOT_FOUND => {
            return Err(anyhow!(t!(i18n, "error-file-not-found")));
        }
        status => {
            return Err(anyhow!(t!(
                i18n,
                "error-download-failed",
                status = status.to_string(),
            )));
        }
    }
    Ok(())
}
