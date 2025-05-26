use crate::{leagues_page, yahoo_utils};
use futures::{TryStreamExt, stream::FuturesUnordered};
use leagues_page::{League, LeaguesPage};
use log::*;

use core::panic;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use sleeper_fantasy_rs::{self as sleeper, Sport};
use std::{
    io::{self, Stdout},
    time::Duration,
};
use tokio::time::sleep;
use tui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    terminal::CompletedFrame,
    widgets::{Block, Borders},
};
use yahoo_fantasy_rs::{GameCode, GameKey, Team};

pub async fn exec() -> Result<(), Box<dyn std::error::Error>> {
    // Set up terminal in raw mode
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let handle = tokio::spawn(async {
        let _ = do_it(stdout).await;
    });

    match handle.await {
        Ok(_) => {}
        Err(err) if err.is_panic() => {
            // Handle panic
            eprintln!("Caught a panic in the future: {}", err);
        }
        Err(err) => {
            // Handle cancellation or other errors
            eprintln!("Future was cancelled or errored: {}", err);
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}

async fn do_it(stdout: Stdout) -> Result<(), Box<dyn std::error::Error>> {
    let (mut tx, mut rx) = tokio::sync::mpsc::channel::<Vec<League>>(100);

    // let mut errors: Vec<String> = vec![];

    let _handle = tokio::spawn(async move {
        info!("Fetching leagues...");
        // let tx_clone = tx.clone();

        match get_all_leagues(vec![
            "2015".to_string(),
            "2016".to_string(),
            "2017".to_string(),
            "2018".to_string(),
            "2019".to_string(),
            "2020".to_string(),
            "2021".to_string(),
            "2022".to_string(),
            "2023".to_string(),
            "2024".to_string(),
        ])
        .await
        {
            Ok(leagues) => {
                if tx.send(leagues).await.is_err() {
                    eprintln!("Failed to send leagues through channel.");
                }
            }
            Err(err) => {
                error!("Error fetching leagues: {:?}", err);
                eprintln!("Error fetching leagues: {:?}", err);
            }
        };
    });

    // Main loop
    render_loop(stdout, &mut rx).await?;
    _handle.await?;

    Ok(())
}

async fn render_loop(
    stdout: Stdout,
    rx: &mut tokio::sync::mpsc::Receiver<Vec<League>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut page = 0;
    let mut leagues_page = LeaguesPage::new();

    let mut i = 0;
    loop {
        i += 1;
        info!("Render loop iteration: {}", i);

        // Poll channel for new value (non-blocking)
        while let Ok(Some(new_leagues_state)) =
            rx.try_recv().map(Some).or::<Option<Vec<League>>>(Ok(None))
        {
            for league in new_leagues_state {
                leagues_page.add_league(league);
            }
        }

        match page {
            0 => {
                draw_blank_screen_with_q_exit(&mut terminal).unwrap();
            }
            1 => {
                leagues_page.draw(&mut terminal).unwrap();
            }
            _ => {}
        }

        // Exit on 'q'
        if event::poll(std::time::Duration::from_millis(99))? {
            if let Event::Key(key) = event::read()? {
                let handled = match page {
                    0 => false,
                    1 => leagues_page.handle_key(&key.code),
                    _ => false,
                };

                if !handled {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('l') => page = 1,
                        KeyCode::Char('w') => page = 0,
                        _ => {}
                    }
                }
            }
        }
        sleep(Duration::from_millis(1)).await;
    }

    Ok(())
}

fn draw_blank_screen_with_q_exit(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> io::Result<CompletedFrame> {
    terminal.draw(|f| {
        let size = f.size();
        let block = Block::default()
            .title("My TUI - Press 'q' to quit, 'l' to list")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        f.render_widget(block, size);
    })
}

async fn get_all_leagues(
    seasons: Vec<String>,
) -> Result<Vec<League>, Box<dyn std::error::Error + Send + Sync>> {
    let sleeper_user_id = "727598467304811923".to_string();
    let sleeper_sport = Sport::NFL;
    let yahoo_game_code = GameCode::NFL;
    let yahoo_client = match yahoo_utils::build_client().await {
        Ok(client) => client,
        Err(err) => return Err(Box::new(err)),
    };
    let sleeper_client = sleeper_fantasy_rs::Client::new(None);

    let mut leagues: Vec<League> = vec![];
    let sleeper_leagues_future =
        get_sleeper_leagues(&sleeper_client, &seasons, &sleeper_user_id, &sleeper_sport);
    let yahoo_leagues_future = get_yahoo_leagues(&yahoo_client, &seasons, yahoo_game_code);
    let (sleeper_leagues, yahoo_leagues) =
        tokio::join!(sleeper_leagues_future, yahoo_leagues_future);
    match sleeper_leagues {
        Ok(sleeper_leagues) => {
            leagues.extend(sleeper_leagues);
        }
        Err(err) => {
            error!("Error fetching Sleeper leagues: {:?}", err);
            return Err(err);
        }
    }
    match yahoo_leagues {
        Ok(yahoo_leagues) => {
            leagues.extend(yahoo_leagues);
        }
        Err(err) => {
            error!("Error fetching Yahoo leagues: {:?}", err);
            return Err(err);
        }
    }
    info!("Total leagues fetched: {}", leagues.len());
    // Sort leagues by season in descending order
    leagues.sort_by(|a, b| b.season().cmp(&a.season()));

    Ok(leagues)
}

async fn get_sleeper_leagues(
    sleeper_client: &sleeper_fantasy_rs::Client,
    seasons: &[String],
    sleeper_user_id: &String,
    sleeper_sport: &Sport,
) -> Result<Vec<League>, Box<dyn std::error::Error + Send + Sync>> {
    let mut sleeper_leagues_for_season_tasks: FuturesUnordered<_> = seasons
        .iter()
        .map(|season| sleeper_client.get_leagues_for_user(sleeper_user_id, sleeper_sport, season))
        .collect();

    // let mut sleeper_leagues_tasks = FuturesUnordered::new();
    // // Now you can poll tasks as they complete:
    let mut sleeper_leagues: Vec<sleeper::League> = vec![];
    loop {
        match sleeper_leagues_for_season_tasks.try_next().await {
            Ok(Some(leagues)) => {
                for league in leagues {
                    sleeper_leagues.push(league);
                }
            }
            Ok(None) => {
                break;
            } // exit loop as all tasks are done
            Err(e) => {
                error!("Error fetching Sleeper leagues: {:?}", e);
                panic!("Error fetching Sleeper leagues: {:?}", e); //TDOO handle this gracefully
                // return Err(Box::new(e));
            }
        }
        // handle result
    }

    let mut leagues = vec![];
    for sleeper_league in sleeper_leagues {
        // let sleeper_user = sleeper_client
        //     .get_users_for_league(&sleeper_league.league_id)
        //     .await
        //     .unwrap()
        //     .iter()
        //     .find(|u| u.user_id == sleeper_user_id)
        //     .unwrap();
        // sleeper_user.league_details.unwrap();
        let rosters = sleeper_client
            .get_rosters_for_league(&sleeper_league.league_id)
            .await
            .unwrap();
        leagues.push(League::Sleeper {
            league: sleeper_league,
            rosters,
            user_id: sleeper_user_id.clone(),
        });
    }

    Ok(leagues)
}

async fn get_yahoo_leagues(
    yahoo_client: &yahoo_fantasy_rs::Client,
    seasons: &[String],
    yahoo_game_code: GameCode,
) -> Result<Vec<League>, Box<dyn std::error::Error + Send + Sync>> {
    let games = yahoo_client
        .get_games(
            vec![yahoo_game_code],
            seasons.iter().map(|s| s.as_str()).collect(),
        )
        .await
        .unwrap();

    let mut yahoo_teams: Vec<Team> = vec![];
    for game in games {
        let game_key: GameKey = GameKey::Id(game.game_id);
        let teams_in_season = yahoo_client.get_teams_for_user(game_key).await?;
        for team in teams_in_season {
            yahoo_teams.push(team);
        }
    }

    let mut leagues: Vec<League> = vec![];

    let mut yahoo_teams_tasks: FuturesUnordered<_> = FuturesUnordered::new();
    for yahoo_team in yahoo_teams {
        yahoo_teams_tasks.push(async move {
            let yahoo_league = yahoo_client
                .get_league(&yahoo_team.team_key.league_key)
                .await
                .unwrap()
                .unwrap();
            let standings = yahoo_client
                .get_league_standings(&yahoo_team.team_key.league_key)
                .await
                .unwrap();
            Ok::<League, yahoo_fantasy_rs::Error>(League::Yahoo {
                team: yahoo_team,
                league: yahoo_league,
                standings,
            })
        });
    }
    while let Some(yahoo_league) = yahoo_teams_tasks.try_next().await? {
        leagues.push(yahoo_league);
    }

    Ok(leagues)
}
