use chrono::NaiveDate;
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{self, IntoDeserializer, MapAccess, Visitor},
};
use serde_json::Value;
use std::{fmt, str::FromStr};

#[derive(Deserialize, Debug)]
#[serde(rename = "yahoo:error")]
pub(crate) struct Error {
    #[serde(rename = "description")]
    pub description: Option<String>,
    #[serde(rename = "yahoo:description")]
    pub yahoo_description: Option<String>,
    pub detail: Option<String>,
}
impl Error {
    pub fn description(&self) -> String {
        self.description
            .clone()
            .or_else(|| self.yahoo_description.clone())
            .unwrap_or_else(|| "Unknown error".to_string())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Token {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
}
#[derive(Deserialize, Debug)]
pub(crate) struct FantasyContentWrapper {
    #[serde(rename = "$value")]
    pub content: FantasyContent,
}

#[derive(Deserialize, Debug)]
#[serde(rename = "$value")]
pub enum FantasyContent {
    #[serde(rename = "league")]
    League(League),
    #[serde(rename = "game")]
    Game(Game),
    #[serde(rename = "users")]
    Users(Users),
    #[serde(rename = "games")]
    Games(Games),
    #[serde(rename = "leagues")]
    Leagues(Leagues),
    #[serde(rename = "team")]
    Team(Team),
}

#[derive(Deserialize, Debug)]
pub(crate) struct Leagues {
    #[serde(rename = "league")]
    pub leagues: Vec<League>,
}

#[derive(Deserialize, Debug)]
pub struct League {
    pub league_key: LeagueKey,
    pub league_id: String,
    pub name: String,
    pub url: String,
    pub draft_status: DraftStatus, // known values: predraft, postdraft
    pub num_teams: u32,
    pub edit_key: u32,
    #[serde(deserialize_with = "empty_string_as_none")]
    pub league_update_timestamp: Option<u64>,
    pub scoring_type: String,
    pub current_week: String,
    pub start_week: u32,
    pub end_week: u32,
    pub is_finished: Option<bool>,
    pub players: Option<Players>,
    pub scoreboard: Option<Scoreboard>,
    pub settings: Option<Settings>,
    pub standings: Option<Standings>,
    pub teams: Option<Teams>,
    pub transactions: Option<Transactions>,
    pub league_type: String, // TODO: make this into am enum
    #[serde(deserialize_with = "deserialize_naive_date")]
    pub start_date: NaiveDate,
    #[serde(deserialize_with = "deserialize_naive_date")]
    pub end_date: NaiveDate,
    pub season: String,
    pub draft_results: Option<DraftResults>,
    pub game_code: GameCode,
    pub is_plus_league: bool,
    pub is_pro_league: bool,
    pub is_cash_league: bool,
    pub renewed: Option<Value>,
    pub renew: Option<String>, // I think this is a reference to the previous year league (if exists)
    pub weekly_deadline: Option<Value>,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DraftStatus {
    PreDraft,
    PostDraft,
    // TODO: Handle fallback case with value
    // #[serde(untagged)]
    // Other(String),
}

#[derive(Deserialize, Debug)]
pub struct DraftResults {
    #[serde(rename = "draft_result")]
    pub draft_results: Vec<DraftResult>,
}

#[derive(Deserialize, Debug)]
pub struct DraftResult {
    pub pick: u32,
    pub round: u32,
    pub team_key: TeamKey,
    pub player_key: PlayerKey, // e.g. nfl.p.7241
}

///
/// e.g. nfl.p.7241
#[derive(Debug)]
pub struct PlayerKey {
    pub game_key: GameKey, // e.g. 7241
    pub player_id: u32,
}
impl std::fmt::Display for PlayerKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.p.{}", self.game_key, self.player_id)
    }
}
impl<'de> Deserialize<'de> for PlayerKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserialize_player_key(deserializer)
    }
}

#[derive(Deserialize, Debug)]
pub struct Scoreboard {
    pub week: u32,
    pub matchups: Matchups,
}

