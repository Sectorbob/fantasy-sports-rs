use super::helpers::empty_string_as_none;
use core::fmt;
use serde::{Deserialize, Deserializer, de};

use crate::{
    Player,
    data::{game::GameKey, league::LeagueKey},
};

// TODO: I want to do away with the wrapper struct (at least publicly)
#[derive(Deserialize, Debug, PartialEq, Clone)]
struct Teams {
    #[serde(rename = "team", default)]
    pub teams: Vec<Team>,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct Team {
    pub team_key: TeamKey,
    pub team_id: u32,
    pub name: String,
    pub is_owned_by_current_login: Option<bool>, // 0 or 1
    pub url: String,
    #[serde(rename = "team_logos")]
    _team_logos: TeamLogos,
    pub division_id: Option<u32>,
    #[serde(deserialize_with = "empty_string_as_none")]
    pub waiver_priority: Option<u32>,
    pub faab_balance: Option<u32>,
    pub number_of_moves: u32,
    pub number_of_trades: u32,
    // roster_adds: RosterAdds,
    pub clinched_playoffs: Option<bool>, // 0 or 1
    pub league_scoring_type: String,     // knwon values: head
    pub has_draft_grade: bool,           // 0 or 1
    pub draft_grade: Option<String>,
    pub draft_recap_url: Option<String>,
    #[serde(rename = "managers")]
    _managers: Managers,
    pub team_points: Option<TeamPoints>,
    pub team_standings: Option<TeamStandings>,
    pub roster: Option<Roster>,
}
impl Team {
    pub fn managers(&self) -> &Vec<Manager> {
        &self._managers.managers
    }
    pub fn team_logos(&self) -> &Vec<TeamLogo> {
        &self._team_logos.team_logos
    }
    pub(crate) fn deserialize_teams<'de, D>(deserializer: D) -> Result<Vec<Team>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Teams::deserialize(deserializer).map(|r| r.teams)
    }
}

///
/// Deserialize a TeamKey from a string in the format `league_key.t.team_id`
/// where the format for league key is `game_key.l.league_id`.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct TeamKey {
    pub league_key: LeagueKey,
    pub team_id: u32,
}
impl TeamKey {
    pub fn new(league_key: LeagueKey, team_id: u32) -> Self {
        TeamKey {
            league_key: league_key.clone(),
            team_id,
        }
    }
}
impl std::fmt::Display for TeamKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.t.{}", self.league_key, self.team_id)
    }
}
impl<'de> Deserialize<'de> for TeamKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserialize_team_key(deserializer)
    }
}
fn deserialize_team_key<'de, D>(deserializer: D) -> Result<TeamKey, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let mut parts = s.split('.');

    let game_key_str = parts
        .next()
        .ok_or_else(|| de::Error::custom("missing game_key"))?
        .to_string();
    let game_key = game_key_str
        .parse::<u32>()
        .map_err(|e| de::Error::custom(format!("game_key parse error: {}", e)))?;

    // skip the middle "l"
    let _ = parts.next();

    let league_id_str = parts
        .next()
        .ok_or_else(|| de::Error::custom("missing league_id"))?;

    let league_id = league_id_str
        .parse::<u32>()
        .map_err(|e| de::Error::custom(format!("league_id parse error: {}", e)))?;

    // skip the middle "t"
    let _ = parts.next();

    let team_id_str = parts
        .next()
        .ok_or_else(|| de::Error::custom("missing team_id"))?;

    let team_id = team_id_str
        .parse::<u32>()
        .map_err(|e| de::Error::custom(format!("team_id parse error: {}", e)))?;

    Ok(TeamKey {
        league_key: LeagueKey::new(GameKey::Id(game_key), league_id),
        team_id,
    })
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
struct TeamLogos {
    #[serde(rename = "team_logo", default)]
    pub team_logos: Vec<TeamLogo>,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct TeamLogo {
    pub size: String,
    pub url: String,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct TeamPoints {
    pub coverage_type: String, // known values: season, week
    pub season: Option<String>,
    pub week: Option<i32>,
    #[serde(deserialize_with = "empty_string_as_none")]
    pub total: Option<f32>,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct TeamStandings {
    #[serde(deserialize_with = "empty_string_as_none")]
    pub rank: Option<u32>,
    pub outcome_totals: OutcomeTotals,
    pub divisional_outcome_totals: Option<OutcomeTotals>,
    pub streak: Option<Streak>,
    pub points_for: f32,
    pub points_against: f32,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
struct Managers {
    #[serde(rename = "manager", default)]
    pub managers: Vec<Manager>,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Manager {
    pub manager_id: u32,
    pub nickname: String,
    pub guid: String,
    pub is_commissioner: Option<bool>,  // 0 or 1
    pub is_current_login: Option<bool>, // 0 or 1
    pub image_url: Option<String>,
    pub felo_score: Option<u32>,
    pub felo_tier: Option<FeloTier>,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum FeloTier {
    Diamond,
    Platinum,
    Gold,
    Silver,
    Bronze,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct OutcomeTotals {
    pub wins: u32,
    pub losses: u32,
    pub ties: u32,
    #[serde(deserialize_with = "empty_string_as_none", default)]
    pub percentage: Option<f32>,
}

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub struct Roster {
    pub coverage_type: String, // date or week
    pub date: Option<String>,
    pub week: Option<String>,
    pub is_prescoring: Option<bool>,
    pub is_editable: Option<bool>,
    #[serde(default, deserialize_with = "Player::deserialize_players")]
    pub players: Vec<Player>,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Streak {
    pub r#type: String,
    pub value: String,
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::BufReader};

    use crate::{FantasyContent, data::FantasyContentWrapper};

    #[test]
    fn test_deserialize_games() {
        let reader = BufReader::new(
            File::open("tests/data/fantasy/v2/league/380.l.129397/teams/roster.xml")
                .expect("failed to open test data file"),
        );
        let w: FantasyContentWrapper = quick_xml::de::from_reader(reader)
            .expect("failed to deserialize league with roster xml");
        let teams = if let FantasyContent::League(league) = w.content {
            league.teams
        } else {
            debug_assert!(false, "fuck me");
            return;
        };

        debug_assert_eq!(12, teams.len());
    }
}
