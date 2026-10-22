use std::collections::HashSet;

use fantasy_sports_core::{self as core, ExternalId};
use futures::{TryStreamExt, stream::FuturesUnordered};
use fantasy_sports_sleeper::{self as sleeper};
use fantasy_sports_yahoo::{self as yahoo};

pub async fn get_all_leagues(
    seasons: Vec<String>,
    league_accessor: &core::LeagueAccessor,
    sleeper_user_id: Option<&str>,
) -> Result<Vec<core::League>, Box<dyn std::error::Error + Send + Sync>> {
    let sleeper_sport = sleeper::Sport::NFL;
    let yahoo_game_code = yahoo::GameCode::NFL;

    // let sleeper_players = sleeper_client.fetch_all_players(&sleeper_sport).await?;
    let mut leagues: Vec<core::League> = vec![];
    let sleeper_leagues_future = async {
        match sleeper_user_id {
            Some(user_id) => {
                get_sleeper_leagues(league_accessor, &seasons, &user_id.to_string(), &sleeper_sport).await
            }
            None => {
                log::info!("sleeper_user_id is not configured, skipping Sleeper leagues");
                Ok(vec![])
            }
        }
    };
    let yahoo_leagues_future = get_yahoo_leagues(league_accessor, &seasons, yahoo_game_code);
    let (sleeper_leagues, yahoo_leagues) =
        tokio::join!(sleeper_leagues_future, yahoo_leagues_future);
    match sleeper_leagues {
        Ok(sleeper_leagues) => {
            leagues.extend(sleeper_leagues);
        }
        Err(err) => {
            log::error!("Error fetching Sleeper leagues: {:?}", err);
            return Err(err);
        }
    }
    match yahoo_leagues {
        Ok(yahoo_leagues) => {
            leagues.extend(yahoo_leagues);
        }
        Err(err) => {
            log::error!("Error fetching Yahoo leagues: {:?}", err);
            return Err(err);
        }
    }
    log::info!("Total leagues fetched: {}", leagues.len());

    // Sort leagues by season in descending order
    leagues.sort_by(|a, b| b.season().cmp(&a.season()));

    Ok(leagues)
}

async fn get_sleeper_leagues(
    league_accessor: &core::LeagueAccessor,
    seasons: &[String],
    sleeper_user_id: &String,
    sleeper_sport: &sleeper::Sport,
    // players: &HashMap<String, sleeper::Player>,
) -> Result<Vec<core::League>, Box<dyn std::error::Error + Send + Sync>> {
    let sleeper_cli = league_accessor
        .sleeper_cli
        .as_ref()
        .ok_or(core::Error::DevError("sleeper_cli not set".to_string()))?;

    let mut sleeper_leagues_for_season_tasks: FuturesUnordered<_> = seasons
        .iter()
        .map(|season| sleeper_cli.get_leagues_for_user(sleeper_user_id, sleeper_sport, season))
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
                eprintln!("Error fetching Sleeper leagues: {:?}", e);
                // panic!("Error fetching Sleeper leagues: {:?}", e); //TDOO handle this gracefully
                // return Err(Box::new(e));
            }
        }
        // handle result
    }

    let mut leagues = vec![];

    for sleeper_league in sleeper_leagues {
        let league_context = league_accessor
            .fetch_league_context(
                &core::ExternalId {
                    platform: fantasy_sports_core::Platform::Sleeper,
                    id: sleeper_league.league_id.clone(),
                },
                false,
            )
            .await?;

        leagues.push(league_context.with_owner_id(ExternalId::new(
            fantasy_sports_core::Platform::Sleeper,
            sleeper_user_id.clone(),
        )));
    }

    Ok(leagues)
}

async fn get_yahoo_leagues(
    league_accessor: &core::LeagueAccessor,
    seasons: &[String],
    yahoo_game_code: yahoo::GameCode,
) -> Result<Vec<core::League>, Box<dyn std::error::Error + Send + Sync>> {
    let yahoo_cli = league_accessor
        .yahoo_cli
        .as_ref()
        .ok_or(core::Error::DevError("yahoo_cli not set".to_string()))?;

    let games = yahoo_cli
        .get_games(
            vec![yahoo_game_code],
            seasons.iter().map(|s| s.as_str()).collect(),
        )
        .await?;

    let mut yahoo_teams: Vec<yahoo::Team> = vec![];
    let mut user_ids = HashSet::new();
    for game in games {
        let game_key: yahoo::GameKey = yahoo::GameKey::Id(game.game_id);
        let teams_in_season = match yahoo_cli.get_teams_for_user(game_key.clone()).await {
            Ok(t) => t,
            Err(e) => {
                log::warn!("failed to fetch yahoo teams for user in {game_key}: {e}");
                continue;
            }
        };
        for team in teams_in_season {
            user_ids.extend(team.managers().iter().filter_map(|m| {
                if m.is_current_login.unwrap_or(false) {
                    Some(m.guid.clone())
                } else {
                    None
                }
            }));
            yahoo_teams.push(team);
        }
    }

    log::info!("Yahoo User id Guids: {user_ids:?}");
    let yahoo_user_guid = user_ids.into_iter().next();

    let mut leagues: Vec<core::League> = vec![];

    let mut yahoo_teams_tasks: FuturesUnordered<_> = FuturesUnordered::new();
    for yahoo_team in yahoo_teams {
        yahoo_teams_tasks.push(async move {
            let league_key = &yahoo_team.team_key.league_key;
            let league_ctx = league_accessor
                .fetch_league_context(
                    &core::ExternalId {
                        platform: fantasy_sports_core::Platform::Yahoo,
                        id: league_key.to_string(),
                    },
                    false,
                )
                .await?;

            Ok::<core::League, fantasy_sports_core::Error>(league_ctx)
        });
    }
    while let Some(yahoo_league) = yahoo_teams_tasks.try_next().await? {
        if let Some(user_guid) = &yahoo_user_guid {
            leagues.push(yahoo_league.with_owner_id(ExternalId {
                platform: fantasy_sports_core::Platform::Yahoo,
                id: user_guid.clone(),
            }));
        } else {
            leagues.push(yahoo_league);
        }
    }

    Ok(leagues)
}
