use crate::{
    ExternalId, Platform,
    data::{self as core},
    player_cache::PlayerCache,
    sleeper_utils,
};
use fantasy_sports_fleaflicker as fleaflicker;
use fantasy_sports_sleeper as sleeper;
use fantasy_sports_yahoo as yahoo;
use futures::{
    StreamExt, TryStreamExt,
    stream::{FuturesOrdered, FuturesUnordered},
};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    str::FromStr,
};
use tokio::io;
use walkdir::WalkDir;

pub struct LeagueAccessor {
    pub fleaflicker_cli: Option<fleaflicker::Client>,
    pub sleeper_cli: Option<sleeper::Client>,
    players_cache: Option<PlayerCache>,
    pub yahoo_cli: Option<yahoo::Client>,
    cache_dir: Option<PathBuf>,
    yahoo_settings: Option<(String, String, Option<String>, Option<yahoo::Token>)>,
}
impl LeagueAccessor {
    pub fn new() -> Self {
        LeagueAccessor {
            fleaflicker_cli: None,
            sleeper_cli: None,
            players_cache: None,
            yahoo_cli: None,
            cache_dir: None,
            yahoo_settings: None,
        }
    }

    pub fn with_cache_dir<T: Into<PathBuf>>(mut self, cache_dir: T) -> Self {
        self.cache_dir = Some(cache_dir.into());
        self
    }

    pub fn with_yahoo_settings<C: Into<String>, R: Into<String>, T: Into<String>>(
        mut self,
        client_id: C,
        redirect_uri: R,
        token_file: Option<T>,
        token: Option<yahoo::Token>,
    ) -> Self {
        self.yahoo_settings = Some((
            client_id.into(),
            redirect_uri.into(),
            token_file.map(|t| t.into()),
            token,
        ));
        self
    }

    pub async fn init(&mut self) -> Result<(), AccessorInitError> {
        let (fleaflicker_cache_dir, sleeper_cache_dir, yahoo_cache_dir, player_cache) =
            match &self.cache_dir {
                Some(cache_dir) => (
                    Some(cache_dir.join("fleaflicker")),
                    Some(cache_dir.join("sleeper")),
                    Some(cache_dir.join("yahoo")),
                    Some(cache_dir.join("players")),
                ),
                None => (None, None, None, None),
            };
        let player_cache = player_cache
            .and_then(|player_cache| PlayerCache::new_with_path(&player_cache, Sport::NFL).ok());
        if let Some(player_cache) = &player_cache {
            player_cache
                .init()
                .await
                .map_err(|e| AccessorInitError::FailedToSetupPlayerCache(e.to_string()))?;
        }

        let fleaflicker_cli = match fleaflicker_cache_dir {
            Some(fleaflicker_cache_dir) => fleaflicker::Client::new()
                .with_cache(fleaflicker_cache_dir)
                .await
                .map_err(|e| {
                    AccessorInitError::FailedToSetupPlatformClient(
                        Platform::FleaFlicker,
                        e.to_string(),
                    )
                })?,
            None => fleaflicker::Client::new(),
        };

        let sleeper_cli = match sleeper_cache_dir {
            Some(sleeper_cache_dir) => sleeper::Client::new()
                .with_cache(sleeper_cache_dir)
                .await
                .map_err(|e| {
                AccessorInitError::FailedToSetupPlatformClient(Platform::Sleeper, e.to_string())
            })?,
            None => sleeper::Client::new(),
        };

        let mut yahoo_cli = if let Some((client_id, redirect_uri, opt_token_file, opt_token)) =
            &self.yahoo_settings
        {
            let mut yahoo_cli = yahoo::Client::new(client_id, redirect_uri);
            if let Some(token) = opt_token {
                yahoo_cli = yahoo_cli.with_token(token.clone());
                yahoo_cli
            } else if let Some(token_file) = opt_token_file {
                yahoo_cli = yahoo_cli
                    .with_token_file(Some(token_file.to_string()))
                    .await
                    .map_err(|e| {
                        AccessorInitError::FailedToSetupPlatformClient(
                            Platform::Yahoo,
                            format!("unable to bootstrap yahoo client due to bad token file: {e}"),
                        )
                    })?;
                yahoo_cli
            } else {
                return Err(AccessorInitError::FailedToSetupPlatformClient(
                    Platform::Yahoo,
                    "unable to bootstrap yahoo client due to no token configuration".to_string(),
                ));
            }
        } else {
            yahoo::Client::new_from_env().map_err(|e| {
                AccessorInitError::FailedToSetupPlatformClient(
                    Platform::Yahoo,
                    format!("unable to bootstrap yahoo client due to missing env variables: {e}"),
                )
            })?.with_token_file::<String>(None).await.map_err(|err| {
                yahoo::Error::UnknownError(format!("failed to load token file: {err}"))
            }).map_err(|e| {
                AccessorInitError::FailedToSetupPlatformClient(
                    Platform::Yahoo,
                    format!("unable to bootstrap yahoo client due to failure to load token file: {e}"),
                )
            })?
        };

        // TODO: only refresh if it needs to be
        yahoo_cli.refresh_token(false).await.map_err(|e| {
            AccessorInitError::FailedToSetupPlatformClient(Platform::Yahoo, e.to_string())
        })?;

        if let Some(yahoo_cache_dir) = yahoo_cache_dir {
            yahoo_cli = yahoo_cli
                .with_cache_dir(yahoo_cache_dir)
                .await
                .map_err(|e| {
                    AccessorInitError::FailedToSetupPlatformClient(Platform::Yahoo, e.to_string())
                })?;
        }
        self.players_cache = player_cache;
        self.fleaflicker_cli = Some(fleaflicker_cli);
        self.sleeper_cli = Some(sleeper_cli);
        self.yahoo_cli = Some(yahoo_cli);
        Ok(())
    }

