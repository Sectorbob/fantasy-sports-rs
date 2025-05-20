mod client;
mod data;

use client::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    let players = client.fetch_all_players(data::Sport::NFL).await.unwrap();
    // println!("Players: {:#?}", players);
    println!("Count: {}", players.iter().count());

    let trending_players =
        client.get_trending_players(data::Sport::NFL, String::from("add"), None, Some(100)).await.unwrap();

    for trending_player in trending_players {
        if let Some(player) = players.get(trending_player.player_id.as_str()) {
            println!("{} {} added {} times", player.first_name, player.last_name, trending_player.count);
        }
    }
    Ok(())
}
