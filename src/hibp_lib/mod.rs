mod buffered_string_writer;
pub mod config;
mod consts;
mod download;
mod progress_style;
mod sort;
mod stats;
mod tasks;

use config::Config;
use consts::{LENGTH, USER_AGENT};
use progress_style::{get_span, progress_style_download};
use reqwest::Client;
use tasks::{download_task, progress_task, writer_task};
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::Instrument;
use tracing_indicatif::IndicatifLayer;
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

pub type ChannelData = (u32, String);
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
        let _enter = span.enter();
        let (client, tx, rx) = init_client_channels(concurrent_requests);
        let progress_task = tokio::spawn(progress_task().instrument(span.clone()));
        let writer_task = tokio::spawn(writer_task(rx, config.output_file.clone()));
        let download_task =
            tokio::spawn(download_task(client, concurrent_requests, tx, config.ntlm));

        download_task.await??;
        writer_task.await??;
        progress_task.abort();
        Ok(())
    };

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(config.workers)
        .build()
        .unwrap()
        .block_on(body)
}
