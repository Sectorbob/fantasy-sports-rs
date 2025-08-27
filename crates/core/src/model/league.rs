use crate::{
    Draft, DraftPick, ExternalId, LeagueSettings, Player, Roster, Standings, StandingsEntry, Team,
    Transaction, data::Scoreboard,
};
use serde_json::Number;
use sleeper_fantasy_rs as sleeper;
use std::{collections::HashMap, fmt};
use yahoo_fantasy_rs as yahoo;

const N_A: &str = "N/A";

#[derive(Debug, Clone)]
pub enum League {
    Sleeper {
        draft: Option<sleeper::Draft>,
        draft_picks: Vec<sleeper::DraftPick>,
        league: sleeper::League,
        matchups: HashMap<usize, Vec<sleeper::Matchup>>,
        owners: HashMap<String, sleeper::User>,
        players: HashMap<String, sleeper::Player>,
        winners_bracket: Option<sleeper::Bracket>,
        losers_bracket: Option<sleeper::Bracket>,
        rosters: HashMap<u8, sleeper::Roster>,
        transactions: Vec<sleeper::Transaction>,
        user_id: Option<String>,
    },
    Yahoo {
        draft_results: Vec<yahoo::DraftResult>,
        league: yahoo::League,
        scoreboards: Vec<yahoo::Scoreboard>,
        players: HashMap<yahoo::PlayerKey, crate::Player>,
        rosters: HashMap<u32, yahoo::Roster>,
        settings: yahoo::Settings,
        standings: yahoo::Standings,
        team: Option<yahoo::Team>,
        teams: Vec<yahoo::Team>,
        transactions: Vec<yahoo::Transaction>,
    },
}
impl League {
    pub fn with_owner_id(self, id: ExternalId) -> Self {
        match self {
            League::Sleeper {
                draft,
                draft_picks,
                league,
                matchups,
                owners,
                players,
                winners_bracket,
                losers_bracket,
                rosters,
                transactions,
                user_id: _,
            } => League::Sleeper {
                draft,
                draft_picks,
                league,
                matchups,
                owners,
                players,
                winners_bracket,
                losers_bracket,
                rosters,
                transactions,
                user_id: Some(id.id),
            },
            League::Yahoo {
                draft_results,
                league,
                scoreboards,
                players,
                rosters,
                settings,
                standings,
                team: _,
                teams,
                transactions,
            } => League::Yahoo {
                draft_results,
                league,
                scoreboards,
                players,
                rosters,
                settings,
                standings,
                team: teams
                    .iter()
                    .find(|t| t.managers().iter().find(|m| m.guid == id.id).is_some())
                    .map(yahoo::Team::clone),
                teams,
                transactions,
            },
        }
    }
    pub fn selected_owner_id(&self) -> Option<String> {
        match self {
            League::Sleeper { user_id, .. } => user_id.as_ref().map(String::clone),
            League::Yahoo { team, .. } => team
                .as_ref()
                .map(|t| t.managers().first().map(|m| m.guid.clone()))
                .flatten(),
        }
    }
    pub fn status(&self) -> LeagueStatus {
        match self {
            League::Sleeper { league, .. } => LeagueStatus::from(league),
            League::Yahoo { league, .. } => LeagueStatus::from(league),
        }
    }
    pub fn id(&self) -> String {
        match self {
            League::Sleeper { league, .. } => league.league_id.clone(),
            League::Yahoo { league, .. } => league.league_key.to_string(),
        }
    }
    pub fn name(&self) -> String {
        match self {
            League::Sleeper { league, .. } => league.name.clone(),
            League::Yahoo { league, .. } => league.name.clone(),
        }
    }
    pub fn platform(&self) -> String {
        match self {
            League::Sleeper { .. } => String::from("Sleeper"),
            League::Yahoo { .. } => String::from("Yahoo"),
        }
    }
    pub fn season(&self) -> String {
        match self {
            League::Sleeper { league, .. } => league.season.clone(),
            League::Yahoo { league, .. } => league.season.clone(),
        }
    }
    pub fn standings(&self) -> Standings {
        match self {
            League::Sleeper {
                rosters, owners, ..
            } => Standings {
                entries: rosters
                    .iter()
                    .map(|(id, roster)| {
                        let wins = roster
                            .settings
                            .get("wins")
                            .map(Number::as_u64)
                            .flatten()
                            .unwrap_or_default();
                        let losses = roster
                            .settings
                            .get("losses")
                            .map(Number::as_u64)
                            .flatten()
                            .unwrap_or_default();
                        let ties = roster
                            .settings
                            .get("ties")
                            .map(Number::as_u64)
                            .flatten()
                            .unwrap_or_default();
                        let owner_id = &roster.owner_id;
                        let owner = owner_id.as_ref().map(|id| owners.get(id)).flatten();
                        StandingsEntry {
                            roster_id: roster.roster_id.to_string(),
                            team_name: owner.map_or(format!("Team {id}"), sleeper::User::team_name),
                            owner_name: owner
                                .as_ref()
                                .map_or(format!("Owner {owner_id:?}"), |o| o.display_name.clone()),
                            owner_id: owner_id.as_ref().map_or(String::new(), |o| o.clone()),
                            wins,
                            losses,
                            ties,
                        }
                    })
                    .collect(),
            },
            League::Yahoo { standings, .. } => Standings {
                entries: standings
                    .teams
                    .iter()
                    .map(|team| {
                        let wins = team
                            .team_standings
                            .as_ref()
                            .map(|s| s.outcome_totals.wins as u64)
                            .unwrap_or_default();
                        let losses = team
                            .team_standings
                            .as_ref()
                            .map(|s| s.outcome_totals.losses as u64)
                            .unwrap_or_default();
                        let ties = team
                            .team_standings
                            .as_ref()
                            .map(|s| s.outcome_totals.ties as u64)
                            .unwrap_or_default();
                        let manager = team.managers().first().clone().unwrap();
                        StandingsEntry {
                            roster_id: team.team_key.to_string(),
                            team_name: team.name.clone(),
                            owner_name: manager.nickname.clone(),
                            owner_id: manager.guid.clone(),
                            wins,
                            losses,
                            ties,
                        }
                    })
                    .collect(),
            },
        }
    }
    pub fn playoff_results(&self) -> Option<HashMap<u8, Team>> {
        let mut teams = self
            .teams()
            .into_iter()
            .map(|t| (t.id.clone(), t))
            .collect::<HashMap<_, _>>();
        match self {
            League::Sleeper {
                winners_bracket,
                losers_bracket: _,
                league,
                ..
            } => {
                match &league.settings.playoff_type {
                    sleeper::PlayoffType::Default => {
                        // implemented below
                    }
                    sleeper::PlayoffType::ReSeed => {
                        //TODO
                        log::warn!("sleeper playoff setting for reseed not yet implemented");
                        return None;
                    }
                };

                let winners_bracket = if let Some(w) = winners_bracket {
                    w
                } else {
                    return None;
                };
                let mut s = HashMap::new();
                if let Some(champ_roster_id) = winners_bracket.champion() {
                    if let Some(t) = teams.remove(&champ_roster_id.to_string()) {
                        s.insert(1, t);
                    }
                }
                if let Some(runner_up_roster_id) = winners_bracket.runner_up() {
                    if let Some(t) = teams.remove(&runner_up_roster_id.to_string()) {
                        s.insert(2, t);
                    }
                }
                if let Some(third_place_roster_id) = winners_bracket.third_place() {
                    if let Some(t) = teams.remove(&third_place_roster_id.to_string()) {
                        s.insert(3, t);
                    }
                }
                if let Some(fourth_place_roster_id) = winners_bracket.fourth_place() {
                    if let Some(t) = teams.remove(&fourth_place_roster_id.to_string()) {
                        s.insert(4, t);
                    }
                }
                log::warn!(
                    "sleeper playoff results not collected for 5+ place in winners bracket and any of the losers bracket"
                );
                Some(s)
            }
            League::Yahoo {
                scoreboards: _,
                settings: _,
                ..
            } => {
                log::warn!("league playoff results not yet implemented for yahoo leagues");
                None
            }
        }
    }