#[derive(Deserialize, Debug)]
pub struct Settings {
    pub draft_type: String,   // known values: live
    pub scoring_type: String, // known values: head
    pub uses_playoff: u8,     // 0 or 1
    pub playoff_start_week: u32,
    pub uses_playoff_reseeding: u8,     // 0 or 1
    pub uses_lock_eliminated_teams: u8, // 0 or 1
    pub uses_faab: u8,                  // 0 or 1
    pub trade_end_date: String,         // format: 2009-11-27
    pub trade_ratify_type: String,      // knwon values: commish
    pub trade_reject_time: u32,
    // roster_positions: RosterPositions,
    // stat_categories: StatCategories,
    pub divisions: Option<Divisions>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct Divisions {
    #[serde(rename = "division")]
    pub divisions: Vec<Division>,
}

#[derive(Deserialize, Debug)]
pub struct Division {
    division_id: u32,
    name: String,
}

#[derive(Deserialize, Debug)]
pub struct Standings {
    #[serde(rename = "teams")]
    _teams: Teams,
}
impl Standings {
    pub fn teams(&self) -> &Vec<Team> {
        &self._teams.teams
    }
}

#[derive(Deserialize, Debug)]
pub struct Matchups {
    #[serde(rename = "matchup")]
    pub matchups: Vec<Matchup>,
}

#[derive(Deserialize, Debug)]
pub struct Matchup {
    pub week: Option<u32>,
    pub status: String, // known values: postevent
    pub is_tied: bool,
    pub winner_team_key: String,
    pub teams: Teams,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum GameType {
    Full,
    PickemTeam,
    PickemGroup,
    PickemTeamList,
    Prediction,
    Squares,
}

pub struct Filters {
    pub is_available: Option<bool>,
    pub game_types: Option<Vec<GameType>>,
    pub game_codes: Option<Vec<GameCode>>,
    pub seasons: Option<Vec<String>>,
}
impl Filters {
    pub fn new() -> Self {
        Filters {
            is_available: None,
            game_types: None,
            game_codes: None,
            seasons: None,
        }
    }
    pub fn set_is_available(&mut self, is_available: bool) {
        self.is_available = Some(is_available);
    }
    pub fn set_game_types(&mut self, game_types: Vec<GameType>) {
        self.game_types = Some(game_types);
    }
    pub fn set_game_codes(&mut self, game_codes: Vec<GameCode>) {
        self.game_codes = Some(game_codes);
    }
    pub fn set_seasons(&mut self, seasons: Vec<String>) {
        self.seasons = Some(seasons);
    }

