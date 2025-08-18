use serde::{Deserialize, Deserializer};

use crate::Game;

#[derive(Deserialize, Debug)]
#[serde(rename = "users")]
struct Users {
    #[serde(rename = "user", default)]
    pub users: Vec<User>,
}

//region User Resource
#[derive(Deserialize, Debug)]
pub struct User {
    // user_id: String,
    pub guid: String,
    #[serde(deserialize_with = "Game::deserialize_games")]
    pub games: Vec<Game>,
    // nickname: String,
    // profile_url: String,
    // image_url: String,
    // status: String,
    // #[serde(rename = "yahoo_guid")]
    // yahoo_guid: String,
    // #[serde(rename = "yahoo_id")]
    // yahoo_id: String,
}
impl User {
    // This method is called when the deserializer encounters a newtype struct
    pub(crate) fn deserialize_users<'de, D>(deserializer: D) -> Result<Vec<User>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Users::deserialize(deserializer).map(|r| r.users)
    }
}