    pub fn teams(&self) -> Vec<Team> {
        match self {
            League::Sleeper {
                owners, rosters, ..
            } => rosters
                .iter()
                .map(|r| Team::from((r.1, owners)))
                .collect::<Vec<_>>(),
            League::Yahoo { teams, .. } => teams.iter().map(Team::from).collect::<Vec<_>>(),
        }
    }
    pub fn record(&self) -> String {
        self.team().map_or(String::from(N_A), |t| {
            self.record_for_roster(t.roster_id.clone())
        })
    }
    pub fn record_for_roster(&self, roster_id: String) -> String {
        match self {
            League::Sleeper { rosters, .. } => match rosters
                .iter()
                .find(|(_, r)| r.roster_id.to_string() == roster_id)
            {
                Some((_, roster)) => format!(
                    "{}-{}-{}",
                    match roster.settings.get("wins") {
                        Some(n) => n.to_string(),
                        None => 0.to_string(),
                    },
                    match roster.settings.get("losses") {
                        Some(n) => n.to_string(),
                        None => 0.to_string(),
                    },
                    match roster.settings.get("ties") {
                        Some(n) => n.to_string(),
                        None => 0.to_string(),
                    }
                ),
                None => "n/a".to_string(),
            },
            League::Yahoo { standings, .. } => match standings
                .teams
                .iter()
                .find(|t| t.team_id.to_string() == roster_id)
            {
                Some(team_w_standings) => match &team_w_standings.team_standings {
                    Some(team_standings) => format!(
                        "{}-{}-{}",
                        team_standings.outcome_totals.wins,
                        team_standings.outcome_totals.losses,
                        team_standings.outcome_totals.ties
                    ),
                    None => String::from(N_A),
                },
                None => String::from(N_A),
            },
        }
    }
    pub fn team(&self) -> Option<Team> {
        self.selected_owner_id()
            .as_ref()
            .map(|id| {
                self.teams()
                    .iter()
                    .find(|t| {
                        if let Some(owner_id) = &t.owner_id {
                            owner_id == id
                        } else {
                            false
                        }
                    })
                    .map(Team::clone)
            })
            .flatten()
    }
    pub fn team_name(&self) -> String {
        self.team()
            .map_or(String::from(N_A), |t| t.team_name.clone())
    }
    pub fn total_points(&self) -> f64 {
        match self {
            League::Sleeper {
                rosters, user_id, ..
            } => match rosters
                .iter()
                .find(|(_, r)| r.owner_id.is_some() && user_id.is_some() && &r.owner_id == user_id)
            {
                Some((_, roster)) => roster
                    .settings
                    .get("points")
                    .and_then(|p| p.as_f64())
                    .unwrap_or(-2.0) as f64,
                None => -1.0,
            },
            League::Yahoo {
                standings, team, ..
            } => match standings
                .teams
                .iter()
                .find(|t| Some(t.team_id) == team.as_ref().map(|t| t.team_id))
            {
                Some(team_w_standings) => match &team_w_standings.team_standings {
                    Some(team_standings) => team_standings.points_for as f64,
                    None => -1.0,
                },
                None => -2.0,
            },
        }
    }
    pub fn settings(&self) -> LeagueSettings {
        match self {
            League::Sleeper { league, draft, .. } => LeagueSettings::from_sleeper(&league, draft),
            League::Yahoo { settings, .. } => LeagueSettings::from_yahoo(settings),
        }
    }
    #[allow(dead_code)]
    pub fn raw(&self) -> String {
        match self {
            League::Sleeper { league, .. } => format!("Raw: {:#?}", league.settings),
            League::Yahoo { settings, .. } => format!("Raw: {:#?}", settings),
        }
    }
    pub fn scoreboards(&self) -> Vec<Scoreboard> {
        match self {
            League::Sleeper {
                matchups,
                owners,
                rosters,
                ..
            } => matchups
                .iter()
                .map(|(week, matchups)| {
                    Scoreboard::from_sleeper(week.clone() as u32, matchups.clone(), rosters, owners)
                })
                .collect(),
            League::Yahoo { scoreboards, .. } => scoreboards.iter().map(Scoreboard::from).collect(), // Yahoo leagues do not have matchups in the same way
        }
    }
    pub fn players(&self) -> HashMap<String, Player> {
        match self {
            League::Sleeper { players, .. } => players
                .iter()
                .map(|(id, p)| (id.clone(), Player::from(p)))
                .collect::<HashMap<String, Player>>(),
            League::Yahoo { players, .. } => players
                .iter()
                .map(|(_, player)| (player._id.clone(), player.clone()))
                .collect(),
        }
    }
    pub fn draft(&self) -> Option<Draft> {
        match self {
            League::Sleeper {
                draft,
                draft_picks,
                rosters,
                ..
            } => draft
                .as_ref()
                .map(|d| Draft::from((d, draft_picks, rosters))),
            League::Yahoo {
                draft_results,
                league,
                players,
                settings,
                ..
            } => match draft_results.len() {
                0 => None,
                _ => Some(Draft {
                    draft_type: settings.draft_type.to_string(),
                    draft_id: String::from(N_A),
                    league_id: league.league_id.clone(),
                    picks: draft_results
                        .iter()
                        .map(|p| {
                            if let Some(player_key) = &p.player_key {
                                if let Some(player) = players.get(player_key) {
                                    DraftPick {
                                        round: p.round,
                                        pick: p.pick,
                                        overall_pick: p.pick,
                                        player: player.clone(),
                                        roster_id: p.team_key.to_string(),
                                    }
                                } else {
                                    DraftPick {
                                        round: p.round,
                                        pick: p.pick,
                                        overall_pick: p.pick,
                                        player: Player {
                                            _id: player_key.to_string(),
                                            name: player_key.to_string(),
                                            positions: vec![],
                                            irl_team: None,
                                        },
                                        roster_id: p.team_key.to_string(),
                                    }
                                }
                            } else {
                                // No player_key onm the draft pick
                                DraftPick {
                                    round: p.round,
                                    pick: p.pick,
                                    overall_pick: p.pick,
                                    player: Player {
                                        _id: String::from(N_A),
                                        name: String::from(N_A),
                                        positions: vec![],
                                        irl_team: None,
                                    },
                                    roster_id: p.team_key.to_string(),
                                }
                            }
                        })
                        .collect(),
                    draft_order: None, //TODO: implement me
                }),
            },
        }
    }
    pub fn rosters(&self) -> Vec<Roster> {
        match self {
            League::Sleeper {
                rosters, owners, ..
            } => rosters
                .iter()
                .map(|r| r.1)
                .map(|r| Roster::from((r, owners)))
                .collect(),
            League::Yahoo { rosters, teams, .. } => rosters
                .iter()
                .map(|(roster_id, roster)| Roster::from((roster_id, roster, teams)))
                .collect(),
        }
    }
    pub fn transactions(&self) -> Vec<Transaction> {
        match self {
            League::Sleeper { transactions, .. } => {
                transactions.iter().map(Transaction::from).collect()
            }
            League::Yahoo { transactions, .. } => {
                transactions.iter().map(Transaction::from_yahoo).collect()
            }
        }
    }
}
impl std::fmt::Display for League {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let team = self.team();
        let placement_sfx = match team {
            Some(t) => match self.playoff_results() {
                Some(playoff_results) => playoff_results
                    .iter()
                    .find(|(_, t2)| t.id == t2.id)
                    .map_or(String::new(), |(place, _)| match *place {
                        1 => format!(" 👑"),
                        2 => format!(" 🥈"),
                        3 => format!(" 🥉"),
                        _ => String::new(),
                    }),
                None => String::new(),
            },
            None => String::new(),
        };
        write!(
            f,
            "[{season}] {name} ({platform}) - {status}       Record: {record}{placement_sfx}",
            season = self.season(),
            name = self.name(),
            platform = self.platform(),
            status = self.status(),
            record = self.record(),
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LeagueStatus {
    PreDraft,
    Drafting,
    InSeason,
    Complete,
    InComplete,
}
impl fmt::Display for LeagueStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LeagueStatus::PreDraft => write!(f, "Pre-draft"),
            LeagueStatus::Drafting => write!(f, "Drafting"),
            LeagueStatus::InSeason => write!(f, "In-Season"),
            LeagueStatus::Complete => write!(f, "Complete"),
            LeagueStatus::InComplete => write!(f, "Incomplete"),
        }
    }
}
impl From<&sleeper::League> for LeagueStatus {
    fn from(value: &sleeper::League) -> Self {
        match value.status {
            sleeper::LeagueStatus::PreDraft => LeagueStatus::PreDraft,
            sleeper::LeagueStatus::Drafting => LeagueStatus::Drafting,
            sleeper::LeagueStatus::InSeason => LeagueStatus::InSeason,
            sleeper::LeagueStatus::Complete => LeagueStatus::Complete,
        }
    }
}
impl From<&yahoo::League> for LeagueStatus {
    fn from(value: &yahoo::League) -> Self {
        match value.is_finished {
            Some(is_finished) => match is_finished {
                true => LeagueStatus::Complete,
                false => LeagueStatus::InSeason,
            },
            None => LeagueStatus::InComplete,
        }
    }
}

