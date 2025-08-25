use serde::{Deserialize, Deserializer, Serialize, de::Visitor};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Bracket {
    pub matchups: Vec<PlayoffBracketEntry>,
    pub is_losers: bool,
}
// FIXME: these don't properly determin the toilet bowl matchup
impl Bracket {
    pub fn champion(&self) -> Option<u8> {
        match self.championship_match() {
            Some(matchup_id) => match self.matchups.iter().find(|m| m.matchup_id == matchup_id) {
                Some(matchup) => {
                    if self.is_losers {
                        Some(matchup.loser_roster_id)
                    } else {
                        Some(matchup.winner_roster_id)
                    }
                }
                None => None,
            },
            None => None,
        }
    }
    pub fn runner_up(&self) -> Option<u8> {
        match self.championship_match() {
            Some(matchup_id) => match self.matchups.iter().find(|m| m.matchup_id == matchup_id) {
                Some(matchup) => {
                    if self.is_losers {
                        Some(matchup.winner_roster_id)
                    } else {
                        Some(matchup.loser_roster_id)
                    }
                }
                None => None,
            },
            None => None,
        }
    }
    pub fn championship_match(&self) -> Option<u8> {
        let last_round_matches: Option<Vec<&PlayoffBracketEntry>> =
            self.matchups.iter().max_by_key(|e| e.round).map(|e| {
                self.matchups
                    .iter()
                    .filter(|f| {
                        if f.round != e.round {
                            return false;
                        }
                        if self.is_losers {
                            if let Some(UnresolvedPlayoffSlot::WinnerOfMatch(_)) = f.team_1_from {
                                return false;
                            }
                            if let Some(UnresolvedPlayoffSlot::WinnerOfMatch(_)) = f.team_2_from {
                                return false;
                            }
                        } else {
                            // winners bracket
                            if let Some(UnresolvedPlayoffSlot::LoserOfMatch(_)) = f.team_1_from {
                                return false;
                            }
                            if let Some(UnresolvedPlayoffSlot::LoserOfMatch(_)) = f.team_2_from {
                                return false;
                            }
                        }

                        true
                    })
                    .collect()
            });
        // println!("LRM: {:#?}", last_round_matches);
        match last_round_matches {
            Some(m) => match m.len() {
                0 => None,
                1 => Some(m[0].matchup_id),
                _ => None,
            },
            None => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct PlayoffBracketEntry {
    #[serde(rename = "r")]
    pub round: u8,
    #[serde(rename = "m")]
    pub matchup_id: u8,
    #[serde(rename = "t1")]
    pub team_1_roster_id: Option<u8>, //TODO: I'd love to combine this with t1_from in an enum
    #[serde(rename = "t2")]
    pub team_2_roster_id: Option<u8>, //TODO: I'd love to combine this with t2_from in an enum
    #[serde(rename = "w")]
    pub winner_roster_id: u8,
    #[serde(rename = "l")]
    pub loser_roster_id: u8,
    #[serde(rename = "t1_from")]
    pub team_1_from: Option<UnresolvedPlayoffSlot>,
    #[serde(rename = "t2_from")]
    pub team_2_from: Option<UnresolvedPlayoffSlot>,
}
impl PlayoffBracketEntry {
    pub fn team1(&self) -> BracketTeam {
        if let Some(team_1) = self.team_1_roster_id {
            team_1.into()
        } else {
            self.team_1_from.clone().unwrap().into()
        }
    }
    pub fn team2(&self) -> BracketTeam {
        if let Some(team_2) = self.team_2_roster_id {
            team_2.into()
        } else {
            self.team_2_from.clone().unwrap().into()
        }
    }
}
impl<'de> serde::Deserialize<'de> for UnresolvedPlayoffSlot {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct OuterVisitor;

        impl<'de> Visitor<'de> for OuterVisitor {
            type Value = UnresolvedPlayoffSlot;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a nonempty sequence of a sequence of numbers")
            }

            #[inline]
            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let map_value: serde_json::Map<String, serde_json::Value> =
                    serde::de::Deserialize::deserialize(
                        serde::de::value::MapAccessDeserializer::new(map),
                    )?;

                if map_value.contains_key("w") {
                    // Deserialize into VariantA
                    Ok(UnresolvedPlayoffSlot::WinnerOfMatch(
                        serde_json::from_value(map_value["w"].clone())
                            .map_err(serde::de::Error::custom)?,
                    ))
                } else if map_value.contains_key("l") {
                    // Deserialize into VariantB
                    Ok(UnresolvedPlayoffSlot::LoserOfMatch(
                        serde_json::from_value(map_value["l"].clone())
                            .map_err(serde::de::Error::custom)?,
                    ))
                } else {
                    // Default to VariantC
                    Err(serde::de::Error::missing_field("\"w\" or \"l\""))
                }
            }
        }
        deserializer.deserialize_map(OuterVisitor)
    }
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub enum UnresolvedPlayoffSlot {
    LoserOfMatch(u8),
    WinnerOfMatch(u8),
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub enum BracketTeam {
    Roster(u8),
    LoserOfMatch(u8),
    WinnerOfMatch(u8),
}
impl From<u8> for BracketTeam {
    fn from(value: u8) -> Self {
        BracketTeam::Roster(value)
    }
}
impl From<UnresolvedPlayoffSlot> for BracketTeam {
    fn from(value: UnresolvedPlayoffSlot) -> Self {
        match value {
            UnresolvedPlayoffSlot::LoserOfMatch(v) => BracketTeam::LoserOfMatch(v),
            UnresolvedPlayoffSlot::WinnerOfMatch(v) => BracketTeam::WinnerOfMatch(v),
        }
    }
}

#[cfg(test)]
mod data_tests {
    use crate::data::{BracketTeam, PlayoffBracketEntry};

    #[test]
    fn test_playoff_bracket_deserialization() {
        let bracket_json = r#"[{"m":1,"r":1,"l":12,"w":7,"t1":7,"t2":12},{"m":2,"r":1,"l":5,"w":9,"t1":5,"t2":9},{"m":3,"r":2,"l":4,"w":7,"t1":4,"t2":7,"t2_from":{"w":1}},{"m":4,"r":2,"l":1,"w":9,"t1":1,"t2":9,"t2_from":{"w":2}},{"p":5,"m":5,"r":2,"l":12,"w":5,"t1":12,"t2":5,"t2_from":{"l":2},"t1_from":{"l":1}},{"p":1,"m":6,"r":3,"l":9,"w":7,"t1":7,"t2":9,"t2_from":{"w":4},"t1_from":{"w":3}},{"p":3,"m":7,"r":3,"l":1,"w":4,"t1":4,"t2":1,"t2_from":{"l":4},"t1_from":{"l":3}}]"#;
        let bracket: Vec<PlayoffBracketEntry> =
            serde_json::from_str(bracket_json).expect("deserialize did not pass");
        assert_eq!(7, bracket.len());
        println!("{:#?}", bracket);
        assert_eq!(BracketTeam::Roster(7), bracket[0].team1());
        assert_eq!(BracketTeam::Roster(12), bracket[0].team2());
        assert_eq!(1, bracket[0].round);
        assert_eq!(1, bracket[0].matchup_id);

        assert_eq!(BracketTeam::Roster(5), bracket[1].team1());
        assert_eq!(BracketTeam::Roster(9), bracket[1].team2());
        assert_eq!(1, bracket[1].round);
        assert_eq!(2, bracket[1].matchup_id);

        assert_eq!(BracketTeam::Roster(4), bracket[6].team1());
        assert_eq!(BracketTeam::Roster(1), bracket[6].team2());
        assert_eq!(3, bracket[6].round);
        assert_eq!(7, bracket[6].matchup_id);
    }
}
