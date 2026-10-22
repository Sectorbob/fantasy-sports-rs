use crate::{ExternalId, Player, PlayerPosition};
use chrono::{DateTime, Utc};
use fleaflicker_fantasy_rs as fleaflicker;
use sleeper_fantasy_rs as sleeper;
use std::collections::{HashMap, HashSet};
use std::fmt;
use yahoo_fantasy_rs as yahoo;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Draft {
    pub draft_type: String,
    pub draft_id: String,
    pub league_id: String,
    pub picks: Vec<DraftPick>,
    pub draft_order: Option<Vec<Option<String>>>,
}
impl Draft {
    pub fn check_for_valid_draft(drafts: &Vec<sleeper::Draft>) -> Option<sleeper::Draft> {
        match drafts.len() {
            0 => None,
            1 => Some(drafts[0].clone()),
            _ => {
                // prioritize drafts with Some(start_time)
                let mut filtered = drafts
                    .iter()
                    .filter(|d| d.start_time.is_some())
                    .collect::<Vec<&sleeper::Draft>>();
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
        &sleeper::Draft,
        &Vec<sleeper::DraftPick>,
        &HashMap<u8, sleeper::Roster>,
    )> for Draft
{
    fn from(
        (draft, picks, rosters): (
            &sleeper::Draft,
            &Vec<sleeper::DraftPick>,
            &HashMap<u8, sleeper::Roster>,
        ),
    ) -> Self {
        Draft {
            draft_type: draft._type.to_string(),
            draft_id: draft.draft_id.clone(),
            league_id: draft.league_id.clone(),
            picks: picks.iter().map(DraftPick::from).collect(),
            draft_order: draft.draft_order.as_ref().map(|order| {
                (1..draft.settings.rounds)
                    .into_iter()
                    .map(|slot| {
                        order
                            .iter()
                            .find(|(_, v)| *v == &slot)
                            .map(|(owner_id, _)| {
                                rosters
                                    .iter()
                                    .find(|(_, r)| {
                                        if let Some(id) = &r.owner_id {
                                            owner_id == id
                                        } else {
                                            false
                                        }
                                    })
                                    .map(|(_, r)| r.roster_id.to_string())
                            })
                    })
                    .flatten()
                    .collect()
            }),
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
impl From<&sleeper::DraftPick> for DraftPick {
    fn from(value: &sleeper::DraftPick) -> Self {
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
                positions: vec![PlayerPosition::from(&value.metadata.position)],
                irl_team: Some(value.metadata.team.clone()),
            },
            roster_id: value.roster_id.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Roster {
    pub id: String,
    pub team_name: String,
    pub owner_name: String,
    pub player_ids: Vec<String>,
}
impl From<&fleaflicker::Roster> for Roster {
    fn from(value: &fleaflicker::Roster) -> Self {
        let owner = value
            .team
            .owners
            .as_ref()
            .map(|owners| owners.first())
            .flatten();
        Roster {
            id: value.team.id.clone().unwrap().to_string(),
            team_name: value.team.name.clone().unwrap(),
            owner_name: owner.unwrap().display_name.clone(),
            player_ids: value
                .players
                .iter()
                .map(|p| p.pro_player.id.to_string())
                .collect(),
        }
    }
}

impl From<(&sleeper::Roster, &HashMap<String, sleeper::User>)> for Roster {
    fn from((roster, owners): (&sleeper::Roster, &HashMap<String, sleeper::User>)) -> Self {
        let roster_id = &roster.roster_id;
        let owner = roster.owner_id.as_ref().map(|id| owners.get(id)).flatten();
        Roster {
            id: roster_id.to_string(),
            team_name: owner.map_or(format!("Team {roster_id}"), sleeper::User::team_name),
            owner_name: owner.map_or(format!("Team {roster_id}"), |o| o.display_name.clone()),
            player_ids: roster.players.clone(),
        }
    }
}
impl From<(&u32, &yahoo::Roster, &Vec<yahoo::Team>)> for Roster {
    fn from((roster_id, roster, teams): (&u32, &yahoo::Roster, &Vec<yahoo::Team>)) -> Self {
        let team = teams.iter().find(|t| &t.team_id == roster_id);
        Roster {
            id: roster_id.to_string(),
            team_name: team.map_or(format!("Roster {roster_id}"), |t| t.name.clone()),
            owner_name: team.map_or(format!("Roster {roster_id}"), |t| {
                t.managers()
                    .iter()
                    .map(|m| m.nickname.clone())
                    .collect::<Vec<_>>()
                    .join(",")
            }),
            player_ids: roster
                .players
                .iter()
                .map(|p| p.player_id.to_string())
                .collect(),
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
    pub fn from_yahoo(value: &yahoo::Transaction) -> Self {
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
            yahoo::Transaction::Commish { .. } => Transaction::CommissionerAction {
                id,
                description: String::new(),
                created,
                updated,
                player_moves,
                draft_pick_moves,
            },
            yahoo::Transaction::Trade { .. } => Transaction::Trade {
                id,
                created,
                updated,
                player_moves,
                draft_pick_moves,
            },
            yahoo::Transaction::Add { .. }
            | yahoo::Transaction::Drop { .. }
            | yahoo::Transaction::AddDrop { .. } => {
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
impl From<&fleaflicker::LeagueActivityItem> for Transaction {
    fn from(value: &fleaflicker::LeagueActivityItem) -> Self {
        todo!()
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
    pub fn is_add<'a>(self: &'a &Self) -> bool {
        match self {
            PlayerMove::Add { .. } => true,
            _ => false,
        }
    }
    pub fn is_drop<'a>(self: &'a &Self) -> bool {
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
    pub fn player_id(&self) -> String {
        match self {
            PlayerMove::Add { player_id, .. } => player_id.clone(),
            PlayerMove::Drop { player_id, .. } => player_id.clone(),
            PlayerMove::Trade { player_id, .. } => player_id.clone(),
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

#[derive(Debug, Clone)]
pub struct Standings {
    pub entries: Vec<StandingsEntry>,
}

#[derive(Debug, Clone)]
pub struct StandingsEntry {
    pub roster_id: String,
    pub team_name: String,
    pub owner_name: String,
    pub owner_id: String,
    pub wins: u64,
    pub losses: u64,
    pub ties: u64,
}

#[cfg(test)]
mod tests {
    use crate as core;
    use chrono::DateTime;
    use sleeper_fantasy_rs as sleeper;
    use std::collections::HashMap;
    use yahoo_fantasy_rs as yahoo;

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
            core::Transaction::Waiver {
                id: String::from("359.l.564503.tr.130"),
                created: DateTime::parse_from_rfc3339("2016-09-28T07:33:23Z")
                    .unwrap()
                    .to_utc(),
                updated: DateTime::parse_from_rfc3339("2016-09-28T07:33:23Z")
                    .unwrap()
                    .to_utc(),
                player_moves: vec![core::PlayerMove::Add {
                    player_id: "359.p.28493".to_string(),
                    dest: core::RosterSpot::Roster("123.l.564503.t.5".to_string()),
                    src: core::RosterSpot::WaiverWire
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
            core::Transaction::CommissionerAction {
                id: String::from("338372223848552968"),
                created: DateTime::parse_from_rfc3339("2019-03-08T15:57:36.943Z")
                    .unwrap()
                    .to_utc(),
                updated: DateTime::parse_from_rfc3339("2019-03-08T15:57:36.943Z")
                    .unwrap()
                    .to_utc(),
                player_moves: vec![core::PlayerMove::Drop {
                    player_id: "1903".to_string(),
                    src: core::RosterSpot::Roster("12".to_string()),
                    dest: core::RosterSpot::WaiverWire
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
            creator: "862237889002904882".to_string(),
            transaction_id: "338372223848552968".to_string(),
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
                            league_key: yahoo::LeagueKey::new(yahoo::GameKey::Id(123), 564503),
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
