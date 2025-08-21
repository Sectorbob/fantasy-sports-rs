use serde_json::Number;
use sleeper_fantasy_rs as sleeper;
use std::collections::HashMap;
use yahoo_fantasy_rs as yahoo;

use crate::{
    Draft, DraftPick, ExternalId, LeagueSettings, Player, Roster, Standings, StandingsEntry,
    Transaction, data::Scoreboard,
};

#[derive(Debug, Clone)]
pub enum League {
    Sleeper {
        draft: Option<sleeper::Draft>,
        draft_picks: Vec<sleeper::DraftPick>,
        league: sleeper::League,
        matchups: HashMap<usize, Vec<sleeper::Matchup>>,
        owners: HashMap<String, sleeper::User>,
        players: HashMap<String, sleeper::Player>,
        rosters: HashMap<u8, sleeper::Roster>,
        transactions: Vec<sleeper::Transaction>,
        user_id: Option<String>,
    },
    Yahoo {
        draft_results: Vec<yahoo::DraftResult>,
        league: yahoo::League,
        scoreboards: Vec<yahoo::Scoreboard>,
        players: HashMap<yahoo::PlayerKey, crate::Player>,
        rosters: HashMap<u32, yahoo::Roster>,
        settings: yahoo::Settings,
        standings: yahoo::Standings,
        team: Option<yahoo::Team>,
        teams: Vec<yahoo::Team>,
        transactions: Vec<yahoo::Transaction>,
    },
}
impl League {
    pub fn with_team(self, id: ExternalId) -> Self {
        match self {
            League::Sleeper {
                draft,
                draft_picks,
                league,
                matchups,
                owners,
                players,
                rosters,
                transactions,
                user_id: _,
            } => League::Sleeper {
                draft,
                draft_picks,
                league,
                matchups,
                owners,
                players,
                rosters,
                transactions,
                user_id: Some(id.id),
            },
            League::Yahoo {
                draft_results,
                league,
                scoreboards,
                players,
                rosters,
                settings,
                standings,
                team: _,
                teams,
                transactions,
            } => League::Yahoo {
                draft_results,
                league,
                scoreboards,
                players,
                rosters,
                settings,
                standings,
                team: teams
                    .iter()
                    .find(|t| t.managers().iter().find(|m| m.guid == id.id).is_some())
                    .map(yahoo::Team::clone),
                teams,
                transactions,
            },
        }
    }

    pub fn id(&self) -> String {
        match self {
            League::Sleeper { league, .. } => league.league_id.clone(),
            League::Yahoo { league, .. } => league.league_key.to_string(),
        }
    }
    pub fn name(&self) -> String {
        match self {
            League::Sleeper { league, .. } => league.name.clone(),
            League::Yahoo { league, .. } => league.name.clone(),
        }
    }
    pub fn platform(&self) -> String {
        match self {
            League::Sleeper { .. } => String::from("Sleeper"),
            League::Yahoo { .. } => String::from("Yahoo"),
        }
    }
    pub fn season(&self) -> String {
        match self {
            League::Sleeper { league, .. } => league.season.clone(),
            League::Yahoo { league, .. } => league.season.clone(),
        }
    }
    pub fn standings(&self) -> Standings {
        match self {
            League::Sleeper {
                rosters, owners, ..
            } => Standings {
                entries: rosters
                    .iter()
                    .map(|(id, roster)| {
                        let wins = roster
                            .settings
                            .get("wins")
                            .map(Number::as_u64)
                            .flatten()
                            .unwrap_or_default();
                        let losses = roster
                            .settings
                            .get("losses")
                            .map(Number::as_u64)
                            .flatten()
                            .unwrap_or_default();
                        let ties = roster
                            .settings
                            .get("ties")
                            .map(Number::as_u64)
                            .flatten()
                            .unwrap_or_default();
                        let owner_id = &roster.owner_id;
                        let owner = owner_id.as_ref().map(|id| owners.get(id)).flatten();
                        StandingsEntry {
                            roster_id: roster.roster_id.to_string(),
                            team_name: owner.map_or(format!("Team {id}"), sleeper::User::team_name),
                            owner_name: owner
                                .as_ref()
                                .map_or(format!("Owner {owner_id:?}"), |o| o.display_name.clone()),
                            owner_id: owner_id.as_ref().map_or(String::new(), |o| o.clone()),
                            wins,
                            losses,
                            ties,
                        }
                    })
                    .collect(),
            },
            League::Yahoo { standings, .. } => Standings {
                entries: standings
                    .teams
                    .iter()
                    .map(|team| {
                        let wins = team
                            .team_standings
                            .as_ref()
                            .map(|s| s.outcome_totals.wins as u64)
                            .unwrap_or_default();
                        let losses = team
                            .team_standings
                            .as_ref()
                            .map(|s| s.outcome_totals.losses as u64)
                            .unwrap_or_default();
                        let ties = team
                            .team_standings
                            .as_ref()
                            .map(|s| s.outcome_totals.ties as u64)
                            .unwrap_or_default();
                        let manager = team.managers().first().clone().unwrap();
                        StandingsEntry {
                            roster_id: team.team_key.to_string(),
                            team_name: team.name.clone(),
                            owner_name: manager.nickname.clone(),
                            owner_id: manager.guid.clone(),
                            wins,
                            losses,
                            ties,
                        }
                    })
                    .collect(),
            },
        }
    }