    pub fn to_string(&self) -> String {
        let mut filters = Vec::new();
        // if let Some(is_available) = self.is_available {
        //     filters.push(format!("is_available={}", is_available ? "1" : "0"));
        // }
        // if let Some(game_types) = &self.game_types {
        //     for game_type in game_types {
        //         filters.push(format!("game_types={}", game_type.to_string()));
        //     }
        // }
        if let Some(game_codes) = &self.game_codes {
            for game_code in game_codes {
                filters.push(format!("game_codes={}", game_code.to_string()));
            }
        }
        if let Some(seasons) = &self.seasons {
            for season in seasons {
                filters.push(format!("seasons={}", season));
            }
        }
        filters.join(",")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameKey {
    Code(GameCode),
    Id(u32),
}
impl std::fmt::Display for GameKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameKey::Code(game_code) => write!(f, "{}", game_code.to_string()),
            GameKey::Id(id) => write!(f, "{}", id),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeagueKey {
    game_key: GameKey,
    league_id: u32,
}
impl LeagueKey {
    pub fn new(game_key: GameKey, league_id: u32) -> Self {
        LeagueKey {
            game_key,
            league_id,
        }
    }
}
impl std::fmt::Display for LeagueKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.l.{}", self.game_key, self.league_id)
    }
}
impl<'de> Deserialize<'de> for LeagueKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserialize_league_key(deserializer)
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

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum GameCode {
    NCAAF,            // NCCAF College Football Season-long
    NFL,              // NFL Season-long
    NHL,              // NHL Season-long
    NBA,              // NBA Season-long
    MLB,              // MLB Season-long
    EPLPickem,        // sogb - english premier league pickem
    BundesligaPickem, // sode - german bundesliga pickem
    LaLigaPickem,     // soes - spanish la liga pickem
    SerieAPickem,     // soit - italian Serie A pickem
    MLSPickem,        // sous - Major League Soccer Pickem
    NFLPickem,        // NFL Pickem
    NFLSurvival,      // NFL Survivor Pool
    NCAAFPickem,      // NCAAF Pickem
    PGAPickem,        // PGA Tour Pickem
    NASCARPickem,     // NASCAR Pickem
    MLBPlus,          // ran from 2002-2009
    NFLPlus,          // ran from 2001-2009
    OTHER(String),
}
impl<'de> Deserialize<'de> for GameCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "nfl" => Ok(GameCode::NFL),
            "nflp" => Ok(GameCode::NFLPickem),
            "nfls" => Ok(GameCode::NFLSurvival),
            "nhl" => Ok(GameCode::NHL),
            "nba" => Ok(GameCode::NBA),
            "mlb" => Ok(GameCode::MLB),
            "sogb" => Ok(GameCode::EPLPickem),
            "cfb" => Ok(GameCode::NCAAF),
            "ncaaf" => Ok(GameCode::NCAAFPickem),
            "pga" => Ok(GameCode::PGAPickem),
            "nascar" => Ok(GameCode::NASCARPickem),
            "pmlb" => Ok(GameCode::MLBPlus),
            "pnfl" => Ok(GameCode::NFLPlus),
            "sode" => Ok(GameCode::BundesligaPickem),
            "soes" => Ok(GameCode::LaLigaPickem),
            "soit" => Ok(GameCode::SerieAPickem),
            "sous" => Ok(GameCode::MLSPickem),
            other => Ok(GameCode::OTHER(other.to_string())),
        }
    }
}
impl fmt::Display for GameCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameCode::NFL => write!(f, "nfl"),
            GameCode::NFLPickem => write!(f, "nflp"),
            GameCode::NHL => write!(f, "nhl"),
            GameCode::NBA => write!(f, "nba"),
            GameCode::MLB => write!(f, "mlb"),
            GameCode::EPLPickem => write!(f, "sogb"),
            GameCode::BundesligaPickem => write!(f, "sode"),
            GameCode::LaLigaPickem => write!(f, "soes"),
            GameCode::SerieAPickem => write!(f, "soit"),
            GameCode::MLSPickem => write!(f, "sous"),
            GameCode::NCAAF => write!(f, "cfb"),
            GameCode::NCAAFPickem => write!(f, "ncaaf"),
            GameCode::PGAPickem => write!(f, "pga"),
            GameCode::NASCARPickem => write!(f, "nascar"),
            GameCode::NFLSurvival => write!(f, "nfls"),
            GameCode::MLBPlus => write!(f, "pmlb"),
            GameCode::NFLPlus => write!(f, "pnfl"),
            GameCode::OTHER(str) => write!(f, "{}", str),
        }
    }
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Game {
    pub game_key: u32,
    pub game_id: u32,
    pub name: String,
    pub code: GameCode,
    #[serde(rename = "type")]
    _type: GameType,
    pub url: String,
    pub season: String,
    pub is_registration_over: Option<bool>,
    pub is_game_over: Option<bool>,
    pub is_offseason: Option<bool>,
    pub teams: Option<Teams>,
}

#[derive(Deserialize, Debug)]
#[serde(rename = "games")]
pub(crate) struct Games {
    #[serde(rename = "game", default)]
    pub games: Vec<Game>,
}

#[derive(Deserialize, Debug)]
#[serde(rename = "users")]
pub(crate) struct Users {
    #[serde(rename = "user", default)]
    pub users: Vec<User>,
}

