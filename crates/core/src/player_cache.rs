use crate as core;
use chrono::{DateTime, Duration, Utc};
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use sleeper_fantasy_rs as sleeper;
use std::path::Path;
use yahoo_fantasy_rs as yahoo;

const SLEEPER_PLAYER_TTL: Duration = Duration::hours(24);
const DEFAULT_CACHE_DIR: &'static str = "players";
const SLEEPER_CACHE_SUBDIR: &'static str = "sleeper";
const YAHOO_CACHE_SUBDIR: &'static str = "yahoo";
const METADATA_CACHE_SUBDIR: &'static str = "metadata";

/// This player cache is a data model repository that saves player lookup data
/// for fantasy football p[latforms to disk for easy lookup. It is using
/// sled::Db for now.
pub struct PlayerCache {
    yahoo: sled::Db,
    sleeper: sled::Db,
    yahoo_to_sleeper: sled::Db,
    metadata: sled::Db,
    sleeper_client: sleeper::Client,
    sport: core::Sport,
}
impl PlayerCache {
    pub fn new<S: Into<core::Sport>>(sport: S) -> Result<Self> {
        PlayerCache::new_with_path(Path::new(DEFAULT_CACHE_DIR), sport)
    }

    pub fn new_with_path<S: Into<core::Sport>>(store_path: &Path, sport: S) -> Result<Self> {
        let sleeper = sled::open(store_path.join(SLEEPER_CACHE_SUBDIR))?;
        let yahoo = sled::open(store_path.join(YAHOO_CACHE_SUBDIR))?;
        let yahoo_to_sleeper = sled::open(store_path.join("yahoo_to_sleeper"))?;
        let metadata = sled::open(store_path.join(METADATA_CACHE_SUBDIR))?;
        Ok(PlayerCache {
            sleeper,
            yahoo,
            yahoo_to_sleeper,
            metadata,
            sleeper_client: sleeper::Client::new(),
            sport: sport.into(),
        })
    }

    /// Initialize the player cache. Should be run at process startup
    pub async fn init(&self) -> Result<()> {
        let sleeper_metadata = self.ensure_sleeper_metadata().await?;

        // check to see if the player cache needs to be updated
        if let Some(last_sync) = sleeper_metadata.last_full_sync {
            if Utc::now() - last_sync > sleeper_metadata.ttl {
                log::info!("Sleeper player cache is stale. Syncing now...");
                let start_ts = Utc::now();
                self.sync_sleeper_players().await?;
                log::info!(
                    "Sleeper player cache synced in {d}",
                    d = Utc::now().signed_duration_since(start_ts)
                );
            }
        } else {
            log::warn!("Sleeper player cache  has never been synced. Syncing now...");
            let start_ts = Utc::now();
            self.sync_sleeper_players().await?;
            log::info!(
                "Sleeper player cache synced in {d}",
                d = Utc::now().signed_duration_since(start_ts)
            );
        }

        Ok(())
    }

    pub async fn close(&self) -> Result<()> {
        for r in join_all([
            self.yahoo.flush_async(),
            self.sleeper.flush_async(),
            self.metadata.flush_async(),
            self.yahoo_to_sleeper.flush_async(),
        ])
        .await
        {
            r?;
        }
        Ok(())
    }

    pub(crate) async fn ensure_sleeper_metadata(&self) -> Result<Metadata> {
        match PlayerCache::get_metadata(&self.metadata, core::Platform::Sleeper) {
            Ok(s) => Ok(s),
            Err(e) if e.is_none() => {
                let m = Metadata {
                    platform: crate::Platform::Sleeper,
                    last_full_sync: None,
                    ttl: SLEEPER_PLAYER_TTL,
                };
                PlayerCache::insert_metadata(&self.metadata, &m).await?;
                Ok(m)
            }
            Err(e) => return Err(e.into()),
        }
    }

    pub(crate) async fn sync_sleeper_players(&self) -> Result<()> {
        let players = self
            .sleeper_client
            .fetch_all_players_request(&self.sport.to_sport())
            .force_update()
            .send()
            .await?;
        for (_, player) in players.into_iter() {
            self.insert_sleeper_player(&player)?;
        }
        // update metadata
        PlayerCache::insert_metadata(
            &self.metadata,
            &Metadata {
                platform: crate::Platform::Sleeper,
                last_full_sync: Some(Utc::now()),
                ttl: SLEEPER_PLAYER_TTL,
            },
        )
        .await?;
        Ok(())
    }