    pub fn record(&self) -> String {
        match self {
            League::Sleeper {
                rosters, user_id, ..
            } => match rosters
                .iter()
                .find(|(_, r)| r.owner_id.is_some() && &r.owner_id == user_id)
            {
                Some((_, roster)) => format!(
                    "{}-{}-{}",
                    match roster.settings.get("wins") {
                        Some(n) => n.to_string(),
                        None => 0.to_string(),
                    },
                    match roster.settings.get("losses") {
                        Some(n) => n.to_string(),
                        None => 0.to_string(),
                    },
                    match roster.settings.get("ties") {
                        Some(n) => n.to_string(),
                        None => 0.to_string(),
                    }
                ),
                None => "n/a".to_string(),
            },
            League::Yahoo {
                standings, team, ..
            } => match standings
                .teams
                .iter()
                .find(|t| Some(t.team_id) == team.as_ref().map(|t| t.team_id))
            {
                Some(team_w_standings) => match &team_w_standings.team_standings {
                    Some(team_standings) => format!(
                        "{}-{}-{}",
                        team_standings.outcome_totals.wins,
                        team_standings.outcome_totals.losses,
                        team_standings.outcome_totals.ties
                    ),
                    None => "n/a".to_string(),
                },
                None => "n/a".to_string(),
            },
        }
    }
    pub fn team_name(&self) -> String {
        match self {
            League::Sleeper {
                owners, user_id, ..
            } => user_id
                .as_ref()
                .map(|user_id| match owners.get(user_id) {
                    Some(owner) => owner.team_name(),
                    None => format!("Unknown User ({})", user_id),
                })
                .unwrap_or(format!("Unknown User (None)")),
            League::Yahoo { team, .. } => team.as_ref().map(|t| t.name.clone()).unwrap_or_default(),
        }
    }
    pub fn total_points(&self) -> f64 {
        match self {
            League::Sleeper {
                rosters, user_id, ..
            } => match rosters
                .iter()
                .find(|(_, r)| r.owner_id.is_some() && user_id.is_some() && &r.owner_id == user_id)
            {
                Some((_, roster)) => roster
                    .settings
                    .get("points")
                    .and_then(|p| p.as_f64())
                    .unwrap_or(-2.0) as f64,
                None => -1.0,
            },
            League::Yahoo {
                standings, team, ..
            } => match standings
                .teams
                .iter()
                .find(|t| Some(t.team_id) == team.as_ref().map(|t| t.team_id))
            {
                Some(team_w_standings) => match &team_w_standings.team_standings {
                    Some(team_standings) => team_standings.points_for as f64,
                    None => -1.0,
                },
                None => -2.0,
            },
        }
    }
    pub fn settings(&self) -> LeagueSettings {
        match self {
            League::Sleeper { league, draft, .. } => LeagueSettings::from_sleeper(&league, draft),
            League::Yahoo { settings, .. } => LeagueSettings::from_yahoo(settings),
        }
    }
    #[allow(dead_code)]
    pub fn raw(&self) -> String {
        match self {
            League::Sleeper { league, .. } => format!("Raw: {:#?}", league.settings),
            League::Yahoo { settings, .. } => format!("Raw: {:#?}", settings),
        }
    }
    pub fn scoreboards(&self) -> Vec<Scoreboard> {
        match self {
            League::Sleeper {
                matchups,
                owners,
                rosters,
                ..
            } => matchups
                .iter()
                .map(|(week, matchups)| {
                    Scoreboard::from_sleeper(week.clone() as u32, matchups.clone(), rosters, owners)
                })
                .collect(),
            League::Yahoo { scoreboards, .. } => scoreboards.iter().map(Scoreboard::from).collect(), // Yahoo leagues do not have matchups in the same way
        }
    }
    pub fn players(&self) -> HashMap<String, Player> {
        match self {
            League::Sleeper { players, .. } => players
                .iter()
                .map(|(id, p)| (id.clone(), Player::from(p)))
                .collect::<HashMap<String, Player>>(),
            League::Yahoo { players, .. } => players
                .iter()
                .map(|(_, player)| (player._id.clone(), player.clone()))
                .collect(),
        }
    }
    pub fn draft(&self) -> Option<Draft> {
        match self {
            League::Sleeper {
                draft, draft_picks, ..
            } => draft.as_ref().map(|d| Draft::from((d, draft_picks))),
            League::Yahoo {
                draft_results,
                league,
                players,
                settings,
                ..
            } => match draft_results.len() {
                0 => None,
                _ => Some(Draft {
                    draft_type: settings.draft_type.to_string(),
                    draft_id: "N/A".to_string(),
                    league_id: league.league_id.clone(),
                    picks: draft_results
                        .iter()
                        .map(|p| {
                            if let Some(player_key) = &p.player_key {
                                if let Some(player) = players.get(player_key) {
                                    DraftPick {
                                        round: p.round,
                                        pick: p.pick,
                                        overall_pick: p.pick,
                                        player: player.clone(),
                                        roster_id: p.team_key.to_string(),
                                    }
                                } else {
                                    DraftPick {
                                        round: p.round,
                                        pick: p.pick,
                                        overall_pick: p.pick,
                                        player: Player {
                                            _id: player_key.to_string(),
                                            name: player_key.to_string(),
                                            positions: vec![],
                                            irl_team: None,
                                        },
                                        roster_id: p.team_key.to_string(),
                                    }
                                }
                            } else {
                                // No player_key onm the draft pick
                                DraftPick {
                                    round: p.round,
                                    pick: p.pick,
                                    overall_pick: p.pick,
                                    player: Player {
                                        _id: String::from("N/A"),
                                        name: String::from("N/A"),
                                        positions: vec![],
                                        irl_team: None,
                                    },
                                    roster_id: p.team_key.to_string(),
                                }
                            }
                        })
                        .collect(),
                }),
            },
        }
    }
    pub fn rosters(&self) -> Vec<Roster> {
        match self {
            League::Sleeper {
                rosters, owners, ..
            } => rosters
                .iter()
                .map(|r| r.1)
                .map(|r| Roster::from((r, owners)))
                .collect(),
            League::Yahoo { rosters, .. } => rosters.iter().map(Roster::from).collect(),
        }
    }
    pub fn transactions(&self) -> Vec<Transaction> {
        match self {
            League::Sleeper { transactions, .. } => {
                transactions.iter().map(Transaction::from).collect()
            }
            League::Yahoo { transactions, .. } => {
                transactions.iter().map(Transaction::from_yahoo).collect()
            }
        }
    }
}
impl std::fmt::Display for League {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            League::Sleeper { league, .. } => write!(
                f,
                "[{season}] {name} (Sleeper) - {status}    Record: {record}",
                season = league.season,
                name = league.name,
                status = match league.status {
                    sleeper_fantasy_rs::LeagueStatus::PreDraft => "predraft",
                    sleeper_fantasy_rs::LeagueStatus::Drafting => "drafting",
                    sleeper_fantasy_rs::LeagueStatus::InSeason => "inseason",
                    sleeper_fantasy_rs::LeagueStatus::Complete => "complete",
                },
                record = self.record(),
            ),
            League::Yahoo { league, .. } => write!(
                f,
                "[{}] {} (Yahoo) - {}       Record: {}",
                league.season,
                league.name,
                match league.is_finished {
                    Some(is_finished) => match is_finished {
                        true => "complete",
                        false => "in-progress",
                    },
                    None => "incomplete",
                },
                self.record(),
            ),
        }
    }
}
