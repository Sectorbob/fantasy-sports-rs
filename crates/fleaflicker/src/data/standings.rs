use crate::{CommishNote, League, Team};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LeagueStandings {
    pub divisions: Vec<Division>,
    pub season: i32,
    pub commish_note: Option<CommishNote>,
    pub league: League,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Division {
    pub id: i32,
    pub name: String,
    pub teams: Vec<Team>,
}
