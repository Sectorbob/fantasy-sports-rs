mod fs_cache;
use chrono::{DateTime, Utc};
pub use fs_cache::FileSystemCacher;
use serde::{Deserialize, Serialize};

// The trait for caching
#[async_trait::async_trait]
pub trait Cacheable: Sync {
    async fn get_metadata(&self, k: String) -> Result<Option<Entry>, std::io::Error>;
    async fn save_metadata(&self, k: String, v: Entry) -> Result<(), std::io::Error>;
    async fn load_cache(&self, key: &str) -> Option<Vec<u8>>;
    async fn save_cache<'a>(&self, key: &str, value: &'a [u8]) -> Result<(), std::io::Error>;
}

pub struct NoOpCacher {}
impl NoOpCacher {
    pub fn new() -> Self {
        NoOpCacher {}
    }
}
impl Default for NoOpCacher {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Cacheable for NoOpCacher {
    async fn get_metadata(&self, _: String) -> Result<Option<Entry>, std::io::Error> {
        Ok(None)
    }
    async fn save_metadata(&self, _: String, _: Entry) -> Result<(), std::io::Error> {
        Ok(())
    }
    async fn load_cache(&self, _key: &str) -> Option<Vec<u8>> {
        None
    }
    async fn save_cache<'a>(&self, _key: &str, _value: &'a [u8]) -> Result<(), std::io::Error> {
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Entry {
    #[serde(rename = "t", with = "chrono::serde::ts_milliseconds")]
    pub ts: DateTime<Utc>,
    #[serde(rename = "e")]
    pub ttl: u64,
}
impl Entry {
    pub fn is_expired(&self) -> bool {
        let elapsed = Utc::now()
            .signed_duration_since(self.ts)
            .as_seconds_f64()
            .round() as u64;
        log::trace!("elapsed: {elapsed}; ttl: {ttl}", ttl = self.ttl);
        elapsed > self.ttl
    }
}

#[cfg(test)]
mod cache_tests {
    use crate::Entry;
    use chrono::{TimeDelta, Utc};

    #[test]
    fn test_entry_is_expired() {
        let entry = Entry {
            ts: Utc::now()
                .checked_sub_signed(TimeDelta::seconds(3599))
                .expect(""),
            ttl: 3600,
        };
        debug_assert!(!entry.is_expired(), "-3599 should not be expired");
        let entry = Entry {
            ts: Utc::now()
                .checked_sub_signed(TimeDelta::seconds(3600))
                .expect(""),
            ttl: 3600,
        };
        debug_assert!(!entry.is_expired(), "-3600 should not be expired");
        let entry = Entry {
            ts: Utc::now()
                .checked_sub_signed(TimeDelta::seconds(3601))
                .expect(""),
            ttl: 3600,
        };
        debug_assert!(entry.is_expired(), "-3601 should be expired");
    }
}
