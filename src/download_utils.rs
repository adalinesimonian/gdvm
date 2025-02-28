use anyhow::{Result, anyhow};
use std::{
    fs,
    io::{Read, Write},
    path::Path,
    time::Duration,
};

use indicatif::{ProgressBar, ProgressStyle};

use crate::{eprintln_i18n, i18n::I18n};

pub fn download_file(url: &str, dest: &Path, i18n: &I18n) -> Result<()> {
    // Print downloading URL message
    eprintln_i18n!(i18n, "operation-downloading-url", [("url", url)]);

    let response = reqwest::blocking::get(url)?;

    match response.status() {
        reqwest::StatusCode::OK => {
            let total_size = response
                .content_length()
                .ok_or_else(|| anyhow!("Failed to get content length"))?;
            let pb = ProgressBar::new(total_size);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template(
                        "{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})",
                    )?
                    .progress_chars("#>-"),
            );
            pb.enable_steady_tick(Duration::from_millis(100));
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
        }
        reqwest::StatusCode::NOT_FOUND => {
            return Err(anyhow!(i18n.t("error-file-not-found")));
        }
        status => {
            return Err(anyhow!(i18n.t_args(
                "error-download-failed",
                &[(
                    "status",
                    fluent_bundle::FluentValue::from(status.to_string())
                )]
            )));
        }
    }
    Ok(())
}
