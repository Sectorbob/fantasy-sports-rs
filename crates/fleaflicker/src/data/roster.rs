use serde::{Deserialize, Serialize};

use crate::{LeaguePlayer, Team};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FetchRostersResponse {
    pub rosters: Vec<Roster>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Roster {
    pub team: Team,
    pub players: Vec<LeaguePlayer>,
}
