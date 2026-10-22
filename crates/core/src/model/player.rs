use fantasy_sports_fleaflicker as fleaflicker;
use fantasy_sports_sleeper as sleeper;
use fantasy_sports_yahoo as yahoo;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fmt, str::FromStr};

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
    /// offensive line: tackles, guards and centers
    OL,
    /// defensive backs that are not specifically a cornerback or safety
    DB,
    /// defensive line that is not specifically an end or tackle
    DL,
    /// anything that has no fantasy position of its own (coaches, athletes, long snappers, ...)
    Other,
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
            PlayerPosition::OL => "OL",
            PlayerPosition::DB => "DB",
            PlayerPosition::DL => "DL",
            PlayerPosition::Other => "Other",
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
            sleeper::Position::KP => PlayerPosition::K,
            sleeper::Position::TEAM => PlayerPosition::DEF,

            sleeper::Position::CB => PlayerPosition::CB,
            sleeper::Position::DE => PlayerPosition::DE,
            sleeper::Position::DT | sleeper::Position::NT => PlayerPosition::DT,
            sleeper::Position::LB | sleeper::Position::ILB | sleeper::Position::OLB => {
                PlayerPosition::LB
            }
            sleeper::Position::LEO => PlayerPosition::EDR, // a hybrid end / outside linebacker
            sleeper::Position::S | sleeper::Position::FS | sleeper::Position::SS => {
                PlayerPosition::S
            }
            sleeper::Position::P => PlayerPosition::P,
            sleeper::Position::DB => PlayerPosition::DB,
            sleeper::Position::DL => PlayerPosition::DL,
            sleeper::Position::OL
            | sleeper::Position::T
            | sleeper::Position::OT
            | sleeper::Position::OG
            | sleeper::Position::G
            | sleeper::Position::C => PlayerPosition::OL,
            sleeper::Position::HC | sleeper::Position::LS | sleeper::Position::ATH => {
                PlayerPosition::Other
            }
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
            "OL" => Ok(PlayerPosition::OL),
            "DB" => Ok(PlayerPosition::DB),
            "DL" => Ok(PlayerPosition::DL),
            "Other" => Ok(PlayerPosition::Other),
            other => {
                log::error!("invalid player position: {other}");
                Err("Invalid player position")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    /// every position name Sleeper uses, as it appears on the wire
    const SLEEPER_POSITIONS: [&str; 33] = [
        "QB", "RB", "WR", "TE", "K", "DEF", "DST", "LB", "DB", "DL", "OL", "P", "HC", "CB", "S",
        "FS", "SS", "DE", "DT", "T", "OT", "OG", "G", "C", "LS", "ILB", "NT", "FB", "OLB", "LEO",
        "ATH", "K/P", "TEAM",
    ];

    const ALL_POSITIONS: [PlayerPosition; 18] = [
        PlayerPosition::QB,
        PlayerPosition::RB,
        PlayerPosition::WR,
        PlayerPosition::TE,
        PlayerPosition::K,
        PlayerPosition::DEF,
        PlayerPosition::LB,
        PlayerPosition::EDR,
        PlayerPosition::DE,
        PlayerPosition::IL,
        PlayerPosition::DT,
        PlayerPosition::P,
        PlayerPosition::CB,
        PlayerPosition::S,
        PlayerPosition::OL,
        PlayerPosition::DB,
        PlayerPosition::DL,
        PlayerPosition::Other,
    ];

    fn sleeper_position(name: &str) -> sleeper::Position {
        serde_json::from_str(&format!("\"{name}\"")).expect("a Sleeper position")
    }

    #[test]
    fn every_sleeper_position_converts_instead_of_panicking() {
        for name in SLEEPER_POSITIONS {
            let _ = PlayerPosition::from(&sleeper_position(name));
        }
    }

    #[test]
    fn related_sleeper_positions_share_a_core_position() {
        let to = |name: &str| PlayerPosition::from(&sleeper_position(name));
        assert_eq!(PlayerPosition::DEF, to("DST"));
        assert_eq!(PlayerPosition::DEF, to("TEAM"));
        assert_eq!(PlayerPosition::RB, to("FB"));
        assert_eq!(PlayerPosition::K, to("K/P"));
        for name in ["T", "OT", "OG", "G", "C", "OL"] {
            assert_eq!(PlayerPosition::OL, to(name), "{name}");
        }
        for name in ["FS", "SS", "S"] {
            assert_eq!(PlayerPosition::S, to(name), "{name}");
        }
        for name in ["ILB", "OLB", "LB"] {
            assert_eq!(PlayerPosition::LB, to(name), "{name}");
        }
        assert_eq!(PlayerPosition::Other, to("HC"));
    }

    #[test]
    fn the_whole_recorded_sleeper_player_universe_converts() {
        // this used to panic on the first offensive lineman
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../sleeper/testdata/players_nfl.json.gz"
        );
        let mut json = String::new();
        flate2::read::GzDecoder::new(std::fs::File::open(path).unwrap())
            .read_to_string(&mut json)
            .unwrap();
        let players: std::collections::HashMap<String, sleeper::Player> =
            serde_json::from_str(&json).unwrap();
        assert!(players.len() > 10_000);

        let converted: Vec<Player> = players.values().map(Player::from).collect();

        assert_eq!(players.len(), converted.len());
        assert!(
            converted
                .iter()
                .any(|p| p.positions == [PlayerPosition::OL])
        );
    }

    #[test]
    fn positions_round_trip_through_their_text_form() {
        for position in ALL_POSITIONS {
            assert_eq!(
                Ok(position.clone()),
                position.to_string().parse(),
                "{position}"
            );
        }
        assert!("QUARTERBACK".parse::<PlayerPosition>().is_err());
        assert!(
            "qb".parse::<PlayerPosition>().is_err(),
            "positions are case sensitive"
        );
    }

    #[test]
    fn yahoo_positions_map_one_to_one() {
        for (yahoo_position, expected) in [
            (yahoo::PlayerPosition::QB, PlayerPosition::QB),
            (yahoo::PlayerPosition::RB, PlayerPosition::RB),
            (yahoo::PlayerPosition::WR, PlayerPosition::WR),
            (yahoo::PlayerPosition::TE, PlayerPosition::TE),
            (yahoo::PlayerPosition::K, PlayerPosition::K),
            (yahoo::PlayerPosition::DEF, PlayerPosition::DEF),
        ] {
            assert_eq!(expected, PlayerPosition::from(&yahoo_position));
        }
    }

    #[test]
    fn a_missing_player_says_so_and_displays_without_a_team() {
        let player = Player::missing(&"42".to_string());
        assert_eq!("42", player._id);
        assert_eq!("Missing Player (42)", player.name);
        assert_eq!("[] Missing Player (42) ", player.to_string());
    }

    #[test]
    fn a_player_displays_positions_name_and_team() {
        let player = Player {
            _id: "1".into(),
            name: "Casey Catcher".into(),
            positions: vec![PlayerPosition::WR, PlayerPosition::TE],
            irl_team: Some("EXR".into()),
        };
        assert_eq!("[WR,TE] Casey Catcher EXR", player.to_string());
    }

    #[test]
    fn a_fleaflicker_player_gets_sorted_unique_positions() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../fleaflicker/testdata/league_rosters.json"
        );
        let response: fleaflicker::FetchRostersResponse =
            serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let mut pro_player = response.rosters[0].players[0].pro_player.clone();
        pro_player.position_eligibility = vec!["WR".into(), "TE".into(), "WR".into(), "??".into()];

        let player = Player::from(&pro_player);

        assert_eq!(pro_player.id.to_string(), player._id);
        assert_eq!(pro_player.name_full, player.name);
        assert_eq!(
            Some(pro_player.pro_team_abbreviation.clone()),
            player.irl_team
        );
        // duplicates collapse, unknown positions are dropped, the rest is in name order
        assert_eq!(
            vec![PlayerPosition::TE, PlayerPosition::WR],
            player.positions
        );
    }
}
