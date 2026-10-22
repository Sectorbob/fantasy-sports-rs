use serde::{Deserialize, Deserializer, Serialize, de::Visitor};
use std::{collections::HashSet, fmt};

#[derive(Debug, Clone, PartialEq)]
pub struct Bracket {
    pub matchups: Vec<PlayoffBracketEntry>,
    pub is_losers: bool,
}
// FIXME: these don't properly determin the toilet bowl matchup
impl Bracket {
    pub fn rosters(&self) -> Vec<u8> {
        let set = self
            .matchups
            .iter()
            .flat_map(|b| {
                let mut v = vec![];
                if let Some(rid) = b.team_1_roster_id {
                    v.push(rid);
                }
                if let Some(rid) = b.team_2_roster_id {
                    v.push(rid);
                }
                v
            })
            .collect::<HashSet<_>>();
        let mut v = set.into_iter().collect::<Vec<_>>();
        v.sort();
        v
    }
    pub fn champion(&self) -> Option<u8> {
        match self.championship_match() {
            Some(m) => match self
                .matchups
                .iter()
                .find(|m2| m.matchup_id == m2.matchup_id)
            {
                Some(matchup) => {
                    if self.is_losers {
                        matchup.loser_roster_id
                    } else {
                        matchup.winner_roster_id
                    }
                }
                None => None,
            },
            None => None,
        }
    }
    pub fn runner_up(&self) -> Option<u8> {
        match self.championship_match() {
            Some(m2) => match self.matchups.iter().find(|m| m.matchup_id == m2.matchup_id) {
                Some(matchup) => {
                    if self.is_losers {
                        matchup.winner_roster_id
                    } else {
                        matchup.loser_roster_id
                    }
                }
                None => None,
            },
            None => None,
        }
    }
    pub fn third_place(&self) -> Option<u8> {
        match self.third_place_match() {
            Some(m2) => match self.matchups.iter().find(|m| m.matchup_id == m2.matchup_id) {
                Some(matchup) => {
                    if self.is_losers {
                        matchup.loser_roster_id
                    } else {
                        matchup.winner_roster_id
                    }
                }
                None => None,
            },
            None => None,
        }
    }
    pub fn fourth_place(&self) -> Option<u8> {
        match self.third_place_match() {
            Some(m2) => match self.matchups.iter().find(|m| m.matchup_id == m2.matchup_id) {
                Some(matchup) => {
                    if self.is_losers {
                        matchup.winner_roster_id
                    } else {
                        matchup.loser_roster_id
                    }
                }
                None => None,
            },
            None => None,
        }
    }
    pub fn rosters_with_byes(&self) -> Vec<u8> {
        let mut rosters = self.rosters();
        for m in self.matchups.iter() {
            if m.team_1_from.is_none() && m.team_2_from.is_none() {
                if let Some(rid) = m.team_1_roster_id
                    && let Some(i) = rosters.iter().position(|i| *i == rid)
                {
                    rosters.remove(i);
                }
                if let Some(rid) = m.team_2_roster_id
                    && let Some(i) = rosters.iter().position(|i| *i == rid)
                {
                    rosters.remove(i);
                }
            }
        }
        rosters
    }
    pub fn championship_match(&self) -> Option<&PlayoffBracketEntry> {
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
                1 => Some(m[0]),
                _ => None,
            },
            None => None,
        }
    }
    pub fn third_place_match(&self) -> Option<&PlayoffBracketEntry> {
        let championship_match = self.championship_match()?;
        let semi_final_match_1 = championship_match.team_1_from.as_ref()?.source_matchup_id();
        let semi_final_match_2 = championship_match.team_2_from.as_ref()?.source_matchup_id();
        let semi_final_match_ids = [semi_final_match_1, semi_final_match_2];
        let third_place_match_round = championship_match.round;
        self.matchups.iter().find(|m| {
            if let Some(s) = &m.team_1_from
                && !semi_final_match_ids.contains(&s.source_matchup_id())
            {
                return false;
            }
            if let Some(s) = &m.team_2_from
                && !semi_final_match_ids.contains(&s.source_matchup_id())
            {
                return false;
            }
            m.round == third_place_match_round && m.matchup_id != championship_match.matchup_id
        })
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
    pub winner_roster_id: Option<u8>,
    #[serde(rename = "l")]
    pub loser_roster_id: Option<u8>,
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
impl UnresolvedPlayoffSlot {
    pub fn source_matchup_id(&self) -> u8 {
        match self {
            UnresolvedPlayoffSlot::LoserOfMatch(id) => *id,
            UnresolvedPlayoffSlot::WinnerOfMatch(id) => *id,
        }
    }
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
mod tests {
    use crate::{Bracket, Roster, User, data::BracketTeam};
    use std::fs;

    const LEAGUE_ID_2019: &str = "164974877262287888";

    fn load_test_winners_bracket(league_id: &str) -> Bracket {
        let json = fs::read_to_string(format!(
            "testdata/v1/league/{league_id}/winners_bracket.json"
        ))
        .expect("failed to load test data file");

        Bracket {
            matchups: serde_json::from_str(json.as_str())
                .expect("failed to deserialize playoff bracket json"),
            is_losers: false,
        }
    }

    fn load_rosters_in_league(league_id: &str) -> Vec<Roster> {
        let json = fs::read_to_string(format!("testdata/v1/league/{league_id}/rosters.json"))
            .expect("failed to load test data file");

        serde_json::from_str(json.as_str()).expect("failed to deserialize league rosters json")
    }

    fn load_users_in_league(league_id: &str) -> Vec<User> {
        let json = fs::read_to_string(format!("testdata/v1/league/{league_id}/users.json"))
            .expect("failed to load test data file");

        serde_json::from_str(json.as_str()).expect("failed to deserialize league users json")
    }

    fn team_name_for_roster(
        roster_id: u8,
        rosters: &Vec<Roster>,
        users: &Vec<User>,
    ) -> Option<String> {
        let owner_id = rosters
            .iter()
            .find(|r| r.roster_id == roster_id)
            .and_then(|r| r.owner_id.as_ref())?;
        users
            .iter()
            .find(|user| &user.user_id == owner_id)
            .map(User::team_name)
    }

    #[test]
    fn test_playoff_bracket_deserialization() {
        let bracket = load_test_winners_bracket(LEAGUE_ID_2019).matchups;
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

    #[test]
    fn test_playoff_rosters() {
        let bracket = load_test_winners_bracket(LEAGUE_ID_2019);
        let rosters = load_rosters_in_league(LEAGUE_ID_2019);
        let users = load_users_in_league(LEAGUE_ID_2019);
        let expected_team_names: Vec<String> = [
            "Stormy Badgers 01",
            "Team ManagerStormy10",
            "Team ManagerBrave17",
            "Team ManagerGolden06",
            "Team ManagerMighty58",
            "Team ManagerBrave24",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
        let actual = bracket.rosters();
        for roster_id in actual {
            let team_name = team_name_for_roster(roster_id, &rosters, &users)
                .expect("can't find team name for {roster_id}");
            debug_assert!(
                expected_team_names.contains(&team_name),
                "unexpected team in winners bracket: {team_name}",
            );
        }
    }

    #[test]
    fn test_playoff_rosters_with_bye() {
        let bracket = load_test_winners_bracket(LEAGUE_ID_2019);
        let rosters = load_rosters_in_league(LEAGUE_ID_2019);
        let users = load_users_in_league(LEAGUE_ID_2019);
        let expected_team_names: Vec<String> = ["Stormy Badgers 01", "Team ManagerStormy10"]
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        let actual = bracket.rosters_with_byes();
        for roster_id_with_bye in actual {
            let team_name = team_name_for_roster(roster_id_with_bye, &rosters, &users)
                .expect("can't find team name for {roster_id_with_bye}");
            debug_assert!(
                expected_team_names.contains(&team_name),
                "unexpected team with bye: {team_name}",
            );
        }
    }

    #[test]
    fn test_playoff_champion() {
        let bracket = load_test_winners_bracket(LEAGUE_ID_2019);
        let rosters = load_rosters_in_league(LEAGUE_ID_2019);
        let users = load_users_in_league(LEAGUE_ID_2019);
        let actual = bracket.champion().expect("unable to find champion");
        let team_name = team_name_for_roster(actual, &rosters, &users)
            .expect("can't find team name for {actual}");
        debug_assert_eq!(
            "Team ManagerGolden06", team_name,
            "unexpected team with bye: {team_name}",
        );
    }

    #[test]
    fn test_playoff_runner_up() {
        let bracket = load_test_winners_bracket(LEAGUE_ID_2019);
        let rosters = load_rosters_in_league(LEAGUE_ID_2019);
        let users = load_users_in_league(LEAGUE_ID_2019);
        let actual = bracket.runner_up().expect("unable to find runner-up");
        let team_name = team_name_for_roster(actual, &rosters, &users)
            .expect("can't find team name for {actual}");
        debug_assert_eq!(
            "Team ManagerMighty58", team_name,
            "unexpected team with bye: {team_name}",
        );
    }

    #[test]
    fn test_playoff_third_place() {
        let bracket = load_test_winners_bracket(LEAGUE_ID_2019);
        let rosters = load_rosters_in_league(LEAGUE_ID_2019);
        let users = load_users_in_league(LEAGUE_ID_2019);
        let actual = bracket
            .third_place()
            .expect("unable to find third place team");
        let team_name = team_name_for_roster(actual, &rosters, &users)
            .expect("can't find team name for {actual}");
        debug_assert_eq!(
            "Team ManagerStormy10", team_name,
            "unexpected team with bye: {team_name}",
        );
    }

    #[test]
    fn test_playoff_fourth_place() {
        let bracket = load_test_winners_bracket(LEAGUE_ID_2019);
        let rosters = load_rosters_in_league(LEAGUE_ID_2019);
        let users = load_users_in_league(LEAGUE_ID_2019);
        let actual = bracket
            .fourth_place()
            .expect("unable to find fourth place team");
        let team_name = team_name_for_roster(actual, &rosters, &users)
            .expect("can't find team name for {actual}");
        debug_assert_eq!(
            "Stormy Badgers 01", team_name,
            "unexpected team with bye: {team_name}",
        );
    }

    #[test]
    fn test_playoff_get_third_place_matchup() {
        let bracket = load_test_winners_bracket(LEAGUE_ID_2019);
        let rosters = load_rosters_in_league(LEAGUE_ID_2019);
        let users = load_users_in_league(LEAGUE_ID_2019);
        let actual = bracket
            .third_place_match()
            .expect("unable to find third place match");
        let roster_1 = actual.team_1_roster_id.expect("roster 1 is not resolved");
        let roster_2 = actual.team_2_roster_id.expect("roster 2 is not resolved");
        debug_assert_eq!(
            "Team ManagerStormy10",
            team_name_for_roster(roster_1, &rosters, &users).unwrap(),
            "unexpected team 1",
        );
        debug_assert_eq!(
            "Stormy Badgers 01",
            team_name_for_roster(roster_2, &rosters, &users).unwrap(),
            "unexpected team 2",
        );
    }
}
