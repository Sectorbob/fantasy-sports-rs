use std::{collections::HashMap, error::Error};

use futures::FutureExt;
use log::error;
use serde::Serialize;
use sleeper_fantasy_rs::{
    Client, Matchup, Player, Roster, Sport, Transaction, User, custom::FantasyMatchup,
};
use tokio::fs; // instead of std::fs

pub async fn exec() -> Result<(), Box<dyn std::error::Error>> {
    let sport = Sport::NFL;
    let client = Client::new(None);
    let players = client.fetch_all_players(&sport).await?;
    check_trending_players(&players, &client, &sport).await;
    println!("Sport: {}", sport);
    let user = client.get_user(String::from("sectorbob")).await.unwrap();
    println!("User: {:#?}", user);
    check_all_leagues_in(&client, "2019", &user, &sport).await;
    check_all_leagues_in(&client, "2024", &user, &sport).await;
    check_all_ecr_leagues(&client).await;
    match get_league_matchups(
        &String::from("4581480575134427121"),
        None,
        &client,
        // &players,
    )
    .await
    {
        Ok(fantasy_matchups_for_week) => {
            let num_of_weeks = 17; //league.settings.weeks;
            println!("League matchups checked successfully.");
            for week in 1..=num_of_weeks {
                println!("Week: {}", week);
                if let Some(fantasy_matchups) = fantasy_matchups_for_week.get(&week) {
                    for matchup in fantasy_matchups.iter() {
                        matchup.printy(50, &players);
                        println!();
                    }
                } else {
                }
            }
        }
        Err(e) => eprintln!("Error checking league matchups: {}", e),
    }
    check_transactions(&client, &String::from("4581480575134427121"), &sport).await;

    Ok(())
}

async fn check_transactions(client: &Client, league_id: &String, sport: &Sport) {
    let players = client.fetch_all_players(sport).await.unwrap();
    let mut owners = client.get_users_for_league(league_id).await.unwrap();
    owners.sort_by(|a, b| a.user_id.cmp(&b.user_id));
    let mut rosters = client.get_rosters_for_league(league_id).await.unwrap();
    rosters.sort_by(|a, b| a.roster_id.cmp(&b.roster_id));

    let transactions = client
        .get_transactions(league_id, &String::from("1"))
        .await
        .unwrap();
    for transaction in transactions.iter() {
        let added: Vec<(&Player, &Roster)> = transaction
            .adds
            .iter()
            .map(|add| {
                (
                    players.get(add.0).unwrap(),
                    rosters
                        .iter()
                        .find(|roster| &roster.roster_id == add.1)
                        .unwrap(),
                )
            })
            .collect();
        let dropped: Vec<(&Player, &Roster)> = transaction
            .drops
            .iter()
            .map(|drop| {
                (
                    players.get(drop.0).unwrap(),
                    rosters
                        .iter()
                        .find(|roster| &roster.roster_id == drop.1)
                        .unwrap(),
                )
            })
            .collect();

        println!(
            "[{}] ({}) - {} Added: {:?} Dropped: {:?}",
            transaction.status_updated,
            transaction.status,
            transaction._type,
            added
                .iter()
                .map(|t| t.0.to_string())
                .collect::<Vec<String>>(),
            dropped
                .iter()
                .map(|t| t.0.to_string())
                .collect::<Vec<String>>()
        );
    }
}

async fn check_trending_players(players: &HashMap<String, Player>, client: &Client, sport: &Sport) {
    println!("Player Count: {}", players.iter().count());

    let trending_players = client
        .get_trending_players(&sport, sleeper_fantasy_rs::TrendType::ADD, None, Some(100))
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
            &league.season,
            &league.name,
            &league.league_id,
            &league.previous_league_id.unwrap_or(String::from("null")),
        );

        let users = client
            .get_users_for_league(&String::from(league_id))
            .await
            .unwrap();
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

