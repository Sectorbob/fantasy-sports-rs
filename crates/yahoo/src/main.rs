mod client;
mod data;

use chrono::DateTime;
use client::Client;
use data::{GameCode, GameKey, LeagueKey, Player, PlayerName, Token, Transaction, Transactions};
use serde_json;
use std::{
    collections::HashSet,
    env,
    error::Error,
    fmt,
    io::{self, BufWriter, Write},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    let new_token = client.refresh_token().await.unwrap();
    save_token_file(&yahoo_token_file, new_token);

    // let users = client.get_users(true).await.unwrap();
    // println!("Users: {:#?}", users);

    // let games = client
    //     .get_games(
    //         vec![GameCode::NFL],
    //         vec![
    //             "2009", "2010", "2011", "2012", "2013", "2014", "2015", "2016", "2017", "2018",
    //             "2019", "2020",
    //         ],
    //     )
    //     .await
    //     .unwrap();

    // println!("Games: {:#?}", games);

    // client.get("https://fantasysports.yahooapis.com/fantasy/v2/users;use_login=1;/leagues").await; // can't do this

    // let teams = client.get_teams_for_user(GameKey::Id(371)).await.unwrap();
    // println!("Teams: {:#?}", teams);

    // let league = client
    //     .get_league(LeagueKey::new("371", 20028))
    //     .await
    //     .unwrap();
    // println!("League: {:#?}", league);

    // let league_settings = client
    //     .get_league_settings(LeagueKey::new("371", 20028))
    //     .await
    //     .unwrap();
    // println!("League Settings: {:#?}", league_settings);

    let public_league_key = LeagueKey::new(GameKey::Id(223).to_string().as_str(), 431);
    // let ecr_2017 = LeagueKey::new("371", 20028);

    // let league_standings = client
    //     .get_league_standings(&public_league_key)
    //     .await
    //     .unwrap();
    // println!("League Standings: {:#?}", league_standings);

    let transactions = client
        .get_league_transactions(&public_league_key)
        .await
        .unwrap();
    fancy_navigate_transactions(&transactions);

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
