use fleaflicker_fantasy_rs as fleaflicker;
use serde::{Deserialize, Serialize};
use sleeper_fantasy_rs as sleeper;
use std::{collections::HashSet, fmt, str::FromStr};
use yahoo_fantasy_rs as yahoo;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Player {
    pub _id: String,
    pub name: String,
    pub positions: Vec<PlayerPosition>,
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
impl From<fleaflicker::ProPlayer> for Player {
    fn from(value: fleaflicker::ProPlayer) -> Self {
        Player::from(&value)
    }
}
impl From<&fleaflicker::ProPlayer> for Player {
    fn from(value: &fleaflicker::ProPlayer) -> Self {
        let mut positions = value
            .position_eligibility
            .iter()
            .filter_map(|p| {
                PlayerPosition::from_str(p)
                    .inspect_err(|err| log::error!("invalid pos: {err}"))
                    .ok()
            })
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        positions.sort_by_key(|p| p.to_string());

        Player {
            _id: value.id.to_string(),
            name: value.name_full.clone(),
            positions,
            irl_team: Some(value.pro_team_abbreviation.clone()),
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
            positions: match &value.position {
                Some(position) => vec![PlayerPosition::from(position)],
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
            positions: match &value.display_position {
                yahoo::PlayerPositions::One(pos) => vec![PlayerPosition::from(pos)],
                yahoo::PlayerPositions::Many(pos) => {
                    pos.iter().map(PlayerPosition::from).collect::<Vec<_>>()
                }
            },
            irl_team: Some(value.editorial_team_abbr.to_string()),
        }
    }
}
impl fmt::Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {} {}",
            self.positions
                .iter()
                .map(PlayerPosition::to_string)
                .collect::<Vec<_>>()
                .join(","),
            self.name,
            self.irl_team.as_ref().map_or(String::new(), |t| t.clone())
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PlayerPosition {
    QB,
    RB,
    WR,
    TE,
    K,
    DEF,
    LB,
    EDR,
    DE,
    IL,
    DT,
    P,
    CB,
    S,
}
impl fmt::Display for PlayerPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            PlayerPosition::QB => "QB",
            PlayerPosition::RB => "RB",
            PlayerPosition::WR => "WR",
            PlayerPosition::TE => "TE",
            PlayerPosition::K => "K",
            PlayerPosition::DEF => "DEF",
            PlayerPosition::LB => "LB",
            PlayerPosition::EDR => "EDR",
            PlayerPosition::DE => "DE",
            PlayerPosition::IL => "IL",
            PlayerPosition::DT => "DT",
            PlayerPosition::P => "P",
            PlayerPosition::CB => "CB",
            PlayerPosition::S => "S",
        };
        write!(f, "{}", s)
    }
}
impl From<&yahoo::PlayerPosition> for PlayerPosition {
    fn from(value: &yahoo::PlayerPosition) -> Self {
        match value {
            yahoo::PlayerPosition::QB => PlayerPosition::QB,
            yahoo::PlayerPosition::RB => PlayerPosition::RB,
            yahoo::PlayerPosition::WR => PlayerPosition::WR,
            yahoo::PlayerPosition::TE => PlayerPosition::TE,
            yahoo::PlayerPosition::K => PlayerPosition::K,
            yahoo::PlayerPosition::DEF => PlayerPosition::DEF,
            // _ => Err("Unsupported Yahoo player position"),
        }
    }
}
impl From<&sleeper::Position> for PlayerPosition {
    fn from(value: &sleeper::Position) -> Self {
        match value {
            sleeper::Position::QB => PlayerPosition::QB,
            sleeper::Position::RB => PlayerPosition::RB,
            sleeper::Position::WR => PlayerPosition::WR,
            sleeper::Position::TE => PlayerPosition::TE,
            sleeper::Position::K => PlayerPosition::K,
            sleeper::Position::DEF => PlayerPosition::DEF,
            sleeper::Position::DST => PlayerPosition::DEF,
            sleeper::Position::FB => PlayerPosition::RB, // map FB to RB

            sleeper::Position::CB => PlayerPosition::CB,
            sleeper::Position::DE => PlayerPosition::DE,
            sleeper::Position::DT => PlayerPosition::DT,
            sleeper::Position::LB => PlayerPosition::LB,
            sleeper::Position::S => PlayerPosition::S,
            sleeper::Position::FS => PlayerPosition::S,
            sleeper::Position::SS => PlayerPosition::S,
            sleeper::Position::P => PlayerPosition::P,
            _ => panic!("Unsupported Sleeper position: {value:?}",),
            // sleeper::Position::DB => todo!(),
            // sleeper::Position::DL => todo!(),
            // sleeper::Position::OL => todo!(),
            // sleeper::Position::HC => todo!(),
            // sleeper::Position::T => todo!(),
            // sleeper::Position::OT => todo!(),
            // sleeper::Position::OG => todo!(),
            // sleeper::Position::G => todo!(),
            // sleeper::Position::C => todo!(),
            // sleeper::Position::LS => todo!(),
            // sleeper::Position::ILB => todo!(),
            // sleeper::Position::NT => todo!(),
            // sleeper::Position::OLB => todo!(),
            // sleeper::Position::LEO => todo!(),
            // sleeper::Position::ATH => todo!(),
            // sleeper::Position::KP => todo!(),
        }
    }
}

impl FromStr for PlayerPosition {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "QB" => Ok(PlayerPosition::QB),
            "RB" => Ok(PlayerPosition::RB),
            "WR" => Ok(PlayerPosition::WR),
            "TE" => Ok(PlayerPosition::TE),
            "K" => Ok(PlayerPosition::K),
            "DEF" => Ok(PlayerPosition::DEF),
            "LB" => Ok(PlayerPosition::LB),
            "EDR" => Ok(PlayerPosition::EDR),
            "DE" => Ok(PlayerPosition::DE),
            "IL" => Ok(PlayerPosition::IL),
            "DT" => Ok(PlayerPosition::DT),
            "P" => Ok(PlayerPosition::P),
            "CB" => Ok(PlayerPosition::CB),
            "S" => Ok(PlayerPosition::S),
            other => {
                log::error!("invalid player position: {other}");
                Err("Invalid player position")
            }
        }
    }
}
