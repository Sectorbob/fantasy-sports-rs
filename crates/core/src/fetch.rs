use crate::{ExternalId, Platform, data as core, player_cache::PlayerCache, sleeper_utils};
use futures::{TryStreamExt, stream::FuturesUnordered};
use sleeper_fantasy_rs::{self as sleeper};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    str::FromStr,
};
use yahoo_fantasy_rs::{self as yahoo};

pub struct LeagueAccessor {
    sleeper_cli: Option<sleeper::Client>,
    players_cache: Option<PlayerCache>,
    yahoo_cli: Option<yahoo::Client>,
    cache_dir: Option<PathBuf>,
}
impl LeagueAccessor {
    pub fn new() -> Self {
        LeagueAccessor {
            sleeper_cli: None,
            players_cache: None,
            yahoo_cli: None,
            cache_dir: None,
        }
    }
    pub fn with_cache_dir<T: Into<PathBuf>>(mut self, cache_dir: T) -> Self {
        self.cache_dir = Some(cache_dir.into());
        self
    }
    pub async fn init(&mut self) -> Result<(), Error> {
        let (sleeper_cache_dir, yahoo_cache_dir, player_cache) = match &self.cache_dir {
            Some(cache_dir) => (
                Some(cache_dir.join("sleeper")),
                Some(cache_dir.join("yahoo")),
                Some(cache_dir.join("players")),
            ),
            None => (None, None, None),
        };
        let player_cache =
            player_cache.map(|player_cache| PlayerCache::new_with_path(&player_cache));

        let sleeper_cli = match sleeper_cache_dir {
            Some(sleeper_cache_dir) => sleeper::Client::new_with_cache(None, sleeper_cache_dir),
            None => sleeper::Client::new(None),
        };
        let mut yahoo_cli = yahoo::Client::new_from_env()
            .inspect_err(|err| log::warn!("failed to setup yahoo cli: {err}"))
            .ok();
        if let Some(cli) = yahoo_cli {
            let tmp = cli.with_token_file::<String>(None).await.map_err(|err| {
                yahoo::Error::UnknownError(format!("failed to load token file: {err}"))
            })?;
            tmp.refresh_token().await?;
            yahoo_cli = Some(tmp)
        }
        if let Some(yahoo_cache_dir) = yahoo_cache_dir {
            if let Some(cli) = yahoo_cli {
                yahoo_cli = Some(cli.with_cache_dir(yahoo_cache_dir));
            }
        }
        self.players_cache = player_cache;
        self.sleeper_cli = Some(sleeper_cli);
        self.yahoo_cli = yahoo_cli;
        Ok(())
    }

    pub fn sleeper_enabled(&self) -> bool {
        self.sleeper_cli.is_some()
    }

    pub fn yahoo_enabled(&self) -> bool {
        self.yahoo_cli.is_some()
    }

    pub async fn fetch_league_context(&self, id: &ExternalId) -> Result<core::League, Error> {
        match &id.platform {
            Platform::Sleeper => {
                if let Some(cli) = &self.sleeper_cli {
                    fetch_sleeper_league_context_by_id(
                        cli,
                        self.players_cache.clone().as_ref(),
                        id.id.clone(),
                    )
                    .await
                } else {
                    Err(Error::new("sleeper cli not configured"))
                }
            }
            Platform::Yahoo => {
                if let Some(cli) = &self.yahoo_cli {
                    fetch_yahoo_league_context(
                        cli,
                        self.players_cache.clone().as_ref(),
                        yahoo::LeagueKey::from_str(&id.id).map_err(|err| {
                            Error::DevError(format!("unable to parse yahoo league key: {err}"))
                        })?,
                    )
                    .await
                } else {
                    Err(Error::new("yahoo cli not configured"))
                }
            }
            other => Err(Error::new(format!("{other} is not supported"))),
        }
    }
}

pub(crate) async fn fetch_sleeper_league_context_by_id<T: Into<String>>(
    cli: &sleeper::Client,
    player_cache: Option<&PlayerCache>,
    league_id: T,
) -> Result<core::League, Error> {
    let sleeper_league = cli.get_league(league_id.into()).await?;
    fetch_sleeper_league_context(cli, player_cache, &sleeper_league).await
}

