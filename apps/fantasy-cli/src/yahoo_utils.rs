use fantasy_sports_core::LeagueAccessor;
use fantasy_sports_yahoo::{
    Client, Error as YahooError, GameCode, GameKey, LeagueKey, PlayerPosition, PlayerPositions,
    Team, TeamStandings, Token, Transaction,
};
use log::*;
use std::{
    collections::{HashMap, HashSet},
    io::Error as IOError,
};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};

pub async fn exec(league_accessor: LeagueAccessor) -> Result<(), fantasy_sports_yahoo::Error> {
    let enable_check_users = false;
    let enable_check_game = false;
    let enable_check_games = false;
    let enable_check_teams = false;
    let enable_check_league = false;
    let enable_check_league_scoreboard = false;
    let enable_check_league_settings = false;
    let enable_check_league_standings = false;
    let enable_league_transactions = false;
    let enable_check_players = false;
    let enable_check_team_in_seasons = true;

    debug!("Building Yahoo Fantasy Client...");
    let client = league_accessor
        .yahoo_cli
        .ok_or(fantasy_sports_yahoo::Error::UnknownError(
            "no yahoo_cli is set".to_string(),
        ))?;
    debug!("Yahoo Fantasy Client built");

    let public_league_key = LeagueKey::new(GameKey::Id(223), 431);
    let ecr_2017 = LeagueKey::new(GameKey::Id(371), 20028);

    if enable_check_users {
        let users = client.get_users(true).await.unwrap();
        info!("Users: {:#?}", users);
    }

    if enable_check_games {
        let mut seasons = vec![];
        for season in 2009..2025 {
            seasons.push(format!("{}", season));
        }

        let games = client
            .get_games(
                vec![GameCode::NFL],
                seasons.iter().map(|s| s.as_str()).collect(),
            )
            .await?;

        let mut game_codes: HashSet<&GameCode> = HashSet::new();
        for game in games.iter() {
            game_codes.insert(&game.code);
        }

        println!(
            "Games: {:#?}",
            games // .iter()
                  // .filter(|item| matches!(item.code, GameCode::OTHER(_)))
                  // .collect::<Vec<&Game>>()
        );
        println!(
            "Game Codes: {:#?}",
            game_codes // .iter()
                       // .filter(|item| matches!(item, GameCode::OTHER(_)))
                       // .collect::<Vec<&&GameCode>>()
        );
    }

    if enable_check_game {
        let game = client.get_game(GameKey::Code(GameCode::NFL)).await?;
        println!("Game: {:#?}", game);
    }

    if enable_check_teams {
        let teams = client.get_teams_for_user(GameKey::Id(371)).await?;
        println!("Teams: {:#?}", teams);
    }

    if enable_check_league {
        let league = client.get_league(&ecr_2017).await?;
        println!("League: {:#?}", league);

        let mut team_standings: HashMap<String, &TeamStandings> = HashMap::new();

        let standings = client.get_league_standings(&ecr_2017).await?;

        for team in &standings.teams {
            if let Some(standings) = &team.team_standings {
                team_standings.insert(team.team_key.to_string(), standings);
            }
        }

        let teams = client.get_teams_in_league(&ecr_2017).await?;
        println!("Teams: {:#?}", teams);
        for team in teams {
            println!(" {}", team.name);
            if let Some(team_points) = team.team_points {
                println!("   Points {:?}", team_points);
            }
            if let Some(standings) = team_standings.get(&team.team_key.to_string()) {
                println!(
                    "   Standings Rank={} Record={}-{}-{}",
                    standings.rank.unwrap_or(0),
                    standings.outcome_totals.wins,
                    standings.outcome_totals.losses,
                    standings.outcome_totals.ties,
                );
            }

            // let roster = client.get_roster_for_team(&TeamKey::new(&ecr_2017, team.team_id)).await.unwrap();
            // println!("   Roster: ");
            // for player in roster.players.players {
            //     println!("     {} ({})", player.name.full, player.display_position);
            // }
        }

        // let scoreboard = client.get_league_scoreboard(&ecr_2017).await.unwrap();
        // println!("Scorebaord Week: {}", scoreboard.)
    }

    if enable_check_league_scoreboard {
        let scoreboard = client
            .get_league_scoreboard(&public_league_key, None, None)
            .await?;
        println!("Scoreboard: {:#?}", scoreboard);
    }

    if enable_check_league_settings {
        let league_settings = client
            .get_league_settings(&LeagueKey::new(GameKey::Id(371), 20028))
            .await?;
        println!("League Settings: {:#?}", league_settings);
    }

    if enable_check_league_standings {
        check_league_standings(&client, &public_league_key).await;
    }

    if enable_league_transactions {
        let transactions = client.get_league_transactions(&public_league_key).await?;
        fancy_navigate_transactions(&transactions);
    }

    if enable_check_players {
        let players = client.get_players_for_league(&ecr_2017, None).await?;
        info!("got {} yahoo players in response", players.len());

        let mut player_statuses: HashSet<String> = HashSet::new();
        for player in players {
            if let Some(status) = player.status {
                player_statuses.insert(status);
            }
        }
        println!("Players Statuses: {:#?}", player_statuses);
        // println!("Players: {:#?}", players);
    }

    if enable_check_team_in_seasons {
        let teams = get_teams_for_seasons(
            &client,
            vec![
                "2015".to_string(),
                "2016".to_string(),
                "2017".to_string(),
                "2018".to_string(),
                "2019".to_string(),
            ],
            GameCode::NFL,
        )
        .await?;
        for team in teams {
            println!("{}", team.name)
        }
    }

    Ok(())
}