    pub(crate) fn get_metadata(
        metadata_db: &sled::Db,
        platform: core::Platform,
    ) -> Result<Metadata> {
        match metadata_db.get(platform) {
            Ok(opt_json) => match opt_json {
                Some(json) => Ok(serde_json::from_slice(&json)?),
                None => Err(Error::NotFound),
            },
            Err(err) => Err(err.into()),
        }
    }

    pub(crate) async fn insert_metadata(metadata_db: &sled::Db, metadata: &Metadata) -> Result<()> {
        metadata_db
            .insert(&metadata.platform, serde_json::to_vec(metadata)?)
            .map(|_| ())?;
        metadata_db.flush_async().await?;
        Ok(())
    }

    pub fn get(&self, player_id: &core::ExternalId) -> Result<core::Player> {
        match &player_id.platform {
            crate::Platform::Sleeper => self
                .get_sleeper_player(&player_id.id)
                .map(sleeper::Player::into),
            crate::Platform::Yahoo => self.get_yahoo_player(&player_id.id),
            // crate::Platform::Yahoo => self // TODO: this is not quite ready
            //     .get_sleeper_player_with_yahoo_id(
            //         yahoo::PlayerKey::from_str(player_id.id.as_str())
            //             .map_err(|e| Error::new(e))?
            //             .player_id,
            //     )
            //     .map(sleeper::Player::into),
            other => Err(Error::DevError(format!(
                "unsupported player platform: {other}"
            ))),
        }
    }

    pub(crate) fn get_sleeper_player<T: Into<String>>(
        &self,
        player_id: T,
    ) -> Result<sleeper::Player> {
        self.sleeper
            .get(player_id.into())?
            .map_or(Err(Error::NotFound), |json| {
                Ok(serde_json::from_slice(&json)?)
            })
    }

    pub(crate) fn get_yahoo_player<T: Into<String>>(&self, player_id: T) -> Result<core::Player> {
        self.yahoo
            .get(player_id.into())?
            .map_or(Err(Error::NotFound), |json| {
                Ok(serde_json::from_slice(&json)?)
            })
    }

    pub(crate) fn get_sleeper_player_with_yahoo_id(
        &self,
        yahoo_id: u32,
    ) -> Result<sleeper::Player> {
        if let Some(sleeper_id) = self
            .yahoo
            .get(yahoo_id.to_le_bytes())?
            .map(|b| String::from_utf8(b.to_vec()).expect("fucking hell"))
        {
            Ok(self.get_sleeper_player(sleeper_id)?)
        } else {
            Err(Error::NotFound)
        }
    }

    pub(crate) fn insert_sleeper_player(&self, player: &sleeper::Player) -> Result<()> {
        self.sleeper
            .insert(player.player_id.to_string(), serde_json::to_vec(player)?)?;
        if let Some(yahoo_id) = &player.yahoo_id {
            self.yahoo_to_sleeper
                .insert(yahoo_id.to_le_bytes(), player.player_id.as_bytes())?;
        }
        Ok(())
    }

    pub(crate) fn insert_yahoo_player(&self, player: &yahoo::Player) {
        let core_player = core::Player::from(player);
        self.yahoo
            .insert(
                player.player_key.to_string(),
                serde_json::to_vec(&core_player)
                    .expect("failed to serialize yahoo player (core) to json"),
            )
            .ok();
    }

    // fn list_collection<T: for<'de> Deserialize<'de>>(
    //     db: &sled::Db,
    // ) -> Result<Vec<T>, DataStoreError> {
    //     let mut items = Vec::new();
    //     for item in db.iter() {
    //         if let Ok((_key, value)) = item {
    //             items.push(serde_json::from_slice::<T>(&value)?);
    //         }
    //     }
    //     Ok(items)
    // }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Metadata {
    platform: core::Platform,
    last_full_sync: Option<DateTime<Utc>>,
    ttl: Duration,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("resource not found")]
    NotFound,
    #[error("sled error: {0}")]
    SledError(#[from] sled::Error),
    #[error("sleeper error: {0}")]
    SleeperError(#[from] sleeper::Error),
    #[error("serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("dev error: {0}")]
    DevError(String),
}
impl Error {
    pub fn new<T: Into<String>>(msg: T) -> Self {
        Error::DevError(msg.into())
    }
    pub fn is_none(&self) -> bool {
        match self {
            Error::NotFound => true,
            _ => false,
        }
    }
}