//region User Resource
#[derive(Deserialize, Debug)]
pub struct User {
    // user_id: String,
    pub guid: String,
    pub games: Option<Games>,
    // nickname: String,
    // profile_url: String,
    // image_url: String,
    // status: String,
    // #[serde(rename = "yahoo_guid")]
    // yahoo_guid: String,
    // #[serde(rename = "yahoo_id")]
    // yahoo_id: String,
}

//endregion

#[derive(Deserialize, Debug, PartialEq)]
pub struct Teams {
    #[serde(rename = "team", default)]
    pub teams: Vec<Team>,
}

#[derive(Deserialize, Debug, PartialEq)]
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
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
struct TeamLogos {
    #[serde(rename = "team_logo", default)]
    pub team_logos: Vec<TeamLogo>,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct TeamLogo {
    pub size: String,
    pub url: String,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
struct Managers {
    #[serde(rename = "manager", default)]
    pub managers: Vec<Manager>,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
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

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FeloTier {
    Diamond,
    Platinum,
    Gold,
    Silver,
    Bronze,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct TeamPoints {
    pub coverage_type: String, // known values: season, week
    pub season: Option<String>,
    pub week: Option<i32>,
    #[serde(deserialize_with = "empty_string_as_none")]
    pub total: Option<f32>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct TeamStandings {
    #[serde(deserialize_with = "empty_string_as_none")]
    pub rank: Option<u32>,
    pub outcome_totals: OutcomeTotals,
    pub divisional_outcome_totals: Option<OutcomeTotals>,
    pub streak: Option<Streak>,
    pub points_for: f32,
    pub points_against: f32,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Roster {
    pub coverage_type: String, // date or week
    pub date: Option<String>,
    pub week: Option<String>,
    pub is_prescoring: Option<bool>,
    pub is_editable: Option<bool>,
    pub players: Players,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct OutcomeTotals {
    pub wins: u32,
    pub losses: u32,
    pub ties: u32,
    #[serde(deserialize_with = "empty_string_as_none", default)]
    pub percentage: Option<f32>,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct Streak {
    #[serde(rename = "type")]
    _type: String,
    value: String,
}

#[derive(Deserialize, Debug)]
pub struct Transactions {
    #[serde(rename = "transaction")]
    pub transactions: Vec<Transaction>,
}

#[derive(Debug)]
pub enum Transaction {
    Commish {
        transaction_key: String,
        transaction_id: u32,
        status: String,
        timestamp: i64,
    },
    Add {
        transaction_key: String,
        transaction_id: u32,
        status: String,
        timestamp: i64,
        players: Option<Vec<Player>>,
    },
    Drop {
        transaction_key: String,
        transaction_id: u32,
        status: String,
        timestamp: i64,
        players: Option<Vec<Player>>,
    },
    AddDrop {
        transaction_key: String,
        transaction_id: u32,
        status: String,
        timestamp: i64,
        players: Option<Vec<Player>>,
    },
    Trade {
        transaction_key: String,
        transaction_id: u32,
        status: String,
        timestamp: i64,
        players: Option<Vec<Player>>,
    },
    Other {
        type_name: String,
        transaction_key: String,
        transaction_id: u32,
        status: String,
        timestamp: i64,
        players: Option<Vec<Player>>,
        rest: Vec<(String, String)>,
    },
}
impl Transaction {
    pub fn key(&self) -> String {
        match self {
            Transaction::Commish {
                transaction_key, ..
            } => transaction_key.clone(),
            Transaction::Add {
                transaction_key, ..
            } => transaction_key.clone(),
            Transaction::Drop {
                transaction_key, ..
            } => transaction_key.clone(),
            Transaction::Other {
                transaction_key, ..
            } => transaction_key.clone(),
            Transaction::AddDrop {
                transaction_key, ..
            } => transaction_key.clone(),
            Transaction::Trade {
                transaction_key, ..
            } => transaction_key.clone(),
        }
    }
    pub fn timestamp(&self) -> &i64 {
        match self {
            Transaction::Commish { timestamp, .. } => timestamp,
            Transaction::Add { timestamp, .. } => timestamp,
            Transaction::Drop { timestamp, .. } => timestamp,
            Transaction::Other { timestamp, .. } => timestamp,
            Transaction::AddDrop { timestamp, .. } => timestamp,
            Transaction::Trade { timestamp, .. } => timestamp,
        }
    }
}
impl<'de> Deserialize<'de> for Transaction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TransactionVisitor;

        impl<'de> Visitor<'de> for TransactionVisitor {
            type Value = Transaction;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("Transaction struct")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Transaction, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut tx_type = None;
                let mut transaction_key = None;
                let mut transaction_id = None;
                let mut status = None;
                let mut timestamp = None;
                let mut players = None;
                let mut rest = Vec::new();

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "type" => tx_type = Some(map.next_value::<String>()?),
                        "transaction_key" => transaction_key = Some(map.next_value::<String>()?),
                        "transaction_id" => transaction_id = Some(map.next_value::<u32>()?),
                        "status" => status = Some(map.next_value::<String>()?),
                        "timestamp" => timestamp = Some(map.next_value::<i64>()?),
                        "players" => players = Some(map.next_value::<Players>()?),
                        // Collect all other fields for the "Other" variant
                        _ => {
                            let value = map.next_value::<String>()?;
                            rest.push((key, value));
                        }
                    }
                }

                let tx_type = tx_type.ok_or_else(|| de::Error::missing_field("Type"))?;
                match tx_type.as_str() {
                    "commish" => Ok(Transaction::Commish {
                        transaction_key: transaction_key
                            .ok_or_else(|| de::Error::missing_field("transaction_key"))?,
                        transaction_id: transaction_id
                            .ok_or_else(|| de::Error::missing_field("transaction_id"))?,
                        status: status.ok_or_else(|| de::Error::missing_field("status"))?,
                        timestamp: timestamp
                            .ok_or_else(|| de::Error::missing_field("timestamp"))?,
                    }),
                    "add" => Ok(Transaction::Add {
                        transaction_key: transaction_key
                            .ok_or_else(|| de::Error::missing_field("transaction_key"))?,
                        transaction_id: transaction_id
                            .ok_or_else(|| de::Error::missing_field("transaction_id"))?,
                        status: status.ok_or_else(|| de::Error::missing_field("status"))?,
                        timestamp: timestamp
                            .ok_or_else(|| de::Error::missing_field("timestamp"))?,
                        players: players.map(|players| players.players),
                    }),
                    "drop" => Ok(Transaction::Drop {
                        transaction_key: transaction_key
                            .ok_or_else(|| de::Error::missing_field("transaction_key"))?,
                        transaction_id: transaction_id
                            .ok_or_else(|| de::Error::missing_field("transaction_id"))?,
                        status: status.ok_or_else(|| de::Error::missing_field("status"))?,
                        timestamp: timestamp
                            .ok_or_else(|| de::Error::missing_field("timestamp"))?,
                        players: players.map(|players| players.players),
                    }),
                    "add/drop" => Ok(Transaction::AddDrop {
                        transaction_key: transaction_key
                            .ok_or_else(|| de::Error::missing_field("transaction_key"))?,
                        transaction_id: transaction_id
                            .ok_or_else(|| de::Error::missing_field("transaction_id"))?,
                        status: status.ok_or_else(|| de::Error::missing_field("status"))?,
                        timestamp: timestamp
                            .ok_or_else(|| de::Error::missing_field("timestamp"))?,
                        players: players.map(|players| players.players),
                    }),
                    "trade" => Ok(Transaction::Trade {
                        transaction_key: transaction_key
                            .ok_or_else(|| de::Error::missing_field("transaction_key"))?,
                        transaction_id: transaction_id
                            .ok_or_else(|| de::Error::missing_field("transaction_id"))?,
                        status: status.ok_or_else(|| de::Error::missing_field("status"))?,
                        timestamp: timestamp
                            .ok_or_else(|| de::Error::missing_field("timestamp"))?,
                        players: players.map(|players| players.players),
                    }),
                    other => Ok(Transaction::Other {
                        type_name: other.to_string(),
                        transaction_key: transaction_key
                            .ok_or_else(|| de::Error::missing_field("transaction_key"))?,
                        transaction_id: transaction_id
                            .ok_or_else(|| de::Error::missing_field("transaction_id"))?,
                        status: status.ok_or_else(|| de::Error::missing_field("status"))?,
                        timestamp: timestamp
                            .ok_or_else(|| de::Error::missing_field("timestamp"))?,
                        players: players.map(|players| players.players),
                        rest,
                    }),
                }
            }
        }

        deserializer.deserialize_map(TransactionVisitor)
    }
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Players {
    #[serde(rename = "player", default)]
    pub players: Vec<Player>,
}

#[derive(Deserialize, Debug, PartialEq)]

pub struct Player {
    pub player_key: String,
    pub player_id: u32,
    pub name: PlayerName,
    pub url: String,                      // https://sports.yahoo.com/nfl/players/7241
    pub status: Option<String>,           // NA
    pub status_full: Option<String>,      // Inactive: Coach's Decision or Not on Roster
    pub editorial_player_key: String,     // nfl.p.7241
    pub editorial_team_key: String,       // nfl.t.20
    pub editorial_team_full_name: String, // New York Jets
    pub editorial_team_abbr: String,      // NYJ
    pub editorial_team_url: String,       // https://sports.yahoo.com/nfl/teams/ny-jets/
    pub bye_weeks: Weeks,
    pub is_keeper: KeeperSummary,
    pub uniform_number: Option<String>, // can be an empty string
    pub display_position: PlayerPositions, //PlayerPosition,
    pub headshot: Headshot,
    pub image_url: String,
    pub is_undroppable: bool,
    pub position_type: PositionType,
    // pub eligible_positions:
    // pub eligible_positions_to_add:
    pub has_player_notes: Option<bool>,
    pub player_notes_last_timestamp: Option<i64>, //>1747583890</player_notes_last_timestamp>
    pub transaction_data: Option<TransactionData>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Weeks {
    #[serde(rename = "week")]
    pub weeks: Vec<String>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct KeeperSummary {
    // #[serde(default)]
    pub status: String, // Option<bool>,
    pub cost: String,   // Option<u32>,
    // #[serde(default)]
    pub kept: String, // Option<bool>,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Headshot {
    pub url: String,
    pub size: String,
}

// pub struct PlayerKey {}

#[derive(Deserialize, Debug, PartialEq)]
pub struct PlayerName {
    pub full: String,
    pub first: String,
    pub last: String,
    pub ascii_first: String,
    pub ascii_last: String,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct TransactionData {
    #[serde(rename = "type")]
    pub _type: String,
    pub source_type: String,             // known values: team, freeagents
    pub source_team_key: Option<String>, // will be present for a trade or drop
    pub source_team_name: Option<String>,
    pub destination_type: String,
}

#[derive(Debug, PartialEq)]
pub enum PlayerPositions {
    One(PlayerPosition),
    Many(Vec<PlayerPosition>),
}
impl<'de> Deserialize<'de> for PlayerPositions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PlayerPositionsVisitor;

        impl<'de> Visitor<'de> for PlayerPositionsVisitor {
            type Value = PlayerPositions;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a comma-delimited string of colors or a single color")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                println!("Value: {}", v);
                let items: Vec<PlayerPosition> = v
                    .split(',')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .map(|s| PlayerPosition::deserialize(s.into_deserializer()))
                    .collect::<Result<Vec<PlayerPosition>, _>>()?;
                match items.len() {
                    0 => Err(E::custom("no colors found")),
                    1 => Ok(PlayerPositions::One(items.first().unwrap().clone())),
                    _ => Ok(PlayerPositions::Many(items)),
                }
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut positions: Option<Result<PlayerPositions, M::Error>> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "$text" => {
                            let v: &str = map.next_value()?;

                            let items: Vec<PlayerPosition> = v
                                .split(',')
                                .map(|s| s.trim())
                                .filter(|s| !s.is_empty())
                                .map(|s| PlayerPosition::deserialize(s.into_deserializer()))
                                .collect::<Result<Vec<PlayerPosition>, _>>()?;
                            positions = Some(match items.len() {
                                0 => Err(de::Error::invalid_value(
                                    de::Unexpected::Str(""),
                                    &"comma-delimited-list",
                                )),
                                1 => Ok(PlayerPositions::One(items.first().unwrap().clone())),
                                _ => Ok(PlayerPositions::Many(items)),
                            })
                        }
                        _ => {
                            // println!("Key: {}", key);
                            let _: de::IgnoredAny = map.next_value()?;
                        }
                    }
                }
                match positions {
                    Some(positions) => positions,
                    None => Err(de::Error::invalid_value(
                        de::Unexpected::Str(""),
                        &"comma-delimited-list",
                    )),
                }
            }
        }

        deserializer.deserialize_any(PlayerPositionsVisitor)
    }
}
impl std::fmt::Display for PlayerPositions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlayerPositions::One(position) => write!(f, "{}", position),
            PlayerPositions::Many(positions) => {
                write!(
                    f,
                    "{}",
                    positions
                        .iter()
                        .map(|pos| pos.to_string())
                        .collect::<Vec<String>>()
                        .join(",")
                )
            }
        }
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum PlayerPosition {
    QB,
    RB,
    TE,
    WR,
    K,
    DEF,
}
impl std::fmt::Display for PlayerPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlayerPosition::QB => write!(f, "QB"),
            PlayerPosition::RB => write!(f, "RB"),
            PlayerPosition::TE => write!(f, "TE"),
            PlayerPosition::WR => write!(f, "WR"),
            PlayerPosition::K => write!(f, "K"),
            PlayerPosition::DEF => write!(f, "DEF"),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Hash)]