pub async fn get_league_matchups(
    league_id: &String,
    _league: Option<&sleeper_fantasy_rs::League>,
    client: &Client,
) -> Result<HashMap<usize, Vec<FantasyMatchup>>, Box<dyn std::error::Error + Send + Sync>> {
    // let league_future = if let Some(_) = league {
    //     None
    // } else {
    //     Some(client.get_league(league_id))
    // };

    let owners_future = client.get_users_for_league(league_id);
    let rosters_future = client.get_rosters_for_league(league_id);

    let mut owners = match owners_future.await {
        Ok(owners) => owners,
        Err(err) => {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to fetch owners: {:?}", err),
            )));
        }
    };
    owners.sort_by(|a, b| a.user_id.cmp(&b.user_id));
    let mut rosters = match rosters_future.await {
        Ok(rosters) => rosters,
        Err(err) => {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to fetch rosters: {:?}", err),
            )));
        }
    };
    rosters.sort_by(|a, b| a.roster_id.cmp(&b.roster_id));

    let num_of_weeks: usize = 17; // TODO: league.settings.weeks;

    let matchups_for_week_futures = (1..=num_of_weeks)
        .into_iter()
        .map(|week| {
            let result = client
                .get_matchups(league_id.clone(), week.to_string())
                .map(move |res| {
                    let result: Result<(usize, Vec<Matchup>), Box<dyn Error + Send + Sync>> =
                        match res {
                            Ok(raw_matchups) => Ok((week, raw_matchups)),
                            Err(e) => {
                                eprintln!("Error fetching matchups for week {}: {:?}", week, e);
                                Ok((week, Vec::new())) // Return empty vector on error
                            }
                        };
                    result
                });
            result
        })
        .collect::<Vec<_>>();

    let mut matchups_for_week: HashMap<usize, Vec<FantasyMatchup>> = HashMap::new();
    for thing in futures::future::join_all(matchups_for_week_futures)
        .await
        .into_iter()
    {
        match thing {
            Ok((week, matchups)) => {
                let m = match FantasyMatchup::from_stuff(week, matchups, &rosters, &owners) {
                    Ok(m) => m,
                    Err(err) => {
                        return Err(Box::new(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Error creating FantasyMatchup: {:?}", err),
                        )));
                    }
                };

                matchups_for_week.insert(week, m);
            }
            Err(err) => {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Error processing matchups: {:?}", err),
                )));
            }
        }
    }
    Ok(matchups_for_week)
}

pub async fn get_league_transactions(
    client: &Client,
    league_id: String,
) -> Result<HashMap<usize, Vec<Transaction>>, sleeper_fantasy_rs::Error> {
    let num_of_weeks: usize = 17; // TODO: league.settings.weeks;

    let transactions_for_week_futures = (1..=num_of_weeks)
        .into_iter()
        .map(|week| {
            let result = client
                .get_transactions(&league_id, &week.to_string())
                .map(move |res| {
                    let result: Result<(usize, Vec<Transaction>), sleeper_fantasy_rs::Error> =
                        match res {
                            Ok(raw_txns) => Ok((week, raw_txns)),
                            Err(e) => {
                                error!("Error fetching transactions for week {}: {:?}", week, e);
                                Ok((week, Vec::new())) // Return empty vector on error
                            }
                        };
                    result
                });
            result
        })
        .collect::<Vec<_>>();

    let mut transactions_for_week: HashMap<usize, Vec<Transaction>> = HashMap::new();
    for thing in futures::future::join_all(transactions_for_week_futures)
        .await
        .into_iter()
    {
        match thing {
            Ok((week, txns)) => {
                transactions_for_week.insert(week, txns);
            }
            Err(err) => {
                return Err(err);
            }
        }
    }
    Ok(transactions_for_week)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_league_matchups_runs_without_panic() {
        let client = Client::new(None);
        // let sport = Sport::NFL;
        // let players = client.fetch_all_players(&sport).await.unwrap();
        let league_ids = vec![
            "4581480575134427121", // 2024
            "383786432088067320",  // 2023
            "323299806782982483",  // 2022
            "652490366221046118",  // 2021
            "121226934266994911",  // 2020
            "164974877262287888",  // 2019
        ]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

        // This test checks that the function runs without panicking.
        // It uses dummy values and expects the function to handle errors gracefully.

        for league_id in league_ids {
            let _matchups = get_league_matchups(&league_id, None, &client /*&players*/)
                .await
                .unwrap();
        }
    }
}

/// Saves a serializable struct as JSON to a file in a subdirectory of the user's local cache directory.
/// `subdir` is the subdirectory name, `filename` is the file name.
pub async fn save_to_cache_dir<T: Serialize>(
    subdir: &str,
    filename: &str,
    value: &T,
) -> std::io::Result<()> {
    let cache_dir = dirs::cache_dir()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "Cache dir not found"))?;
    let dir_path = cache_dir.join(subdir);
    fs::create_dir_all(&dir_path).await?;
    let file_path = dir_path.join(filename);
    let json = serde_json::to_string_pretty(value)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    let mut file = fs::File::create(file_path).await?;
    use tokio::io::AsyncWriteExt;
    file.write_all(json.as_bytes()).await?;
    Ok(())
}
