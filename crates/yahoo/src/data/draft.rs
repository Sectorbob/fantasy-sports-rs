use serde::Deserialize;

use crate::data::{player::PlayerKey, team::TeamKey};

#[derive(Deserialize, Debug, Clone)]
pub struct DraftResults {
    #[serde(rename = "draft_result")]
    pub draft_results: Vec<DraftResult>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DraftResult {
    pub pick: u32,
    pub round: u32,
    pub team_key: TeamKey,
    pub player_key: PlayerKey, // e.g. nfl.p.7241
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DraftStatus {
    PreDraft,
    PostDraft,
    // TODO: Handle fallback case with value
    // #[serde(untagged)]
    // Other(String),
}
