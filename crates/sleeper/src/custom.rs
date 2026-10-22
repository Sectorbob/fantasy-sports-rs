use crate::{
    Client, PlayoffBracketEntry,
    data::{LeagueSettings, Matchup, Player, Roster, User},
};
use fantasy_sports_cache::Error;
use rand::{rng, seq::SliceRandom as _};
use std::collections::{HashMap, HashSet};

/// I want to place my own custom consumer code here
///
///
#[derive(Debug, Clone)]
pub struct FantasyMatchup {
    pub week: usize,
    pub left: FantasyMatchupSide,
    pub right: FantasyMatchupSide,
}
impl FantasyMatchup {
    pub fn from_stuff(
        week: usize,
        matchups: &Vec<Matchup>,
        rosters: &[Roster],
        owners: &[User],
    ) -> Result<Vec<FantasyMatchup>, Box<dyn std::error::Error + Send + Sync>> {
        let mut games: Vec<(Matchup, Matchup)> = Vec::with_capacity(matchups.len() / 2);
        let mut wip: Vec<Matchup> = vec![];
        let mut orphaned: Vec<Matchup> = vec![];
        for matchup in matchups {
            if let Some(index) = wip
                .iter()
                .position(|wip_entry| matchup.matchup_id == wip_entry.matchup_id)
            {
                let tmp = wip.remove(index);
                games.push((tmp, matchup.clone()));
            } else if matchup.matchup_id.is_none() {
                orphaned.push(matchup.clone());
            } else {
                wip.push(matchup.clone());
            }
        }
        if !wip.is_empty() {
            return Err(Box::new(std::io::Error::other(format!(
                "wip was not empty: {}",
                wip.len()
            ))));
        }
        if !orphaned.is_empty() {
            // info!("had {} orphaned statlines", orphaned.len());
        }

        let mut new_fantasy_matchups: Vec<FantasyMatchup> = vec![];

        for game in games {
            let roster_0 =
                match rosters.binary_search_by_key(&game.0.roster_id, |roster| roster.roster_id) {
                    Ok(index) => rosters[index].clone(),
                    Err(_) => {
                        return Err(Box::new(std::io::Error::other(format!(
                            "invalid roster {}",
                            game.0.roster_id
                        ))));
                    }
                };
            let roster_1 =
                match rosters.binary_search_by_key(&game.1.roster_id, |roster| roster.roster_id) {
                    Ok(index) => rosters[index].clone(),
                    Err(_) => {
                        return Err(Box::new(std::io::Error::other(format!(
                            "invalid roster {}",
                            game.1.roster_id
                        ))));
                    }
                };
            let owner_0 = if let Some(owner_id) = &roster_0.owner_id {
                match owners.binary_search_by_key(&owner_id, |user| &user.user_id) {
                    Ok(index) => owners[index].clone(),
                    Err(_) => {
                        return Err(Box::new(std::io::Error::other(format!(
                            "invalid owner '{}'",
                            owner_id
                        ))));
                    }
                }
            } else {
                return Err(Box::new(std::io::Error::other(format!(
                    "roster {} does not have an owner",
                    roster_0.roster_id
                ))));
            };
            let owner_1 = if let Some(owner_id) = &roster_1.owner_id {
                match owners.binary_search_by_key(&owner_id, |user| &user.user_id) {
                    Ok(index) => owners[index].clone(),
                    Err(_) => {
                        return Err(Box::new(std::io::Error::other(format!(
                            "invalid owner '{}'",
                            owner_id
                        ))));
                    }
                }
            } else {
                return Err(Box::new(std::io::Error::other(format!(
                    "roster {} does not have an owner",
                    roster_1.roster_id
                ))));
            };

            let players_0 = game.0.players.clone();
            let players_1 = game.1.players.clone();
            let starters_0 = game.0.starters.clone();
            let starters_1 = game.1.starters.clone();

            let f = FantasyMatchup {
                week,
                left: FantasyMatchupSide {
                    matchup: game.0,
                    roster: roster_0,
                    owner: owner_0,
                    starters: starters_0.clone(),
                    bench: players_0
                        .into_iter()
                        .filter(|p| starters_0.contains(p))
                        .collect(),
                    ir: vec![],
                    // players: players,
                },
                right: FantasyMatchupSide {
                    matchup: game.1,
                    roster: roster_1,
                    owner: owner_1,
                    starters: starters_1.clone(),
                    bench: players_1
                        .into_iter()
                        .filter(|p| starters_1.contains(p))
                        .collect(),
                    ir: vec![],
                    // players: players,
                },
            };

            new_fantasy_matchups.push(f);
        }

        Ok(new_fantasy_matchups)
    }

