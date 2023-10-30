use std::{sync::atomic, time::Instant};

use tracing::info;

use super::{
    consts::HIBP_ROOT,
    stats::{AVG_TIME_MS, CACHE_HITS, DOWNLOADED, IN_ROUTE},
};

pub async fn download_prefix(
    client: &reqwest::Client,
    n: u32,
    ntlm: bool,
) -> anyhow::Result<(u32, String)> {
    let n_str = format!("{n:05X}");
    let ntlm_str = if ntlm { "?mode=ntlm" } else { "" };
    let url = format!("{HIBP_ROOT}{n_str}{ntlm_str}");

    let mut retries = 5;
    let mut cache_hit;

    IN_ROUTE.fetch_add(1, atomic::Ordering::AcqRel);

    let now = Instant::now();
    let res_bytes = loop {
        match client.get(&url).send().await {
            Ok(r) => {
                // Keep track of CloudFlare cache hits
                cache_hit = r
                    .headers()
                    .get("CF-Cache-Status")
                    .map(|v| v.as_bytes() == b"HIT")
                    .unwrap_or(false);
                match r.bytes().await {
                    Ok(b) => break b,
                    Err(e) => {
                        if retries == 0 {
                            return Err(e.into());
                        }
                        info!(
                            "Failed response body processing. Retrying 0x{} {}/5...",
                            n_str, retries
                        );
                        retries -= 1;
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                }
            }
            Err(e) => {
                if retries == 0 {
                    return Err(e.into());
                }
                info!("Failed request. Retrying 0x{} {}/5...", n_str, retries);
                retries -= 1;
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
    };
    let req_time_ms = now.elapsed().as_millis() as u64;
    let total_downloaded = DOWNLOADED.load(atomic::Ordering::Acquire) + 1;
    AVG_TIME_MS
        .fetch_update(
            atomic::Ordering::AcqRel,
            atomic::Ordering::Acquire,
            |prev| {
                Some((prev * total_downloaded.saturating_sub(1) + req_time_ms) / total_downloaded)
            },
        )
        .ok();

    IN_ROUTE.fetch_sub(1, atomic::Ordering::AcqRel);
    DOWNLOADED.fetch_add(1, atomic::Ordering::AcqRel);
    if cache_hit {
        CACHE_HITS.fetch_add(1, atomic::Ordering::AcqRel);
    }

    let new_lines: String = String::from_utf8_lossy(&res_bytes)
        .lines()
        .map(|s| format!("{n_str}{s}\n"))
        .collect();

    Ok((n, new_lines))
}
