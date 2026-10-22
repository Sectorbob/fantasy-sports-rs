use crate::ecr::service::DraftResult;
use fantasy_sports_sleeper::DraftPick;
use std::collections::{HashMap, HashSet};

pub fn get_keeper_status<T: Into<String>>(player_id: T, picks: &[DraftPick]) -> KeeperStatus {
    let player_id: String = player_id.into();
    if let Some(pick) = picks.iter().find(|p| p.player_id == player_id) {
        KeeperStatus {
            is_keeper: pick.is_keeper,
            was_drafted: !pick.is_keeper,
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

pub fn get_player_history<T: Into<String>>(
    player_id: T,
    draft_history: &HashMap<String, DraftResult>,
) -> HashMap<String, PlayerHistorySeason> {
    let player_id: String = player_id.into();
    let mut history = HashMap::<String, PlayerHistorySeason>::new();
    for (season, draft_result) in draft_history {
        if let Some(pick) = draft_result.picks.iter().find(|p| p.player_id == player_id) {
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
    history
}

pub struct KeepablesReport {
    pub season: String,
    pub keepables: Vec<Keepable>,
}
impl KeepablesReport {
    pub fn text_report(&self) -> String {
        let mut text = String::new();

        let mut owners = self
            .keepables
            .iter()
            .map(|k| k.owner_name.to_string())
            .collect::<HashSet<String>>()
            .into_iter()
            .collect::<Vec<String>>();
        owners.sort_by_key(|owner| owner.to_ascii_lowercase());
        for owner in owners.iter() {
            text.push_str(owner);
            text.push('\n');

            let max_player_name_length = self
                .keepables
                .iter()
                .map(|k| k.player_name.len())
                .max()
                .unwrap_or_default();

            for keepable in &self
                .keepables
                .iter()
                .filter(|k| &k.owner_name == owner)
                .collect::<Vec<&Keepable>>()
            {
                let cost = keepable.cost(self.season.as_str());
                // lookup current ranking
                let cost = if let Some(player_rank) = &keepable.player_rank {
                    Some(format!(
                        "COST={cost:?} ADP~{player_rank}",
                        cost = cost.unwrap()
                    ))
                } else {
                    Some(format!("COST={cost:?} ADP~UNKNOWN", cost = cost.unwrap()))
                };
                text.push_str(
                    format!(
                        " {player:<max_player_name_length$} {cost}\n",
                        player = keepable.player_name,
                        cost = cost.map(|v| format!(" ({v})")).unwrap_or_default()
                    )
                    .as_str(),
                );
            }
        }

        text
    }
}

#[allow(dead_code)]
pub struct Keepable {
    pub owner_id: String,
    pub owner_name: String,
    pub was_keeper_last_year: bool,
    pub player_id: String,
    pub player_name: String,
    pub position: String,
    pub round: u8,
    pub history: HashMap<String, PlayerHistorySeason>,
    pub player_rank: Option<String>,
}
impl Keepable {
    pub fn cost<T: Into<String>>(&self, season: T) -> Option<String> {
        let mut kept_since: Option<String> = None;
        let season_int: i32 = season.into().parse().unwrap();

        let mut seasons: Vec<String> = self.history.keys().map(Into::into).collect();
        seasons.sort();
        seasons.reverse();

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
            return self
                .history
                .get(season_drafted.to_string().as_str())
                .map(|season_entry| format!("{}", season_entry.rd.unwrap()));
        }

        let ket_since_int: i32 = kept_since.unwrap().parse().unwrap();

        // contract renegotiation
        if season_int - ket_since_int >= 3 {
            return Some("ADP".to_string());
        }

        Some(
            self.history
                .get(ket_since_int.to_string().as_str())
                .unwrap()
                .rd
                .unwrap()
                .to_string(),
        )
    }
}

pub struct KeeperStatus {
    pub is_keeper: bool,
    pub was_drafted: bool,
    pub draft_round: Option<u8>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct PlayerHistorySeason {
    pub season: String,
    pub drafted: bool,
    pub kept: bool,
    pub rd: Option<u8>,
    pub roster_id: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn history(entries: &[(&str, bool, u8)]) -> HashMap<String, PlayerHistorySeason> {
        entries
            .iter()
            .map(|(season, kept, rd)| {
                (
                    season.to_string(),
                    PlayerHistorySeason {
                        season: season.to_string(),
                        drafted: !kept,
                        kept: *kept,
                        rd: Some(*rd),
                        roster_id: 1,
                    },
                )
            })
            .collect()
    }

    fn keepable(history: HashMap<String, PlayerHistorySeason>) -> Keepable {
        Keepable {
            owner_id: "1".into(),
            owner_name: "owner".into(),
            was_keeper_last_year: false,
            player_id: "9001".into(),
            player_name: "Casey Catcher".into(),
            position: "WR".into(),
            round: 3,
            history,
            player_rank: None,
        }
    }

    #[test]
    fn cost_is_draft_round_the_first_time_a_player_is_kept() {
        let k = keepable(history(&[("2024", false, 7)]));
        assert_eq!(Some("7".to_string()), k.cost("2025"));
    }

    #[test]
    fn cost_is_round_of_first_keep_in_current_streak() {
        // drafted 2023 (rd 5), kept 2024 (rd 4)
        let k = keepable(history(&[("2023", false, 5), ("2024", true, 4)]));
        assert_eq!(Some("4".to_string()), k.cost("2025"));
    }

    #[test]
    fn cost_resets_to_adp_after_three_straight_keeps() {
        let k = keepable(history(&[
            ("2022", true, 3),
            ("2023", true, 3),
            ("2024", true, 3),
        ]));
        assert_eq!(Some("ADP".to_string()), k.cost("2025"));
        // two straight keeps is still priced by the original round
        let k = keepable(history(&[("2023", true, 3), ("2024", true, 3)]));
        assert_eq!(Some("3".to_string()), k.cost("2025"));
    }

    #[test]
    fn cost_is_none_when_not_drafted_last_year() {
        let k = keepable(history(&[("2022", false, 9)]));
        assert_eq!(None, k.cost("2025"));
    }

    #[test]
    fn keeper_status_reflects_the_draft_pick() {
        assert!(!get_keeper_status("9999", &[]).was_drafted);
    }
}