#[allow(dead_code)]
#[cfg(test)]
mod tests {
    use crate::{ExternalId, League, LeagueAccessor};
    use sleeper_fantasy_rs as sleeper;
    use std::{collections::HashMap, ops::Index};
    use yahoo_fantasy_rs as yahoo;

    // #[tokio::test]
    //TODO: stub out data, vs using cache
    async fn testme() {
        let mut accessor = LeagueAccessor::new().with_cache_dir("cache");
        accessor
            .init()
            .await
            .expect("failed to setup league accessor");

        let l = accessor
            .fetch_league_context(&ExternalId::try_from("yahoo:359.l.564503").unwrap(), false)
            .await
            .unwrap();

        if let League::Yahoo {
            scoreboards,
            settings,
            ..
        } = l
        {
            let matchups = extract_playoffs_into_matchups(&scoreboards, &settings);
            for matchup in matchups {
                let team_1 = matchup.2.teams.index(0);
                let team_2 = matchup.2.teams.index(1);
                let left_side = format!(
                    "{} {}",
                    team_1.name,
                    team_1
                        .team_points
                        .as_ref()
                        .map_or(0.0, |team_points| team_points.total.map_or(0.0, |p| p))
                );
                let right_side = format!(
                    "{} {}",
                    team_2.name,
                    team_2
                        .team_points
                        .as_ref()
                        .map_or(0.0, |team_points| team_points.total.map_or(0.0, |p| p))
                );
                println!(
                    "Round: {rd} MatchupId {id} {left_side} vs {right_side}",
                    rd = matchup.0,
                    id = matchup.1
                );
                println!(
                    "{:?}",
                    convert_yahoo_to_sleeper_playoff_bracket(matchup.0, matchup.1, matchup.2),
                );
            }
        }
        assert!(false)
    }

