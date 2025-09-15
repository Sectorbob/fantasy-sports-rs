use crate::{GameCode, GameType};
use std::fmt;

/// Filters ths is not yet implemented. TODO
#[derive(Default)]
pub struct Filters {
    pub is_available: Option<bool>,
    pub game_types: Option<Vec<GameType>>,
    pub game_codes: Option<Vec<GameCode>>,
    pub seasons: Option<Vec<String>>,
}
impl Filters {
    pub fn new() -> Self {
        Filters::default()
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
}
impl fmt::Display for Filters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut filters = Vec::new();
        // if let Some(is_available) = self.is_available {
        //     filters.push(format!("is_available={}", is_available ? "1" : "0"));
        // }
        // if let Some(game_types) = &self.game_types {
        //     for game_type in game_types {
        //         filters.push(format!("game_types={game_type}"));
        //     }
        // }
        if let Some(game_codes) = &self.game_codes {
            for game_code in game_codes {
                filters.push(format!("game_codes={game_code}"));
            }
        }
        if let Some(seasons) = &self.seasons {
            for season in seasons {
                filters.push(format!("seasons={season}"));
            }
        }

        f.write_str(&filters.join(","))
    }
}
