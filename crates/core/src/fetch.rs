use crate::{data as core, sleeper_utils};
use futures::{TryStreamExt, stream::FuturesUnordered};
use sleeper_fantasy_rs::{self as sleeper};
use std::{
    collections::{HashMap, HashSet},
    path,
    str::FromStr,
};
use yahoo_fantasy_rs::{self as yahoo};

pub struct LeagueAccessor {
    sleeper_cli: Option<sleeper::Client>,
    sleeper_players_cache: HashMap<sleeper::Sport, HashMap<String, sleeper::Player>>,
    yahoo_cli: Option<yahoo::Client>,
    yahoo_players_cache: HashMap<yahoo::PlayerKey, yahoo::Player>,
}
impl LeagueAccessor {
    pub fn new() -> Self {
        let cache_dir = path::Path::new("cache");
        let sleeper_cache_dir = cache_dir.join("sleeper");
        let sleeper_cli = sleeper::Client::new_with_cache(None, sleeper_cache_dir);
        let yahoo_cache_dir = cache_dir.join("yahoo");
        let yahoo_cli = yahoo::Client::new_from_env()
            .expect("failed to configure yahoo client from env")
            .with_cache_dir(yahoo_cache_dir);
        let sleeper_players_store = HashMap::new();
        let yahoo_players_store = HashMap::new();
        LeagueAccessor {
            sleeper_cli: Some(sleeper_cli),
            sleeper_players_cache: sleeper_players_store,
            yahoo_cli: Some(yahoo_cli),
            yahoo_players_cache: yahoo_players_store,
        }
    }

    pub async fn fetch_league_context(&self, id: ExternalId) -> Result<core::League, Error> {
        match id.platform {
            Platform::Sleeper => {
                if let Some(cli) = &self.sleeper_cli {
                    fetch_sleeper_league_context_by_id(cli, id.id).await
                } else {
                    Err(Error::new("sleeper cli not configured"))
                }
            }
            Platform::Yahoo => {
                if let Some(cli) = &self.yahoo_cli {
                    fetch_yahoo_league_context(
                        cli,
                        yahoo::LeagueKey::from_str(&id.id).map_err(|err| {
                            Error::DevError(format!("unable to parse yahoo league key: {err}"))
                        })?,
                    )
                    .await
                } else {
                    Err(Error::new("yahoo cli not configured"))
                }
            }
        }
    }
}

pub(crate) async fn fetch_sleeper_league_context_by_id<T: Into<String>>(
    cli: &sleeper::Client,
    league_id: T,
) -> Result<core::League, Error> {
    let sleeper_league = cli.get_league(league_id.into()).await?;
    fetch_sleeper_league_context(cli, &sleeper_league).await
}

pub(crate) async fn fetch_sleeper_league_context(
    cli: &sleeper::Client,
    sleeper_league: &sleeper::League,
) -> Result<core::League, Error> {
    let players_future = cli.fetch_all_players(&sleeper_league.sport);
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
    let players = players_future.await?;
    // TODO: add player_ids for drafted players
    let players_in_league: HashMap<String, sleeper::Player> = player_id_set
        .into_iter()
        .filter_map(|owned_player_id| match players.get(&owned_player_id) {
            Some(player_ref) => Some((owned_player_id, player_ref.clone())),
            None => None,
        })
        .collect::<HashMap<String, sleeper::Player>>();

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
    players: &HashMap<String, sleeper::Player>,
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
        league_futures.push(fetch_sleeper_league_context(sleeper_client, sleeper_league));
    }

    let mut leagues = vec![];
    while let Some(yahoo_league) = league_futures.try_next().await? {
        leagues.push(yahoo_league);
    }

    Ok(leagues)
}

async fn get_yahoo_leagues_for_user(
    yahoo_client: &yahoo_fantasy_rs::Client,
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
            yahoo_team.team_key.league_key,
        ));
    }
    while let Some(yahoo_league) = yahoo_teams_tasks.try_next().await? {
        leagues.push(yahoo_league);
    }

    Ok(leagues)
}

pub struct ExternalId {
    pub id: String,
    pub platform: Platform,
}

pub enum Platform {
    Sleeper,
    Yahoo,
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
