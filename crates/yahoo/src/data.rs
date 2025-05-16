use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Token {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: String,
    pub scope: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "fantasy_content")]
pub struct FantasyContent {
    #[serde(rename = "league")]
    pub league: Option<League>,
    pub game: Option<Game>,
    #[serde(rename = "users")]
    pub users: Option<Users>,
}

#[derive(Serialize, Deserialize, Debug)]
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
    pub scoreboard: Scoreboard
}

#[derive(Serialize, Deserialize, Debug )]
pub struct Scoreboard {
    week: u32,
    // matchups: Vec<Matchup>,
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


#[derive(Serialize, Deserialize, Debug)]
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
    Id(u32)
}
impl GameKey {
    pub fn to_string(&self) -> String {
        match self {
            GameKey::Code(code) => code.to_string(),
            GameKey::Id(id) => id.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
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
impl GameCode {
    pub fn to_string(&self) -> String {
        match self {
            GameCode::NFL => "nfl".to_string(),
            GameCode::NHL => "nhl".to_string(),
            GameCode::NBA => "nba".to_string(),
            GameCode::MLB => "mlb".to_string(),
            GameCode::EPL => "epl".to_string(),
            GameCode::MLS => "mls".to_string(),
            GameCode::MLL => "mll".to_string(),
            GameCode::NCAAF => "ncaaf".to_string(),
            GameCode::NCAAB => "ncaab".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Game {
    game_key: u32,
    game_id: u32,
    name: String,
    // code: GameCode,
    #[serde(rename = "type")]
    _type: String,
    url: String,
    season: String,
    is_registration_over: u32,
    is_game_over: u32,
    is_offseason: u32,
}


#[derive(Serialize, Deserialize, Debug)]
#[serde(rename = "users")]
pub struct Users {
    #[serde(rename = "user")]
    pub users: Vec<User>,
}


//region User Resource
#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    // user_id: String,
    guid: String,
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
