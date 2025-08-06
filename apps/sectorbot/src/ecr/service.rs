use std::{
    collections::{HashMap, HashSet},
    u64,
};

use serde::{Deserialize, Serialize};
use sleeper_fantasy_rs::{Client, Draft, DraftPick, League, Player, Roster, Sport, User};

use crate::ecr::keepers::{Keepable, KeepablesReport, get_keeper_status, get_player_history};

const LEAGUE_ID: &str = "2489066589662560510";
const SPORT: Sport = Sport::NFL;

pub struct Service {
    league_id: String,
    sport: Sport,
    sleeper_cli: Client,
}
impl Service {
    pub fn new() -> Self {
        Service {
            league_id: LEAGUE_ID.to_string(),
            sport: SPORT,
            sleeper_cli: Client::new(None),
        }
    }

    async fn fetch_keepable_context<T: Into<String>>(
        &self,
        league_id: T,
        sport: &Sport,
    ) -> Result<KeepablesContext, ECRError> {
        let league_id: String = league_id.into();
        let players_future = self.sleeper_cli.fetch_all_players(&sport);
        let league_future = self.sleeper_cli.get_league(league_id.clone());
        let rosters_future = self.sleeper_cli.get_rosters_for_league(league_id.clone());
        let users_future = self.sleeper_cli.get_users_for_league(league_id.clone());
        let league = league_future
            .await
            .map_err(|e| ECRError::SleeperError(e.to_string()))?;
        let draft_history = self.fetch_all_draft_history(league.clone()).await?;
        let mut players = players_future
            .await
            .map_err(|e| ECRError::SleeperError(e.to_string()))?;
        let rosters = rosters_future
            .await
            .map_err(|e| ECRError::SleeperError(e.to_string()))?;
        let users = users_future
            .await
            .map_err(|e| ECRError::SleeperError(e.to_string()))?;

        // look through drafts and rosters (later transactions?????)
        // and get a set of player_ids to include in players_map
        reduce_players_map(&mut players, Some(&draft_history), Some(&rosters));

        // let current_season: i32 = league
        //     .season
        //     .parse()
        //     .map_err(|e: ParseIntError| ECRError::Unknown(e.to_string()))?;

        Ok(KeepablesContext {
            current_league: league,
            draft_history: draft_history
                .into_iter()
                .map(|e| (e.draft.season.clone(), e))
                .collect(),
            players,
            rosters,
            users,
        })
    }

    async fn get_keepables(&self, ctx: KeepablesContext) -> Result<KeepablesReport, ECRError> {
        let mut keepables = vec![];
        let last_season: i32 = &ctx.current_league.season.parse().unwrap() - 1;

        let draft_result = ctx
            .draft_history
            .get(last_season.to_string().as_str())
            .unwrap();
        for roster in ctx.rosters {
            let user = ctx.users.iter().find(|u| u.user_id == roster.owner_id);
            if user.is_none() {
                panic!("fuck");
            }
            let user = user.unwrap();
            println!("User: {}", user.display_name);
            for player_id in roster.players {
                let player = ctx.players.get(player_id.as_str());
                if player.is_none() {
                    panic!("fuck");
                }
                let player = player.unwrap();
                let keeper_status = get_keeper_status(player, &draft_result.picks);
                if keeper_status.is_keeper || keeper_status.was_drafted {
                    let player_history = get_player_history(player, &ctx.draft_history);
                    keepables.push(Keepable {
                        ownerId: user.user_id.clone(),
                        ownerName: user.display_name.clone(),
                        wasKeeperLastYear: keeper_status.is_keeper,
                        playerId: player.player_id.clone(),
                        playerName: player.to_string(),
                        position: player.position.clone().unwrap_or_default(),
                        round: keeper_status.draft_round.unwrap_or_default(),
                        history: player_history,
                    });
                    println!(
                        "\t\t{name} [${position}] [{team}] Keeper[{is_keeper}] Rd[{draft_round}]",
                        name = player.to_string(),
                        position = player.position.clone().unwrap_or_default(),
                        team = player.team.clone().unwrap_or_default(),
                        is_keeper = keeper_status.is_keeper,
                        draft_round = keeper_status.draft_round.unwrap_or_default(),
                    );
                    // println!("\t\t\t${player_history:#?}")
                }
            }
        }

        // try fancy sort of keepables
        keepables.sort_by_key(|k| {
            k.ownerId
                .parse::<u64>()
                .unwrap_or(u64::MIN)
                .checked_add(k.round as u64)
                .unwrap_or(u64::MAX)
        });

        Ok(KeepablesReport { keepables })
    }

    async fn fetch_all_draft_history(&self, league: League) -> Result<Vec<DraftResult>, ECRError> {
        let mut drafts: Vec<DraftResult> = vec![];
        let mut draft_futures = vec![];
        let mut current_league = league;
        while let Some(league_id) = current_league.previous_league_id {
            if league_id == "" || league_id == "0" {
                // end of the line
                break;
            }
            println!("Getting league details for {league_id}");
            current_league = self
                .sleeper_cli
                .get_league(league_id)
                .await
                .map_err(|e| ECRError::SleeperError(e.to_string()))?;
            println!("Getting draft details for {}", current_league.league_id);
            draft_futures.push(self.fetch_draft_picks_for_league(current_league.league_id));
        }
        for f in draft_futures {
            drafts.push(f.await?);
        }
        drafts.sort_by_key(|d| d.draft.season.to_string());

        Ok(drafts)
    }