pub(crate) async fn fetch_sleeper_league_context(
    cli: &sleeper::Client,
    optional_player_cache: Option<&PlayerCache>,
    sleeper_league: &sleeper::League,
) -> Result<core::League, Error> {
    let mut optional_players_future = if optional_player_cache.is_none() {
        Some(cli.fetch_all_players(&sleeper_league.sport))
    } else {
        None
    };
    let owners_future = cli.get_users_for_league(&sleeper_league.league_id);
    let rosters_future = cli.get_rosters_for_league(&sleeper_league.league_id);
    let (owners_result, rosters_result) = tokio::join!(owners_future, rosters_future);
    let owners = owners_result?
        .into_iter()
        .map(|owner| (owner.user_id.clone(), owner))
        .collect::<HashMap<String, sleeper::User>>();
    let rosters = rosters_result?;

    let matchups = match sleeper_utils::get_league_matchups(&sleeper_league, &cli).await {
        Ok(matchups) => matchups,
        Err(e) => {
            eprintln!(
                "Error fetching matchups for league {}: {:?}",
                sleeper_league.league_id, e
            );
            HashMap::new()
        }
    };

    let drafts = match cli
        .get_drafts_for_league(sleeper_league.league_id.clone())
        .await
    {
        Ok(d) => d,
        Err(err) => {
            log::error!(
                "failed to fetch drafts for sleeper league {}: {:?}",
                sleeper_league.league_id,
                err
            );
            vec![]
        }
    };
    let draft: Option<sleeper::Draft> = core::Draft::check_for_valid_draft(&drafts);
    let mut draft_picks: Vec<sleeper::DraftPick> = vec![];
    if let Some(draft) = &draft {
        draft_picks = match cli.get_draft_picks(draft.draft_id.clone()).await {
            Ok(picks) => picks,
            Err(err) => {
                log::error!(
                    "failed to fetch draft picks for sleeper draft {}: {:?}",
                    draft.draft_id,
                    err
                );
                vec![]
            }
        }
    }

    let transactions: Vec<sleeper::Transaction> =
        match sleeper_utils::get_league_transactions(cli, &sleeper_league).await {
            Ok(txns) => txns,
            Err(err) => {
                log::error!(
                    "Failed to load transactions for league {}: {:?}",
                    sleeper_league.league_id,
                    err
                );
                vec![]
            }
        };

    // Obtain set of all player_ids used by this league
    let mut player_id_set: HashSet<String> = HashSet::new();
    rosters.iter().for_each(|r| {
        r.players.iter().for_each(|pid| {
            player_id_set.insert(pid.clone());
        })
    });
    matchups.iter().for_each(|(_, fmups)| {
        fmups.iter().for_each(|fmup| {
            fmup.left.players().into_iter().for_each(|owned_player_id| {
                player_id_set.insert(owned_player_id);
            });
            fmup.right
                .players()
                .into_iter()
                .for_each(|owned_player_id| {
                    player_id_set.insert(owned_player_id);
                });
        })
    });
    transactions.iter().for_each(|txn| {
        txn.adds.keys().for_each(|k| {
            player_id_set.insert(k.to_string());
        });
        txn.drops.keys().for_each(|k| {
            player_id_set.insert(k.to_string());
        });
    });
    draft_picks.iter().for_each(|pick| {
        player_id_set.insert(pick.player_id.clone());
    });

    // try to find data on all players
    let mut player_cache_misses = HashSet::new();
    let mut players_in_league: HashMap<String, sleeper::Player> = HashMap::new();
    if let Some(player_cache) = optional_player_cache {
        for player_id in player_id_set.into_iter() {
            if let Some(player) = player_cache.get_sleeper_player(&player_id) {
                players_in_league.insert(player.player_id.clone(), player);
            } else {
                // cache miss
                player_cache_misses.insert(player_id);
            }
        }
    }

    // if we have cache missed and a sleep player update request is not yet in flight, then fetch players
    if !player_cache_misses.is_empty() && optional_players_future.is_none() {
        optional_players_future = Some(cli.fetch_all_players(&sleeper_league.sport));
    }

    let mut unmatched_players = HashSet::new();
    if let Some(players_future) = optional_players_future {
        let players = players_future.await?;
        for player_id in player_cache_misses.into_iter() {
            if let Some(player) = players.get(&player_id) {
                // add to cache
                if let Some(player_cache) = optional_player_cache {
                    player_cache.insert_sleeper_player(player);
                }
                players_in_league.insert(player_id, player.clone());
            } else {
                unmatched_players.insert(player_id);
                continue;
            }
        }
    }

    if !unmatched_players.is_empty() {
        log::warn!(
            "unable to match all players for sleeper league. unmatched: {}",
            unmatched_players.len()
        );
    }

    Ok(core::League::Sleeper {
        draft,
        draft_picks,
        league: sleeper_league.clone(),
        owners,
        players: players_in_league,
        rosters,
        transactions,
        matchups,
        user_id: None,
    })
}