    pub fn fleaflicker_enabled(&self) -> bool {
        self.fleaflicker_cli.is_some()
    }

    pub fn sleeper_enabled(&self) -> bool {
        self.sleeper_cli.is_some()
    }

    pub fn yahoo_enabled(&self) -> bool {
        self.yahoo_cli.is_some()
    }

    pub async fn fetch_league_ids_for_user(
        &self,
        user_id: &ExternalId,
        seasons: Vec<String>,
        sport: &Sport,
    ) -> Result<Vec<ExternalId>, Error> {
        match &user_id.platform {
            Platform::FleaFlicker => {
                if let Some(cli) = &self.fleaflicker_cli {
                    fetch_fleaflicker_leagues_for_user(
                        cli,
                        &seasons,
                        &user_id.id,
                        &sport.to_fleaflicker_sport(),
                    )
                    .await
                } else {
                    Err(Error::new("fleaflicker cli not configured"))
                }
            }
            Platform::Sleeper => {
                if let Some(cli) = &self.sleeper_cli {
                    fetch_sleeper_league_ids_for_user(cli, &seasons, &user_id.id, &sport.to_sport())
                        .await
                } else {
                    Err(Error::new("sleeper cli not configured"))
                }
            }
            Platform::Yahoo => {
                if let Some(cli) = &self.yahoo_cli {
                    fetch_yahoo_league_keys_for_user(cli, &seasons, sport.to_game_code()).await
                } else {
                    Err(Error::new("yahoo cli not configured"))
                }
            }
            other => Err(Error::new(format!("{other} is not supported"))),
        }
    }

