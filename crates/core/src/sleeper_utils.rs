use futures::FutureExt;
use sleeper_fantasy_rs::{
    Client, League, Matchup, Player, Roster, Sport, Transaction, User, custom::FantasyMatchup,
};
use std::{collections::HashMap, error::Error, ops::Range};

const START_WEEK: usize = 1;

/// Developer harness: prints a tour of what the Sleeper client can fetch for `user` and the given leagues.
pub async fn exec(user: &str, league_ids: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let force_update = false;
    let sport = Sport::NFL;
    let client = Client::new(); //.with_cache(cache_dir);
    let players = client.fetch_all_players(&sport).await?;
    check_trending_players(&players, &client, &sport).await;
    log::debug!("Sport: {sport}");
    let user = client.get_user(user).await?;
    log::debug!("User: {user:#?}");
    let Some(first_league_id) = league_ids.first() else {
        return Err("pass at least one sleeper league id".into());
    };
    check_all_ecr_leagues(&client, league_ids).await;
    let league = client.get_league(first_league_id).await?;
    check_all_leagues_in(&client, &league.season, &user, &sport).await;
    let owners_future = client.get_users_for_league(&league.league_id);
    let rosters_future = client.get_rosters_in_league(&league.league_id);

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
    match get_league_matchups(&league, &client, &force_update).await {
        Ok(fantasy_matchups_for_week) => {
            let num_of_weeks = 17; //league.settings.weeks;
            println!("League matchups checked successfully.");
            for week in 1..=num_of_weeks {
                println!("Week: {}", week);
                if let Some(matchups) = fantasy_matchups_for_week.get(&week) {
                    for matchup in FantasyMatchup::from_stuff(week, matchups, &rosters, &owners)
                        .expect("failed to build out fantasy fantasy matchup for sleeper")
                    {
                        matchup.printy(50, &players);
                        println!();
                    }
                } else {
                }
            }
        }
        Err(e) => eprintln!("Error checking league matchups: {}", e),
    }
    check_transactions(&client, first_league_id, &sport).await;

    Ok(())
}

async fn check_transactions(client: &Client, league_id: &String, sport: &Sport) {
    let players = client.fetch_all_players(sport).await.unwrap();
    let mut owners = client.get_users_for_league(league_id).await.unwrap();
    owners.sort_by(|a, b| a.user_id.cmp(&b.user_id));
    let mut rosters = client.get_rosters_in_league(league_id).await.unwrap();
    rosters.sort_by(|a, b| a.roster_id.cmp(&b.roster_id));

    let transactions = client.get_transactions(league_id, 1).await.unwrap();
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

async fn check_all_ecr_leagues(client: &Client, ecr_league_ids: &[String]) {
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

fn determine_weeks_to_scan(league: &League) -> Range<usize> {
    let first_leg = START_WEEK;
    // leg, last_leg, last_scored_leg
    let last_leg: Option<usize> = league.settings.last_leg.map(|n| n as usize);
    let leg: Option<usize> = league.settings.leg.map(|n| n as usize);
    let last_scored_leg: Option<usize> = league.settings.last_scored_leg.map(|n| n as usize);
    log::debug!(
        "{} Sleeper League ({}): last_leg: {:?}",
        league.season,
        league.league_id,
        last_leg
    );
    log::debug!(
        "{} Sleeper League ({}): leg: {:?}",
        league.season,
        league.league_id,
        leg
    );
    log::debug!(
        "{} Sleeper League ({}): last_scored_leg: {:?}",
        league.season,
        league.league_id,
        last_scored_leg
    );
    let total_num_of_weeks =
        last_leg.unwrap_or(leg.unwrap_or(last_scored_leg.unwrap_or(first_leg))) - first_leg + 1;

    let weeks_to_scan = first_leg..(total_num_of_weeks + first_leg);
    log::debug!(
        "{} Sleeper League ({}): weeks_to_scan: {:?}",
        league.season,
        league.league_id,
        weeks_to_scan
    );
    weeks_to_scan
}

pub async fn get_league_matchups(
    league: &League,
    client: &Client,
    force_update: &bool,
) -> Result<HashMap<usize, Vec<Matchup>>, Box<dyn std::error::Error + Send + Sync>> {
    let weeks_to_scan = determine_weeks_to_scan(league);

    let matchups_for_week_futures = weeks_to_scan
        .into_iter()
        .map(|week| {
            let result = client
                .get_matchups_request(&league.league_id, week)
                .with_force_update(force_update)
                .send()
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

    let mut matchups_for_week: HashMap<usize, Vec<Matchup>> = HashMap::new();
    for thing in futures::future::join_all(matchups_for_week_futures)
        .await
        .into_iter()
    {
        match thing {
            Ok((week, matchups)) => {
                matchups_for_week.insert(week, matchups);
                // let m = match FantasyMatchup::from_stuff(week, matchups, &rosters, &owners) {
                //     Ok(m) => m,
                //     Err(err) => {
                //         return Err(Box::new(std::io::Error::new(
                //             std::io::ErrorKind::Other,
                //             format!("Error creating FantasyMatchup: {:?}", err),
                //         )));
                //     }
                // };

                // matchups_for_week.insert(week, m);
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
    league: &League,
    force_update: &bool,
) -> Result<Vec<Transaction>, cached_client_rs::Error> {
    let weeks_to_scan = determine_weeks_to_scan(league);

    let transactions_for_week_futures = weeks_to_scan
        .into_iter()
        .map(move |week| {
            let result = client
                .get_transactions_request(&league.league_id, week)
                .with_force_update(&force_update)
                .send()
                .map(move |res| {
                    let result: Result<(usize, Vec<Transaction>), cached_client_rs::Error> =
                        match res {
                            Ok(raw_txns) => Ok((week, raw_txns)),
                            Err(e) => {
                                log::error!("Error fetching transactions for week {week}: {e:?}");
                                Ok((week, Vec::new())) // Return empty vector on error
                            }
                        };
                    result
                });
            result
        })
        .collect::<Vec<_>>();

    // let mut transactions_for_week: HashMap<usize, Vec<Transaction>> = HashMap::new();
    let mut transactions: Vec<Transaction> = vec![];
    for thing in futures::future::join_all(transactions_for_week_futures)
        .await
        .into_iter()
    {
        match thing {
            Ok((_week, txns)) => {
                // transactions_for_week.insert(week, txns);
                transactions.extend(txns);
            }
            Err(err) => {
                return Err(err);
            }
        }
    }
    Ok(transactions)
}