    async fn fetch_draft_picks_for_league<T: Into<String>>(
        &self,
        league_id: T,
    ) -> Result<DraftResult, ECRError> {
        let league_id: String = league_id.into();
        let mut drafts: Vec<Draft> = self
            .sleeper_cli
            .get_drafts_for_league(league_id.clone())
            .await
            .map_err(|e| ECRError::SleeperError(e.to_string()))?
            .into_iter()
            .filter(|d| d.status == "complete" && d.start_time.is_some() && d.last_picked.is_some())
            .collect();
        drafts.sort_by_key(|d| d.last_picked.unwrap());
        drafts.reverse();

        match drafts.len() {
            0 => Err(ECRError::NotFound(format!(
                "draft results for league: {league_id}",
            ))),
            1 => {
                let draft = drafts.remove(0);
                let picks = self
                    .sleeper_cli
                    .get_draft_picks(&draft.draft_id)
                    .await
                    .map_err(|e| ECRError::SleeperError(e.to_string()))?;
                Ok(DraftResult { draft, picks })
            }
            n => {
                let draft = drafts.remove(0);
                let picks = self
                    .sleeper_cli
                    .get_draft_picks(&draft.draft_id)
                    .await
                    .map_err(|e| ECRError::SleeperError(e.to_string()))?;
                Ok(DraftResult { draft, picks })
            } //Err(ECRError::UndeterminableDraft(league_id)),
        }
    }
}

/// This function will look through provided sleeper league data structures and
/// find all player_ids referenced, place them into a HashSet and then reduce
/// the provided player_map to reduce memory footprint and such.
///
/// In my limited testing I was able to reduce the overall storage footprint of
/// a 5 year league history context from 14MB to 1MB on disk. This was
/// effectively a player_map size of 11388 reduced down to 476 entries.
fn reduce_players_map(
    players: &mut HashMap<String, Player>,
    draft_results: Option<&Vec<DraftResult>>,
    rosters: Option<&Vec<Roster>>,
    // transactions: Option<&Vec<Transaction>>,
) {
    let mut player_ids: HashSet<String> = HashSet::new();
    if let Some(draft_results) = draft_results {
        player_ids.extend(
            draft_results
                .iter()
                .flat_map(|d| d.picks.iter().map(|p| p.player_id.clone())),
        );
    }
    if let Some(rosters) = rosters {
        player_ids.extend(rosters.iter().flat_map(|r| r.players.clone()));
    }
    // if let Some(transactions) = transactions {
    //     player_ids.extend(transactions.iter().flat_map(|t| t.adds.keys().));
    // }

    // reduce the player map
    players.retain(|key, _| player_ids.contains(key));
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DraftResult {
    pub draft: Draft,
    pub picks: Vec<DraftPick>,
}

#[derive(thiserror::Error, Debug)]
pub enum ECRError {
    #[error("Sleeper Encountered an error {0}")]
    SleeperError(String),

    #[error("Sleeper element not found {0}")]
    NotFound(String),

    #[error("Unable to determine correct draft for league {0}")]
    UndeterminableDraft(String),

    #[error("Unknown error {0}")]
    Unknown(String),
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use sleeper_fantasy_rs::{Roster, User};
    use tokio::fs;

    use crate::{
        ecr::service::{KeepablesContext, LEAGUE_ID, SPORT, Service},
        fantasypros,
    };

    #[tokio::test]
    async fn test_me() {
        let svc = Service::new();

        let keepables_context: KeepablesContext =
            svc.fetch_keepable_context(LEAGUE_ID, &SPORT).await.unwrap();
        fs::write(
            "keepables_ctx.json",
            serde_json::to_vec(&keepables_context).expect("serialization should aslways succeed"),
        )
        .await
        .expect("should not fail");
        // let keepables_context: KeepablesContext = serde_json::from_str(
        //     fs::read_to_string("keepables_ctx.json")
        //         .await
        //         .unwrap()
        //         .as_str(),
        // )
        // .unwrap();

        assert_eq!(6, keepables_context.draft_history.len());
        assert_eq!(476, keepables_context.players.len());
        assert_eq!(12, keepables_context.rosters.len());
        assert_eq!(12, keepables_context.users.len());

        let fantasy_pros_cli = fantasypros::Client::new();

        let ecr_rankings = fantasy_pros_cli
            .fetch_ecr_half_ppr()
            .await
            .expect("asdfasdfasdfasdf");
        let num_teams = keepables_context.current_league.settings.num_teams;

        let keepables = svc.get_keepables(keepables_context).await.unwrap();
        for keepable in keepables.keepables {
            let mut cost = keepable.cost("2025".to_string());
            //if cost == Some("ADP".to_string()) {
            // lookup current ranking
            if let Some(fp_player) = ecr_rankings
                .players
                .iter()
                .find(|p| p.player_name == keepable.playerName)
            {
                let round = fp_player.rank_ecr / num_teams + 1;
                let pick_in_round = fp_player.rank_ecr % num_teams;
                cost = Some(format!(
                    "({cost:?}) ADP ~ {round}.{pick_in_round}",
                    cost = cost.unwrap()
                ))
            }
            //}
            let cost = if let Some(cost) = cost {
                format!(" ({cost})")
            } else {
                String::new()
            };
            println!("{} - {} {cost}", keepable.ownerName, keepable.playerName);
        }
        assert!(false)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct KeepablesContext {
    pub current_league: League,
    pub draft_history: HashMap<String, DraftResult>,
    pub players: HashMap<String, Player>,
    pub rosters: Vec<Roster>,
    pub users: Vec<User>,
}