pub(crate) async fn fetch_yahoo_league_context(
    cli: &yahoo::Client,
    player_cache: Option<&PlayerCache>,
    league_key: yahoo::LeagueKey,
) -> Result<core::League, Error> {
    let yahoo_league_future = cli.get_league(&league_key);

    let standings_future = cli.get_league_standings(&league_key);
    let rosters_future = cli.get_rosters_in_league(&league_key);
    let settings_future = cli.get_league_settings(&league_key);
    let draft_results = cli.get_draft_results(&league_key).await?;

    let mut player_key_set: HashSet<yahoo::PlayerKey> = HashSet::new();
    draft_results.iter().for_each(|p| {
        player_key_set.insert(p.player_key.clone());
    });
    //TODO: add player keys from transactions and ending rosters
    // we may be able to borrow the player info from the rosters

    let mut player_keys: Vec<yahoo::PlayerKey> = player_key_set.into_iter().collect();
    log::debug!(
        "{} player keys must be looked up for {league_key}",
        player_keys.len()
    );

    let mut players: Vec<yahoo_fantasy_rs::Player> = vec![];
    while !player_keys.is_empty() {
        let popped = if player_keys.len() > 25 {
            player_keys.split_off(player_keys.len() - 25)
        } else {
            player_keys.split_off(0)
        };
        players.extend(
            cli.get_players_for_league(&league_key, Some(&popped))
                .await?,
        );
    }

    log::debug!("got {} yahoo players in {league_key}", players.len());
    Ok(core::League::Yahoo {
        draft_results,
        league: yahoo_league_future
            .await?
            .ok_or(Error::new("yahoo league not found"))?,
        matchups: HashMap::new(), // TODO: Yahoo does not provide matchups in the same way as Sleeper
        players: players
            .into_iter()
            .map(|p| (p.player_key.clone(), p))
            .collect(),
        rosters: rosters_future
            .await?
            .into_iter()
            .map(|team| (team.team_id, team.roster.unwrap()))
            .collect::<HashMap<u32, yahoo::Roster>>(),
        settings: settings_future.await?,
        standings: standings_future.await?,
        team: None,
    })
}

async fn get_sleeper_leagues(
    sleeper_client: &sleeper::Client,
    seasons: &[String],
    sleeper_user_id: &String,
    sleeper_sport: &sleeper::Sport,
    player_cache: Option<&PlayerCache>,
) -> Result<Vec<core::League>, Box<dyn std::error::Error + Send + Sync>> {
    let mut sleeper_leagues_for_season_tasks: FuturesUnordered<_> = seasons
        .iter()
        .map(|season| sleeper_client.get_leagues_for_user(sleeper_user_id, sleeper_sport, season))
        .collect();

    // let mut sleeper_leagues_tasks = FuturesUnordered::new();
    // // Now you can poll tasks as they complete:
    let mut sleeper_leagues: Vec<sleeper::League> = vec![];
    loop {
        match sleeper_leagues_for_season_tasks.try_next().await {
            Ok(Some(leagues)) => {
                for league in leagues {
                    sleeper_leagues.push(league);
                }
            }
            Ok(None) => {
                break;
            } // exit loop as all tasks are done
            Err(e) => {
                log::error!("Error fetching Sleeper leagues: {:?}", e);
                // panic!("Error fetching Sleeper leagues: {:?}", e); //TDOO handle this gracefully
                // return Err(Box::new(e));
            }
        }
        // handle result
    }

    let mut league_futures = FuturesUnordered::new();

    for sleeper_league in sleeper_leagues.iter() {
        league_futures.push(fetch_sleeper_league_context(
            sleeper_client,
            player_cache,
            sleeper_league,
        ));
    }

    let mut leagues = vec![];
    while let Some(yahoo_league) = league_futures.try_next().await? {
        leagues.push(yahoo_league);
    }

    Ok(leagues)
}

