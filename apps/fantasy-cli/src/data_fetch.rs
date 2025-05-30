use std::collections::{HashMap, HashSet};

use futures::{TryStreamExt, stream::FuturesUnordered};
use log::{error, info};
use sleeper_fantasy_rs::{self as sleeper};
use yahoo_fantasy_rs::{GameCode, GameKey, Roster, Team};

use crate::{data::League, sleeper_utils, yahoo_utils};

pub async fn get_all_leagues(
    seasons: Vec<String>,
) -> Result<Vec<League>, Box<dyn std::error::Error + Send + Sync>> {
    let sleeper_user_id = "727598467304811923".to_string();
    let sleeper_sport = sleeper::Sport::NFL;
    let yahoo_game_code = GameCode::NFL;
    let yahoo_client = match yahoo_utils::build_client().await {
        Ok(client) => client,
        Err(err) => return Err(Box::new(err)),
    };
    let sleeper_client = sleeper_fantasy_rs::Client::new(None);

    let sleeper_players = sleeper_client.fetch_all_players(&sleeper_sport).await?;
    let mut leagues: Vec<League> = vec![];
    let sleeper_leagues_future = get_sleeper_leagues(
        &sleeper_client,
        &seasons,
        &sleeper_user_id,
        &sleeper_sport,
        &sleeper_players,
    );
    let yahoo_leagues_future = get_yahoo_leagues(&yahoo_client, &seasons, yahoo_game_code);
    let (sleeper_leagues, yahoo_leagues) =
        tokio::join!(sleeper_leagues_future, yahoo_leagues_future);
    match sleeper_leagues {
        Ok(sleeper_leagues) => {
            leagues.extend(sleeper_leagues);
        }
        Err(err) => {
            error!("Error fetching Sleeper leagues: {:?}", err);
            return Err(err);
        }
    }
    match yahoo_leagues {
        Ok(yahoo_leagues) => {
            leagues.extend(yahoo_leagues);
        }
        Err(err) => {
            error!("Error fetching Yahoo leagues: {:?}", err);
            return Err(err);
        }
    }
    info!("Total leagues fetched: {}", leagues.len());
    // Sort leagues by season in descending order
    leagues.sort_by(|a, b| b.season().cmp(&a.season()));

    Ok(leagues)
}

async fn get_sleeper_leagues(
    sleeper_client: &sleeper::Client,
    seasons: &[String],
    sleeper_user_id: &String,
    sleeper_sport: &sleeper::Sport,
    players: &HashMap<String, sleeper::Player>,
) -> Result<Vec<League>, Box<dyn std::error::Error + Send + Sync>> {
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
                eprintln!("Error fetching Sleeper leagues: {:?}", e);
                // panic!("Error fetching Sleeper leagues: {:?}", e); //TDOO handle this gracefully
                // return Err(Box::new(e));
            }
        }
        // handle result
    }

    let mut leagues = vec![];

    for sleeper_league in sleeper_leagues {
        let owners_future = sleeper_client.get_users_for_league(&sleeper_league.league_id);
        let rosters_future = sleeper_client.get_rosters_for_league(&sleeper_league.league_id);
        let (owners_result, rosters_result) = tokio::join!(owners_future, rosters_future);
        let owners = match owners_result {
            Ok(owners) => owners
                .into_iter()
                .map(|owner| (owner.user_id.clone(), owner))
                .collect::<HashMap<String, sleeper::User>>(),
            Err(e) => {
                eprintln!(
                    "Error fetching owners for league {}: {:?}",
                    sleeper_league.league_id, e
                );
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "Error fetching owners for league {}: {:?}",
                        sleeper_league.league_id, e
                    ),
                )));
            }
        };
        let rosters = match rosters_result {
            Ok(rosters) => rosters,
            Err(e) => {
                eprintln!(
                    "Error fetching rosters for league {}: {:?}",
                    sleeper_league.league_id, e
                );
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "Error fetching rosters for league {}: {:?}",
                        sleeper_league.league_id, e
                    ),
                )));
            }
        };

        let matchups = match sleeper_utils::get_league_matchups(
            &sleeper_league.league_id,
            Some(&sleeper_league),
            &sleeper_client,
        )
        .await
        {
            Ok(matchups) => matchups,
            Err(e) => {
                eprintln!(
                    "Error fetching matchups for league {}: {:?}",
                    sleeper_league.league_id, e
                );
                HashMap::new()
                // return Err(Box::new(std::io::Error::new(
                //     std::io::ErrorKind::Other,
                //     format!(
                //         "Error fetching matchups for league {}: {:?}",
                //         sleeper_league.league_id, e
                //     ),
                // )));
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
        // TODO: add player_ids for drafted players
        let players_in_league: HashMap<String, sleeper::Player> = player_id_set
            .into_iter()
            .filter_map(|owned_player_id| match players.get(&owned_player_id) {
                Some(player_ref) => Some((owned_player_id, player_ref.clone())),
                None => None,
            })
            .collect::<HashMap<String, sleeper::Player>>();

        leagues.push(League::Sleeper {
            league: sleeper_league,
            owners,
            players: players_in_league,
            rosters,
            matchups,
            user_id: sleeper_user_id.clone(),
        });
    }

    Ok(leagues)
}

async fn get_yahoo_leagues(
    yahoo_client: &yahoo_fantasy_rs::Client,
    seasons: &[String],
    yahoo_game_code: GameCode,
) -> Result<Vec<League>, Box<dyn std::error::Error + Send + Sync>> {
    let games = yahoo_client
        .get_games(
            vec![yahoo_game_code],
            seasons.iter().map(|s| s.as_str()).collect(),
        )
        .await
        .unwrap();

    let mut yahoo_teams: Vec<Team> = vec![];
    for game in games {
        let game_key: GameKey = GameKey::Id(game.game_id);
        let teams_in_season = yahoo_client.get_teams_for_user(game_key).await?;
        for team in teams_in_season {
            yahoo_teams.push(team);
        }
    }

    let mut leagues: Vec<League> = vec![];

    let mut yahoo_teams_tasks: FuturesUnordered<_> = FuturesUnordered::new();
    for yahoo_team in yahoo_teams {
        yahoo_teams_tasks.push(async move {
            let yahoo_league = yahoo_client
                .get_league(&yahoo_team.team_key.league_key)
                .await
                .unwrap()
                .unwrap();
            let standings = yahoo_client
                .get_league_standings(&yahoo_team.team_key.league_key)
                .await
                .unwrap();
            let rosters = yahoo_client
                .get_rosters_in_league(&yahoo_team.team_key.league_key)
                .await
                .unwrap()
                .into_iter()
                .map(|team| (team.team_id, team.roster.unwrap()))
                .collect::<HashMap<u32, Roster>>();
            let settings = yahoo_client
                .get_league_settings(&yahoo_team.team_key.league_key)
                .await
                .unwrap();
            Ok::<League, yahoo_fantasy_rs::Error>(League::Yahoo {
                league: yahoo_league,
                matchups: HashMap::new(), // TODO: Yahoo does not provide matchups in the same way as Sleeper
                rosters,
                settings,
                standings,
                team: yahoo_team,
            })
        });
    }
    while let Some(yahoo_league) = yahoo_teams_tasks.try_next().await? {
        leagues.push(yahoo_league);
    }

    Ok(leagues)
}
