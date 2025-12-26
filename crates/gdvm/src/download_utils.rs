use anyhow::{Result, anyhow};
use std::{
    fs,
    io::{Read, Write},
    path::Path,
    time::Duration,
};

use indicatif::{ProgressBar, ProgressStyle};

use crate::{eprintln_i18n, i18n::I18n, t};

pub fn download_file(url: &str, dest: &Path, i18n: &I18n) -> Result<()> {
    // Print downloading URL message
    eprintln_i18n!(i18n, "operation-downloading-url", url = url);

    let response = reqwest::blocking::get(url)?;

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
