mod client;
mod data;

use client::Client;
use data::{GameCode, Token};
use serde_json;
use std::{env, error::Error, io, io::Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let yahoo_client_id = env::var("YAHOO_CLIENT_ID").unwrap();
    let yahoo_redirect_uri = env::var("YAHOO_REDIRECT_URI").unwrap();
    let yahoo_token_file = env::var("YAHOO_TOKEN_FILE").unwrap();
    let mut client = client::Client::new(yahoo_client_id, yahoo_redirect_uri);

    // Try to load token from token file
    match load_token_file(yahoo_token_file) {
        Ok(token) => client.token = Some(token),
        // If fails, perform auth flow
        Err(_) => perform_oauth_flow(&mut client).await,
    }

    let users = client.get_users(true).await.unwrap();
    println!("Users: {:#?}", users);

    let games = client
        .get_games(vec![GameCode::NFL], vec!["2019", "2020"])
        .await
        .unwrap();
    println!("Games: {:#?}", games);

    Ok(())
}

fn load_token_file(file: String) -> Result<Token, Box<dyn Error>> {
    let file = std::fs::File::open(file)?;
    let reader = std::io::BufReader::new(file);

    // Read the JSON contents of the file as an instance of `Token`.
    let token = serde_json::from_reader(reader)?;
    Ok(token)
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
