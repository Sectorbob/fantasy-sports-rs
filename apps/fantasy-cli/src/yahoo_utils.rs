use chrono::DateTime;
use log::*;
use serde_json;
use std::{
    collections::{HashMap, HashSet},
    env,
    error::Error,
    io::{self, BufWriter, Write},
};

use yahoo_fantasy_rs::{
    Client, GameCode, GameKey, LeagueKey, PlayerPosition, PlayerPositions, Team, TeamStandings,
    Token, Transaction,
};

const DEBUG_ENABLE: bool = false;

pub async fn build_client() -> Result<Client, yahoo_fantasy_rs::Error> {
    let yahoo_client_id = match env::var("YAHOO_CLIENT_ID") {
        Ok(id) => id,
        Err(_) => {
            return Err(yahoo_fantasy_rs::Error::UnknownError(
                "YAHOO_CLIENT_ID environment variable not set".to_string(),
            ));
        }
    };
    let yahoo_redirect_uri = match env::var("YAHOO_REDIRECT_URI") {
        Ok(uri) => uri,
        Err(_) => {
            return Err(yahoo_fantasy_rs::Error::UnknownError(
                "YAHOO_REDIRECT_URI environment variable not set".to_string(),
            ));
        }
    };
    let yahoo_token_file = match env::var("YAHOO_TOKEN_FILE") {
        Ok(file) => file,
        Err(_) => {
            return Err(yahoo_fantasy_rs::Error::UnknownError(
                "YAHOO_TOKEN_FILE environment variable not set".to_string(),
            ));
        }
    };
    let mut client = Client::new(yahoo_client_id, yahoo_redirect_uri, None);

    debug!("loading token file: {}", yahoo_token_file);

    let token = match load_token_file(&yahoo_token_file) {
        Ok(token) => token,
        // If fails, perform auth flow
        Err(err) => {
            return Err(yahoo_fantasy_rs::Error::UnknownError(format!(
                "Failed to load token file: {}",
                err
            )));
            // println!(
            //     "No token found or failed to load ({}). Performing OAuth flow...",
            //     err
            // );
            // match perform_oauth_flow(&mut client).await {
            //     Ok(token) => token,
            //     Err(_) => {
            //         println!("Failed to perform OAuth flow. Exiting.");
            //         return Err(yahoo_fantasy_rs::Error::UnknownError(
            //             "Failed to perform OAuth flow".to_string(),
            //         ));
            //     }

            // }
        }
    };
    client.token = Some(token);
    debug!("loaded token: {:#?}", client.token);
    info!("refreshing token");
    let new_token = client.refresh_token().await?;
    save_token_file(&yahoo_token_file, new_token);
    Ok(client)
}

