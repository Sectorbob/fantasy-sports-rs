use std::{
    collections::{HashMap, HashSet},
    u64,
};

use chrono::{DateTime, Utc, serde::ts_milliseconds};
use serde::{Deserialize, Serialize};
use sleeper_fantasy_rs::{Draft, DraftPick, League, Player, Roster, Sport, User};

use crate::{
    ecr::keepers::{Keepable, KeepablesReport, get_keeper_status, get_player_history},
    fantasypros,
};

pub const LEAGUE_ID: &str = "2489066589662560510";
pub const SPORT: Sport = Sport::NFL;

pub struct Service {
    league_id: String,
    sport: Sport,
    sleeper_cli: sleeper_fantasy_rs::Client,
    fantasypros_cli: fantasypros::Client,
}
impl Service {
    pub fn new() -> Self {
        Service {
            league_id: LEAGUE_ID.to_string(),
            sport: SPORT,
            sleeper_cli: sleeper_fantasy_rs::Client::new(None),
            fantasypros_cli: fantasypros::Client::new(),
        }
    }

    pub async fn fetch_keepable_context<T: Into<String>>(
        &self,
        league_id: T,
        sport: &Sport,
    ) -> Result<KeepablesContext, ECRError> {
        let ecr_response_future = self.fantasypros_cli.fetch_ecr_half_ppr();
        let league_id: String = league_id.into();
        let players_future = self.sleeper_cli.fetch_all_players(&sport);
        let league_future = self.sleeper_cli.get_league(league_id.clone());
        let rosters_future = self.sleeper_cli.get_rosters_for_league(league_id.clone());
        let users_future = self.sleeper_cli.get_users_for_league(league_id.clone());
        let league = league_future.await?;
        let draft_history = self.fetch_all_draft_history(league.clone()).await?;
        let mut players = players_future.await?;
        let rosters = rosters_future.await?;
        let users = users_future.await?;
        let ecr_response = ecr_response_future.await?;

        // look through drafts and rosters (later transactions?????)
        // and get a set of player_ids to include in players_map
        reduce_players_map(&mut players, Some(&draft_history), Some(&rosters));

        Ok(KeepablesContext {
            fetched_at: Utc::now(), //.with_timezone(&chrono_tz::America::New_York),
            current_league: league,
            draft_history: draft_history
                .into_iter()
                .map(|e| (e.draft.season.clone(), e))
                .collect(),
            players,
            rosters,
            users,
            rankings: ecr_response,
        })
    }

