mod client;
mod data;

use chrono::{DateTime};
use client::Client;
use data::{GameCode, GameKey, LeagueKey, TeamStandings, Token, Transaction};
use serde_json;
use std::{
    collections::{HashMap, HashSet},
    env,
    error::Error,
    io::{self, BufWriter, Write},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let enable_auto_token_refresh = true;
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

    let yahoo_client_id = env::var("YAHOO_CLIENT_ID").unwrap();
    let yahoo_redirect_uri = env::var("YAHOO_REDIRECT_URI").unwrap();
    let yahoo_token_file = env::var("YAHOO_TOKEN_FILE").unwrap();
    let mut client = client::Client::new(yahoo_client_id, yahoo_redirect_uri);

    // Try to load token from token file
    match load_token_file(&yahoo_token_file) {
        Ok(token) => client.token = Some(token),
        // If fails, perform auth flow
        Err(_) => perform_oauth_flow(&mut client).await,
    }

    if enable_auto_token_refresh {
        let new_token = client.refresh_token().await.unwrap();
        save_token_file(&yahoo_token_file, new_token);
    }

    let public_league_key = LeagueKey::new(GameKey::Id(223).to_string().as_str(), 431);
    let ecr_2017 = LeagueKey::new("371", 20028);

    if enable_check_users {
        let users = client.get_users(true).await.unwrap();
        println!("Users: {:#?}", users);
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

        let mut team_standings: HashMap<String, TeamStandings> = HashMap::new();

        for team in client
            .get_league_standings(&ecr_2017)
            .await
            .unwrap()
            .teams
            .teams
        {
            if let Some(standings) = team.team_standings {
                team_standings.insert(team.team_key, standings);
            }
        }

        let teams = client.get_teams_in_league(&ecr_2017).await.unwrap();
        println!("Teams: {:#?}", teams);
        for team in teams {
            println!(" {}", team.name);
            if let Some(team_points) = team.team_points {
                println!("   Points {:?}", team_points);
            }
            if let Some(standings) = team_standings.get(&team.team_key) {
                println!(
                    "   Standings Rank={} Record={}-{}-{}",
                    standings.rank,
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
            .get_league_settings(&LeagueKey::new("371", 20028))
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

async fn perform_oauth_flow(client: &mut Client) {
    let auth_request_url = client.get_request_auth_url().await.unwrap();
    println!("Auth request URL: {}", auth_request_url);

    let auth_code = receive_auth_code_from_stdin();
    println!("Auth code: {}", auth_code);
    let token = client.get_token(&auth_code).await.unwrap();
    println!("Token: {:#?}", token);
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
    let mut player_positions: HashSet<&data::PlayerPosition> = HashSet::new();
    let mut player_position_types = HashSet::new();

    for transaction in transactions {
        let transaction_time = DateTime::from_timestamp(*transaction.timestamp(), 0).unwrap();
        // println!("{}: {}", transaction.key(), transaction_time);
        match transaction {
            data::Transaction::Commish {
                transaction_key,
                transaction_id,
                status,
                timestamp,
            } => {
                //println!("Commish({}): {}", transaction_key, transaction_time)
            }
            data::Transaction::Add {
                transaction_key,
                transaction_id,
                status,
                timestamp,
                players,
            } => {
                if let Some(players) = players {
                    for player in players {
                        match &player.display_position {
                            data::PlayerPositions::One(position) => {
                                player_positions.insert(position);
                            }
                            data::PlayerPositions::Many(positions) => {
                                positions.into_iter().for_each(|pos| {
                                    player_positions.insert(pos);
                                });
                            }
                        }
                        player_position_types.insert(&player.position_type);
                    }
                }
            }
            data::Transaction::Drop {
                transaction_key,
                transaction_id,
                status,
                timestamp,
                players,
            } => {
                if let Some(players) = players {
                    for player in players {
                        match &player.display_position {
                            data::PlayerPositions::One(position) => {
                                player_positions.insert(position);
                            }
                            data::PlayerPositions::Many(positions) => {
                                positions.into_iter().for_each(|pos| {
                                    player_positions.insert(pos);
                                });
                            }
                        }
                        player_position_types.insert(&player.position_type);
                    }
                }
            }
            data::Transaction::AddDrop {
                transaction_key,
                transaction_id,
                status,
                timestamp,
                players,
            } => {
                if let Some(players) = players {
                    for player in players {
                        match &player.display_position {
                            data::PlayerPositions::One(position) => {
                                player_positions.insert(position);
                            }
                            data::PlayerPositions::Many(positions) => {
                                positions.into_iter().for_each(|pos| {
                                    player_positions.insert(pos);
                                });
                            }
                        }
                        player_position_types.insert(&player.position_type);
                    }
                }
            }
            data::Transaction::Trade { players, .. } => {
                // println!("trade: {:#?}", transaction);
                if let Some(players) = players {
                    for player in players {
                        match &player.display_position {
                            data::PlayerPositions::One(position) => {
                                player_positions.insert(position);
                            }
                            data::PlayerPositions::Many(positions) => {
                                positions.into_iter().for_each(|pos| {
                                    player_positions.insert(pos);
                                });
                            }
                        }
                        player_position_types.insert(&player.position_type);
                    }
                }
            }
            data::Transaction::Other {
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