async fn get_yahoo_leagues_for_user(
    yahoo_client: &yahoo_fantasy_rs::Client,
    player_cache: Option<&PlayerCache>,
    seasons: &[String],
    yahoo_game_code: yahoo::GameCode,
) -> Result<Vec<core::League>, Box<dyn std::error::Error + Send + Sync>> {
    let games = yahoo_client
        .get_games(
            vec![yahoo_game_code],
            seasons.iter().map(|s| s.as_str()).collect(),
        )
        .await
        .unwrap();

    let mut yahoo_teams: Vec<yahoo::Team> = vec![];
    for game in games {
        let game_key: yahoo::GameKey = yahoo::GameKey::Id(game.game_id);
        let teams_in_season = yahoo_client.get_teams_for_user(game_key).await?;
        for team in teams_in_season {
            yahoo_teams.push(team);
        }
    }

    let mut leagues: Vec<core::League> = vec![];

    let mut yahoo_teams_tasks: FuturesUnordered<_> = FuturesUnordered::new();
    for yahoo_team in yahoo_teams {
        yahoo_teams_tasks.push(fetch_yahoo_league_context(
            yahoo_client,
            player_cache,
            yahoo_team.team_key.league_key,
        ));
    }
    while let Some(yahoo_league) = yahoo_teams_tasks.try_next().await? {
        leagues.push(yahoo_league);
    }

    Ok(leagues)
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("sleeper error: {0}")]
    SleeperError(#[from] sleeper::Error),
    #[error("yahoo error: {0}")]
    YahooError(#[from] yahoo::Error),
    #[error("dev error: {0}")]
    DevError(String),
}
impl Error {
    pub fn new<T: Into<String>>(msg: T) -> Self {
        Error::DevError(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use crate::fetch::{ExternalId, LeagueAccessor};

    #[tokio::test]
    async fn test_thing() {
        let cases = [
            ("yahoo:359.l.564503", "2016", "Yahoo", 0),
            ("sleeper:164974877262287888", "2019", "Sleeper", 334), // was 278 before adding all player_ids from draft and transactions
        ];

        let mut fixture = LeagueAccessor::new().with_cache_dir("cache");
        fixture.init().await.expect("failed to initialized");

        debug_assert!(fixture.sleeper_enabled(), "sleeper not enabled");
        debug_assert!(fixture.yahoo_enabled(), "yahoo not enabled");

        // Execute and Assert
        for (league_id, season, platform, num_of_players) in cases {
            let external_id = ExternalId::try_from(league_id).expect("invalid league_id");
            let league = fixture
                .fetch_league_context(&external_id)
                .await
                .expect("failed to get sleeper league");
            debug_assert_eq!(season, league.season(), "season is incorrect");
            debug_assert_eq!(platform, league.platform(), "platform is incorrect");
            debug_assert_eq!(
                num_of_players,
                league.players().len(),
                "num of players is incorrect in {league_id}",
            );
        }

        // assert that only cache is hit
        debug_assert_eq!(
            0,
            fixture.sleeper_cli.unwrap().request_count(),
            "sleeper api was hit"
        );
        debug_assert_eq!(0, fixture.yahoo_cli.unwrap().request_count(), "yahoo api was hit");
    }
}
