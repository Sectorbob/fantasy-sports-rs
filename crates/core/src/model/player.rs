use serde::{Deserialize, Serialize};
use sleeper_fantasy_rs as sleeper;
use std::fmt;
use yahoo_fantasy_rs as yahoo;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Player {
    pub _id: String,
    pub name: String,
    pub positions: Vec<String>,
    pub irl_team: Option<String>,
}
impl Player {
    pub fn missing(id: &String) -> Self {
        Player {
            _id: id.clone(),
            name: format!("Missing Player ({})", id),
            positions: vec![],
            irl_team: None,
        }
    }
}
impl From<sleeper::Player> for Player {
    fn from(value: sleeper::Player) -> Self {
        Player::from(&value)
    }
}
impl From<&sleeper::Player> for Player {
    fn from(value: &sleeper::Player) -> Self {
        Player {
            _id: value.player_id.clone(),
            name: value.to_string(),
            positions: match value.position.clone() {
                Some(position) => vec![position.clone()],
                None => vec![],
            },
            irl_team: value.team.clone(),
        }
    }
}
impl From<yahoo::Player> for Player {
    fn from(value: yahoo::Player) -> Self {
        Player::from(&value)
    }
}
impl From<&yahoo::Player> for Player {
    fn from(value: &yahoo::Player) -> Self {
        Player {
            _id: value.player_key.to_string(),
            name: value.name.full.to_string(),
            positions: vec![value.display_position.to_string()],
            irl_team: Some(value.editorial_team_abbr.to_string()),
        }
    }
}
impl fmt::Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {} {}",
            self.positions.join(","),
            self.name,
            self.irl_team.clone().unwrap_or("".to_string())
        )
    }
}
