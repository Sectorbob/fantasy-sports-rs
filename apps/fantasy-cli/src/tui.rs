use crate::{leagues_page, yahoo_utils};
use leagues_page::{League, LeaguesPage};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use sleeper_fantasy_rs::{self as sleeper, Sport};
use std::io::{self, Stdout};
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

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut page = 0;

    let mut leagues_page = LeaguesPage::new();

    let mut leagues_future = Some(get_all_leagues(vec![
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
    ]));

    // leagues_page.add_league(League::Sleeper("blerp".to_string()));
    // leagues_page.add_league(League::Sleeper("blarp".to_string()));
    // leagues_page.add_league(League::Yahoo("flarp".to_string()));

    let mut errors: Vec<String> = vec![];
    let mut updated_leagues = match leagues_future {
        Some(lf) => match lf.await {
            Ok(leagues) => Some(leagues),
            Err(err) => panic!("fffff: {}", err),
        },
        None => None,
    };

    // Main loop
    loop {
        if let Some(new_leagues_state) = updated_leagues {
            for league in new_leagues_state {
                leagues_page.add_league(league);
            }
            updated_leagues = None;
        }
        // if let Some(f) = leagues_future {
        //     tokio::spawn(async {
        //         let l = match f.await {
        //             Ok(leagues) => {
        //                 updated_leagues = Some(leagues);
        //             }
        //             Err(error) => errors.push(format!("{}", error)),
        //         };
        //         ()
        //     });
        //     leagues_future = None;
        // }

        match page {
            0 => {
                draw_blank_screen_with_q_exit(&mut terminal)?;
            }
            1 => {
                leagues_page.draw(&mut terminal)?;
            }
            _ => {}
        }

        // Exit on 'q'
        if event::poll(std::time::Duration::from_millis(100))? {
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
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
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

async fn get_all_leagues(seasons: Vec<String>) -> Result<Vec<League>, Box<dyn std::error::Error>> {
    let sleeper_user_id = "727598467304811923".to_string();
    let sleeper_sport = Sport::NFL;

    let yahoo_game_code = GameCode::NFL;

    let yahoo_client = yahoo_utils::build_client().await?;
    let sleeper_client = sleeper_fantasy_rs::Client::new(None);

    let mut sleeper_leagues: Vec<sleeper::League> = vec![];
    for season in &seasons {
        for sleeper_league in sleeper_client
            .get_leagues_for_user(&sleeper_user_id, &sleeper_sport, season)
            .await
            .unwrap()
        {
            sleeper_leagues.push(sleeper_league);
        }
    }

    let games = yahoo_client
        .get_games(
            vec![yahoo_game_code],
            seasons.iter().map(|s| s.as_str()).collect(),
        )
        .await
        .unwrap();

    let mut yahoo_teams: Vec<Team> = vec![];
    for game in games {
        if game.game_id == 390 {
            continue; // TODO due to data issue
        }
        let game_key: GameKey = GameKey::Id(game.game_id);
        let teams_in_season = yahoo_client.get_teams_for_user(game_key).await?;
        for team in teams_in_season {
            yahoo_teams.push(team);
        }
    }

    let mut leagues: Vec<League> = vec![];
    for yahoo_team in yahoo_teams {
        leagues.push(League::Yahoo(format!(
            "{} ({})",
            yahoo_team.name, yahoo_team.team_key
        )));
    }
    for sleeper_league in sleeper_leagues {
        leagues.push(League::Sleeper(format!(
            "{} ({})",
            sleeper_league.name, sleeper_league.season
        )));
    }

    Ok(leagues)
}
