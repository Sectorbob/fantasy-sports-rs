use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use core::fmt;
use sleeper_fantasy_rs::{self as sleeper, custom::FantasyMatchup};
use std::collections::HashMap;
use yahoo_fantasy_rs as yahoo;

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
                let filtered = drafts
                    .iter()
                    .filter(|d| d.start_time.is_some())
                    .collect::<Vec<&sleeper_fantasy_rs::Draft>>();
                if filtered.len() == 1 {
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
        matchups: HashMap<usize, Vec<FantasyMatchup>>,
        owners: HashMap<String, sleeper::User>,
        players: HashMap<String, sleeper::Player>,
        rosters: Vec<sleeper::Roster>,
        transactions: Vec<sleeper::Transaction>,
        user_id: Option<String>,
    },
    Yahoo {
        draft_results: Vec<yahoo::DraftResult>,
        league: yahoo::League,
        matchups: HashMap<usize, Vec<FantasyMatchup>>,
        players: HashMap<yahoo::PlayerKey, yahoo::Player>,
        rosters: HashMap<u32, yahoo::Roster>,
        settings: yahoo::Settings,
        standings: yahoo::Standings,
        team: Option<yahoo::Team>,
    },
}
impl League {
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
                .find(|r| r.owner_id.is_some() && &r.owner_id == user_id)
            {
                Some(roster) => format!(
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
                .find(|r| r.owner_id.is_some() && user_id.is_some() && &r.owner_id == user_id)
            {
                Some(roster) => roster
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
    pub fn matchups(&self) -> &HashMap<usize, Vec<FantasyMatchup>> {
        match self {
            League::Sleeper { matchups, .. } => matchups,
            League::Yahoo { matchups, .. } => matchups, // Yahoo leagues do not have matchups in the same way
        }
    }
    pub fn players(&self) -> HashMap<String, Player> {
        match self {
            League::Sleeper { players, .. } => players
                .iter()
                .map(|(id, p)| (id.clone(), Player::from(p)))
                .collect::<HashMap<String, Player>>(),
            League::Yahoo {
                draft_results: _,
                league: _,
                matchups: _,
                players: _,
                rosters: _,
                settings: _,
                standings: _,
                team: _,
            } => HashMap::new(),
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
                            if let Some(player) = players.get(&p.player_key) {
                                DraftPick {
                                    round: p.round,
                                    pick: p.pick,
                                    overall_pick: p.pick,
                                    player: Player::from(player),
                                    roster_id: p.team_key.to_string(),
                                }
                            } else {
                                DraftPick {
                                    round: p.round,
                                    pick: p.pick,
                                    overall_pick: p.pick,
                                    player: Player {
                                        _id: p.player_key.to_string(),
                                        name: p.player_key.to_string(),
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
            League::Sleeper { rosters, .. } => rosters.iter().map(Roster::from).collect(),
            League::Yahoo { rosters, .. } => rosters.iter().map(Roster::from).collect(),
        }
    }
    pub fn transactions(&self) -> Vec<Transaction> {
        match self {
            League::Sleeper { transactions, .. } => {
                transactions.iter().map(Transaction::from_sleeper).collect()
            }
            League::Yahoo {
                draft_results: _,
                league: _,
                matchups: _,
                players: _,
                rosters: _,
                settings: _,
                standings: _,
                team: _,
            } => vec![],
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
impl From<yahoo_fantasy_rs::League> for League {
    fn from(value: yahoo_fantasy_rs::League) -> Self {
        todo!()
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

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Transaction {
    pub id: String,
    pub transaction_type: String,
    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub status: String,
}
impl Transaction {
    pub fn from_sleeper(sleeper_transaction: &sleeper_fantasy_rs::Transaction) -> Self {
        Transaction {
            id: sleeper_transaction.transaction_id.clone(),
            transaction_type: sleeper_transaction._type.to_string(),
            created: sleeper_transaction.created.clone(),
            updated: sleeper_transaction.status_updated.clone(),
            status: sleeper_transaction.status.to_string(),
        }
    }
    pub fn from_yahoo(yahoo_transaction: &yahoo_fantasy_rs::Transaction) -> Self {
        match yahoo_transaction {
            yahoo_fantasy_rs::Transaction::Commish {
                transaction_key,
                transaction_id: _,
                status,
                timestamp,
            } => Transaction {
                id: transaction_key.clone(),
                transaction_type: "commish".to_string(),
                created: DateTime::from_timestamp_millis(*timestamp).unwrap(),
                updated: DateTime::from_timestamp_millis(*timestamp).unwrap(),
                status: status.to_string(),
            },
            yahoo_fantasy_rs::Transaction::Add {
                transaction_key,
                transaction_id: _,
                status,
                timestamp,
                players: _,
            } => Transaction {
                id: transaction_key.clone(),
                transaction_type: "add".to_string(),
                created: DateTime::from_timestamp_millis(*timestamp).unwrap(),
                updated: DateTime::from_timestamp_millis(*timestamp).unwrap(),
                status: status.to_string(),
            },
            yahoo_fantasy_rs::Transaction::Drop {
                transaction_key,
                transaction_id: _,
                status,
                timestamp,
                players: _,
            } => Transaction {
                id: transaction_key.clone(),
                transaction_type: "drop".to_string(),
                created: DateTime::from_timestamp_millis(*timestamp).unwrap(),
                updated: DateTime::from_timestamp_millis(*timestamp).unwrap(),
                status: status.to_string(),
            },
            yahoo_fantasy_rs::Transaction::AddDrop {
                transaction_key,
                transaction_id: _,
                status,
                timestamp,
                players: _,
            } => Transaction {
                id: transaction_key.clone(),
                transaction_type: "add/drop".to_string(),
                created: DateTime::from_timestamp_millis(*timestamp).unwrap(),
                updated: DateTime::from_timestamp_millis(*timestamp).unwrap(),
                status: status.to_string(),
            },
            yahoo_fantasy_rs::Transaction::Trade {
                transaction_key,
                transaction_id: _,
                status,
                timestamp,
                players: _,
            } => Transaction {
                id: transaction_key.clone(),
                transaction_type: "trade".to_string(),
                created: DateTime::from_timestamp_millis(*timestamp).unwrap(),
                updated: DateTime::from_timestamp_millis(*timestamp).unwrap(),
                status: status.to_string(),
            },
            yahoo_fantasy_rs::Transaction::Other {
                type_name: _,
                transaction_key,
                transaction_id: _,
                status,
                timestamp,
                players: _,
                rest: _,
            } => Transaction {
                id: transaction_key.clone(),
                transaction_type: "other".to_string(),
                created: DateTime::from_timestamp_millis(*timestamp).unwrap(),
                updated: DateTime::from_timestamp_millis(*timestamp).unwrap(),
                status: status.to_string(),
            },
        }
    }
}
impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{updated}: {_type} [{status}]",
            updated = self.updated,
            _type = self.transaction_type,
            status = self.status
        )
    }
}
