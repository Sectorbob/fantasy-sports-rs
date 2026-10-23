use crate as core;
use chrono::{DateTime, Duration, Utc};
use fantasy_sports_sleeper as sleeper;
use fantasy_sports_yahoo as yahoo;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use std::path::Path;

const SLEEPER_PLAYER_TTL: Duration = Duration::hours(24);
const DEFAULT_CACHE_DIR: &str = "players";
const SLEEPER_CACHE_SUBDIR: &str = "sleeper";
const YAHOO_CACHE_SUBDIR: &str = "yahoo";
const METADATA_CACHE_SUBDIR: &str = "metadata";

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

    /// Uses `client` to download the Sleeper player universe (for another base url, for example).
    pub fn with_sleeper_client(mut self, client: sleeper::Client) -> Self {
        self.sleeper_client = client;
        self
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
            Err(e) => Err(e),
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

    pub(crate) fn insert_sleeper_player(&self, player: &sleeper::Player) -> Result<()> {
        self.sleeper
            .insert(&player.player_id, serde_json::to_vec(player)?)?;
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
        matches!(self, Error::NotFound)
    }
}
impl From<fantasy_sports_cache::Error> for Error {
    fn from(value: fantasy_sports_cache::Error) -> Self {
        match value {
            fantasy_sports_cache::Error::CacheError(cache_err) => {
                Error::DevError(format!("cache error: {cache_err:?}"))
            }
            fantasy_sports_cache::Error::ClientError {
                status: status_code,
                ..
            } => Error::DevError(format!("http client error: {status_code}")),
            fantasy_sports_cache::Error::DevError(msg) => Error::DevError(msg),
            fantasy_sports_cache::Error::ReqwestError(error) => {
                Error::DevError(format!("reqwest error: {error:?}"))
            }
            fantasy_sports_cache::Error::NotFound(_) => Error::NotFound,
            fantasy_sports_cache::Error::SerdeError(serde_error) => match serde_error {
                fantasy_sports_cache::SerdeError::JsonError(json_err) => {
                    Error::SerializationError(json_err)
                }
                fantasy_sports_cache::SerdeError::XMLError(de_error) => {
                    Error::DevError(format!("xml deserialization error: {de_error}"))
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::io::Read;
    use tempfile::TempDir;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    /// a few real (public) players from the recorded Sleeper player universe
    fn some_players(n: usize) -> HashMap<String, sleeper::Player> {
        let fixture = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../sleeper/testdata/players_nfl.json.gz"
        );
        let mut json = String::new();
        flate2::read::GzDecoder::new(std::fs::File::open(fixture).unwrap())
            .read_to_string(&mut json)
            .unwrap();
        let all: HashMap<String, sleeper::Player> = serde_json::from_str(&json).unwrap();
        let mut ids: Vec<&String> = all.keys().collect();
        ids.sort();
        ids.into_iter()
            .take(n)
            .map(|id| (id.clone(), all[id].clone()))
            .collect()
    }

    fn cache(dir: &TempDir) -> PlayerCache {
        PlayerCache::new_with_path(dir.path(), core::Sport::NFL).unwrap()
    }

    async fn players_server(players: &HashMap<String, sleeper::Player>) -> MockServer {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/players/nfl"))
            .respond_with(ResponseTemplate::new(200).set_body_json(players))
            .mount(&server)
            .await;
        server
    }

    #[test]
    fn a_stored_sleeper_player_can_be_read_back_and_unknown_ids_are_not_found() {
        let dir = TempDir::new().unwrap();
        let cache = cache(&dir);
        let players = some_players(3);
        let (id, player) = players.iter().next().unwrap();

        assert!(cache.get_sleeper_player(id.as_str()).unwrap_err().is_none());
        cache.insert_sleeper_player(player).unwrap();

        assert_eq!(player, &cache.get_sleeper_player(id.as_str()).unwrap());
        assert!(cache.get_sleeper_player("nope").unwrap_err().is_none());
    }

    #[test]
    fn players_are_looked_up_by_external_id() {
        let dir = TempDir::new().unwrap();
        let cache = cache(&dir);
        let players = some_players(1);
        let player = players.values().next().unwrap();
        cache.insert_sleeper_player(player).unwrap();

        let found = cache
            .get(&core::ExternalId::new(
                core::Platform::Sleeper,
                player.player_id.clone(),
            ))
            .unwrap();
        assert_eq!(player.player_id, found._id);

        let not_cached = cache
            .get(&core::ExternalId::new(
                core::Platform::Yahoo,
                "nfl.p.1".into(),
            ))
            .unwrap_err();
        assert!(not_cached.is_none());
        let unsupported = cache
            .get(&core::ExternalId::new(core::Platform::Discord, "1".into()))
            .unwrap_err();
        assert!(
            unsupported
                .to_string()
                .contains("unsupported player platform")
        );
    }

    #[test]
    fn a_players_yahoo_id_is_indexed_to_their_sleeper_id() {
        let dir = TempDir::new().unwrap();
        let cache = cache(&dir);
        let mut player = some_players(1).into_values().next().unwrap();
        player.yahoo_id = Some(7241);
        cache.insert_sleeper_player(&player).unwrap();

        let stored = cache.yahoo_to_sleeper.get(7241u32.to_le_bytes()).unwrap();
        assert_eq!(Some(player.player_id.as_bytes()), stored.as_deref());
    }

    #[tokio::test]
    async fn init_downloads_the_player_universe_once_and_not_again_while_fresh() {
        let dir = TempDir::new().unwrap();
        let players = some_players(25);
        let server = players_server(&players).await;
        let client = sleeper::Client::new().with_base_url(server.uri());
        let cache = cache(&dir).with_sleeper_client(client);

        cache.init().await.unwrap();
        for id in players.keys() {
            assert!(
                cache.get_sleeper_player(id.as_str()).is_ok(),
                "{id} was not synced"
            );
        }
        assert_eq!(1, server.received_requests().await.unwrap().len());

        let metadata = PlayerCache::get_metadata(&cache.metadata, core::Platform::Sleeper).unwrap();
        assert!(metadata.last_full_sync.is_some());

        // the data is fresh, so a second start-up does not hit the network again
        cache.init().await.unwrap();
        assert_eq!(1, server.received_requests().await.unwrap().len());
        cache.close().await.unwrap();
    }

    #[tokio::test]
    async fn init_refreshes_a_stale_cache() {
        let dir = TempDir::new().unwrap();
        let players = some_players(5);
        let server = players_server(&players).await;
        let client = sleeper::Client::new().with_base_url(server.uri());
        let cache = cache(&dir).with_sleeper_client(client);
        cache.init().await.unwrap();

        // pretend the last sync was long ago
        PlayerCache::insert_metadata(
            &cache.metadata,
            &Metadata {
                platform: core::Platform::Sleeper,
                last_full_sync: Some(Utc::now() - Duration::hours(48)),
                ttl: SLEEPER_PLAYER_TTL,
            },
        )
        .await
        .unwrap();
        cache.init().await.unwrap();

        assert_eq!(2, server.received_requests().await.unwrap().len());
    }

    #[tokio::test]
    async fn a_failing_player_download_is_an_error_not_an_empty_cache() {
        let dir = TempDir::new().unwrap();
        let server = MockServer::start().await; // every path is a 404
        let client = sleeper::Client::new().with_base_url(server.uri());
        let cache = cache(&dir).with_sleeper_client(client);

        assert!(cache.init().await.is_err());
        let metadata = PlayerCache::get_metadata(&cache.metadata, core::Platform::Sleeper).unwrap();
        assert!(
            metadata.last_full_sync.is_none(),
            "a failed sync must not be recorded"
        );
    }

    #[test]
    fn missing_metadata_is_reported_as_not_found() {
        let dir = TempDir::new().unwrap();
        let cache = cache(&dir);
        let err = PlayerCache::get_metadata(&cache.metadata, core::Platform::Sleeper).unwrap_err();
        assert!(err.is_none());
    }
}