    fn extract_playoffs_into_matchups<'a>(
        scoreboards: &'a Vec<yahoo::Scoreboard>,
        settings: &yahoo::Settings,
    ) -> Vec<(u8, u8, &'a yahoo::Matchup)> {
        let mut matchups = vec![];
        if settings.uses_playoff {
            if let Some(first_week_of_playoffs) = settings.playoff_start_week {
                let num_of_playoff_teams = settings.num_playoff_teams;
                log::warn!(
                    "still need to do a yahoo check for the num of teams allowed in playoffs: {num_of_playoff_teams}"
                );
                let mut playoff_rounds: HashMap<u8, &yahoo::Scoreboard> = HashMap::new();
                let mut rounds = vec![];
                let mut matchup_counter: u8 = 0;
                for scoreboard in scoreboards {
                    if scoreboard.week >= first_week_of_playoffs {
                        // is playoff week
                        let round: u8 = scoreboard.week as u8 - first_week_of_playoffs as u8 + 1;
                        playoff_rounds.insert(round as u8, scoreboard);
                        rounds.push(round);
                    }
                }
                rounds.sort();
                for round in rounds {
                    if let Some(scoreboard) = playoff_rounds.get(&round) {
                        for matchup in &scoreboard.matchups.matchups {
                            if matchup.teams.len() != 2 {
                                continue;
                            }
                            // let team_1 = matchup.teams.index(0);
                            // let team_2 = matchup.teams.index(1);
                            matchup_counter = matchup_counter + 1;
                            matchups.push((round as u8, matchup_counter, matchup));
                        }
                    }
                }
            }
        }
        matchups
    }

    fn convert_yahoo_to_sleeper_playoff_bracket(
        round: u8,
        matchup_id: u8,
        matchup: &yahoo::Matchup,
    ) -> sleeper::PlayoffBracketEntry {
        sleeper::PlayoffBracketEntry {
            round,
            matchup_id,
            team_1_roster_id: matchup.teams.get(0).map(|t| t.team_id as u8),
            team_2_roster_id: matchup.teams.get(1).map(|t| t.team_id as u8),
            winner_roster_id: matchup.winner_team_key.as_ref().map(|k| k.team_id as u8),
            loser_roster_id: matchup
                .winner_team_key
                .as_ref()
                .map(|winner_team_key| {
                    if Some(winner_team_key.team_id) == matchup.teams.get(0).map(|t| t.team_id) {
                        matchup.teams.get(1).map(|t| t.team_id as u8)
                    } else {
                        matchup.teams.get(0).map(|t| t.team_id as u8)
                    }
                })
                .flatten(),
            team_1_from: None,
            team_2_from: None,
        }
    }
}
