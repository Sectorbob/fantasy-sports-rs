use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub struct DataStore {
    feeds: sled::Db,
}
impl DataStore {
    pub fn new() -> Self {
        let feeds = sled::open("data_store/feeds").expect("Failed to open data store");
        DataStore { feeds }
    }

    pub fn get_feed(&self, id: u64) -> Option<Feed> {
        self.feeds
            .get(id.to_string())
            .ok()
            .flatten()
            .and_then(|data| serde_json::from_slice(&data).ok())
    }

    pub fn save_feed(&self, feed: &Feed) -> Result<(), Box<dyn std::error::Error>> {
        let data = serde_json::to_vec(feed)?;
        self.feeds.insert(feed.id.to_string(), data)?;
        self.feeds.flush()?;
        Ok(())
    }

    // Add methods to interact with the store as needed
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Feed {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub sources: Vec<String>,
    pub last_updated: Option<String>,
    pub destination: String,
    pub pool_interval: u64,
    pub quiet_time: String,
    pub entries: Vec<FeedEntry>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FeedEntry {
    pub id: u64,
    pub title: String,
    pub link: String,
    pub source_ts: DateTime<Utc>,
    pub content: String,
    pub published_at: Option<DateTime<Utc>>,
}
