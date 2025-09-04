use crate::{FormattedDecimal, LeaguePlayer, RosterPosition, Team, serde_epoch_millis};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BoxScore {
    pub game: FantasyGame,
    pub lineups: Vec<MatchupRosterGroup>,
    pub points_away: MatchupTeamScore,
    pub points_home: MatchupTeamScore,
    pub eligible_periods: Vec<ScoringPeriod>,
    pub scoring_period: ScoringPeriod,
    #[serde(default)]
    pub is_in_progress: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FantasyGame {
    pub id: String,
    pub away: Team,
    pub home: Team,
    pub away_score: GameResult,
    pub home_score: GameResult,
    pub is_divisional: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MatchupRosterGroup {
    pub group: Option<RosterPositionGroup>,
    pub slots: Vec<MatchupRosterSlot>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MatchupRosterSlot {
    pub position: RosterPosition,
    /// Can be None when an owner has not set a full lineup
    pub away: Option<LeaguePlayer>,
    /// Can be None when an owner has not set a full lineup
    pub home: Option<LeaguePlayer>,
    pub position_color: Option<Vec<DraftBoardCellColor>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MatchupTeamScore {
    pub total: MatchupTeamSum,
    pub scoring_period: MatchupTeamSum,
    pub win_probability: f64,
    pub is_win_probability_set: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MatchupTeamSum {
    pub value: FormattedDecimal,
    pub optimum: Option<FormattedDecimal>,
    pub projected: FormattedDecimal,
    pub optimum_lineup: Option<Vec<Value>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ScoringPeriod {
    pub ordinal: i32,
    #[serde(rename = "startEpochMilli", with = "serde_epoch_millis")]
    pub start_ts: DateTime<Utc>,
    pub is_now: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GameResult {
    pub yet_to_play: i32,
    pub score: FormattedDecimal,
    pub yet_to_play_positions: Vec<String>,
    pub projected: FormattedDecimal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum RosterPositionGroup {
    BENCH,
    START,
    INJURED,
    TAXI,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", deny_unknown_fields)]
pub enum DraftBoardCellColor {
    DraftBoardEmpty,
    DraftBoardGreen,
    DraftBoardBlue,
    DraftBoardYellow,
    DraftBoardRed,
    DraftBoardGray,
    DraftBoardPurple,
}
impl DraftBoardCellColor {
    pub fn as_str(&self) -> &'static str {
        match self {
            DraftBoardCellColor::DraftBoardEmpty => "DRAFT_BOARD_EMPTY",
            DraftBoardCellColor::DraftBoardGreen => "DRAFT_BOARD_GREEN",
            DraftBoardCellColor::DraftBoardBlue => "DRAFT_BOARD_BLUE",
            DraftBoardCellColor::DraftBoardYellow => "DRAFT_BOARD_YELLOW",
            DraftBoardCellColor::DraftBoardRed => "DRAFT_BOARD_RED",
            DraftBoardCellColor::DraftBoardGray => "DRAFT_BOARD_GRAY",
            DraftBoardCellColor::DraftBoardPurple => "DRAFT_BOARD_PURPLE",
        }
    }
    pub fn as_rgb(&self) -> (u8, u8, u8) {
        todo!("implement the to rgb fn for DraftBoardCellColor")
    }
}
