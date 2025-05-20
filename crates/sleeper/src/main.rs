mod client;
mod data;

use client::Client;
use data::Sport;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sport = Sport::NBA;
    let client = Client::new();
    // check_trending_players(&client, &sport).await;

    println!("Sport: {}", sport);

    let user = client.get_user(String::from("sectorbob")).await.unwrap();
    println!("User: {:#?}", user);

    Ok(())
}

async fn check_trending_players(client: &Client, sport: &Sport) {
    let players = client.fetch_all_players(&sport).await.unwrap();
    // println!("Players: {:#?}", players);
    println!("Player Count: {}", players.iter().count());

    let trending_players = client
        .get_trending_players(&sport, String::from("add"), None, Some(100))
        .await
        .unwrap();
    println!("Trending Count: {}", trending_players.iter().count());

    for trending_player in trending_players {
        if let Some(player) = players.get(trending_player.player_id.as_str()) {
            println!(
                "{} {} added {} times",
                player
                    .first_name
                    .as_ref()
                    .map_or(String::from("<null>"), |name| name.to_string()),
                player
                    .last_name
                    .as_ref()
                    .map_or(String::from("<null>"), |name| name.to_string()),
                trending_player.count
            );
        }
    }
}