    pub async fn fetch_league_context(
        &self,
        id: &ExternalId,
        force_update: bool,
    ) -> Result<crate::model::League, Error> {
        match &id.platform {
            Platform::FleaFlicker => {
                if let Some(cli) = &self.fleaflicker_cli {
                    fetch_fleaflicker_league_context(
                        cli,
                        &self.players_cache,
                        id.id.clone(),
                        force_update,
                    )
                    .await
                } else {
                    Err(Error::new("fleaflicker cli not configured"))
                }
            }
            Platform::Sleeper => {
                if let Some(cli) = &self.sleeper_cli {
                    fetch_sleeper_league_context_by_id(
                        cli,
                        &self.players_cache,
                        id.id.clone(),
                        force_update,
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
                        &self.players_cache,
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

    pub fn disk_space_used(&self) -> Result<u64, io::Error> {
        let mut total_size = 0;
        if let Some(cache_dir) = &self.cache_dir {
            for entry in WalkDir::new(cache_dir).into_iter().filter_map(|e| e.ok()) {
                let metadata = entry.metadata()?;
                if metadata.is_file() {
                    total_size += metadata.len();
                }
            }
        }
        Ok(total_size)
    }
}

impl Default for LeagueAccessor {
    fn default() -> Self {
        Self::new()
    }
}
unsafe impl Send for LeagueAccessor {
    //TODO: WTF
}

pub(crate) async fn fetch_fleaflicker_leagues_for_user(
    fleaflicker_client: &fleaflicker::Client,
    seasons: &[String],
    fleaflicker_email: &String,
    sport: &fleaflicker::Sport,
) -> Result<Vec<ExternalId>, Error> {
    let mut fleaflicker_leagues_for_season_tasks: FuturesUnordered<_> = seasons
        .iter()
        .map(|season| {
            fleaflicker_client.get_leagues_for_user(
                fleaflicker_email,
                Some(sport.clone()),
                Some(season.as_str().parse().unwrap()),
            )
        })
        .collect();

    let mut leagues: Vec<fleaflicker::League> = vec![];
    loop {
        match fleaflicker_leagues_for_season_tasks.try_next().await {
            Ok(Some(resp)) => {
                for league in resp.leagues {
                    leagues.push(league);
                }
            }
            Ok(None) => {
                break;
            } // exit loop as all tasks are done
            Err(e) => {
                log::error!("Error fetching FleaFlicker leagues: {:?}", e);
                // panic!("Error fetching FleaFlicker leagues: {:?}", e); //TDOO handle this gracefully
                // return Err(Box::new(e));
            }
        }
    }

    Ok(leagues.into_iter().map(ExternalId::from).collect())
}

pub(crate) async fn fetch_fleaflicker_league_context(
    cli: &fleaflicker::Client,
    _optional_player_cache: &Option<PlayerCache>,
    fleaflicker_league_id: String,
    _force_update: bool,
) -> Result<crate::League, Error> {
    let fleaflicker_league_id: i32 = fleaflicker_league_id.parse().map_err(|_| {
        Error::DevError(format!(
            "fleaflicker league ids are numbers, got {fleaflicker_league_id:?}"
        ))
    })?;
    let league_standings = cli
        .fetch_league_standings(fleaflicker_league_id, None, None)
        .await?;
    let fleaflicker_league = league_standings.league.clone(); // this needs to come as a param

    // TODO: result offset
    let transaction_history_future = cli.get_transaction_history(
        fleaflicker_league.id,
        fleaflicker_league.sport.clone(),
        None,
        None,
    );

    let league_rosters_future = cli.fetch_league_rosters(
        fleaflicker_league.id,
        fleaflicker_league.sport.clone(),
        None,
        None,
        None,
    );

    let league_rules_future =
        cli.fetch_league_rules(fleaflicker_league.id, fleaflicker_league.sport.clone());

    let league_standings_future = cli.fetch_league_standings(
        fleaflicker_league.id,
        fleaflicker_league.sport.clone(),
        None,
    );

    Ok(crate::League::FleaFlicker {
        league: fleaflicker_league.clone(),
        rules: league_rules_future.await?,
        rosters: league_rosters_future.await?.rosters,
        standings: league_standings_future.await?,
        transactions: transaction_history_future.await?.items,
        team: fleaflicker_league.owned_team.clone(),
    })
}

pub(crate) async fn fetch_sleeper_league_context_by_id<T: Into<String>>(
    cli: &sleeper::Client,
    player_cache: &Option<PlayerCache>,
    league_id: T,
    force_update: bool,
) -> Result<crate::League, Error> {
    let league_id: String = league_id.into();
    let sleeper_league = cli
        .get_league_request(league_id.as_str())
        .with_force_update(&force_update)
        .send()
        .await?;
    fetch_sleeper_league_context(cli, player_cache, &sleeper_league, force_update).await
}

pub(crate) async fn fetch_sleeper_league_context(
    cli: &sleeper::Client,
    optional_player_cache: &Option<PlayerCache>,
    sleeper_league: &sleeper::League,
    force_update: bool,
) -> Result<crate::League, Error> {
    let mut optional_players_future = if optional_player_cache.is_none() {
        Some(
            cli.fetch_all_players_request(&sleeper_league.sport)
                .with_force_update(&force_update)
                .send(),
        )
    } else {
        None
    };
    let owners_future = cli
        .get_users_for_league_request(&sleeper_league.league_id)
        .with_force_update(&force_update)
        .send();
    let rosters_future = cli
        .get_rosters_in_league_request(&sleeper_league.league_id)
        .with_force_update(&force_update)
        .send();
    let winners_bracket_future = cli
        .get_playoff_bracket_request(&sleeper_league.league_id, false)
        .with_force_update(&force_update)
        .send();
    let losers_bracket_future = cli
        .get_playoff_bracket_request(&sleeper_league.league_id, true)
        .with_force_update(&force_update)
        .send();
    let (owners_result, rosters_result) = tokio::join!(owners_future, rosters_future);
    let owners = owners_result?
        .into_iter()
        .map(|owner| (owner.user_id.clone(), owner))
        .collect::<HashMap<String, sleeper::User>>();
    let rosters = rosters_result?;

    let matchups =
        match sleeper_utils::get_league_matchups(sleeper_league, cli, &force_update).await {
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
        .get_drafts_for_league_request(&sleeper_league.league_id)
        .with_force_update(&force_update)
        .send()
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
        draft_picks = match cli
            .get_draft_picks_request(&draft.draft_id)
            .with_force_update(&force_update)
            .send()
            .await
        {
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
        match sleeper_utils::get_league_transactions(cli, sleeper_league, &force_update).await {
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
    matchups.iter().for_each(|(_, matchups)| {
        matchups.iter().for_each(|fmup| {
            fmup.players.iter().for_each(|owned_player_id| {
                player_id_set.insert(owned_player_id.to_string());
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
            match player_cache.get_sleeper_player(&player_id) {
                Ok(player) => {
                    // cache hit
                    players_in_league.insert(player.player_id.clone(), player);
                }
                Err(e) if e.is_none() => {
                    // cache miss
                    player_cache_misses.insert(player_id);
                }
                Err(err) => {
                    // cache error
                    log::error!("failed to lookup sleeper player {player_id} in cache: {err}")
                }
            }
        }
    } else {
        // without a player cache every player the league uses has to come from the downloaded
        // player universe; this used to leave the league without any players
        player_cache_misses = player_id_set;
    }

    // if we have cache missed and a sleep player update request is not yet in flight, then fetch players
    if !player_cache_misses.is_empty() && optional_players_future.is_none() {
        let tmp = cli
            .fetch_all_players_request(&sleeper_league.sport)
            .with_force_update(&force_update)
            .send();

        optional_players_future = Some(tmp);
    }

    let mut unmatched_players = HashSet::new();
    if let Some(players_future) = optional_players_future {
        let players = players_future.await?;
        for player_id in player_cache_misses.into_iter() {
            if let Some(player) = players.get(&player_id) {
                // add to cache
                if let Some(player_cache) = optional_player_cache {
                    player_cache.insert_sleeper_player(player)?;
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
            "unable to match all players for sleeper league ({season}: {league_id}). unmatched: {count}",
            count = unmatched_players.len(),
            season = sleeper_league.season,
            league_id = sleeper_league.league_id
        );
    }

    Ok(crate::League::Sleeper {
        draft,
        draft_picks,
        league: sleeper_league.clone(),
        owners,
        players: players_in_league,
        rosters: rosters.into_iter().map(|r| (r.roster_id, r)).collect(),
        winners_bracket: match winners_bracket_future.await {
            Ok(b) => Some(sleeper::Bracket {
                matchups: b,
                is_losers: false,
            }),
            Err(e) => {
                log::warn!("failed to get winners bracket: {e}");
                None
            }
        },
        losers_bracket: match losers_bracket_future.await {
            Ok(b) => Some(sleeper::Bracket {
                matchups: b,
                is_losers: true,
            }),
            Err(e) => {
                log::warn!("failed to get losers bracket: {e}");
                None
            }
        },
        transactions,
        matchups,
        user_id: None,
    })
}

pub(crate) async fn fetch_yahoo_league_context(
    cli: &yahoo::Client,
    optional_player_cache: &Option<PlayerCache>,
    league_key: yahoo::LeagueKey,
) -> Result<crate::League, Error> {
    // Fire out http requests
    let yahoo_league_future = cli.get_league(&league_key);
    let standings_future = cli.get_league_standings(&league_key);
    let rosters_future = cli.get_rosters_in_league(&league_key);
    let settings_future = cli.get_league_settings(&league_key);
    let transactions_future = cli.get_league_transactions(&league_key);
    let teams_future = cli.get_teams_in_league(&league_key);

    // Wait for the league settings to come back
    let settings = settings_future.await?;
    let mut scoreboard_settings_futures = FuturesOrdered::new();
    let mut weeks = vec![];
    let playoff_start_week: i32 = settings
        .playoff_start_week
        .unwrap_or(0)
        .try_into()
        .map_err(|err| {
            Error::new(format!(
                "unable to use playoff_start_week ({:?}): {err}",
                settings.playoff_start_week
            ))
        })?;
    let last_week: i32 = playoff_start_week + 2;
    for week in 1..last_week + 1 {
        scoreboard_settings_futures.push_back(cli.get_league_scoreboard(
            &league_key,
            Some(week),
            None,
        ));
        weeks.push(week);
    }

    // Build out draft results
    let draft_results = cli.get_draft_results(&league_key).await?;

    // Build out Rosters
    let rosters = rosters_future
        .await?
        .into_iter()
        .map(|team| (team.team_id, team.roster.unwrap()))
        .collect::<HashMap<u32, yahoo::Roster>>();

    // build out transactions
    let transactions = transactions_future.await?;

    // Gather set of all player keys in the league
    let mut player_key_set: HashSet<yahoo::PlayerKey> = HashSet::new();
    draft_results.iter().for_each(|p| {
        if let Some(player_key) = &p.player_key {
            player_key_set.insert(player_key.clone());
        }
    });
    rosters.iter().for_each(|(_, roster)| {
        roster.players.iter().for_each(|player| {
            player_key_set.insert(player.player_key.clone());
        });
    });
    transactions.iter().for_each(|txn| {
        txn.player_keys().into_iter().for_each(|k| {
            player_key_set.insert(k);
        })
    });
    // we may be able to borrow the player info from the rosters

    let mut player_cache_misses = HashSet::new();
    let mut players_in_league = HashMap::new();
    // check player cache for player
    if let Some(player_cache) = optional_player_cache {
        for player_key in player_key_set.into_iter() {
            match player_cache.get(&ExternalId {
                platform: Platform::Yahoo,
                id: player_key.to_string(),
            }) {
                Ok(player) => {
                    // found player
                    players_in_league.insert(player_key, player);
                }
                Err(e) if e.is_none() => {
                    // cache miss
                    player_cache_misses.insert(player_key);
                }
                Err(err) => {
                    // cache error
                    log::error!("failed to lookup yahoo player {player_key} in cache: {err}")
                }
            }
        }
    } else {
        player_cache_misses.extend(player_key_set);
    }

    if !player_cache_misses.is_empty() {
        log::info!(
            "{count} player keys must be looked up for {league_key}",
            count = player_cache_misses.len()
        );
    }

    let mut player_cache_misses = player_cache_misses.into_iter().collect::<Vec<_>>();
    while !player_cache_misses.is_empty() {
        let popped = if player_cache_misses.len() > 25 {
            player_cache_misses.split_off(player_cache_misses.len() - 25)
        } else {
            player_cache_misses.split_off(0)
        };
        for player in cli
            .get_players_for_league(&league_key, Some(&popped))
            .await?
        {
            if let Some(player_cache) = optional_player_cache {
                player_cache.insert_yahoo_player(&player);
            }
            players_in_league.insert(player.player_key.clone(), player.into());
        }
    }

    // Build out Weekly Scoreboards
    let mut scoreboards = vec![];
    let mut i = 0;
    while let Some(result) = scoreboard_settings_futures.next().await {
        let week = weeks[i];
        i += 1;
        match result {
            Ok(scoreboard) => {
                scoreboards.push(scoreboard);
            }
            Err(err) => {
                log::warn!("failed to get scoreboard for week {week}: {err}")
            }
        }
    }

    log::debug!(
        "got {} yahoo players in {league_key}",
        players_in_league.len()
    );
    Ok(crate::League::Yahoo {
        draft_results,
        league: yahoo_league_future
            .await?
            .ok_or(Error::new("yahoo league not found"))?,
        scoreboards,
        players: players_in_league,
        rosters,
        settings,
        standings: standings_future.await?,
        teams: teams_future.await?,
        team: None,
        transactions,
    })
}

pub(crate) async fn fetch_sleeper_league_ids_for_user(
    sleeper_client: &sleeper::Client,
    seasons: &[String],
    sleeper_user_id: &String,
    sport: &sleeper::Sport,
) -> Result<Vec<ExternalId>, Error> {
    let mut sleeper_leagues_for_season_tasks: FuturesUnordered<_> = seasons
        .iter()
        .map(|season| sleeper_client.get_leagues_for_user(sleeper_user_id, sport, season))
        .collect();

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
    }

    Ok(sleeper_leagues.into_iter().map(ExternalId::from).collect())
}

pub enum Sport {
    NFL,
}
impl Sport {
    pub fn to_game_code(&self) -> yahoo::GameCode {
        match self {
            Sport::NFL => yahoo::GameCode::NFL,
        }
    }
    pub fn to_sport(&self) -> sleeper::Sport {
        match self {
            Sport::NFL => sleeper::Sport::NFL,
        }
    }
    pub fn to_fleaflicker_sport(&self) -> fleaflicker::Sport {
        match self {
            Sport::NFL => fleaflicker::Sport::NFL,
        }
    }
}

async fn fetch_yahoo_league_keys_for_user(
    yahoo_client: &fantasy_sports_yahoo::Client,
    seasons: &[String],
    yahoo_game_code: yahoo::GameCode,
) -> Result<Vec<ExternalId>, Error> {
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

    Ok(yahoo_teams
        .into_iter()
        .map(|t| ExternalId::from(t.team_key.league_key))
        .collect())
}

#[derive(thiserror::Error, Debug)]
pub enum AccessorInitError {
    #[error("failed to setup accessor player cache: {0}")]
    FailedToSetupPlayerCache(String),
    #[error("failed to setup {0} platform client: {1}")]
    FailedToSetupPlatformClient(Platform, String),
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("player cache error: {0}")]
    PlayerCacheError(#[from] crate::player_cache::Error),
    #[error("cached client error: {0}")]
    CachedClientError(#[from] fantasy_sports_cache::Error),
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
    use super::*;
    use std::path::PathBuf;
    use wiremock::{
        Mock, MockServer, Request, Respond, ResponseTemplate,
        matchers::{method, path_regex},
    };

    const LEAGUE_ID: &str = "164974877262287888";
    const DRAFT_ID: &str = "685822547406541191";

    fn fixture(rel: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../sleeper/testdata")
            .join(rel)
    }

    /// Serves the sleeper crate's recorded (pseudonymized) responses for one league, keyed by the
    /// request path. Weeks and transaction legs without a recording are empty, like a quiet week.
    struct SleeperFixtures;
    impl Respond for SleeperFixtures {
        fn respond(&self, req: &Request) -> ResponseTemplate {
            let path = req.url.path();
            let json = |rel: String| match std::fs::read_to_string(fixture(&rel)) {
                Ok(body) => ResponseTemplate::new(200).set_body_raw(body, "application/json"),
                Err(_) => ResponseTemplate::new(404),
            };
            let league = format!("/league/{LEAGUE_ID}");
            match path {
                p if p == league => json(format!("v1/league/{LEAGUE_ID}.json")),
                p if p == format!("{league}/users") => {
                    json(format!("v1/league/{LEAGUE_ID}/users.json"))
                }
                p if p == format!("{league}/rosters") => {
                    json(format!("v1/league/{LEAGUE_ID}/rosters.json"))
                }
                p if p == format!("{league}/drafts") => {
                    json(format!("v1/league/{LEAGUE_ID}/drafts.json"))
                }
                p if p == format!("{league}/traded_picks") => {
                    json(format!("v1/league/{LEAGUE_ID}/traded_picks.json"))
                }
                p if p == format!("{league}/winners_bracket") => {
                    json(format!("v1/league/{LEAGUE_ID}/winners_bracket.json"))
                }
                p if p == format!("{league}/losers_bracket") => {
                    json(format!("v1/league/{LEAGUE_ID}/losers_bracket.json"))
                }
                p if p == format!("{league}/matchups/1") => {
                    json("league_matchups_1.json".to_string())
                }
                p if p.starts_with(&format!("{league}/matchups/")) => {
                    ResponseTemplate::new(200).set_body_string("[]")
                }
                p if p.starts_with(&format!("{league}/transactions/")) => {
                    ResponseTemplate::new(200).set_body_string("[]")
                }
                p if p == format!("/draft/{DRAFT_ID}") => json(format!("v1/draft/{DRAFT_ID}.json")),
                p if p == format!("/draft/{DRAFT_ID}/picks") => {
                    json(format!("v1/draft/{DRAFT_ID}/picks.json"))
                }
                "/players/nfl" => ResponseTemplate::new(200)
                    .set_body_raw(
                        std::fs::read(fixture("players_nfl.json.gz")).unwrap(),
                        "application/json",
                    )
                    .insert_header("Content-Encoding", "gzip"),
                _ => ResponseTemplate::new(404),
            }
        }
    }

    async fn sleeper_client() -> (sleeper::Client, MockServer) {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path_regex(".*"))
            .respond_with(SleeperFixtures)
            .mount(&server)
            .await;
        (sleeper::Client::new().with_base_url(server.uri()), server)
    }

    #[tokio::test]
    async fn a_sleeper_league_is_assembled_from_the_api_responses() {
        let (cli, server) = sleeper_client().await;

        let league = fetch_sleeper_league_context_by_id(&cli, &None, LEAGUE_ID, false)
            .await
            .expect("league should be fetched");

        assert_eq!("Rapid League 17", league.name());
        assert_eq!("2019", league.season());
        assert_eq!(12, league.rosters().len());
        assert_eq!(12, league.standings().entries.len());
        assert_eq!(
            204,
            league.draft().expect("the league has a draft").picks.len()
        );
        assert!(
            league.transactions().is_empty(),
            "no transactions were recorded"
        );

        // one scoreboard per regular season week; week 1 is the recorded matchups, 6 games for 12 teams
        let scoreboards = league.scoreboards();
        assert_eq!(16, scoreboards.len());
        assert_eq!(6, scoreboards[0].matchups.len());
        assert_eq!(1, scoreboards[0].matchups[0].week);
        // always in season order, however the weeks were collected
        let weeks: Vec<u32> = scoreboards
            .iter()
            .filter_map(|s| s.matchups.first().map(|m| m.week))
            .collect();
        assert!(weeks.is_sorted(), "{weeks:?}");

        // everything the context needs was requested, each endpoint through the client
        let requested: Vec<String> = server
            .received_requests()
            .await
            .unwrap()
            .iter()
            .map(|r| r.url.path().to_string())
            .collect();
        let base = format!("/league/{LEAGUE_ID}");
        for expected in [
            base.clone(),
            format!("{base}/users"),
            format!("{base}/rosters"),
            format!("{base}/drafts"),
            format!("{base}/winners_bracket"),
            format!("{base}/losers_bracket"),
            format!("/draft/{DRAFT_ID}/picks"),
            "/players/nfl".to_string(),
        ] {
            assert!(
                requested.contains(&expected),
                "{expected} was not requested: {requested:?}"
            );
        }
    }

    #[tokio::test]
    async fn an_unknown_sleeper_league_is_an_error() {
        let (cli, _server) = sleeper_client().await;
        let result = fetch_sleeper_league_context_by_id(&cli, &None, "999", false).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn fetching_from_a_platform_that_is_not_configured_says_so() {
        // a LeagueAccessor that has not been init()ed has no clients
        let accessor = LeagueAccessor::new();
        for (id, platform) in [
            ("sleeper:1", "sleeper"),
            ("fleaflicker:1", "fleaflicker"),
            ("yahoo:359.l.1", "yahoo"),
        ] {
            let err = accessor
                .fetch_league_context(&ExternalId::try_from(id).unwrap(), false)
                .await
                .expect_err("no client is configured");
            assert!(
                err.to_string()
                    .contains(&format!("{platform} cli not configured")),
                "{err}"
            );
        }
    }

    /// Serves the fleaflicker crate's recorded (pseudonymized) responses by endpoint.
    struct FleaflickerFixtures;
    impl Respond for FleaflickerFixtures {
        fn respond(&self, req: &Request) -> ResponseTemplate {
            let file = match req.url.path() {
                "/FetchLeagueStandings" => "fetch_league_standings_pre.json",
                "/FetchLeagueTransactions" => "league_transactions_page_1.json",
                "/FetchLeagueRosters" => "league_rosters.json",
                "/FetchLeagueRules" => "fetch_league_rules.json",
                _ => return ResponseTemplate::new(404),
            };
            let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../fleaflicker/testdata")
                .join(file);
            ResponseTemplate::new(200)
                .set_body_raw(std::fs::read_to_string(path).unwrap(), "application/json")
        }
    }

    async fn fleaflicker_client() -> (fleaflicker::Client, MockServer) {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path_regex(".*"))
            .respond_with(FleaflickerFixtures)
            .mount(&server)
            .await;
        (
            fleaflicker::Client::new().with_base_url(server.uri()),
            server,
        )
    }

    #[tokio::test]
    async fn a_fleaflicker_league_is_assembled_from_the_api_responses() {
        let (cli, _server) = fleaflicker_client().await;

        let league = fetch_fleaflicker_league_context(&cli, &None, "1".to_string(), false)
            .await
            .expect("league should be fetched");

        assert_eq!("Golden League 00", league.name());
        assert_eq!("2025", league.season());
        assert_eq!(12, league.rosters().len());
        assert_eq!(12, league.standings().entries.len());
        assert!(matches!(league, crate::League::FleaFlicker { .. }));
    }

    #[tokio::test]
    async fn a_non_numeric_fleaflicker_league_id_is_an_error_not_a_panic() {
        let (cli, _server) = fleaflicker_client().await;
        let err = fetch_fleaflicker_league_context(&cli, &None, "abc".to_string(), false)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("abc"), "{err}");
    }

    async fn sleeper_league() -> crate::League {
        let (cli, _server) = sleeper_client().await;
        fetch_sleeper_league_context_by_id(&cli, &None, LEAGUE_ID, false)
            .await
            .unwrap()
    }

    async fn fleaflicker_league() -> crate::League {
        let (cli, _server) = fleaflicker_client().await;
        fetch_fleaflicker_league_context(&cli, &None, "1".to_string(), false)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn a_sleeper_league_describes_itself() {
        let league = sleeper_league().await;

        assert_eq!(format!("sleeper:{LEAGUE_ID}"), league.id().to_string());
        assert_eq!("Sleeper", league.platform());
        assert_eq!(crate::LeagueStatus::Complete, league.status());
        assert_eq!(12, league.teams().len());
        assert!(league.raw().starts_with("Raw:") && league.raw().contains("playoff_week_start"));

        let settings = league.settings();
        assert!(settings.has_playoffs);
        assert_eq!(14, settings.playoff_start_week);
        assert_eq!(6, settings.num_playoff_teams);
        assert_eq!(17, settings.draft_rounds);
        assert!(settings.scoring.is_some());
    }

    #[tokio::test]
    async fn a_sleeper_league_knows_its_players() {
        // no player cache is configured here; the league must still get its players
        let league = sleeper_league().await;
        let players = league.players();
        assert!(players.len() > 300, "only {} players", players.len());

        let missing: Vec<String> = league
            .rosters()
            .iter()
            .flat_map(|r| r.player_ids.clone())
            .filter(|id| !players.contains_key(id))
            .collect();
        assert!(
            missing.is_empty(),
            "rostered players without data: {missing:?}"
        );
    }

    #[tokio::test]
    async fn sleeper_standings_and_rosters_come_back_in_a_stable_order() {
        let league = sleeper_league().await;

        let standings = league.standings().entries;
        assert_eq!(12, standings.len());
        // best record first
        assert!(
            standings.windows(2).all(|w| w[0].wins >= w[1].wins),
            "{standings:#?}"
        );
        assert_eq!("10-3-0", league.record_for_roster("1".to_string()));

        let rosters = league.rosters();
        let ids: Vec<u32> = rosters.iter().map(|r| r.id.parse().unwrap()).collect();
        assert_eq!((1..=12).collect::<Vec<u32>>(), ids);

        // the rosters are kept in a HashMap; the order must not depend on its iteration order
        for _ in 0..10 {
            let again: Vec<String> = sleeper_league_standings_order(&league);
            assert_eq!(
                standings
                    .iter()
                    .map(|e| e.roster_id.clone())
                    .collect::<Vec<_>>(),
                again
            );
        }
    }

    fn sleeper_league_standings_order(league: &crate::League) -> Vec<String> {
        league
            .standings()
            .entries
            .into_iter()
            .map(|e| e.roster_id)
            .collect()
    }

    #[tokio::test]
    async fn a_sleeper_leagues_playoff_results_follow_the_bracket() {
        let league = sleeper_league().await;
        let results = league
            .playoff_results()
            .expect("the league has a winners bracket");
        assert_eq!("7", results[&1].roster_id, "champion");
        assert_eq!("9", results[&2].roster_id, "runner-up");
        assert_eq!("4", results[&3].roster_id, "third place");
    }

    #[tokio::test]
    async fn selecting_an_owner_picks_out_their_team_and_record() {
        let league = sleeper_league().await;
        assert_eq!(None, league.selected_owner_id());
        assert!(league.team().is_none());
        assert_eq!("N/A", league.record());

        let owner = league
            .standings()
            .entries
            .into_iter()
            .find(|e| e.roster_id == "1")
            .unwrap();
        let league =
            league.with_owner_id(ExternalId::new(Platform::Sleeper, owner.owner_id.clone()));

        assert_eq!(Some(owner.owner_id.clone()), league.selected_owner_id());
        let team = league.team().expect("the owner has a team in this league");
        assert_eq!(owner.owner_id, team.owner_id.unwrap());
        assert_eq!(owner.team_name, league.team_name());
        assert_eq!(
            format!("{}-{}-{}", owner.wins, owner.losses, owner.ties),
            league.record()
        );
        // roster 1 scored 1716 points and ten hundredths
        assert!(
            (league.total_points() - 1716.10).abs() < 1e-9,
            "{}",
            league.total_points()
        );
    }

    #[tokio::test]
    async fn a_fleaflicker_league_describes_itself() {
        let league = fleaflicker_league().await;
        assert_eq!(crate::LeagueStatus::PreDraft, league.status());
        assert_eq!("FleaFlicker", league.platform());
        assert!(league.id().to_string().starts_with("fleaflicker:"));
        assert!(league.raw().len() > 1000);
        assert_eq!(12, league.teams().len());
        assert!(
            league.draft().is_none(),
            "the fixture league has not drafted"
        );
        assert!(league.scoreboards().is_empty());
        assert!(league.playoff_results().is_none());
        assert!(league.settings().has_playoffs);
    }

    #[tokio::test]
    async fn fleaflicker_standings_survive_unowned_and_unnamed_teams() {
        // open slots have no owner and may have no name; the standings used to unwrap both
        let mut league = fleaflicker_league().await;
        if let crate::League::FleaFlicker { standings, .. } = &mut league {
            for division in &mut standings.divisions {
                for team in &mut division.teams {
                    team.owners = None;
                    team.name = None;
                }
            }
        }

        let entries = league.standings().entries;

        assert_eq!(12, entries.len());
        for entry in entries {
            assert_eq!(format!("Team {}", entry.roster_id), entry.team_name);
            assert_eq!(format!("Team {}", entry.roster_id), entry.owner_name);
            assert_eq!("", entry.owner_id);
        }
    }
}
