use std::{
    io,
    path::PathBuf,
    sync::{atomic, Arc},
    time::Duration,
};

use tokio::{
    sync::{mpsc::Receiver, mpsc::Sender, Semaphore},
    task::JoinSet,
};
use tracing::Span;
use tracing_indicatif::span_ext::IndicatifSpanExt;

use super::{
    buffered_string_writer::BufferedStringWriter,
    consts::{BEGIN, END},
    download::download_prefix,
    stats::DOWNLOADED,
};

pub async fn writer_task(
    mut rx: Receiver<(u32, String)>,
    output_file: PathBuf,
) -> Result<(), io::Error> {
    let mut file = BufferedStringWriter::from_file(&output_file).await?;

    while let Some(rows) = rx.recv().await {
        file.add_file(rows).await?;
    }

    file.flush(false).await?;
    file.inner_flush().await?;
    Ok::<(), io::Error>(())
}

pub async fn progress_task() {
    let span = Span::current();
    loop {
        tokio::time::sleep(Duration::from_millis(100)).await;
        span.pb_set_position(DOWNLOADED.load(atomic::Ordering::Acquire));
    }
}

pub async fn download_task(
    client: reqwest::Client,
    concurrent_requests: usize,
    tx: Sender<(u32, String)>,
    ntlm: bool,
) -> anyhow::Result<()> {
    let mut handles = JoinSet::new();
    let semaphore = Arc::new(Semaphore::new(concurrent_requests));
    for n in BEGIN..=END {
        let client = client.clone();
        let tx = tx.clone();
        let semaphore = Arc::clone(&semaphore);

        handles.spawn(async move {
            let _permit = semaphore.acquire().await?;
            tx.send(download_prefix(&client, n, ntlm).await?).await?;
            Ok::<(), anyhow::Error>(())
        });
    }
    drop(tx);
    while let Some(res) = handles.join_next().await {
        res??;
    }

    Ok(())
}
