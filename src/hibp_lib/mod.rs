mod buffered_string_writer;
pub mod config;
mod consts;
mod download;
mod progress_style;
mod sort;
mod stats;
mod tasks;

use std::sync::atomic;

use bytes::Bytes;
use config::Config;
use consts::{LENGTH, USER_AGENT};
use progress_style::{get_span, progress_style_download};
use reqwest::Client;
use tasks::{download_task, progress_task, writer_task};
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::Instrument;
use tracing_indicatif::{span_ext::IndicatifSpanExt, IndicatifLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, FmtSubscriber};

pub fn init_logging_and_progress() {
    let indicatif_layer = IndicatifLayer::new();
    tracing::subscriber::set_global_default(
        FmtSubscriber::builder()
            .with_env_filter(EnvFilter::from_default_env())
            .with_writer(indicatif_layer.get_stderr_writer())
            .finish()
            .with(indicatif_layer),
    )
    .expect("Logging subscriber failed");
    LogTracer::init().unwrap();
}

pub type ChannelData = (u32, Bytes);
pub fn init_client_channels(
    concurrent_requests: usize,
) -> (Client, Sender<ChannelData>, Receiver<ChannelData>) {
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap();
    let (tx, rx) = tokio::sync::mpsc::channel::<ChannelData>(concurrent_requests);

    (client, tx, rx)
}

pub use sort::run_sort;
pub fn run_download(config: &Config) -> anyhow::Result<()> {
    let body = async move {
        let concurrent_requests = config.workers * config.multiplier;
        let span = get_span(u64::from(LENGTH), progress_style_download());
        let enter = span.enter();
        let (client, tx, rx) = init_client_channels(concurrent_requests);
        let progress_task = tokio::spawn(progress_task().instrument(span.clone()));
        let file =
            buffered_string_writer::BufferedStringWriter::from_file(&config.output_path).await?;
        let writer_task = tokio::spawn(writer_task(rx, file));
        let download_task =
            tokio::spawn(download_task(client, concurrent_requests, tx, config.ntlm));

        download_task.await??;
        writer_task.await??;
        progress_task.abort();

        // Leak the span so that it never gets cleaned up
        // (We want it to remain after the program finishes so the logs aren't deleted)
        // Give it a chance to write to stderr (since it can't flush in the Drop impl)
        span.pb_set_position(stats::DOWNLOADED.load(atomic::Ordering::Acquire));
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        core::mem::forget(enter);
        core::mem::forget(span);

        anyhow::Ok(())
    };

    // Dropping the runtime took 7 seconds when leaking the logging span.
    // Leaking the runtime since we know that this function call is the last
    // in the program and will be cleaned up after the process exits anyways.
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(config.workers)
        .build()
        .unwrap();
    runtime.block_on(body)?;
    core::mem::forget(runtime);
    Ok(())
}
