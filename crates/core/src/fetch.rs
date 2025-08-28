use crate::{
    ExternalId, Platform,
    data::{self as core},
    player_cache::PlayerCache,
    sleeper_utils,
};
use futures::{
    StreamExt, TryStreamExt,
    stream::{FuturesOrdered, FuturesUnordered},
};
use sleeper_fantasy_rs::{self as sleeper};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    str::FromStr,
};
use tokio::io;
use walkdir::WalkDir;
use yahoo_fantasy_rs::{self as yahoo};

pub struct LeagueAccessor {
    pub sleeper_cli: Option<sleeper::Client>,
    players_cache: Option<PlayerCache>,
    pub yahoo_cli: Option<yahoo::Client>,
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
        let player_cache = player_cache
            .map(|player_cache| PlayerCache::new_with_path(&player_cache, Sport::NFL).ok())
            .flatten();
        if let Some(player_cache) = &player_cache {
            player_cache.init().await?;
        }

        let sleeper_cli = match sleeper_cache_dir {
            Some(sleeper_cache_dir) => sleeper::Client::new().with_cache(sleeper_cache_dir),
            None => sleeper::Client::new(),
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

    pub async fn fetch_league_ids_for_user(
        &self,
        user_id: &ExternalId,
        seasons: Vec<String>,
        sport: &Sport,
    ) -> Result<Vec<ExternalId>, Error> {
        match &user_id.platform {
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
        match sleeper_utils::get_league_matchups(&sleeper_league, &cli, &force_update).await {
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
        match sleeper_utils::get_league_transactions(cli, &sleeper_league, &force_update).await {
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
            season = &sleeper_league.season,
            league_id = &sleeper_league.league_id
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
        .map_or(0, |v| v)
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
        i = i + 1;
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
    seasons: &Vec<String>,
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
}

async fn fetch_yahoo_league_keys_for_user(
    yahoo_client: &yahoo_fantasy_rs::Client,
    seasons: &Vec<String>,
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
pub enum Error {
    #[error("player cache error: {0}")]
    PlayerCacheError(#[from] crate::player_cache::Error),
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
    async fn test_fetch_league_context() {
        let sleeper_user_id = ExternalId::try_from("sleeper:727598467304811923").unwrap();
        let yahoo_user_id = ExternalId::try_from("yahoo:GGQ52IUUT34FUMAXVLVIZ6V2QP").unwrap();

        let cases = [
            (
                "yahoo:359.l.564503",
                "2016",
                "Yahoo",
                291, /* players */
                180, /* draft pick count */
                323, /* transactions count */
                16,  //TODO: FUCK shoudl be 16  /* weeks of matchups */
                yahoo_user_id.clone(),
                "All I Do Is Lose",
            ),
            (
                "yahoo:371.l.20028",
                "2017",
                "Yahoo",
                300, /* players */
                180, /* draft pick count */
                331, /* transactions count */
                16,  //TODO: FUCK shoudl be 16  /* weeks of matchups */
                yahoo_user_id.clone(),
                "All I Do Is Wynn",
            ),
            (
                "yahoo:380.l.129397",
                "2018",
                "Yahoo",
                317, /* players */
                204, /* draft pick count */
                298, /* transactions count */
                16,  //TODO: FUCK shoudl be 16  /* weeks of matchups */
                yahoo_user_id.clone(),
                "Stormy Badgers 01",
            ),
            (
                "sleeper:164974877262287888",
                "2019",
                "Sleeper",
                359, /* players (was 278 before adding all player_ids from draft and transactions) */
                204, /* draft pick count */
                500, /* transactions count */
                16,  /* weeks of matchups */
                sleeper_user_id.clone(),
                "Stormy Badgers 01",
            ),
            (
                "sleeper:121226934266994911",
                "2020",
                "Sleeper",
                369, /* players */
                204, /* draft pick count */
                673, /* transactions count */
                16,  /* weeks of matchups */
                sleeper_user_id.clone(),
                "All Rise",
            ),
            (
                "sleeper:652490366221046118",
                "2021",
                "Sleeper",
                357, /* players */
                204, /* draft pick count */
                538, /* transactions count */
                17,  /* weeks of matchups */
                sleeper_user_id.clone(),
                "All Rise",
            ),
            (
                "sleeper:323299806782982483",
                "2022",
                "Sleeper",
                342, /* players */
                204, /* draft pick count */
                415, /* transactions count */
                17,  /* weeks of matchups */
                sleeper_user_id.clone(),
                "There's Always Next Year",
            ),
            (
                "sleeper:383786432088067320",
                "2023",
                "Sleeper",
                298, /* players */
                192, /* draft pick count */
                379, /* transactions count */
                // TODO: uh oh, 500 might be the max
                17, /* weeks of matchups */
                sleeper_user_id.clone(),
                "Kicked in the Dabolls",
            ),
            (
                "sleeper:4581480575134427121",
                "2024",
                "Sleeper",
                302, /* players */
                192, /* draft pick count */
                294, /* transactions count */
                17,  /* weeks of matchups */
                sleeper_user_id.clone(),
                "Golden Comets 25",
            ),
        ];

        let mut fixture = LeagueAccessor::new().with_cache_dir("cache");
        fixture.init().await.expect("failed to initialized");

        debug_assert!(fixture.sleeper_enabled(), "sleeper not enabled");
        debug_assert!(fixture.yahoo_enabled(), "yahoo not enabled");

        // Execute and Assert
        for (
            league_id,
            season,
            platform,
            num_of_players,
            number_of_draft_picks,
            num_of_txns,
            weeks_of_matchups,
            user_id,
            expected_team_name,
        ) in cases
        {
            let external_id = ExternalId::try_from(league_id).expect("invalid league_id");
            let league = fixture
                .fetch_league_context(&external_id, false)
                .await
                .expect("failed to get sleeper league")
                .with_owner_id(user_id.clone());
            debug_assert_eq!(
                season,
                league.season(),
                "season is incorrect for {league_id}"
            );
            debug_assert_eq!(
                platform,
                league.platform(),
                "platform is incorrect for {league_id}"
            );
            debug_assert_eq!(
                expected_team_name,
                league.team_name(),
                "team_name is incorrect for {league_id}"
            );
            debug_assert_eq!(
                num_of_players,
                league.players().len(),
                "num of players is incorrect in {league_id}",
            );
            debug_assert!(league.draft().is_some(), "no draft for {league_id}");
            debug_assert_eq!(
                number_of_draft_picks,
                league.draft().unwrap().picks.len(),
                "num of draft picks is incorrect in {league_id}",
            );
            debug_assert_eq!(
                num_of_txns,
                league.transactions().len(),
                "num of transactions is incorrect in {league_id}",
            );
            let scoreboards = league.scoreboards();
            debug_assert_eq!(
                weeks_of_matchups,
                scoreboards.len(),
                "num of weeks of scoreboards is incorrect in {league_id}",
            );
            for scoreboard in scoreboards {
                let week = scoreboard.matchups[0].week;
                if week < 14 {
                    debug_assert_eq!(
                        6,
                        scoreboard.matchups.len(),
                        "week {week} matchup count is incorrect in {league_id}",
                    );
                }
            }
        }

        // assert that only cache is hit
        debug_assert_eq!(
            0,
            fixture.sleeper_cli.unwrap().request_count(),
            "sleeper api was hit"
        );
        debug_assert_eq!(
            1, // because of token refresh
            fixture.yahoo_cli.unwrap().request_count(),
            "yahoo api was hit"
        );
    }
}
