use std::path::Path;

use crate::{self as core, ExternalId};
use sleeper_fantasy_rs as sleeper;
use yahoo_fantasy_rs as yahoo;

#[allow(dead_code)]
#[derive(Clone)]
pub struct PlayerCache {
    yahoo: sled::Db,
    sleeper: sled::Db,
}
impl PlayerCache {
    pub fn new() -> Self {
        PlayerCache::new_with_path(Path::new("players"))
    }
    pub fn new_with_path(store_path: &Path) -> Self {
        let sleeper = sled::open(store_path.join("sleeper")).expect("Failed to open data store");
        let yahoo = sled::open(store_path.join("yahoo")).expect("Failed to open data store");
        PlayerCache { sleeper, yahoo }
    }

    pub fn get(&self, player_id: &ExternalId) -> Option<core::Player> {
        match player_id.platform {
            crate::Platform::Sleeper => self
                .get_sleeper_player(&player_id.id)
                .map(sleeper::Player::into),
            crate::Platform::Yahoo => self.get_yahoo_player(&player_id.id),
            _ => None,
        }
    }

    pub fn get_sleeper_player<T: Into<String>>(&self, player_id: T) -> Option<sleeper::Player> {
        self.sleeper
            .get(player_id.into())
            .ok()
            .flatten()
            .map(|json| serde_json::from_slice(&json).ok())
            .flatten()
    }

    pub(crate) fn get_yahoo_player<T: Into<String>>(&self, player_id: T) -> Option<core::Player> {
        self.yahoo
            .get(player_id.into())
            .ok()
            .flatten()
            .map(|json| serde_json::from_slice(&json).ok())
            .flatten()
    }

    pub fn insert_sleeper_player(&self, player: &sleeper::Player) {
        self.sleeper
            .insert(
                player.player_id.to_string(),
                serde_json::to_vec(player).expect("failed to serialize sleeper player to json"),
            )
            .ok();
    }

    pub fn insert_yahoo_player(&self, player: &yahoo::Player) {
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
