use std::collections::HashMap;

use crate::data::{Matchup, Player, Roster, User};

/// I want to place my own custom consumer code here
///
///

pub struct FantasyMatchup<'a> {
    pub left: FantasyMatchupSide<'a>,
    pub right: FantasyMatchupSide<'a>,
}

impl FantasyMatchup<'_> {
    pub fn from_stuff<'a>(
        players: &'a HashMap<String, Player>,
        matchups: Vec<Matchup>,
        rosters: &'a Vec<Roster>,
        owners: &'a Vec<User>,
    ) -> Vec<FantasyMatchup<'a>> {
        let mut games: Vec<(Matchup, Matchup)> = Vec::with_capacity(matchups.len() / 2);
        let mut wip: Vec<Matchup> = vec![];
        let mut orphaned: Vec<Matchup> = vec![];
        for matchup in matchups {
            if let Some(index) = wip
                .iter()
                .position(|wip_entry| matchup.matchup_id == wip_entry.matchup_id)
            {
                let tmp = wip.remove(index);
                games.push((tmp, matchup));
            } else if let None = matchup.matchup_id {
                orphaned.push(matchup);
            } else {
                wip.push(matchup);
            }
        }
        if !wip.is_empty() {
            panic!("wip was not empty: {}", wip.len());
        }
        if !orphaned.is_empty() {
            println!("had {} orphaned statlines", orphaned.len());
        }

        let mut new_fantasy_matchups: Vec<FantasyMatchup<'a>> = vec![];

        for game in games {
            let roster_0 =
                match rosters.binary_search_by_key(&game.0.roster_id, |roster| roster.roster_id) {
                    Ok(index) => &rosters[index],
                    Err(_) => panic!("invalid roster {}", &game.0.roster_id),
                };
            let roster_1 =
                match rosters.binary_search_by_key(&game.1.roster_id, |roster| roster.roster_id) {
                    Ok(index) => &rosters[index],
                    Err(_) => panic!("invalid roster {}", &game.1.roster_id),
                };
            let owner_0 =
                match owners.binary_search_by_key(&(&roster_0.owner_id), |user| &user.user_id) {
                    Ok(index) => &owners[index],
                    Err(_) => panic!("invalid owner '{}'", &roster_0.owner_id),
                };
            let owner_1 =
                match owners.binary_search_by_key(&(&roster_1.owner_id), |user| &user.user_id) {
                    Ok(index) => &owners[index],
                    Err(_) => panic!("invalid owner '{}'", &roster_1.owner_id),
                };

            let f = FantasyMatchup {
                left: FantasyMatchupSide {
                    matchup: game.0,
                    roster: roster_0,
                    owner: owner_0,
                    players: players,
                },
                right: FantasyMatchupSide {
                    matchup: game.1,
                    roster: roster_1,
                    owner: owner_1,
                    players: players,
                },
            };

            new_fantasy_matchups.push(f);
        }

        new_fantasy_matchups
    }

    pub fn printy(&self, width: usize) {
        // let delim = String::from(" ").repeat(if width % 2 == 0 { 2 } else { 1 });
        let col_width = width / 2;

        println!(
            "{:<col_width$}{}{:>col_width$}",
            &self.left.owner.display_name,
            "vs",
            &self.right.owner.display_name,
            col_width = col_width,
        );
        println!(
            "{:<col_width$}{}{:>col_width$}",
            &self.left.matchup.points,
            "TP",
            &self.right.matchup.points,
            col_width = col_width,
        );
        for i in 0..self.left.matchup.starters.len() {
            let left_player = self
                .left
                .players
                .get(&self.left.matchup.starters[i])
                .unwrap();
            let right_player = self
                .right
                .players
                .get(&self.right.matchup.starters[i])
                .unwrap();

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

            println!("{}{}{}", left, "  ", right);
        }
    }
}

pub struct FantasyMatchupSide<'a> {
    pub matchup: Matchup,
    pub roster: &'a Roster,
    pub owner: &'a User,
    pub players: &'a HashMap<String, Player>,
}
impl<'a> FantasyMatchupSide<'a> {
    pub fn max_player_score_width(&'a self) -> usize {
        self.matchup
            .starters_points
            .iter()
            .map(|p| p.to_string())
            .max_by_key(|s| s.len())
            .unwrap_or_default()
            .len()
    }
}
