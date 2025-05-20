mod client;
mod data;

use std::ops::Deref;

use client::Client;
use data::{Sport, User};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sport = Sport::NFL;
    let client = Client::new();
    check_trending_players(&client, &sport).await;

    println!("Sport: {}", sport);

    let user = client.get_user(String::from("sectorbob")).await.unwrap();
    println!("User: {:#?}", user);

    check_all_leagues_in(&client, "2019", &user, &sport).await;
    check_all_leagues_in(&client, "2024", &user, &sport).await;

    check_all_ecr_leagues(&client).await;

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

async fn check_all_ecr_leagues(client: &Client) {
    let ecr_league_ids = vec![
        "4581480575134427121", // 2024
        "383786432088067320",  // 2023
        "323299806782982483",  // 2022
        "652490366221046118",  // 2021
        "121226934266994911",  // 2020
        "164974877262287888",  // 2019
    ];

    for league_id in ecr_league_ids {
        let league = client.get_league(&String::from(league_id)).await.unwrap();
        println!(
            "League ({}): {} (id={} prev={}) ",
            &league.season, &league.name, &league.league_id, &league.previous_league_id.unwrap_or(String::from("null")),
        );

        let users = client.get_users_for_league(&String::from(league_id)).await.unwrap();
        println!("Users:");
        for user in users {
            println!(" - {}", user.display_name,)
        }
        // println!("{:#?}", &league);
    }
}

async fn check_all_leagues_in(client: &Client, season: &str, user: &User, sport: &Sport) {
    let leagues = client
        .get_leagues_for_user(&user.user_id, sport, &String::from(season))
        .await
        .unwrap();
    for league in leagues {
        println!("{:#?}", league);
        // println!("League ({}): {}", league.season, league.name);
        
    }
}