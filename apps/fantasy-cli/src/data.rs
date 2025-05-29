use std::collections::HashMap;

use serde_json::Number;
use sleeper_fantasy_rs::{Player as SleeperPlayer, custom::FantasyMatchup};

use crate::{matchups_list_widget::Player, roster_widget::RosterWidget};

// #[derive(Debug, Clone)]
pub enum League {
    Sleeper {
        league: sleeper_fantasy_rs::League,
        matchups: HashMap<usize, Vec<FantasyMatchup>>,
        owners: HashMap<String, sleeper_fantasy_rs::User>,
        players: HashMap<String, SleeperPlayer>,
        rosters: Vec<sleeper_fantasy_rs::Roster>,
        user_id: String,
    },
    Yahoo {
        team: yahoo_fantasy_rs::Team,
        league: yahoo_fantasy_rs::League,
        standings: yahoo_fantasy_rs::Standings,
        rosters: HashMap<u32, yahoo_fantasy_rs::Roster>,
        settings: yahoo_fantasy_rs::Settings,
        matchups: HashMap<usize, Vec<FantasyMatchup>>,
    },
}
impl League {
    pub fn id(&self) -> String {
        match self {
            League::Sleeper {
                league,
                matchups: _,
                owners: _,
                players: _,
                rosters: _,
                user_id: _,
            } => league.league_id.clone(),
            League::Yahoo {
                league,
                matchups: _,
                rosters: _,
                settings: _,
                standings: _,
                team: _,
            } => league.league_id.clone(),
        }
    }
    pub fn name(&self) -> String {
        match self {
            League::Sleeper {
                league,
                matchups: _,
                owners: _,
                players: _,
                rosters: _,
                user_id: _,
            } => league.name.clone(),
            League::Yahoo {
                league,
                matchups: _,
                rosters: _,
                settings: _,
                standings: _,
                team: _,
            } => league.name.clone(),
        }
    }
    pub fn season(&self) -> String {
        match self {
            League::Sleeper {
                league,
                matchups: _,
                owners: _,
                players: _,
                rosters: _,
                user_id: _,
            } => league.season.clone(),
            League::Yahoo {
                league,
                matchups: _,
                rosters: _,
                settings: _,
                standings: _,
                team: _,
            } => league.season.clone(),
        }
    }
    pub fn record(&self) -> String {
        match self {
            League::Sleeper {
                league: _,
                matchups: _,
                owners: _,
                players: _,
                rosters,
                user_id,
            } => match rosters.iter().find(|r| &r.owner_id == user_id) {
                Some(roster) => format!(
                    "{}-{}-{}",
                    match roster.settings.get("wins") {
                        Some(n) => n.to_string(),
                        None => "0".to_string(),
                    },
                    match roster.settings.get("losses") {
                        Some(n) => n.to_string(),
                        None => "0".to_string(),
                    },
                    match roster.settings.get("ties") {
                        Some(n) => n.to_string(),
                        None => "0".to_string(),
                    }
                ),
                None => "n/a".to_string(),
            },
            League::Yahoo {
                league: _,
                matchups: _,
                rosters: _,
                settings: _,
                standings,
                team,
            } => match standings.teams().iter().find(|t| t.team_id == team.team_id) {
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
                league: _,
                matchups: _,
                owners: _,
                players: _,
                rosters: _,
                user_id: _,
            } => "".to_string(),
            League::Yahoo {
                league: _,
                matchups: _,
                rosters: _,
                settings: _,
                standings: _,
                team,
            } => team.name.clone(),
        }
    }
    pub fn total_points(&self) -> f64 {
        match self {
            League::Sleeper {
                league: _,
                matchups: _,
                owners: _,
                players: _,
                rosters,
                user_id,
            } => match rosters.iter().find(|r| &r.owner_id == user_id) {
                Some(roster) => roster
                    .settings
                    .get("points")
                    .and_then(|p| p.as_f64())
                    .unwrap_or(-2.0) as f64,
                None => -1.0,
            },
            League::Yahoo {
                league: _,
                matchups: _,
                rosters: _,
                settings: _,
                standings,
                team,
            } => match standings.teams().iter().find(|t| t.team_id == team.team_id) {
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
            League::Sleeper {
                league,
                matchups: _,
                owners: _,
                players: _,
                rosters: _,
                user_id: _,
            } => LeagueSettings::from_sleeper(&league.settings),
            League::Yahoo {
                league: _,
                matchups: _,
                rosters: _,
                settings,
                standings: _,
                team: _,
            } => LeagueSettings::from_yahoo(settings),
        }
    }
    pub fn raw(&self) -> String {
        match self {
            League::Sleeper {
                league,
                matchups: _,
                owners: _,
                players: _,
                rosters: _,
                user_id: _,
            } => format!("Raw: {:#?}", league.settings),
            League::Yahoo {
                league: _,
                matchups: _,
                rosters: _,
                settings,
                standings: _,
                team: _,
            } => format!("Raw: {:#?}", settings),
        }
    }
    pub fn roster_widget(&self) -> Option<RosterWidget> {
        match self {
            League::Sleeper {
                league: _,
                matchups: _,
                owners,
                players,
                rosters,
                user_id,
            } => rosters
                .iter()
                .find(|r| &r.owner_id == user_id)
                .map(|roster| RosterWidget::from_sleeper(roster, owners, players)),
            League::Yahoo {
                league: _,
                matchups: _,
                rosters,
                settings: _,
                standings: _,
                team,
            } => RosterWidget::from_yahoo(team, rosters.get(&team.team_id).unwrap()).into(),
        }
    }
    pub fn matchups(&self) -> &HashMap<usize, Vec<FantasyMatchup>> {
        match self {
            League::Sleeper {
                league: _,
                matchups,
                owners: _,
                players: _,
                rosters: _,
                user_id: _,
            } => matchups,
            League::Yahoo {
                league: _,
                matchups,
                rosters: _,
                settings: _,
                standings: _,
                team: _,
            } => matchups, // Yahoo leagues do not have matchups in the same way
        }
    }
    pub fn players(&self) -> HashMap<String, Player> {
        match self {
            League::Sleeper {
                league: _,
                matchups: _,
                owners: _,
                players,
                rosters: _,
                user_id: _,
            } => players
                .iter()
                .map(|(id, p)| (id.clone(), Player::from_sleeper(p)))
                .collect::<HashMap<String, Player>>(),
            League::Yahoo {
                team: _,
                league: _,
                standings: _,
                rosters: _,
                settings: _,
                matchups: _,
            } => HashMap::new(),
        }
    }
}
impl std::fmt::Display for League {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            League::Sleeper {
                league,
                matchups: _,
                owners: _,
                players: _,
                rosters: _,
                user_id: _,
            } => write!(
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
            League::Yahoo {
                league,
                matchups: _,
                rosters: _,
                settings: _,
                standings: _,
                team: _,
            } => write!(
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

#[derive(Debug)]
pub struct LeagueSettings {
    /// The week number in the season when the playoffs begin.
    pub playoff_start_week: u32,
}
impl LeagueSettings {
    pub fn from_yahoo(yahoo_settings: &yahoo_fantasy_rs::Settings) -> Self {
        LeagueSettings {
            playoff_start_week: yahoo_settings.playoff_start_week,
        }
    }
    pub fn from_sleeper(sleeper_settings: &HashMap<String, Number>) -> Self {
        LeagueSettings {
            playoff_start_week: sleeper_settings
                .get("playoff_week_start")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
        }
    }
}
