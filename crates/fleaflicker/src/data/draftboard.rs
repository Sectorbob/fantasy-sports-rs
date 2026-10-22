use crate::{DraftBoardCellColor, LeaguePlayer, Team};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Draftboard {
    pub ordered_selections: Vec<Selection>,
    #[serde(default)]
    pub is_in_progress: bool,
    #[serde(default)]
    pub draft_index: u8,
    // pub draft_order: Option<Vec<Team>>,
    // pub rows: Option<Vec<DraftboardRow>>,
    // pub keepers: Option<Vec<TeamKeepers>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Selection {
    pub team: Team,
    pub player: LeaguePlayer,
    pub slot: DraftSlot,
    pub color: DraftBoardCellColor,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DraftSlot {
    pub round: i32,
    pub slot: i32,
    pub overall: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DraftboardRow {}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TeamKeepers {}
