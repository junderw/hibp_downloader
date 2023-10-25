use std::sync::atomic::AtomicU64;

pub static DOWNLOADED: AtomicU64 = AtomicU64::new(0);
pub static WRITTEN_TO_FILE: AtomicU64 = AtomicU64::new(0);
pub static IN_ROUTE: AtomicU64 = AtomicU64::new(0);
pub static CACHE_HITS: AtomicU64 = AtomicU64::new(0);
pub static AVG_TIME_MS: AtomicU64 = AtomicU64::new(0);