    pub fn printy(&self, width: usize, players: &HashMap<String, Player>) {
        // let delim = String::from(" ").repeat(if width % 2 == 0 { 2 } else { 1 });
        let col_width = width / 2;

        println!(
            "{:<col_width$}vs{:>col_width$}",
            self.left.owner.display_name,
            self.right.owner.display_name,
            col_width = col_width,
        );
        println!(
            "{:<col_width$}TP{:>col_width$}",
            self.left.matchup.points,
            self.right.matchup.points,
            col_width = col_width,
        );
        for i in 0..self.left.matchup.starters.len() {
            let left_player = players.get(&self.left.matchup.starters[i]).unwrap();
            let right_player = players.get(&self.right.matchup.starters[i]).unwrap();

            let left_max_player_points_width = self.left.max_player_score_width();
            let left_player_name_width = col_width - left_max_player_points_width;
            let right_max_player_points_width = self.right.max_player_score_width();
            let right_player_name_width = col_width - right_max_player_points_width;

            let left = format!(
                "{:<left_player_name_width$}{:>left_max_player_points_width$}",
                left_player.to_string().as_str(),
                self.left.matchup.starters_points[i].to_string().as_str(),
                left_player_name_width = left_player_name_width,
                left_max_player_points_width = left_max_player_points_width,
            );
            let right = format!(
                "{:<right_max_player_points_width$}{:>right_player_name_width$}",
                self.right.matchup.starters_points[i].to_string().as_str(),
                right_player.to_string().as_str(),
                right_max_player_points_width = right_max_player_points_width
            );

            println!("{}  {}", left, right);
        }
    }
}

#[derive(Debug, Clone)]
pub struct FantasyMatchupSide {
    pub matchup: Matchup,
    pub roster: Roster,
    pub starters: Vec<String>,
    pub bench: Vec<String>,
    pub ir: Vec<String>,
    pub owner: User,
}
impl FantasyMatchupSide {
    pub fn players(&self) -> Vec<String> {
        let mut players = self.starters.clone();
        players.append(&mut self.bench.clone());
        players.append(&mut self.ir.clone());
        players
    }
    pub fn max_player_score_width(&self) -> usize {
        self.matchup
            .starters_points
            .iter()
            .map(|p| p.to_string())
            .max_by_key(|s| s.len())
            .unwrap_or_default()
            .len()
    }
}

pub struct Standings {
    league_settings: LeagueSettings,
    winners_bracket: Vec<PlayoffBracketEntry>,
    #[allow(dead_code)] // reserved for ordering the toilet bowl, see the FIXME on `impl Bracket`
    losers_bracket: Option<Vec<PlayoffBracketEntry>>,
}
impl Standings {
    pub fn new(
        league_settings: LeagueSettings,
        winners_bracket: Vec<PlayoffBracketEntry>,
        losers_bracket: Option<Vec<PlayoffBracketEntry>>,
    ) -> Self {
        Standings {
            league_settings,
            winners_bracket,
            losers_bracket,
        }
    }
    pub async fn from_sleeper<T: Into<String>>(league_id: T) -> Result<Self, Error> {
        let client = Client::new();
        let league_id: String = league_id.into();
        let league_future = client.get_league(league_id.as_str());
        // let rosters_future = client.get_rosters_for_league(league_id.as_str());
        // let users_future = client
        // .get_users_for_league(league_id.as_str());
        // let roster_map = rosters_future.await?
        //     .into_iter()
        //     .map(|r| (r.roster_id, r))
        //     .collect::<HashMap<u8, Roster>>();
        // let user_map = users_future.await?
        //     .into_iter()
        //     .map(|u| (u.user_id.clone(), u))
        //     .collect::<HashMap<String, User>>();
        let league = league_future.await?;
        let previous_league_id = league.previous_league_id.ok_or(Error::NotFound(format!(
            "previous league does not exist for {league_id}"
        )))?;
        let previous_league = client.get_league(previous_league_id.as_str()).await?;
        let previous_league_bracket = client
            .get_playoff_bracket(previous_league.league_id.clone(), false)
            .await?;
        Ok(Standings::new(
            previous_league.settings,
            previous_league_bracket.matchups,
            None,
        ))
    }

    pub fn list_playoff_roster_ids(&self) -> HashSet<u8> {
        self.winners_bracket
            .iter()
            .flat_map(|e| {
                let mut rids = vec![];
                if let Some(rid) = e.team_1_roster_id {
                    rids.push(rid);
                }
                if let Some(rid) = e.team_2_roster_id {
                    rids.push(rid);
                }
                rids
            })
            .collect()
    }