    pub fn get_keepables(&self, ctx: &KeepablesContext) -> Result<KeepablesReport, ECRError> {
        let num_teams = ctx.current_league.settings.num_teams;
        let mut keepables = vec![];
        let current_season = ctx.current_league.season.clone();
        let last_season: i32 = current_season
            .parse()
            .map_err(|e| {
                ECRError::Unknown(format!(
                    "league.season is invalid ({}): {e}",
                    ctx.current_league.season
                ))
            })
            .map(|v: i32| v - 1)?;

        let draft_result = ctx
            .draft_history
            .get(last_season.to_string().as_str())
            .ok_or(ECRError::NotFound(format!(
                "unable to find draft_history for {}",
                last_season
            )))?;
        for roster in &ctx.rosters {
            let user = ctx
                .users
                .iter()
                .find(|u| u.user_id == roster.owner_id)
                .map(User::clone)
                .unwrap_or_else(|| {
                    log::warn!("for some reason, we could not find the user info {user_id} for roster({roster_id})",user_id =roster.owner_id, roster_id = roster.roster_id);
                    User {
                    username: None,
                    user_id: roster.owner_id.clone(),
                    display_name: format!("Unknown User({})", roster.owner_id),
                    avatar: None,
                    league_details: None,
                    other_data: HashMap::new(),
                }});
            log::debug!("User: {}", user.display_name);
            for player_id in &roster.players {
                let (player_name, player_pos, player_team) =
                    ctx.players.get(player_id.as_str()).map_or_else(
                        || {
                            log::warn!("Unable to find player {}", player_id);
                            (
                                format!("Unknown Player ({player_id})"),
                                String::from(""),
                                String::from(""),
                            )
                        },
                        |player| {
                            (
                                player.to_string(),
                                player.position.clone().unwrap_or_default(),
                                player.team.clone().unwrap_or_default(),
                            )
                        },
                    );
                let keeper_status = get_keeper_status(player_id.as_str(), &draft_result.picks);
                if keeper_status.is_keeper || keeper_status.was_drafted {
                    let player_history = get_player_history(player_id.as_str(), &ctx.draft_history);
                    log::debug!(
                        "{name} [${position}] [{team}] Keeper[{is_keeper}] Rd[{draft_round}]",
                        name = player_name,
                        position = player_pos,
                        team = player_team,
                        is_keeper = keeper_status.is_keeper,
                        draft_round = keeper_status.draft_round.unwrap_or_default(),
                    );
                    let player_rank = ctx
                        .rankings
                        .players
                        .iter()
                        .find(|p| p.player_name == player_name)
                        .map(|pr| {
                            let round = pr.rank_ecr / num_teams + 1;
                            let pick_in_round = pr.rank_ecr % num_teams;
                            format!("{round}.{pick_in_round}")
                        });
                    keepables.push(Keepable {
                        owner_id: user.user_id.clone(),
                        owner_name: user.display_name.clone(),
                        was_keeper_last_year: keeper_status.is_keeper,
                        player_id: player_id.clone(),
                        player_name,
                        position: player_pos,
                        round: keeper_status.draft_round.unwrap_or_default(),
                        history: player_history,
                        player_rank,
                    });
                }
            }
        }

        // try fancy sort of keepables
        keepables.sort_by_key(|k| {
            k.owner_id
                .parse::<u64>()
                .unwrap_or(u64::MIN)
                .checked_add(k.round as u64)
                .unwrap_or(u64::MAX)
        });

        Ok(KeepablesReport {
            season: current_season,
            keepables,
        })
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
            log::debug!("Getting league details for {league_id}");
            current_league = self.sleeper_cli.get_league(league_id).await?;
            log::debug!("Getting draft details for {}", current_league.league_id);
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
            .await?
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
                let picks = self.sleeper_cli.get_draft_picks(&draft.draft_id).await?;
                Ok(DraftResult { draft, picks })
            }
            _ => {
                let draft = drafts.remove(0);
                let picks = self.sleeper_cli.get_draft_picks(&draft.draft_id).await?;
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

#[derive(Serialize, Deserialize, Debug)]
pub struct KeepablesContext {
    #[serde(with = "ts_milliseconds")]
    pub fetched_at: DateTime<Utc>,
    pub current_league: League,
    pub draft_history: HashMap<String, DraftResult>,
    pub players: HashMap<String, Player>,
    pub rosters: Vec<Roster>,
    pub users: Vec<User>,
    pub rankings: fantasypros::ECRResponse,
}

#[derive(thiserror::Error, Debug)]
pub enum ECRError {
    #[error("failed to fetch from sleeper {0}")]
    SleeperError(#[from] sleeper_fantasy_rs::Error),

    #[error("Sleeper element not found {0}")]
    NotFound(String),

    #[error("Unable to determine correct draft for league {0}")]
    UndeterminableDraft(String),

    #[error("error fetching from fantasypros: {0}")]
    FantasyProsError(#[from] fantasypros::FantasyProsError),

    #[error("Unknown error {0}")]
    Unknown(String),
}

#[cfg(test)]
mod tests {
    use flate2::read::GzDecoder;
    use std::error::Error;
    use std::fs::File;
    use std::io::{self, BufReader};
    use tokio::fs::{self};

    use crate::ecr::keepers::Keepable;
    use crate::ecr::service::{KeepablesContext, LEAGUE_ID, SPORT, Service};

    #[tokio::test]
    async fn test_get_keepables_report() {
        let svc = Service::new();

        let keepables_context: KeepablesContext = load_keepables_context_testdata()
            .await
            .expect("failed to load context test data file");
        // let keepables_context: KeepablesContext = fetch_and_save_ctx(&svc, "keepables_ctx.json")
        //     .await
        //     .expect("failed to fetch live context");

        assert_eq!(6, keepables_context.draft_history.len());
        assert_eq!(476, keepables_context.players.len());
        assert_eq!(12, keepables_context.rosters.len());
        assert_eq!(12, keepables_context.users.len());

        // do it
        let keepables = svc
            .get_keepables(&keepables_context)
            .expect("report calculation failed");

        // // some report debugging
        // println!("{}", keepables.text_report());
        // assert!(false);
        debug_assert_eq!(
            151,
            keepables.keepables.len(),
            "expecting 151 total keepables in the league"
        );
        let sectorbob_keepables: Vec<&Keepable> = keepables
            .keepables
            .iter()
            .filter(|k| k.owner_name == "Sectorbob")
            .collect();
        debug_assert_eq!(
            15,
            sectorbob_keepables.len(),
            "expecting 15 total keepables for Sectorbob's roster"
        );
        debug_assert_eq!(
            Some(String::from("ADP")),
            sectorbob_keepables
                .iter()
                .find(|k| k.player_name == "Ja'Marr Chase")
                .map(|k| k.cost("2025"))
                .flatten(),
            "expecting Ja'Marr Chase's keeper cost to be at ADP"
        );
        debug_assert_eq!(
            Some(String::from("2")),
            sectorbob_keepables
                .iter()
                .find(|k| k.player_name == "Malik Nabers")
                .map(|k| k.cost("2025"))
                .flatten(),
            "expecting Malik Nabers' keeper cost to be a second round pick"
        );
        debug_assert_eq!(
            Some(String::from("10")),
            sectorbob_keepables
                .iter()
                .find(|k| k.player_name == "Caleb Williams")
                .map(|k| k.cost("2025"))
                .flatten(),
            "expecting Caleb Williams' keeper cost to be a 10th round pick"
        );
    }

    async fn load_keepables_context_testdata() -> Result<KeepablesContext, io::Error> {
        let file_path = "testdata/ecr/keepables_ctx.json.gz"; // Replace with your file path
        let file = File::open(file_path)?;
        let decoder = GzDecoder::new(BufReader::new(file));
        serde_json::from_reader(BufReader::new(decoder)).map_err(|json_error| json_error.into())
    }

    #[allow(dead_code)]
    async fn fetch_and_save_ctx<T: Into<String>>(
        svc: &Service,
        outfile: T,
    ) -> Result<KeepablesContext, Box<dyn Error>> {
        let keepables_context: KeepablesContext =
            svc.fetch_keepable_context(LEAGUE_ID, &SPORT).await?;
        let outfile = outfile.into();
        fs::write(outfile, serde_json::to_vec(&keepables_context)?).await?;
        Ok(keepables_context)
    }
}
