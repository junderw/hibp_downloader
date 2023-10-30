use std::{collections::VecDeque, path::Path, sync::atomic};
use tokio::io::AsyncWriteExt;

use super::stats::WRITTEN_TO_FILE;

pub struct BufferedStringWriter {
    files: VecDeque<(u32, String)>,
    writer: tokio::io::BufWriter<tokio::fs::File>,
}

impl BufferedStringWriter {
    pub async fn from_file(filename: &Path) -> Result<Self, std::io::Error> {
        Ok(Self {
            files: VecDeque::with_capacity(1024),
            writer: tokio::io::BufWriter::with_capacity(
                1024 * 1024 * 32, // 1 download is around 32kB. This fits around 1024 downloads.
                tokio::fs::File::create(filename).await?,
            ),
        })
    }

    pub async fn add_file(&mut self, (n, content): (u32, String)) -> Result<(), std::io::Error> {
        self.files.push_back((n, content));
        if self.files.len() < 1024 {
            return Ok(());
        }

        self.flush(true).await?;

        Ok(())
    }

    pub async fn flush(&mut self, only_contiguous: bool) -> Result<(), std::io::Error> {
        // Sort by the 5 character key at the beginning of the file
        // This matches the first 5 characters of the first row
        self.files.make_contiguous().sort_unstable_by_key(|v| v.0);
        let Some((mut current_key, _)) = self.files.get(0) else {
            return Ok(());
        };
        // Peek at the key to check it is contiguous
        while let Some(&(key, _)) = self.files.get(0) {
            // Figure out when to break
            if current_key != key {
                if key == current_key + 1 || !only_contiguous {
                    current_key = key;
                } else {
                    break;
                }
            }
            let (_, text) = self.files.pop_front().unwrap();
            self.writer.write_all(text.as_bytes()).await?;
            WRITTEN_TO_FILE.fetch_add(1, atomic::Ordering::AcqRel);
        }

        Ok(())
    }

    pub async fn inner_flush(&mut self) -> Result<(), std::io::Error> {
        self.writer.flush().await
    }
}