pub async fn exec() -> Result<(), Box<dyn std::error::Error>> {
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
    if DEBUG_ENABLE {
        println!("Building Yahoo Fantasy Client...");
    }

    let client = build_client().await?;
    if DEBUG_ENABLE {
        println!("Yahoo Fantasy Client built");
    }

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
            .await
            .unwrap();

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
        let game = client.get_game(GameKey::Code(GameCode::NFL)).await.unwrap();
        println!("Game: {:#?}", game);
    }

    if enable_check_teams {
        let teams = client.get_teams_for_user(GameKey::Id(371)).await.unwrap();
        println!("Teams: {:#?}", teams);
    }

    if enable_check_league {
        let league = client.get_league(&ecr_2017).await.unwrap();
        println!("League: {:#?}", league);

        let mut team_standings: HashMap<String, &TeamStandings> = HashMap::new();

        let standings = client.get_league_standings(&ecr_2017).await.unwrap();

        for team in standings.teams() {
            if let Some(standings) = &team.team_standings {
                team_standings.insert(team.team_key.to_string(), standings);
            }
        }

        let teams = client.get_teams_in_league(&ecr_2017).await.unwrap();
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
            .await
            .unwrap();
        println!("Scoreboard: {:#?}", scoreboard);
    }

    if enable_check_league_settings {
        let league_settings = client
            .get_league_settings(&LeagueKey::new(GameKey::Id(371), 20028))
            .await
            .unwrap();
        println!("League Settings: {:#?}", league_settings);
    }

    if enable_check_league_standings {
        check_league_standings(&client, &public_league_key).await;
    }

    if enable_league_transactions {
        let transactions = client
            .get_league_transactions(&public_league_key)
            .await
            .unwrap();
        fancy_navigate_transactions(&transactions);
    }

    if enable_check_players {
        let players = client.get_players_for_league(&ecr_2017).await.unwrap();

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

fn load_token_file(file: &String) -> Result<Token, Box<dyn Error>> {
    let file = std::fs::File::open(file)?;
    let reader = std::io::BufReader::new(file);

    // Read the JSON contents of the file as an instance of `Token`.
    let token = serde_json::from_reader(reader)?;
    Ok(token)
}

fn save_token_file(file: &String, token: Token) {
    // d

    let f = std::fs::OpenOptions::new()
        .write(true)
        .create(true) // creates if not exists or opens
        .open(file)
        .unwrap();
    let writer = BufWriter::new(f);

    // Read the JSON contents of the file as an instance of `Token`.
    serde_json::to_writer_pretty(writer, &token).unwrap();
}

async fn perform_oauth_flow(client: &mut Client) -> Result<Token, ()> {
    let auth_request_url = client.get_request_auth_url().await.unwrap();
    println!("Auth request URL: {}", auth_request_url);

    let auth_code = receive_auth_code_from_stdin();
    println!("Auth code: {}", auth_code);
    let token = client.get_token(&auth_code).await.unwrap();
    println!("Token: {:#?}", token);
    Ok(token)
}

fn receive_auth_code_from_stdin() -> String {
    print!("Please enter the auth code: ");
    io::stdout().flush().unwrap();
    let mut auth_code = String::new();
    io::stdin().read_line(&mut auth_code).unwrap();

    auth_code.trim().to_string()
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
        let transaction_time = DateTime::from_timestamp(*transaction.timestamp(), 0).unwrap();
        // println!("{}: {}", transaction.key(), transaction_time);
        match transaction {
            Transaction::Commish {
                transaction_key,
                transaction_id,
                status,
                timestamp,
            } => {
                //println!("Commish({}): {}", transaction_key, transaction_time)
            }
            Transaction::Add {
                transaction_key,
                transaction_id,
                status,
                timestamp,
                players,
            } => {
                if let Some(players) = players {
                    for player in players {
                        match &player.display_position {
                            PlayerPositions::One(position) => {
                                player_positions.insert(position);
                            }
                            PlayerPositions::Many(positions) => {
                                positions.into_iter().for_each(|pos| {
                                    player_positions.insert(pos);
                                });
                            }
                        }
                        player_position_types.insert(&player.position_type);
                    }
                }
            }
            Transaction::Drop {
                transaction_key,
                transaction_id,
                status,
                timestamp,
                players,
            } => {
                if let Some(players) = players {
                    for player in players {
                        match &player.display_position {
                            PlayerPositions::One(position) => {
                                player_positions.insert(position);
                            }
                            PlayerPositions::Many(positions) => {
                                positions.into_iter().for_each(|pos| {
                                    player_positions.insert(pos);
                                });
                            }
                        }
                        player_position_types.insert(&player.position_type);
                    }
                }
            }
            Transaction::AddDrop {
                transaction_key,
                transaction_id,
                status,
                timestamp,
                players,
            } => {
                if let Some(players) = players {
                    for player in players {
                        match &player.display_position {
                            PlayerPositions::One(position) => {
                                player_positions.insert(position);
                            }
                            PlayerPositions::Many(positions) => {
                                positions.into_iter().for_each(|pos| {
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
                                positions.into_iter().for_each(|pos| {
                                    player_positions.insert(pos);
                                });
                            }
                        }
                        player_position_types.insert(&player.position_type);
                    }
                }
            }
            Transaction::Other {
                type_name,
                transaction_key,
                transaction_id,
                status,
                timestamp,
                players,
                rest,
            } => println!("Other({}): {:#?}", type_name, rest),
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
) -> Result<Vec<Team>, Box<dyn std::error::Error>> {
    let games = client
        .get_games(
            vec![game_code],
            seasons.iter().map(|s| s.as_str()).collect(),
        )
        .await
        .unwrap();

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
