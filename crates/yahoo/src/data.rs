use serde::{
    Deserialize, Deserializer, Serialize,
    de::{self, IntoDeserializer, MapAccess, Visitor},
};
use std::fmt;

#[derive(Deserialize)]
#[serde(rename = "yahoo:error")]
pub struct Error {
    #[serde(rename = "description")]
    // #[serde(rename = "yahoo:description")]
    pub description: String,
    pub detail: Option<String>,
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
}

#[derive(Deserialize, Debug)]
pub struct League {
    league_key: String,
    league_id: String,
    name: String,
    url: String,
    draft_status: String,
    num_teams: u32,
    edit_key: u32,
    league_update_timestamp: u64,
    scoring_type: String,
    current_week: String,
    start_week: u32,
    end_week: u32,
    is_finished: u32,
    pub scoreboard: Option<Scoreboard>,
    pub settings: Option<Settings>,
    pub standings: Option<Standings>,
    pub transactions: Option<Transactions>,
}

#[derive(Deserialize, Debug)]
pub struct Scoreboard {
    week: u32,
    // matchups: Vec<Matchup>,
}

#[derive(Deserialize, Debug)]
pub struct Settings {
    draft_type: String,   // known values: live
    scoring_type: String, // known values: head
    uses_playoff: u8,     // 0 or 1
    playoff_start_week: u32,
    uses_playoff_reseeding: u8,     // 0 or 1
    uses_lock_eliminated_teams: u8, // 0 or 1
    uses_faab: u8,                  // 0 or 1
    trade_end_date: String,         // format: 2009-11-27
    trade_ratify_type: String,      // knwon values: commish
    trade_reject_time: u32,
    // roster_positions: RosterPositions,
    // stat_categories: StatCategories,
    divisions: Divisions,
}

#[derive(Deserialize, Debug)]
pub struct Divisions {
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
    teams: Teams,
}

// #[derive(Serialize, Deserialize)]
// pub struct Matchup {
//     #[serde(rename = "team")]
//     team: Vec<Team>,
//     #[serde(rename = "teams")]
//     teams: Vec<Team>,
//     #[serde(rename = "matchup_key")]
//     matchup_key: String,
//     #[serde(rename = "matchup_id")]
//     matchup_id: String,
//     #[serde(rename = "week")]
//     week: u32,
// }

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum GameType {
    Full,
    PickemTeam,
    PickemGroup,
    PickemTeamList,
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

pub struct LeagueKey {
    game_key: String,
    league_id: u32,
}
impl LeagueKey {
    pub fn new(game_key: &str, league_id: u32) -> Self {
        LeagueKey {
            game_key: String::from(game_key),
            league_id,
        }
    }
}
impl std::fmt::Display for LeagueKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.l.{}", self.game_key, self.league_id)
    }
}

pub struct TeamKey {
    pub league_key: LeagueKey,
    pub team_id: u32,
}
impl TeamKey {
    pub fn new(league_key: LeagueKey, team_id: u32) -> Self {
        TeamKey {
            league_key,
            team_id,
        }
    }
}
impl std::fmt::Display for TeamKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.t.{}", self.league_key, self.team_id)
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum GameCode {
    NFL,
    NHL,
    NBA,
    MLB,
    EPL,
    MLS,
    MLL,
    NCAAF,
    NCAAB,
}
impl fmt::Display for GameCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameCode::NFL => write!(f, "nfl"),
            GameCode::NHL => write!(f, "nhl"),
            GameCode::NBA => write!(f, "nba"),
            GameCode::MLB => write!(f, "mlb"),
            GameCode::EPL => write!(f, "epl"),
            GameCode::MLS => write!(f, "mls"),
            GameCode::MLL => write!(f, "mll"),
            GameCode::NCAAF => write!(f, "ncaaf"),
            GameCode::NCAAB => write!(f, "ncaab"),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Game {
    game_key: u32,
    game_id: u32,
    name: String,
    code: GameCode,
    #[serde(rename = "type")]
    _type: String,
    url: String,
    season: String,
    is_registration_over: u32,
    is_game_over: u32,
    is_offseason: u32,
    pub teams: Option<Teams>,
}

