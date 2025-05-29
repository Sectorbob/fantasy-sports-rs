use std::collections::HashMap;

use crate::data::{Matchup, Player, Roster, User};

/// I want to place my own custom consumer code here
///
///

pub struct FantasyMatchup {
    pub week: usize,
    pub left: FantasyMatchupSide,
    pub right: FantasyMatchupSide,
}

impl FantasyMatchup {
    pub fn from_stuff(
        week: usize,
        matchups: Vec<Matchup>,
        rosters: &Vec<Roster>,
        owners: &Vec<User>,
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
                games.push((tmp, matchup));
            } else if let None = matchup.matchup_id {
                orphaned.push(matchup);
            } else {
                wip.push(matchup);
            }
        }
        if !wip.is_empty() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("wip was not empty: {}", wip.len()),
            )));
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
                        return Err(Box::new(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("invalid roster {}", &game.0.roster_id),
                        )));
                    }
                };
            let roster_1 =
                match rosters.binary_search_by_key(&game.1.roster_id, |roster| roster.roster_id) {
                    Ok(index) => rosters[index].clone(),
                    Err(_) => {
                        return Err(Box::new(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("invalid roster {}", &game.1.roster_id),
                        )));
                    }
                };
            let owner_0 =
                match owners.binary_search_by_key(&(&roster_0.owner_id), |user| &user.user_id) {
                    Ok(index) => owners[index].clone(),
                    Err(_) => {
                        return Err(Box::new(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("invalid owner '{}'", &roster_0.owner_id),
                        )));
                    }
                };
            let owner_1 =
                match owners.binary_search_by_key(&(&roster_1.owner_id), |user| &user.user_id) {
                    Ok(index) => owners[index].clone(),
                    Err(_) => {
                        return Err(Box::new(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("invalid owner '{}'", &roster_1.owner_id),
                        )));
                    }
                };

            let players_0 = game.0.players.clone();
            let players_1 = game.1.players.clone();
            let starters_0 = game.0.starters.clone();
            let starters_1 = game.1.starters.clone();

            let f = FantasyMatchup {
                week: week,
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

            println!("{}{}{}", left, "  ", right);
        }
    }
}

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
