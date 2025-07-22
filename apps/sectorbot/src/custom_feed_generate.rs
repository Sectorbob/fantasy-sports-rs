use std::error::Error;

use crate::store::{DataStore, Feed, ExternalId};

pub async fn giants(store: &DataStore) -> Result<(), Box<dyn Error>> {
    store.save_feed(&Feed {
        id: 1,
        name: "Giants News".to_string(),
        description: "A curated list of Giants news sources".to_string(),
        sources: vec![
            ExternalId {
                platform: Some(crate::store::Platform::Twitter),
                id: "DanSchneierNFL".to_string(),
            },
            ExternalId {
                platform: Some(crate::store::Platform::Twitter),
                id: "GiantInsider".to_string(),
            },
            ExternalId {
                platform: Some(crate::store::Platform::Twitter),
                id: "CarlBanksGIII".to_string(),
            },
            ExternalId {
                platform: Some(crate::store::Platform::Twitter),
                id: "nickfalato".to_string(),
            },
            ExternalId {
                platform: Some(crate::store::Platform::Twitter),
                id: "BobbySkinner_".to_string(),
            },
        ],
        last_updated: None,
        destination: ExternalId {
            platform: Some(crate::store::Platform::Discord),
            id: "7100738766260628716".to_string(),
        },
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
            ExternalId {
                platform: Some(crate::store::Platform::Twitter),
                id: "SharpFootball".to_string(),
            },
            ExternalId {
                platform: Some(crate::store::Platform::Twitter),
                id: "RapSheet".to_string(),
            },
            ExternalId {
                platform: Some(crate::store::Platform::Twitter),
                id: "TomPelissero".to_string(),
            },
        ],
        last_updated: None,
        destination: ExternalId {
            platform: Some(crate::store::Platform::Discord),
            id: "2067967944778588584".to_string(),
        },
        pool_interval: 5 * 60, // every 5 mins
        quiet_time: "00:00-06:00".to_string(),
        entries: vec![],
    })?;
    Ok(())
}
