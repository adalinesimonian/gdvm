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
use std::{path::Path, time::Duration};
use tokio::{fs, io::AsyncWriteExt};

use crate::{eprintln_i18n, i18n::I18n, t};

pub async fn download_file(url: &str, dest: &Path, i18n: &I18n) -> Result<()> {
    // Print downloading URL message
    eprintln_i18n!(i18n, "operation-downloading-url", url = url);

    // Copy from local file if the registry is on disk.
    if let Some(path) = url.strip_prefix("file://") {
        let src = Path::new(path);
        if !src.is_file() {
            return Err(anyhow!(t!(i18n, "error-file-not-found")));
        }
        fs::copy(src, dest).await?;
        eprintln_i18n!(i18n, "operation-download-complete");
        return Ok(());
    }

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
