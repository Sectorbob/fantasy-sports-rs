use std::error::Error;

use crate::store::{DataStore, Feed};

pub async fn giants(store: &DataStore) -> Result<(), Box<dyn Error>> {
    store.save_feed(&Feed {
        id: 1,
        name: "Giants News".to_string(),
        description: "A curated list of Giants news sources".to_string(),
        sources: vec![
            "DanSchneierNFL".to_string(),
            "GiantInsider".to_string(),
            "CarlBanksGIII".to_string(),
            "nickfalato".to_string(),
            "BobbySkinner_".to_string(),
        ],
        last_updated: None,
        destination: "7100738766260628716".to_string(),
        pool_interval: 5 * 60, // every 5 mins
        quiet_time: "00:00-06:00".to_string(),
        entries: vec![],
    })?;
    Ok(())
}

pub async fn nfl(store: &DataStore) -> Result<(), Box<dyn Error>> {
    store.save_feed(&Feed {
        id: 2,
        name: "NFL News".to_string(),
        description: "A curated list of NFL news sources".to_string(),
        sources: vec![
            "SharpFootball".to_string(),
            "RapSheet".to_string(),
            "TomPelissero".to_string(),
        ],
        last_updated: None,
        destination: "2067967944778588584".to_string(),
        pool_interval: 5 * 60, // every 5 mins
        quiet_time: "00:00-06:00".to_string(),
        entries: vec![],
    })?;
    Ok(())
}