#[allow(dead_code)]
async fn perform_oauth_flow(client: &mut Client) -> Result<Token, IOError> {
    let auth_request_url = match client.get_request_auth_url().await {
        Ok(url) => url,
        Err(err) => {
            return Err(IOError::other(format!(
                "received unexpected error while getting request auth url: {:?}",
                err
            )));
        }
    };
    debug!("Auth request URL: {}", auth_request_url);

    let auth_code = receive_auth_code_from_stdin().await?;
    debug!("Auth code: {}", auth_code);
    let token = match client.get_token(&auth_code).await {
        Ok(t) => t,
        Err(err) => {
            return Err(IOError::other(format!(
                "received unexpected error while getting token: {:?}",
                err
            )));
        }
    };
    debug!("Token: {:#?}", token);
    Ok(token)
}

#[allow(dead_code)]
async fn receive_auth_code_from_stdin() -> Result<String, IOError> {
    print!("Please enter the auth code: ");
    io::stdout().flush().await?;
    let mut auth_code = String::new();
    let mut reader = BufReader::new(io::stdin());

    // Read a line asynchronously
    let _ = reader.read_line(&mut auth_code).await?;
    Ok(auth_code.trim().to_string())
}

async fn check_league_standings(client: &Client, league_key: &LeagueKey) {
    let league_standings = client.get_league_standings(league_key).await.unwrap();
    println!("League Standings: {:#?}", league_standings);
}

fn fancy_navigate_transactions(transactions: &Vec<Transaction>) {
    println!("League Transactions:");
    let mut player_positions: HashSet<&PlayerPosition> = HashSet::new();
    let mut player_position_types = HashSet::new();

    for transaction in transactions {
        match transaction {
            Transaction::Commish { .. } => {
                //println!("Commish({}): {}", transaction_key, transaction_time)
            }
            Transaction::Add { players, .. } => {
                if let Some(players) = players {
                    for player in players {
                        match &player.display_position {
                            PlayerPositions::One(position) => {
                                player_positions.insert(position);
                            }
                            PlayerPositions::Many(positions) => {
                                positions.iter().for_each(|pos| {
                                    player_positions.insert(pos);
                                });
                            }
                        }
                        player_position_types.insert(&player.position_type);
                    }
                }
            }
            Transaction::Drop {
                transaction_key: _,
                transaction_id: _,
                status: _,
                timestamp: _,
                players,
            } => {
                if let Some(players) = players {
                    for player in players {
                        match &player.display_position {
                            PlayerPositions::One(position) => {
                                player_positions.insert(position);
                            }
                            PlayerPositions::Many(positions) => {
                                positions.iter().for_each(|pos| {
                                    player_positions.insert(pos);
                                });
                            }
                        }
                        player_position_types.insert(&player.position_type);
                    }
                }
            }
            Transaction::AddDrop {
                transaction_key: _,
                transaction_id: _,
                status: _,
                timestamp: _,
                players,
            } => {
                if let Some(players) = players {
                    for player in players {
                        match &player.display_position {
                            PlayerPositions::One(position) => {
                                player_positions.insert(position);
                            }
                            PlayerPositions::Many(positions) => {
                                positions.iter().for_each(|pos| {
                                    player_positions.insert(pos);
                                });
                            }
                        }
                        player_position_types.insert(&player.position_type);
                    }
                }
            }
            Transaction::Trade { players, .. } => {
                // println!("trade: {:#?}", transaction);
                if let Some(players) = players {
                    for player in players {
                        match &player.display_position {
                            PlayerPositions::One(position) => {
                                player_positions.insert(position);
                            }
                            PlayerPositions::Many(positions) => {
                                positions.iter().for_each(|pos| {
                                    player_positions.insert(pos);
                                });
                            }
                        }
                        player_position_types.insert(&player.position_type);
                    }
                }
            }
        }
        // if let Some(players) = transaction.players {

        //     for player in players.players {
        //         player_positions.insert(player.display_position);
        //         // player_position_types.insert(player.position_type);
        //     }

        // }
        // if transaction._type != "commish" {
        //     println!("{:#?}", transaction);
        // }

        // println!("{:#?}", transaction);
        println!("Player Positions:      {:#?}", player_positions);
        // println!("Player Position Types: {:#?}", player_position_types);
    }
}

async fn get_teams_for_seasons(
    client: &Client,
    seasons: Vec<String>,
    game_code: GameCode,
) -> Result<Vec<Team>, YahooError> {
    let games = client
        .get_games(
            vec![game_code],
            seasons.iter().map(|s| s.as_str()).collect(),
        )
        .await?;

    let mut yahoo_teams: Vec<Team> = vec![];
    for game in games {
        let game_key: GameKey = GameKey::Id(game.game_id);
        let teams_in_season = client.get_teams_for_user(game_key).await?;
        for team in teams_in_season {
            yahoo_teams.push(team);
        }
    }
    Ok(yahoo_teams)
}
