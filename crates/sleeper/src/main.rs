mod client;
mod data;

use client::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let client = Client::new();

    let players = client.fetch_all_players(data::Sport::NFL).await.unwrap();
    // println!("Players: {:#?}", players);
    println!("Count: {}", players.iter().count());

    Ok(())
}
