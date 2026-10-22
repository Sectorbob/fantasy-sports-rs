use crate::data::Position;

use super::helpers::{bool_from_int, int_from_bool, null_to_false};
use super::league::SeasonType;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Draft {
    pub draft_id: String,
    pub league_id: String,
    pub sport: super::Sport,
    pub season: String,
    pub season_type: SeasonType,
    #[serde(with = "chrono::serde::ts_milliseconds", default)]
    pub created: DateTime<Utc>,
    pub creators: Option<Vec<String>>,
    pub draft_order: Option<HashMap<String, u8>>,
    pub last_message_id: Option<String>,
    #[serde(with = "chrono::serde::ts_milliseconds_option", default)]
    pub last_message_time: Option<DateTime<Utc>>,
    #[serde(with = "chrono::serde::ts_milliseconds_option", default)]
    pub last_picked: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, String>,
    pub settings: DraftSettings,
    /// This denotes the time the draft started. None if draft never started.
    #[serde(with = "chrono::serde::ts_milliseconds_option", default)]
    pub start_time: Option<DateTime<Utc>>,
    pub status: String, // drafting
    #[serde(rename = "type")]
    pub _type: DraftType,
    /// This will only be present when obtained directly with get_draft()
    pub slot_to_roster_id: Option<HashMap<String, u8>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct DraftSettings {
    /// Sort players by name instead of ADP.
    #[serde(
        deserialize_with = "bool_from_int",
        serialize_with = "int_from_bool",
        default
    )]
    pub alpha_sort: bool,
    /// If true, a CPU will autopick for a user when a user runs out of time
    #[serde(deserialize_with = "bool_from_int", serialize_with = "int_from_bool")]
    pub cpu_autopick: bool,
    pub rounds: u8,
    pub player_type: Option<u8>,
    pub pick_timer: u32,
    pub teams: u8,
    #[serde(flatten)]
    pub other: HashMap<String, i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DraftType {
    SNAKE,
    LINEAR,
    AUCTION,
}
impl fmt::Display for DraftType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DraftType::SNAKE => write!(f, "snake"),
            DraftType::LINEAR => write!(f, "linear"),
            DraftType::AUCTION => write!(f, "auction"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct DraftPick {
    pub player_id: String,
    /// user_id this pick will go to (not all leagues have users in every slot, this can be "")
    pub picked_by: String,
    /// roster_id this pick will go to
    pub roster_id: u8,
    pub round: u8,
    /// which column this is on the draftboard
    pub draft_slot: u8,
    pub pick_no: u32,
    pub metadata: DraftPickMetadata,
    #[serde(default, deserialize_with = "null_to_false")]
    pub is_keeper: bool,
    pub draft_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct DraftPickMetadata {
    pub first_name: String,
    pub injury_status: String,
    pub last_name: String,
    pub news_updated: String, // string wrapping a millis epoch
    pub number: String,
    pub player_id: String,
    pub position: Position,
    pub sport: super::Sport,
    pub status: String, // known value: Active
    pub team: String,   // team abbreviation
    pub years_exp: String,
}

#[cfg(test)]
mod data_tests {
    use crate::Draft;
    use std::{fs, io};

    #[test]
    fn ensure_two_way_de_serialization_of_draft() {
        let file = fs::File::open("testdata/v1/league/164974877262287888/drafts.json")
            .expect("unable to load test data file");
        let reader = io::BufReader::new(file);

        // Deserialize it
        let drafts: Vec<Draft> =
            serde_json::from_reader(reader).expect("failed to deserialize test data file");
        log::info!("{:#?}", drafts);
        assert_eq!(2, drafts.len());

        // Serialize it
        let serialized_drafts =
            serde_json::to_string(&drafts).expect("failed to re-serialize the drafts json");

        let re_deserialized_drafts: Vec<Draft> = serde_json::from_str(serialized_drafts.as_str())
            .expect("failed to re-deserialize draft json object");
        assert_eq!(drafts, re_deserialized_drafts)
    }
}