    pub fn expected_playoff_teams(&self) -> usize {
        self.league_settings.playoff_teams
    }

    /// This will group the rosters into 2 groups: playoff and non-playoff
    /// teams. It will then randomize within the two groups and insert the
    /// non-playoff randomize rosters then the randomized playoff teams to
    /// generate the draft order for the following season.
    pub fn randomize_bucketed_draft_order(&self) -> Vec<u8> {
        let rosters: Vec<u8> = (1..=self.league_settings.num_teams as u8).collect();
        let mut playoff_teams: Vec<u8> = self.list_playoff_roster_ids().drain().collect();
        let mut non_playoff_teams = rosters
            .into_iter()
            .filter(|r| !playoff_teams.contains(r))
            .collect::<Vec<u8>>();

        // shuffle
        playoff_teams.shuffle(&mut rng());
        non_playoff_teams.shuffle(&mut rng());

        non_playoff_teams.extend(playoff_teams);
        non_playoff_teams
    }

    /// Renders the draft order with each slot's owner, fetching rosters and users for `league_id`.
    pub async fn visualize_draft_order<T: Into<String>>(
        league_id: T,
        draft_order_roster_ids: Vec<u8>,
    ) -> Result<String, Error> {
        let client = Client::new();
        let league_id: String = league_id.into();
        let roster_map = client
            .get_rosters_in_league(league_id.as_str())
            .await?
            .into_iter()
            .map(|r| (r.roster_id, r))
            .collect::<HashMap<u8, Roster>>();
        let user_map = client
            .get_users_for_league(league_id.as_str())
            .await?
            .into_iter()
            .map(|u| (u.user_id.clone(), u))
            .collect::<HashMap<String, User>>();
        Ok(format_draft_order(
            &draft_order_roster_ids,
            &roster_map,
            &user_map,
        ))
    }
}

/// One line per draft slot: "<slot>: <owner display name>" (or "Roster <id>" when the owner is unknown).
fn format_draft_order(
    draft_order_roster_ids: &[u8],
    roster_map: &HashMap<u8, Roster>,
    user_map: &HashMap<String, User>,
) -> String {
    let mut text = String::new();
    for (i, roster_id) in draft_order_roster_ids.iter().enumerate() {
        let slot = i + 1;
        let username = roster_map
            .get(roster_id)
            .and_then(|roster| roster.owner_id.as_ref())
            .and_then(|owner_id| user_map.get(owner_id.as_str()))
            .map(|user| user.display_name.clone())
            .unwrap_or(format!("Roster {roster_id}"));
        text.push_str(format!("{slot}: {username}").as_str());
        text.push('\n');
    }
    text
}

#[cfg(test)]
mod custom_tests {
    use std::{
        collections::HashMap,
        ffi::OsStr,
        fs::{self, File},
        io::BufReader,
        path::Path,
    };

    use crate::{
        League, LeagueStatus, PlayoffBracketEntry, Roster, User,
        custom::{Standings, format_draft_order},
        data::{LeagueType, PlayoffType, WaiverType},
    };

    fn load_all_testdata_leagues() -> Vec<League> {
        let test_data_leagues_path = "testdata/v1/league";
        let paths: Vec<String> = fs::read_dir(test_data_leagues_path)
            .expect("unable to list test data files")
            .filter_map(|f| match f {
                Ok(dir_entry) => {
                    let tmp = dir_entry.file_name();
                    let p = Path::new(tmp.as_os_str());
                    match p.extension().and_then(OsStr::to_str) {
                        Some(ext) => {
                            if ext == "json" {
                                Some(format!(
                                    "{}/{}",
                                    test_data_leagues_path,
                                    p.to_str().unwrap()
                                ))
                            } else {
                                None
                            }
                        }
                        None => None,
                    }
                }
                Err(_) => None,
            })
            .collect();
        paths
            .iter()
            .map(|league_test_data_filename| {
                let f = File::open(league_test_data_filename.clone()).unwrap_or_else(|_| {
                    panic!("failed to open test data file ({league_test_data_filename})")
                });

                serde_json::from_reader(BufReader::new(f)).unwrap_or_else(|_| {
                    panic!(
                        "failed to deserialize league json file: {}",
                        league_test_data_filename
                    )
                })
            })
            .collect()
    }

