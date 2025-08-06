use std::collections::HashMap;

use sleeper_fantasy_rs::{DraftPick, Player};

use crate::ecr::service::DraftResult;

pub fn get_keeper_status(player: &Player, picks: &Vec<DraftPick>) -> KeeperStatus {
    if let Some(pick) = picks.iter().find(|p| p.player_id == player.player_id) {
        KeeperStatus {
            is_keeper: pick.is_keeper,
            was_drafted: pick.is_keeper,
            draft_round: Some(pick.round),
        }
    } else {
        KeeperStatus {
            is_keeper: false,
            was_drafted: false,
            draft_round: None,
        }
    }
}

pub fn get_player_history(
    player: &Player,
    draft_history: &HashMap<String, DraftResult>,
) -> HashMap<String, PlayerHistorySeason> {
    let mut history = HashMap::<String, PlayerHistorySeason>::new();
    for (season, draft_result) in draft_history {
        if let Some(pick) = draft_result
            .picks
            .iter()
            .find(|p| p.player_id == player.player_id)
        {
            history.insert(
                season.clone(),
                PlayerHistorySeason {
                    season: season.clone(),
                    drafted: !pick.is_keeper,
                    kept: pick.is_keeper,
                    rd: Some(pick.round),
                    roster_id: pick.roster_id,
                },
            );
            // history.insert(season, PlayerHistorySeason());
        }
    }
    return history;
}

pub struct KeepablesReport {
    pub keepables: Vec<Keepable>,
}

pub struct Keepable {
    pub ownerId: String,
    pub ownerName: String,
    pub wasKeeperLastYear: bool,
    pub playerId: String,
    pub playerName: String,
    pub position: String,
    pub round: u8,
    pub history: HashMap<String, PlayerHistorySeason>,
}
impl Keepable {
    pub fn cost(&self, season: String) -> String {
        let mut kept_since: Option<String> = None;
        let season_int: i32 = season.parse().unwrap();

        let mut seasons: Vec<String> = self.history.keys().map(Into::into).collect();
        seasons.sort();

        for s in seasons {
            if let Some(history_entry) = self.history.get(s.as_str()) {
                if history_entry.kept {
                    kept_since = Some(s)
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        // this is the first time this pick is being kept
        if kept_since.is_none() {
            let season_drafted: i32 = season_int - 1;
            let season_entry = self
                .history
                .get(season_drafted.to_string().as_str())
                .unwrap();
            return format!("{}", season_entry.rd.unwrap());
        }

        let ket_since_int: i32 = kept_since.unwrap().parse().unwrap();

        // contract renegotiation
        if season_int - ket_since_int >= 3 {
            return "ADP".to_string();
        }

        self.history
            .get(ket_since_int.to_string().as_str())
            .unwrap()
            .rd
            .unwrap()
            .to_string()
    }
}

pub struct KeeperStatus {
    pub is_keeper: bool,
    pub was_drafted: bool,
    pub draft_round: Option<u8>,
}

#[derive(Debug)]
pub struct PlayerHistorySeason {
    pub season: String,
    pub drafted: bool,
    pub kept: bool,
    pub rd: Option<u8>,
    pub roster_id: u8,
}

mod tests {}