#[derive(Deserialize, Debug)]
#[serde(rename = "games")]
pub struct Games {
    #[serde(rename = "game")]
    pub games: Vec<Game>,
}

#[derive(Deserialize, Debug)]
#[serde(rename = "users")]
pub struct Users {
    #[serde(rename = "user")]
    pub users: Vec<User>,
}

//region User Resource
#[derive(Deserialize, Debug)]
pub struct User {
    // user_id: String,
    guid: String,
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

#[derive(Deserialize, Debug)]
pub struct Teams {
    #[serde(rename = "team")]
    pub teams: Vec<Team>,
}

#[derive(Deserialize, Debug)]
pub struct Team {
    team_key: String,
    team_id: String,
    name: String,
    is_owned_by_current_login: Option<u8>, // 0 or 1
    url: String,
    // team_logos: TeamLogos,
    division_id: Option<u32>,
    waiver_priority: u32,
    faab_balance: Option<u32>,
    number_of_moves: u32,
    number_of_trades: u32,
    // roster_adds: RosterAdds,
    clinched_playoffs: Option<u8>, // 0 or 1
    league_scoring_type: String,   // knwon values: head
    has_draft_grade: u8,           // 0 or 1
    draft_grade: Option<String>,
    draft_recap_url: Option<String>,
    managers: Managers,
    team_points: Option<TeamPoints>,
    team_standings: Option<TeamStandings>,
}

#[derive(Deserialize, Debug)]
pub struct Managers {
    #[serde(rename = "manager")]
    managers: Vec<Manager>,
}

#[derive(Deserialize, Debug)]
pub struct Manager {
    manager_id: u32,
    nickname: String,
    guid: String,
    is_current_login: Option<u8>, // 0 or 1
    image_url: Option<String>,
    felo_score: Option<u32>,
    felo_tier: Option<String>, // known values: gold
}

#[derive(Deserialize, Debug)]
pub struct TeamPoints {
    coverage_type: String, // knwon values: season
    season: String,
    total: f32,
}

#[derive(Deserialize, Debug)]
pub struct TeamStandings {
    rank: u32,
    outcome_totals: OutcomeTotals,
    divisional_outcome_totals: OutcomeTotals,
    streak: Streak,
    points_for: f32,
    points_against: f32,
}

#[derive(Deserialize, Debug)]
pub struct OutcomeTotals {
    wins: u32,
    losses: u32,
    ties: u32,
    percentage: Option<f32>,
}

#[derive(Deserialize, Debug)]
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

#[derive(Deserialize, Debug)]
struct Players {
    #[serde(rename = "player")]
    pub players: Vec<Player>,
}

#[derive(Deserialize, Debug)]
pub struct Player {
    pub player_key: String,
    pub player_id: u32,
    pub name: PlayerName,
    pub editorial_team_abbr: String,
    pub display_position: PlayerPositions, //PlayerPosition,
    pub position_type: PositionType,
    pub transaction_data: TransactionData,
}

// pub struct PlayerKey {}

#[derive(Deserialize, Debug)]
pub struct PlayerName {
    pub full: String,
    pub first: String,
    pub last: String,
    pub ascii_first: String,
    pub ascii_last: String,
}

#[derive(Deserialize, Debug)]
pub struct TransactionData {
    #[serde(rename = "type")]
    pub _type: String,
    pub source_type: String,             // known values: team, freeagents
    pub source_team_key: Option<String>, // will be present for a trade or drop
    pub source_team_name: Option<String>,
    pub destination_type: String,
}

#[derive(Debug)]
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

#[derive(Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum PlayerPosition {
    QB,
    RB,
    TE,
    WR,
    K,
    DEF,
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
