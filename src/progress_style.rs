use std::sync::atomic;

use indicatif::{ProgressState, ProgressStyle};
use tracing::{error_span, Span};
use tracing_indicatif::span_ext::IndicatifSpanExt;

use crate::stats::{AVG_TIME_MS, CACHE_HITS, DOWNLOADED, IN_ROUTE};

pub fn get_span(length: u64, style: ProgressStyle) -> Span {
    // Use error so the progress bar is always shown
    let span = error_span!("Download Progress");
    span.pb_set_style(&style);
    span.pb_set_length(length);
    span
}

/// This function gives a percentage up to 6 decimal places.
fn get_pct(numerator: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        return 0.0;
    }
    (numerator * 100_000_000 / denominator) as f64 / 1_000_000.0
}

/// For use with `indicatif::style::ProgressStyle::with_key`
fn cache_stats_tracker(_: &ProgressState, w: &mut dyn std::fmt::Write) {
    let cache = CACHE_HITS.load(atomic::Ordering::Acquire);
    let total = DOWNLOADED.load(atomic::Ordering::Acquire);
    let in_flight = IN_ROUTE.load(atomic::Ordering::Acquire);
    let pct = get_pct(cache, total);
    w.write_fmt(format_args!(
        "{cache}/{total} {pct}% (In flight requests: {in_flight})"
    ))
    .unwrap();
}

/// For use with `indicatif::style::ProgressStyle::with_key`
fn avg_request_ms_tracker(_: &ProgressState, w: &mut dyn std::fmt::Write) {
    let avg_time = AVG_TIME_MS.load(atomic::Ordering::Acquire);
    w.write_fmt(format_args!("{avg_time}")).unwrap();
}

pub fn progress_style_download() -> ProgressStyle {
    ProgressStyle::with_template(
        "\
        {spinner:.green} \
        [{elapsed_precise}] \
        [ETA: {eta_precise}] \
        [{percent}%] \
        [{wide_bar:.pink/blue}]\n\
        Request speed: {per_sec}\n\
        Avg Request time: {avg_request_ms} ms\n\
        Current: {human_pos}/{human_len}\n\
        Cloudflare cache hits: {cache_stats}",
    )
    .unwrap()
    .with_key("cache_stats", cache_stats_tracker)
    .with_key("avg_request_ms", avg_request_ms_tracker)
    .progress_chars("#>-")
}

pub fn progress_style_sort() -> ProgressStyle {
    ProgressStyle::with_template(
        "\
        {spinner:.green} \
        [{elapsed_precise}] \
        [ETA: {eta_precise}] \
        [{percent}%] \
        [{wide_bar:.pink/blue}]\n\
        Process speed: {per_sec}\n\
        Current: {human_pos}/{human_len}",
    )
    .unwrap()
    .progress_chars("#>-")
}