pub enum PositionType {
    K,
    DT,
    O,
    OTHER(String),
}
impl<'de> Deserialize<'de> for PositionType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "K" => Ok(PositionType::K),
            "DT" => Ok(PositionType::DT),
            "O" => Ok(PositionType::O),
            other => Ok(PositionType::OTHER(other.to_string())),
        }
    }
}
// impl std::fmt::Display for PositionType {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         match self {
//             PositionType::K => write!(f, "K"),
//             PositionType::DT => write!(f, "DT"),
//             PositionType::O => write!(f, "O"),
//             PositionType::OTHER(s) => write!(f, "{}", s),
//         }
//     }
// }

// Custom deserializer to treat empty string as None, otherwise parse as u32
fn empty_string_as_none<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: FromStr,
    T::Err: std::fmt::Display,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt.as_deref() {
        Some("") => Ok(None),
        Some(s) => T::from_str(s)
            .map(Some)
            .map_err(|e| de::Error::custom(format!("parse error: {}", e))),
        None => Ok(None),
    }
}

fn deserialize_naive_date<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    NaiveDate::parse_from_str(&s, "%Y-%m-%d").map_err(serde::de::Error::custom)
}

fn deserialize_league_key<'de, D>(deserializer: D) -> Result<LeagueKey, D::Error>
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

    Ok(LeagueKey {
        game_key: GameKey::Id(game_key),
        league_id,
    })
}

fn deserialize_player_key<'de, D>(deserializer: D) -> Result<PlayerKey, D::Error>
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

    let player_id_str = parts
        .next()
        .ok_or_else(|| de::Error::custom("missing player_id"))?;

    let player_id = player_id_str
        .parse::<u32>()
        .map_err(|e| de::Error::custom(format!("player_id parse error: {}", e)))?;

    Ok(PlayerKey {
        game_key: GameKey::Id(game_key),
        player_id,
    })
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
        league_key: LeagueKey {
            game_key: GameKey::Id(game_key),
            league_id,
        },
        team_id,
    })
}
