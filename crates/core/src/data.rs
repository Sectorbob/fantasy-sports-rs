use crate::ExternalId;
use chrono::{DateTime, Utc};
use core::fmt;
use serde::{Deserialize, Serialize};
use sleeper_fantasy_rs::{self as sleeper};
use std::collections::{HashMap, HashSet};
use yahoo_fantasy_rs::{self as yahoo};

#[derive(Debug)]
#[allow(dead_code)]
pub struct Draft {
    pub draft_type: String,
    pub draft_id: String,
    pub league_id: String,
    pub picks: Vec<DraftPick>,
}
impl Draft {
    pub fn check_for_valid_draft(
        drafts: &Vec<sleeper_fantasy_rs::Draft>,
    ) -> Option<sleeper_fantasy_rs::Draft> {
        match drafts.len() {
            0 => None,
            1 => Some(drafts[0].clone()),
            _ => {
                // prioritize drafts with Some(start_time)
                let mut filtered = drafts
                    .iter()
                    .filter(|d| d.start_time.is_some())
                    .collect::<Vec<&sleeper_fantasy_rs::Draft>>();
                if filtered.len() == 1 {
                    Some(filtered[0].clone())
                } else if filtered.len() > 1 {
                    filtered.sort_by_key(|d| d.start_time);
                    filtered.reverse();
                    Some(filtered[0].clone())
                } else {
                    None
                }
            }
        }
    }
}
impl
    From<(
        &sleeper_fantasy_rs::Draft,
        &Vec<sleeper_fantasy_rs::DraftPick>,
    )> for Draft
{
    fn from(
        value: (
            &sleeper_fantasy_rs::Draft,
            &Vec<sleeper_fantasy_rs::DraftPick>,
        ),
    ) -> Self {
        Draft {
            draft_type: value.0._type.to_string(),
            draft_id: value.0.draft_id.clone(),
            league_id: value.0.league_id.clone(),
            picks: value.1.iter().map(DraftPick::from).collect(),
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct DraftPick {
    pub round: u32,
    pub pick: u32,
    pub overall_pick: u32,
    pub player: Player,
    pub roster_id: String,
}
impl From<&sleeper_fantasy_rs::DraftPick> for DraftPick {
    fn from(value: &sleeper_fantasy_rs::DraftPick) -> Self {
        DraftPick {
            round: value.round as u32,
            pick: value.pick_no as u32, // TODO: this is wrong
            overall_pick: value.pick_no as u32,
            player: Player {
                _id: value.draft_id.clone(), // should I clone?
                name: format!(
                    "{} {}",
                    &value.metadata.first_name, &value.metadata.last_name
                ),
                positions: vec![value.metadata.position.clone()],
                irl_team: Some(value.metadata.team.clone()),
            },
            roster_id: value.roster_id.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum League {
    Sleeper {
        draft: Option<sleeper::Draft>,
        draft_picks: Vec<sleeper::DraftPick>,
        league: sleeper::League,
        matchups: HashMap<usize, Vec<sleeper::Matchup>>,
        owners: HashMap<String, sleeper::User>,
        players: HashMap<String, sleeper::Player>,
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
    pub fn with_team(self, id: ExternalId) -> Self {
        match self {
            League::Sleeper {
                draft,
                draft_picks,
                league,
                matchups,
                owners,
                players,
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
    pub fn record(&self) -> String {
        match self {
            League::Sleeper {
                rosters, user_id, ..
            } => match rosters
                .iter()
                .find(|(_, r)| r.owner_id.is_some() && &r.owner_id == user_id)
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
            League::Yahoo {
                standings, team, ..
            } => match standings
                .teams
                .iter()
                .find(|t| Some(t.team_id) == team.as_ref().map(|t| t.team_id))
            {
                Some(team_w_standings) => match &team_w_standings.team_standings {
                    Some(team_standings) => format!(
                        "{}-{}-{}",
                        team_standings.outcome_totals.wins,
                        team_standings.outcome_totals.losses,
                        team_standings.outcome_totals.ties
                    ),
                    None => "n/a".to_string(),
                },
                None => "n/a".to_string(),
            },
        }
    }
    pub fn team_name(&self) -> String {
        match self {
            League::Sleeper {
                owners, user_id, ..
            } => user_id
                .as_ref()
                .map(|user_id| match owners.get(user_id) {
                    Some(owner) => owner.team_name(),
                    None => format!("Unknown User ({})", user_id),
                })
                .unwrap_or(format!("Unknown User (None)")),
            League::Yahoo { team, .. } => team.as_ref().map(|t| t.name.clone()).unwrap_or_default(),
        }
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
            League::Sleeper { league, .. } => LeagueSettings::from_sleeper(&league),
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
                draft, draft_picks, ..
            } => draft.as_ref().map(|d| Draft::from((d, draft_picks))),
            League::Yahoo {
                draft_results,
                league,
                players,
                settings,
                ..
            } => match draft_results.len() {
                0 => None,
                _ => Some(Draft {
                    draft_type: settings.draft_type.clone(),
                    draft_id: "N/A".to_string(),
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
                                        _id: String::from("N/A"),
                                        name: String::from("N/A"),
                                        positions: vec![],
                                        irl_team: None,
                                    },
                                    roster_id: p.team_key.to_string(),
                                }
                            }
                        })
                        .collect(),
                }),
            },
        }
    }
    pub fn rosters(&self) -> Vec<Roster> {
        match self {
            League::Sleeper { rosters, .. } => {
                rosters.iter().map(|r| r.1).map(Roster::from).collect()
            }
            League::Yahoo { rosters, .. } => rosters.iter().map(Roster::from).collect(),
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
        match self {
            League::Sleeper { league, .. } => write!(
                f,
                "[{season}] {name} (Sleeper) - {status}    Record: {record}",
                season = league.season,
                name = league.name,
                status = match league.status {
                    sleeper_fantasy_rs::LeagueStatus::PreDraft => "predraft",
                    sleeper_fantasy_rs::LeagueStatus::Drafting => "drafting",
                    sleeper_fantasy_rs::LeagueStatus::InSeason => "inseason",
                    sleeper_fantasy_rs::LeagueStatus::Complete => "complete",
                },
                record = self.record(),
            ),
            League::Yahoo { league, .. } => write!(
                f,
                "[{}] {} (Yahoo) - {}       Record: {}",
                league.season,
                league.name,
                match league.is_finished {
                    Some(is_finished) => match is_finished {
                        true => "complete",
                        false => "in-progress",
                    },
                    None => "incomplete",
                },
                self.record(),
            ),
        }
    }
}

#[derive(Debug)]
pub struct LeagueSettings {
    /// The week number in the season when the playoffs begin.
    pub has_playoffs: bool,
    pub playoff_start_week: u32,
    pub scoring: Option<ScoringSettings>,
}
impl LeagueSettings {
    pub fn from_yahoo(yahoo_settings: &yahoo_fantasy_rs::Settings) -> Self {
        LeagueSettings {
            has_playoffs: yahoo_settings.uses_playoff == 1,
            playoff_start_week: yahoo_settings.playoff_start_week,
            scoring: None, // TODO: implement me
        }
    }
    pub fn from_sleeper(sleeper_league: &sleeper_fantasy_rs::League) -> Self {
        let playoff_start_week = sleeper_league.settings.playoff_week_start as u32;

        LeagueSettings {
            has_playoffs: playoff_start_week > 0, //TODO: make sure this is correct logic
            playoff_start_week,
            scoring: match ScoringSettings::from_sleeper(&sleeper_league.scoring_settings) {
                Ok(s) => Some(s),
                Err(_) => None, // TODO: handle me
            },
        }
    }
    pub fn formatted_settings(&self) -> Vec<(String, String)> {
        vec![
            (String::from("has_playoffs"), self.has_playoffs.to_string()),
            (
                String::from("playoff_start_week"),
                self.playoff_start_week.to_string(),
            ),
            // (String::from(""), ),
        ]
    }
}

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
impl From<sleeper_fantasy_rs::Player> for Player {
    fn from(value: sleeper_fantasy_rs::Player) -> Self {
        let name = value.to_string();
        Player {
            _id: value.player_id,
            name,
            positions: value.position.map(|v| vec![v]).unwrap_or_default(),
            irl_team: value.team,
        }
    }
}
impl From<&sleeper_fantasy_rs::Player> for Player {
    fn from(value: &sleeper_fantasy_rs::Player) -> Self {
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
impl From<yahoo_fantasy_rs::Player> for Player {
    fn from(value: yahoo_fantasy_rs::Player) -> Self {
        Player {
            _id: value.player_key.to_string(),
            name: value.name.full,
            positions: vec![value.display_position.to_string()],
            irl_team: Some(value.editorial_team_abbr),
        }
    }
}
impl From<&yahoo_fantasy_rs::Player> for Player {
    fn from(value: &yahoo_fantasy_rs::Player) -> Self {
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

#[derive(Debug, Clone)]
pub struct Roster {
    pub id: String,
    pub player_ids: Vec<String>,
}
impl From<&sleeper_fantasy_rs::Roster> for Roster {
    fn from(value: &sleeper_fantasy_rs::Roster) -> Self {
        Roster {
            id: value.roster_id.to_string(),
            player_ids: value.players.clone(),
        }
    }
}
impl From<(&u32, &yahoo_fantasy_rs::Roster)> for Roster {
    fn from(value: (&u32, &yahoo_fantasy_rs::Roster)) -> Self {
        Roster {
            id: value.0.to_string(),
            player_ids: value
                .1
                .players
                .iter()
                .map(|p| p.player_id.to_string())
                .collect(),
        }
    }
}

#[derive(Debug)]
pub enum ScoringSettings {
    Football {
        // Passing Stats
        pass_2pt: f32,
        pass_int: f32,
        pass_yd: f32,
        pass_td: f32,

        // Receiving Stats
        receptions: f32,
        rec_2pt: f32,
        rec_td: f32,
        rec_yd: f32,

        // Rushing Stats
        rush_2pt: f32,
        rush_td: f32,
        rush_yd: f32,

        // Kicking Stats
        kick_fgmiss: f32,
        kick_fgm_0_19: f32,
        kick_fgm_20_29: f32,
        kick_fgm_30_39: f32,
        kick_fgm_40_49: f32,
        kick_fgm_50p: f32,
        kick_xpm: f32,
        kick_xpmiss: f32,

        // Other Offensive Stats
        fum: f32,
        fum_lost: f32,

        // DST Stats
        pts_allow_0: f32,
        pts_allow_1_6: f32,
        pts_allow_7_13: f32,
        pts_allow_14_20: f32,
        pts_allow_21_27: f32,
        pts_allow_28_34: f32,
        pts_allow_35p: f32,
        int: f32,
        sack: f32,
        safe: f32,
        def_td: f32,
        def_kr_td: f32,
        fum_rec: f32,
        fum_rec_td: f32,
        fum_forced: f32,
        pass_int_td: f32,
        def_st_td: f32,
        def_st_fum_rec: f32,
        def_st_ff: f32,
        st_fum_rec: f32,
        st_ff: f32,
        st_td: f32,
        blk_kick: f32,
        kr_td: f32,
        def_pr_td: f32,
        pr_td: f32,

        // Milestones (cumulative stats)
        bonus_rec_yd_200: f32,
        bonus_rush_yd_200: f32,
        bonus_pass_yd_400: f32,

        // Bonus Stats
        bonus_rec_te: f32,       // TODO: What is this?
        bonus_pass_td_40p: f32,  // points per pass_td of 40+ yards
        bonus_rush_td_40p: f32,  // points per rush_td of 40+ yards
        bonus_pass_cmp_40p: f32, // points per pass_cmp of 40+ yards
        bonus_rec_td_40p: f32,   // points per rec_td of 40+ yards
        bonus_rush_40p: f32,     // points per rush of 40+ yards
        bonus_rec_40p: f32,      // points per reception of 40+ yards
    },
    #[allow(dead_code)]
    Unknown,
}
impl ScoringSettings {
    pub fn from_sleeper(
        sleeper_settings: &sleeper_fantasy_rs::ScoringSettings,
    ) -> Result<Self, std::io::Error> {
        match sleeper_settings {
            sleeper_fantasy_rs::ScoringSettings::Football {
                pass_2pt,
                pass_int,
                pass_yd,
                pass_td,
                rec,
                rec_2pt,
                rec_td,
                rec_yd,
                rush_2pt,
                rush_td,
                rush_yd,
                fgmiss,
                fgm_0_19,
                fgm_20_29,
                fgm_30_39,
                fgm_40_49,
                fgm_50p,
                xpm,
                xpmiss,
                fum,
                fum_lost,
                pts_allow_0,
                pts_allow_1_6,
                pts_allow_7_13,
                pts_allow_14_20,
                pts_allow_21_27,
                pts_allow_28_34,
                pts_allow_35p,
                int,
                sack,
                safe,
                def_td,
                def_kr_td,
                fum_rec,
                fum_rec_td,
                ff,
                pass_int_td,
                def_st_td,
                def_st_fum_rec,
                def_st_ff,
                st_fum_rec,
                st_ff,
                st_td,
                blk_kick,
                kr_td,
                def_pr_td,
                pr_td,
                bonus_rec_yd_200,
                pass_td_40p,
                bonus_rush_yd_200,
                bonus_pass_yd_400,
                rush_td_40p,
                bonus_rec_te,
                pass_cmp_40p,
                rec_td_40p,
                rush_40p,
                rec_40p,
                other: _,
            } => Ok(ScoringSettings::Football {
                pass_2pt: pass_2pt.clone(),
                pass_int: pass_int.clone(),
                pass_yd: pass_yd.clone(),
                pass_td: pass_td.clone(),
                receptions: rec.clone(),
                rec_2pt: rec_2pt.clone(),
                rec_td: rec_td.clone(),
                rec_yd: rec_yd.clone(),
                rush_2pt: rush_2pt.clone(),
                rush_td: rush_td.clone(),
                rush_yd: rush_yd.clone(),
                kick_fgmiss: fgmiss.clone(),
                kick_fgm_0_19: fgm_0_19.clone(),
                kick_fgm_20_29: fgm_20_29.clone(),
                kick_fgm_30_39: fgm_30_39.clone(),
                kick_fgm_40_49: fgm_40_49.clone(),
                kick_fgm_50p: fgm_50p.clone(),
                kick_xpm: xpm.clone(),
                kick_xpmiss: xpmiss.clone(),
                fum: fum.clone(),
                fum_lost: fum_lost.clone(),
                pts_allow_0: pts_allow_0.clone(),
                pts_allow_1_6: pts_allow_1_6.clone(),
                pts_allow_7_13: pts_allow_7_13.clone(),
                pts_allow_14_20: pts_allow_14_20.clone(),
                pts_allow_21_27: pts_allow_21_27.clone(),
                pts_allow_28_34: pts_allow_28_34.clone(),
                pts_allow_35p: pts_allow_35p.clone(),
                int: int.clone(),
                sack: sack.clone(),
                safe: safe.clone(),
                def_td: def_td.clone(),
                def_kr_td: def_kr_td.clone(),
                fum_rec: fum_rec.clone(),
                fum_rec_td: fum_rec_td.clone(),
                fum_forced: ff.clone(),
                pass_int_td: pass_int_td.clone(),
                def_st_td: def_st_td.clone(),
                def_st_fum_rec: def_st_fum_rec.clone(),
                def_st_ff: def_st_ff.clone(),
                st_fum_rec: st_fum_rec.clone(),
                st_ff: st_ff.clone(),
                st_td: st_td.clone(),
                blk_kick: blk_kick.clone(),
                kr_td: kr_td.clone(),
                def_pr_td: def_pr_td.clone(),
                pr_td: pr_td.clone(),
                bonus_rec_yd_200: bonus_rec_yd_200.clone(),
                bonus_rush_yd_200: bonus_rush_yd_200.clone(),
                bonus_pass_yd_400: bonus_pass_yd_400.clone(),
                bonus_rec_te: bonus_rec_te.clone(),
                bonus_pass_td_40p: pass_td_40p.clone(),
                bonus_rush_td_40p: rush_td_40p.clone(),
                bonus_pass_cmp_40p: pass_cmp_40p.clone(),
                bonus_rec_td_40p: rec_td_40p.clone(),
                bonus_rush_40p: rush_40p.clone(),
                bonus_rec_40p: rec_40p.clone(),
            }),
            sleeper_fantasy_rs::ScoringSettings::Basketball {
                pts: _,
                ast: _,
                reb: _,
                fgmi: _,
                ftmi: _,
                tpm: _,
                stl: _,
                blk: _,
                tf: _,
                dd: _,
                td: _,
                ff: _,
                to: _,
                bonus_ast_15p: _,
                bonus_pt_40p: _,
                bonus_pt_50p: _,
                bonus_reb_20p: _,
                other: _,
            } => todo!(),
        }
    }

    pub fn formatted_settings(&self) -> Vec<(String, String)> {
        match self {
            ScoringSettings::Football {
                pass_2pt,
                pass_int,
                pass_yd,
                pass_td,
                receptions,
                rec_2pt,
                rec_td,
                rec_yd,
                rush_2pt,
                rush_td,
                rush_yd,
                kick_fgmiss,
                kick_fgm_0_19,
                kick_fgm_20_29,
                kick_fgm_30_39,
                kick_fgm_40_49,
                kick_fgm_50p,
                kick_xpm,
                kick_xpmiss,
                fum,
                fum_lost,
                pts_allow_0,
                pts_allow_1_6,
                pts_allow_7_13,
                pts_allow_14_20,
                pts_allow_21_27,
                pts_allow_28_34,
                pts_allow_35p,
                int,
                sack,
                safe,
                def_td,
                def_kr_td,
                fum_rec,
                fum_rec_td,
                fum_forced,
                pass_int_td,
                def_st_td,
                def_st_fum_rec,
                def_st_ff,
                st_fum_rec,
                st_ff,
                st_td,
                blk_kick,
                kr_td,
                def_pr_td,
                pr_td,
                bonus_rec_yd_200,
                bonus_rush_yd_200,
                bonus_pass_yd_400,
                bonus_rec_te,
                bonus_pass_td_40p,
                bonus_rush_td_40p,
                bonus_pass_cmp_40p,
                bonus_rec_td_40p,
                bonus_rush_40p,
                bonus_rec_40p,
            } => vec![
                (String::from("pass_2pt"), pass_2pt),
                (String::from("pass_int"), pass_int),
                (String::from("pass_yd"), pass_yd),
                (String::from("pass_td"), pass_td),
                (String::from("receptions"), receptions),
                (String::from("rec_2pt"), rec_2pt),
                (String::from("rec_td"), rec_td),
                (String::from("rec_yd"), rec_yd),
                (String::from("rush_2pt"), rush_2pt),
                (String::from("rush_td"), rush_td),
                (String::from("rush_yd"), rush_yd),
                (String::from("kick_fgmiss"), kick_fgmiss),
                (String::from("kick_fgm_0_19"), kick_fgm_0_19),
                (String::from("kick_fgm_20_29"), kick_fgm_20_29),
                (String::from("kick_fgm_30_39"), kick_fgm_30_39),
                (String::from("kick_fgm_40_49"), kick_fgm_40_49),
                (String::from("kick_fgm_50p"), kick_fgm_50p),
                (String::from("kick_xpm"), kick_xpm),
                (String::from("kick_xpmiss"), kick_xpmiss),
                (String::from("fum"), fum),
                (String::from("fum_lost"), fum_lost),
                (String::from("pts_allow_0"), pts_allow_0),
                (String::from("pts_allow_1_6"), pts_allow_1_6),
                (String::from("pts_allow_7_13"), pts_allow_7_13),
                (String::from("pts_allow_14_20"), pts_allow_14_20),
                (String::from("pts_allow_21_27"), pts_allow_21_27),
                (String::from("pts_allow_28_34"), pts_allow_28_34),
                (String::from("pts_allow_35p"), pts_allow_35p),
                (String::from("int"), int),
                (String::from("sack"), sack),
                (String::from("safe"), safe),
                (String::from("def_td"), def_td),
                (String::from("def_kr_td"), def_kr_td),
                (String::from("fum_rec"), fum_rec),
                (String::from("fum_rec_td"), fum_rec_td),
                (String::from("fum_forced"), fum_forced),
                (String::from("pass_int_td"), pass_int_td),
                (String::from("def_st_td"), def_st_td),
                (String::from("def_st_fum_rec"), def_st_fum_rec),
                (String::from("def_st_ff"), def_st_ff),
                (String::from("st_fum_rec"), st_fum_rec),
                (String::from("st_ff"), st_ff),
                (String::from("st_td"), st_td),
                (String::from("blk_kick"), blk_kick),
                (String::from("kr_td"), kr_td),
                (String::from("def_pr_td"), def_pr_td),
                (String::from("pr_td"), pr_td),
                (String::from("bonus_rec_yd_200"), bonus_rec_yd_200),
                (String::from("bonus_rush_yd_200"), bonus_rush_yd_200),
                (String::from("bonus_pass_yd_400"), bonus_pass_yd_400),
                (String::from("bonus_rec_te"), bonus_rec_te),
                (String::from("bonus_pass_td_40p"), bonus_pass_td_40p),
                (String::from("bonus_rush_td_40p"), bonus_rush_td_40p),
                (String::from("bonus_pass_cmp_40p"), bonus_pass_cmp_40p),
                (String::from("bonus_rec_td_40p"), bonus_rec_td_40p),
                (String::from("bonus_rush_40p"), bonus_rush_40p),
                (String::from("bonus_rec_40p"), bonus_rec_40p),
            ]
            .into_iter()
            .filter_map(|(k, v)| match *v != (0.0 as f32) {
                true => Some((k, format!("{:.2}", v))),
                false => None,
            })
            .collect(),
            ScoringSettings::Unknown => vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Transaction {
    Waiver {
        id: String,
        created: DateTime<Utc>,
        updated: DateTime<Utc>,
        player_moves: Vec<PlayerMove>,
    },
    FreeAgent {
        id: String,
        created: DateTime<Utc>,
        updated: DateTime<Utc>,
        player_moves: Vec<PlayerMove>,
    },
    Trade {
        id: String,
        created: DateTime<Utc>,
        updated: DateTime<Utc>,
        player_moves: Vec<PlayerMove>,
        draft_pick_moves: Vec<PickMove>,
    },
    CommissionerAction {
        id: String,
        description: String,
        created: DateTime<Utc>,
        updated: DateTime<Utc>,
        player_moves: Vec<PlayerMove>,
        draft_pick_moves: Vec<PickMove>,
    },
}
impl Transaction {
    pub fn from_yahoo(value: &yahoo_fantasy_rs::Transaction) -> Self {
        let id = value.key();
        let created = DateTime::from_timestamp(*value.timestamp(), 0).unwrap();
        let updated = DateTime::from_timestamp(*value.timestamp(), 0).unwrap();
        let players = value.players();

        let is_waiver_claim = players
            .iter()
            .find(|p| {
                p.transaction_data
                    .as_ref()
                    .map_or(false, |data| match data.r#type {
                        yahoo::TransactionDataType::Add => data.source_type.is_waivers(),
                        yahoo::TransactionDataType::Drop => false,
                        yahoo::TransactionDataType::Trade => false,
                    })
            })
            .is_some();
        let player_moves = players
            .iter()
            .filter_map(|p| PlayerMove::try_from(p).ok())
            .collect::<Vec<_>>();

        let draft_pick_moves = value
            .draft_picks()
            .into_iter()
            .map(PickMove::from)
            .collect();

        match value {
            yahoo_fantasy_rs::Transaction::Commish { .. } => Transaction::CommissionerAction {
                id,
                description: String::new(),
                created,
                updated,
                player_moves,
                draft_pick_moves,
            },
            yahoo_fantasy_rs::Transaction::Trade { .. } => Transaction::Trade {
                id,
                created,
                updated,
                player_moves,
                draft_pick_moves,
            },
            yahoo_fantasy_rs::Transaction::Add { .. }
            | yahoo_fantasy_rs::Transaction::Drop { .. }
            | yahoo_fantasy_rs::Transaction::AddDrop { .. } => {
                if is_waiver_claim {
                    Transaction::Waiver {
                        id,
                        created,
                        updated,
                        player_moves,
                    }
                } else {
                    Transaction::FreeAgent {
                        id,
                        created,
                        updated,
                        player_moves,
                    }
                }
            }
        }
    }
}
impl Transaction {
    pub fn description(&self) -> String {
        match self {
            Transaction::Waiver { .. } => format!("Waiver Claim"),
            Transaction::FreeAgent { .. } => format!("Free Agent Pickup"),
            Transaction::Trade { player_moves, .. } => {
                let teams_involved = player_moves
                    .iter()
                    .flat_map(|m| {
                        let mut roster_ids = HashSet::new();
                        if let RosterSpot::Roster(roster_id) = m.src() {
                            roster_ids.insert(roster_id);
                        }
                        if let RosterSpot::Roster(roster_id) = m.dest() {
                            roster_ids.insert(roster_id);
                        }
                        roster_ids
                    })
                    .collect::<HashSet<_>>();
                format!("{n}-way Trade", n = teams_involved.len())
            }
            Transaction::CommissionerAction { .. } => format!("Commissioner Action"),
        }
    }
    pub fn draft_pick_moves(&self) -> Vec<PickMove> {
        match self {
            Transaction::Trade {
                draft_pick_moves, ..
            } => draft_pick_moves.clone(),
            Transaction::CommissionerAction {
                draft_pick_moves, ..
            } => draft_pick_moves.clone(),
            _ => vec![],
        }
    }
    pub fn player_moves(&self) -> Vec<PlayerMove> {
        match self {
            Transaction::Trade { player_moves, .. } => player_moves.clone(),
            Transaction::CommissionerAction { player_moves, .. } => player_moves.clone(),
            Transaction::Waiver { player_moves, .. } => player_moves.clone(),
            Transaction::FreeAgent { player_moves, .. } => player_moves.clone(),
        }
    }
    pub fn r#type(&self) -> &'static str {
        match self {
            Transaction::Waiver { .. } => "Waiver",
            Transaction::FreeAgent { .. } => "FreeAgent",
            Transaction::Trade { .. } => "Trade",
            Transaction::CommissionerAction { .. } => "CommissionerAction",
        }
    }
    pub fn ts(&self) -> DateTime<Utc> {
        match self {
            Transaction::Waiver { updated, .. } => updated.clone(),
            Transaction::FreeAgent { updated, .. } => updated.clone(),
            Transaction::Trade { updated, .. } => updated.clone(),
            Transaction::CommissionerAction { updated, .. } => updated.clone(),
        }
    }
}
impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{updated}: {_type}",
            updated = self.ts(),
            _type = self.r#type(),
        )
    }
}
impl From<sleeper::Transaction> for Transaction {
    fn from(value: sleeper::Transaction) -> Self {
        let id = value.transaction_id;
        let created = value.created;
        let updated = value.status_updated;

        let adds = value
            .adds
            .iter()
            .map(|(player_id, dest_roster_id)| PlayerMove::Add {
                player_id: player_id.to_string(),
                dest: RosterSpot::Roster(dest_roster_id.to_string()),
                src: match value._type {
                    sleeper::TransactionType::Trade => value
                        .drops
                        .iter()
                        .find(|(pid, _)| player_id == *pid)
                        .map(|(_, r)| RosterSpot::Roster(r.to_string()))
                        .unwrap_or(RosterSpot::FreeAgentPool),
                    sleeper::TransactionType::FreeAgent => RosterSpot::FreeAgentPool,
                    sleeper::TransactionType::Waiver => RosterSpot::WaiverWire,
                    _other => RosterSpot::FreeAgentPool,
                },
            })
            .collect::<Vec<_>>();
        let drops = value
            .drops
            .iter()
            .map(|(player_id, src_roster_id)| PlayerMove::Drop {
                player_id: player_id.to_string(),
                src: RosterSpot::Roster(src_roster_id.to_string()),
                dest: match value._type {
                    sleeper::TransactionType::Trade => value
                        .adds
                        .iter()
                        .find(|(pid, _)| player_id == *pid)
                        .map(|(_, r)| RosterSpot::Roster(r.to_string()))
                        .unwrap_or(RosterSpot::WaiverWire),
                    sleeper::TransactionType::FreeAgent => RosterSpot::WaiverWire,
                    sleeper::TransactionType::Waiver => RosterSpot::WaiverWire,
                    _other => RosterSpot::WaiverWire,
                },
            })
            .collect::<Vec<_>>();
        let mut player_moves = adds;
        player_moves.extend(drops);

        let draft_pick_moves = value
            .draft_picks
            .into_iter()
            .map(PickMove::from)
            .collect::<Vec<_>>();

        match value._type {
            sleeper::TransactionType::Waiver => Transaction::Waiver {
                id,
                created,
                updated,
                player_moves,
            },
            sleeper::TransactionType::FreeAgent => Transaction::Waiver {
                id,
                created,
                updated,
                player_moves,
            },
            sleeper::TransactionType::Trade => Transaction::Trade {
                id,
                created,
                updated,
                player_moves,
                draft_pick_moves,
            },
            sleeper::TransactionType::Commissioner => Transaction::CommissionerAction {
                id,
                created,
                updated,
                player_moves: player_moves,
                description: "".to_string(),
                draft_pick_moves,
            },
        }
    }
}
impl From<&sleeper::Transaction> for Transaction {
    fn from(value: &sleeper::Transaction) -> Self {
        Transaction::from(value.clone())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlayerMove {
    Add {
        player_id: String,
        dest: RosterSpot,
        src: RosterSpot,
    },
    Drop {
        player_id: String,
        dest: RosterSpot,
        src: RosterSpot,
    },
    Trade {
        player_id: String,
        dest: RosterSpot,
        src: RosterSpot,
    },
}
impl PlayerMove {
    pub fn add(&self) -> bool {
        match self {
            PlayerMove::Add { .. } => true,
            _ => false,
        }
    }
    pub fn drop(&self) -> bool {
        match self {
            PlayerMove::Drop { .. } => true,
            _ => false,
        }
    }
    pub fn dest(&self) -> RosterSpot {
        match self {
            PlayerMove::Add { dest, .. } => dest.clone(),
            PlayerMove::Drop { dest, .. } => dest.clone(),
            PlayerMove::Trade { dest, .. } => dest.clone(),
        }
    }
    pub fn src(&self) -> RosterSpot {
        match self {
            PlayerMove::Add { src, .. } => src.clone(),
            PlayerMove::Drop { src, .. } => src.clone(),
            PlayerMove::Trade { dest, .. } => dest.clone(),
        }
    }
}
impl TryFrom<&yahoo::Player> for PlayerMove {
    type Error = &'static str;

    fn try_from(value: &yahoo::Player) -> Result<Self, Self::Error> {
        value
            .transaction_data
            .as_ref()
            .map_or(Err("player missing transaction_data"), |data| {
                let player_id = value.player_key.to_string();
                let dest = RosterSpot::from_dest(data).ok_or("unable to determine add source")?;
                let src = RosterSpot::from_source(data).ok_or("unable to determine add source")?;
                Ok(match data.r#type {
                    yahoo::TransactionDataType::Add => PlayerMove::Add {
                        player_id,
                        dest,
                        src,
                    },
                    yahoo::TransactionDataType::Drop => PlayerMove::Drop {
                        player_id,
                        dest,
                        src,
                    },
                    yahoo::TransactionDataType::Trade => PlayerMove::Trade {
                        player_id,
                        dest,
                        src,
                    },
                })
            })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RosterSpot {
    WaiverWire,
    FreeAgentPool,
    Roster(String),
}
impl RosterSpot {
    pub fn from_source(data: &yahoo::TransactionData) -> Option<RosterSpot> {
        RosterSpot::try_from((data, true)).ok()
    }

    pub fn from_dest(data: &yahoo::TransactionData) -> Option<RosterSpot> {
        RosterSpot::try_from((data, false)).ok()
    }
}
impl TryFrom<(&yahoo::TransactionData, bool)> for RosterSpot {
    type Error = &'static str;

    fn try_from((data, src): (&yahoo::TransactionData, bool)) -> Result<Self, Self::Error> {
        match if src {
            &data.source_type
        } else {
            &data.destination_type
        } {
            yahoo::TransactionSourceType::Team => {
                if src {
                    data.source_team_key
                        .as_ref()
                        .map(|team_key| RosterSpot::Roster(team_key.to_string()))
                        .ok_or("unable to determine RosterSpot from yahoo TransactionData source")
                } else {
                    data.destination_team_key
                        .as_ref()
                        .map(|team_key| RosterSpot::Roster(team_key.to_string()))
                        .ok_or("unable to determine RosterSpot from yahoo TransactionData source")
                }
            }
            yahoo::TransactionSourceType::Waivers => Ok(RosterSpot::WaiverWire),
            yahoo::TransactionSourceType::FreeAgents => Ok(RosterSpot::FreeAgentPool),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PickMove {
    pub src_team_id: String,
    pub dest_team_id: String,
    pub orig_team_id: String,
    pub round: usize,
}
impl From<&yahoo::Pick> for PickMove {
    fn from(value: &yahoo::Pick) -> Self {
        PickMove {
            src_team_id: value.source_team_key.to_string(),
            dest_team_id: value.destination_team_key.to_string(),
            orig_team_id: value.original_team_key.to_string(),
            round: value.round.clone(),
        }
    }
}
impl From<yahoo::Pick> for PickMove {
    fn from(value: yahoo::Pick) -> Self {
        PickMove::from(&value)
    }
}
impl From<&sleeper::DraftPickMove> for PickMove {
    fn from(value: &sleeper::DraftPickMove) -> Self {
        PickMove {
            src_team_id: value.previous_owner_id.to_string(),
            dest_team_id: value.owner_id.to_string(),
            orig_team_id: value.roster_id.to_string(),
            round: value.round.clone(),
        }
    }
}
impl From<sleeper::DraftPickMove> for PickMove {
    fn from(value: sleeper::DraftPickMove) -> Self {
        PickMove::from(&value)
    }
}

#[derive(Debug, Clone)]
pub struct Scoreboard {
    pub matchups: Vec<Matchup>,
}
impl Scoreboard {
    pub fn from_sleeper(
        week: u32,
        sleeper_matchups: Vec<sleeper::Matchup>,
        rosters: &HashMap<u8, sleeper::Roster>,
        owners: &HashMap<String, sleeper::User>,
    ) -> Self {
        let mut matchups: Vec<(sleeper::Matchup, sleeper::Matchup)> =
            Vec::with_capacity(sleeper_matchups.len() / 2);
        let mut wip: Vec<sleeper::Matchup> = vec![];
        let mut orphaned: Vec<sleeper::Matchup> = vec![];

        for matchup in sleeper_matchups {
            if let Some(index) = wip
                .iter()
                .position(|wip_entry| matchup.matchup_id == wip_entry.matchup_id)
            {
                let tmp = wip.remove(index);
                matchups.push((tmp, matchup));
            } else if let None = matchup.matchup_id {
                orphaned.push(matchup);
            } else {
                wip.push(matchup);
            }
        }

        Scoreboard {
            matchups: matchups
                .into_iter()
                .map(|m| Matchup::from_sleeper(week, m, rosters, owners))
                .collect(),
        }
    }
}
impl From<yahoo::Scoreboard> for Scoreboard {
    fn from(value: yahoo::Scoreboard) -> Self {
        Scoreboard {
            matchups: value
                .matchups
                .matchups
                .into_iter()
                .map(|m| Matchup::from_yahoo(value.week, m))
                .collect(),
        }
    }
}
impl From<&yahoo::Scoreboard> for Scoreboard {
    fn from(value: &yahoo::Scoreboard) -> Self {
        value.clone().into()
    }
}

#[derive(Debug, Clone)]
pub struct Matchup {
    pub id: MatchupId,
    pub week: u32,
    pub status: Option<String>,
    pub left: MatchupSide,
    pub right: Option<MatchupSide>,
}
impl Matchup {
    pub fn from_sleeper(
        week: u32,
        matchup: (sleeper::Matchup, sleeper::Matchup),
        rosters: &HashMap<u8, sleeper::Roster>,
        owners: &HashMap<String, sleeper::User>,
    ) -> Self {
        // yahoo scoreboard doesn't include the player info
        Matchup {
            id: matchup
                .0
                .matchup_id
                .map_or(MatchupId::None, |id| MatchupId::Id(id.to_string())),
            week,
            status: None,
            left: MatchupSide::from_sleeper(matchup.0, rosters, owners),
            right: Some(MatchupSide::from_sleeper(matchup.1, rosters, owners)),
        }
    }
    pub fn from_yahoo(week: u32, mut matchup: yahoo::Matchup) -> Self {
        let team_1 = matchup.teams.remove(0);
        let team_2 = if matchup.teams.is_empty() {
            None
        } else {
            Some(matchup.teams.remove(0))
        };
        let mut id = team_1.team_id.to_string();
        if let Some(team_2) = &team_2 {
            id.push('.');
            id.push_str(&team_2.team_id.to_string());
        }

        Matchup {
            id: MatchupId::Id(id),
            week,
            status: Some(matchup.status),
            left: MatchupSide::from_yahoo(team_1),
            right: team_2.map(MatchupSide::from_yahoo),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MatchupSide {
    pub roster_id: String,
    pub team_name: String,
    pub owner_name: String,
    pub score: f32,
    pub players: Vec<MatchupPlayer>,
}
impl MatchupSide {
    pub fn from_sleeper(
        matchup: sleeper::Matchup,
        rosters: &HashMap<u8, sleeper::Roster>,
        owners: &HashMap<String, sleeper::User>,
    ) -> Self {
        // yahoo scoreboard doesn't include the player info

        let owner = rosters
            .get(&matchup.roster_id)
            .map(|r| r.owner_id.clone())
            .flatten()
            .map(|id| owners.get(&id))
            .flatten();
        let team_name = owner.map_or(String::from(""), |o| o.team_name());
        let owner_name = owner.map_or(String::from(""), |o| o.display_name.clone());

        MatchupSide {
            roster_id: matchup.roster_id.to_string(),
            team_name,
            owner_name,
            score: matchup.points,
            players: MatchupPlayer::from_sleeper_vec(
                matchup.players,
                matchup.players_points,
                matchup.starters,
            ),
        }
    }
    pub fn from_yahoo(team: yahoo::Team) -> Self {
        let owner_name = team
            .managers()
            .iter()
            .map(|m| m.nickname.clone())
            .collect::<Vec<_>>()
            .join(" & ");
        // yahoo scoreboard doesn't include the player info
        MatchupSide {
            roster_id: team.team_id.to_string(),
            team_name: team.name,
            owner_name,
            score: team
                .team_points
                .map(|p| p.total)
                .flatten()
                .unwrap_or_default(),
            players: vec![],
        }
    }
}

#[derive(Debug, Clone)]
pub enum MatchupId {
    None,
    Id(String),
}
impl fmt::Display for MatchupId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MatchupId::None => write!(f, "None"),
            MatchupId::Id(s) => write!(f, "{s}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MatchupPlayer {
    pub player_id: ExternalId,
    pub player_score: f32,
    pub is_starter: bool,
}
impl MatchupPlayer {
    pub fn new<P: Into<ExternalId>>(player_id: P, score: f32, is_starter: bool) -> Self {
        MatchupPlayer {
            player_id: player_id.into(),
            player_score: score,
            is_starter,
        }
    }
    pub fn from_sleeper_vec(
        player_ids: Vec<String>,
        mut players_pts: HashMap<String, f32>,
        starters: Vec<String>,
    ) -> Vec<Self> {
        player_ids
            .into_iter()
            .map(|id| {
                let player_pts = players_pts.remove(&id).unwrap_or_default();
                let is_starter = starters.contains(&id);
                MatchupPlayer::new(
                    ExternalId::new(crate::Platform::Sleeper, id),
                    player_pts,
                    is_starter,
                )
            })
            .collect()
    }
    pub fn from_yahoo(player: &yahoo::Player) -> Self {
        // yahoo scoreboard doesn't include the player info
        MatchupPlayer::new(&player.player_key, 0.0, false)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        self as core, Transaction,
        data::{PlayerMove, RosterSpot},
    };
    use chrono::DateTime;
    use sleeper_fantasy_rs as sleeper;
    use yahoo_fantasy_rs::{self as yahoo, LeagueKey};

    #[test]
    fn test_convert_yahoo_transaction() {
        let yahoo_txn = yahoo_txn_waiver_add();
        let core_txn = core::Transaction::from_yahoo(&yahoo_txn);
        debug_assert_eq!("Waiver", core_txn.r#type(), "expect correct type");
        debug_assert_eq!(
            DateTime::from_timestamp(1475048003, 0).unwrap(),
            core_txn.ts(),
            "expect correct timestamp"
        );
        debug_assert_eq!(
            Transaction::Waiver {
                id: String::from("359.l.564503.tr.130"),
                created: DateTime::parse_from_rfc3339("2016-09-28T07:33:23Z")
                    .unwrap()
                    .to_utc(),
                updated: DateTime::parse_from_rfc3339("2016-09-28T07:33:23Z")
                    .unwrap()
                    .to_utc(),
                player_moves: vec![PlayerMove::Add {
                    player_id: "359.p.28493".to_string(),
                    dest: RosterSpot::Roster("123.l.564503.t.5".to_string()),
                    src: RosterSpot::WaiverWire
                }],
            },
            core_txn,
            "expect correct add players"
        );
        // debug_assert_eq!(
        //     HashMap::new(),
        //     core_txn.drops,
        //     "expect correct drop players"
        // );
    }

    #[test]
    fn test_convert_sleeper_transaction() {
        let sleeper_txn = sleeper_txn_commish_drop();
        let core_txn = core::Transaction::from(&sleeper_txn);
        debug_assert_eq!(
            "CommissionerAction",
            core_txn.r#type(),
            "expect correct type"
        );
        debug_assert_eq!(
            DateTime::from_timestamp_millis(1552060656943).unwrap(),
            core_txn.ts(),
            "expect correct timestamp"
        );
        debug_assert_eq!(
            Transaction::CommissionerAction {
                id: String::from("848755191285740706"),
                created: DateTime::parse_from_rfc3339("2019-03-08T15:57:36.943Z")
                    .unwrap()
                    .to_utc(),
                updated: DateTime::parse_from_rfc3339("2019-03-08T15:57:36.943Z")
                    .unwrap()
                    .to_utc(),
                player_moves: vec![PlayerMove::Drop {
                    player_id: "1903".to_string(),
                    src: RosterSpot::Roster("12".to_string()),
                    dest: RosterSpot::WaiverWire
                }],
                draft_pick_moves: vec![],
                description: String::from(""),
            },
            core_txn,
            "expect correct add players"
        );
        // debug_assert_eq!(
        //     HashMap::new(),
        //     core_txn.drops,
        //     "expect correct drop players"
        // );
    }

    fn sleeper_txn_commish_drop() -> sleeper::Transaction {
        sleeper::Transaction {
            status: sleeper::TransactionStatus::Complete,
            _type: sleeper::TransactionType::Commissioner,
            metadata: HashMap::new(),
            created: DateTime::from_timestamp_millis(1552060656943).unwrap(),
            settings: HashMap::new(),
            leg: 1,
            draft_picks: vec![],
            creator: "727598467304811923".to_string(),
            transaction_id: "848755191285740706".to_string(),
            adds: HashMap::new(),
            drops: HashMap::from([("1903".to_string(), 12)]),
            consenter_ids: None,
            roster_ids: vec![12],
            status_updated: DateTime::from_timestamp_millis(1552060656943).unwrap(),
            waiver_budget: vec![],
        }
    }

    fn yahoo_txn_waiver_add() -> yahoo::Transaction {
        yahoo::Transaction::Add {
            transaction_key: "359.l.564503.tr.130".to_string(),
            transaction_id: 130,
            status: "successful".to_string(),
            timestamp: 1475048003,
            players: Some(
                [yahoo::Player {
                    player_key: yahoo::PlayerKey {
                        game_key: yahoo::GameKey::Id(359),
                        player_id: 28493,
                    },
                    player_id: 28493,
                    name: yahoo::PlayerName {
                        full: "Jamison Crowder".to_string(),
                        first: "Jamison".to_string(),
                        last: "Crowder".to_string(),
                        ascii_first: "Jamison".to_string(),
                        ascii_last: "Crowder".to_string(),
                    },
                    url: None,
                    status: None,
                    status_full: None,
                    editorial_player_key: None,
                    editorial_team_key: None,
                    editorial_team_full_name: None,
                    editorial_team_abbr: "Was".to_string(),
                    editorial_team_url: None,
                    bye_weeks: None,
                    is_keeper: None,
                    uniform_number: None,
                    display_position: yahoo::PlayerPositions::One(yahoo::PlayerPosition::WR),
                    headshot: None,
                    image_url: None,
                    is_undroppable: None,
                    position_type: yahoo::PositionType::O,
                    has_player_notes: None,
                    player_notes_last_timestamp: None,
                    transaction_data: Some(yahoo::TransactionData {
                        r#type: yahoo::TransactionDataType::Add,
                        source_type: yahoo::TransactionSourceType::Waivers,
                        source_team_key: None,
                        source_team_name: None,
                        destination_type: yahoo::TransactionSourceType::Team,
                        destination_team_key: Some(yahoo::TeamKey {
                            league_key: LeagueKey::new(yahoo_fantasy_rs::GameKey::Id(123), 564503),
                            team_id: 5,
                        }),
                        destination_team_name: Some("SmeTeam".to_string()),
                    }),
                }]
                .to_vec(),
            ),
        }
    }
}