    #[test]
    fn load_all_leagues_test_datas_sanity_check() {
        let leagues = load_all_testdata_leagues();
        assert_eq!(5, leagues.len());
        for league in leagues {
            let league_name = league.name.as_str();
            let season = league.season.as_str();
            let league_id = league.league_id.as_str();
            let league_debug_name = format!("{league_name} [{season}] ({league_id})");

            // assertions
            debug_assert_eq!(
                "Rapid League 17", league_name,
                "unusual league name in {league_debug_name}"
            );
            debug_assert_eq!(
                12, league.settings.num_teams,
                "unusual num_teams in {league_debug_name}"
            );
            debug_assert_eq!(
                LeagueType::Keeper,
                league.settings.r#type,
                "unusual league type in {league_debug_name}"
            );
            debug_assert_eq!(
                WaiverType::FAAB,
                league.settings.waiver_type,
                "unusual waiver_type in {league_debug_name}"
            );
            debug_assert_eq!(
                PlayoffType::Default,
                league.settings.playoff_type,
                "unusual playoff_type in {league_debug_name}"
            );
            debug_assert_eq!(
                6, league.settings.playoff_teams,
                "unusual playoff_teams in {league_debug_name}"
            );
            debug_assert!(
                !league.settings.best_ball,
                "unusual best_ball in {league_debug_name}"
            );
            if league.status == LeagueStatus::PreDraft {
                debug_assert!(
                    league.settings.last_scored_leg.is_none(),
                    "unusual last_scored_leg in predraft {league_debug_name}"
                );
            } else {
                debug_assert!(
                    league.settings.last_scored_leg.is_some(),
                    "unusual last_scored_leg in {league_debug_name}"
                );
            }
            debug_assert!(
                league.settings.playoff_week_start > 0,
                "unusual playoff_week_start in {league_debug_name}"
            );
            // assert_eq!(1, league.settings.start_week);
            let reg_season_weeks = league.settings.playoff_week_start - league.settings.start_week;
            debug_assert!(
                reg_season_weeks > 0,
                "unusual calculated regular season weeks [{reg_season_weeks}] in {league_debug_name}"
            );
        }
    }

    fn standings_from_testdata() -> Standings {
        let league_id = "164974877262287888";
        let league: League = serde_json::from_str(
            &fs::read_to_string(format!("testdata/v1/league/{league_id}.json")).unwrap(),
        )
        .unwrap();
        let winners_bracket: Vec<PlayoffBracketEntry> = serde_json::from_str(
            &fs::read_to_string(format!(
                "testdata/v1/league/{league_id}/winners_bracket.json"
            ))
            .unwrap(),
        )
        .unwrap();
        Standings::new(league.settings, winners_bracket, None)
    }

    #[test]
    fn test_randomize_bucketed_draft_order() {
        let standings = standings_from_testdata();
        let playoff_teams = standings.list_playoff_roster_ids();
        assert!(!playoff_teams.is_empty());

        for _ in 0..20 {
            let order = standings.randomize_bucketed_draft_order();

            // every roster appears exactly once
            let mut sorted = order.clone();
            sorted.sort();
            assert_eq!((1..=12).collect::<Vec<u8>>(), sorted);

            // non-playoff teams draft first, playoff teams last
            let first_playoff_slot = order
                .iter()
                .position(|r| playoff_teams.contains(r))
                .unwrap();
            assert_eq!(12 - playoff_teams.len(), first_playoff_slot);
            assert!(
                order[first_playoff_slot..]
                    .iter()
                    .all(|r| playoff_teams.contains(r))
            );
        }
    }

    #[test]
    fn test_format_draft_order_falls_back_to_the_roster_id_for_unknown_owners() {
        let users: Vec<User> =
            serde_json::from_str(&fs::read_to_string("testdata/league_users.json").unwrap())
                .unwrap();
        let rosters: Vec<Roster> =
            serde_json::from_str(&fs::read_to_string("testdata/league_rosters.json").unwrap())
                .unwrap();
        let user_map = users
            .into_iter()
            .map(|u| (u.user_id.clone(), u))
            .collect::<HashMap<_, _>>();
        let mut roster_map = rosters
            .into_iter()
            .map(|r| (r.roster_id, r))
            .collect::<HashMap<_, _>>();
        let known = *roster_map.keys().min().unwrap();
        let owner = roster_map[&known].owner_id.clone().unwrap();
        let expected_name = user_map[&owner].display_name.clone();
        roster_map.get_mut(&known).unwrap().owner_id = Some(owner);
        roster_map.insert(
            99,
            Roster {
                roster_id: 99,
                owner_id: None,
                ..roster_map[&known].clone()
            },
        );

        let text = format_draft_order(&[known, 99], &roster_map, &user_map);
        assert_eq!(format!("1: {expected_name}\n2: Roster 99\n"), text);
    }
}
