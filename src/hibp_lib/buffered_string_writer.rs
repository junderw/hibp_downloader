use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
    sync::atomic,
};
use tokio::io::AsyncWriteExt;

use super::{stats::WRITTEN_TO_FILE, ChannelData};

pub struct BufferedStringWriter {
    files: VecDeque<ChannelData>,
    writer: Option<tokio::io::BufWriter<tokio::fs::File>>,
    path: PathBuf,
}

impl BufferedStringWriter {
    pub async fn from_file(filename: &Path) -> Result<Self, std::io::Error> {
        let exists = tokio::fs::try_exists(filename).await?;
        let is_dir = if exists {
            let metadata = tokio::fs::metadata(filename).await?;
            metadata.is_dir()
        } else {
            false
        };
        let writer = if exists && is_dir {
            let mut dir_contents = tokio::fs::read_dir(filename).await?;
            if dir_contents.next_entry().await?.is_some() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Unsupported,
                    "Directory downloads must be done with an empty directory.",
                ));
            }
            None
        } else {
            Some(tokio::io::BufWriter::with_capacity(
                1024 * 1024 * 32, // 1 download is around 32kB. This fits around 1024 downloads.
                tokio::fs::File::create(filename).await?,
            ))
        };
        Ok(Self {
            files: VecDeque::with_capacity(1024),
            writer,
            path: filename.to_path_buf(),
        })
    }

    pub async fn add_file(&mut self, (n, content): ChannelData) -> Result<(), std::io::Error> {
        self.files.push_back((n, content));
        if self.files.len() < 1024 {
            return Ok(());
        }

        self.flush(true).await?;

        Ok(())
    }

    #[allow(clippy::get_first)]
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
            let (n, text) = self.files.pop_front().unwrap();
            if let Some(writer) = self.writer.as_mut() {
                for line in String::from_utf8_lossy(&text).lines() {
                    writer
                        .write_all(format!("{n:05X}{line}\n").as_bytes())
                        .await?;
                }
            } else {
                let filepath = self.path.join(format!("{n:05X}"));
                let mut file = tokio::fs::File::create(&filepath).await?;
                file.write_all(&text).await?;
            }
            WRITTEN_TO_FILE.fetch_add(1, atomic::Ordering::AcqRel);
        }

        Ok(())
    }

    pub async fn inner_flush(&mut self) -> Result<(), std::io::Error> {
        if let Some(w) = self.writer.as_mut() {
            w.flush().await
        } else {
            Ok(())
        }
    }
}
