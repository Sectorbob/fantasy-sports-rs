use crate::{
    constants::SEASONS_TO_SCAN, data::League, data_fetch,
    league_matchups_all::LeagueMatchupsTilesPage, league_summary_page, leagues_page,
};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use league_summary_page::LeagueSummaryPage;
use leagues_page::LeaguesPage;
use log::*;
use std::{
    io::{self, Stdout},
    time::Duration,
};
use tokio::time::sleep;
use tui::{
    Terminal,
    backend::CrosstermBackend,
    style::{Color, Style},
    terminal::CompletedFrame,
    widgets::{Block, Borders},
};

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
    let (tx, mut rx) = tokio::sync::mpsc::channel::<Vec<League>>(100);

    // let mut errors: Vec<String> = vec![];

    let _handle = tokio::spawn(async move {
        info!("Fetching leagues...");

        match data_fetch::get_all_leagues(
            SEASONS_TO_SCAN
                .into_iter()
                .map(String::from)
                .collect::<Vec<String>>(),
        )
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
    let mut page = NaviItem::WelcomePage;
    let mut leagues_page = LeaguesPage::new();
    let league_matchups_page = LeagueMatchupsTilesPage {};
    let mut navi_stack: Vec<NaviItem> = vec![];

    loop {
        // Poll channel for new value (non-blocking)
        while let Ok(Some(new_leagues_state)) =
            rx.try_recv().map(Some).or::<Option<Vec<League>>>(Ok(None))
        {
            for league in new_leagues_state {
                leagues_page.add_league(league);
            }
        }

        match &page {
            NaviItem::WelcomePage => {
                draw_blank_screen_with_q_exit(&mut terminal).unwrap();
            }
            NaviItem::LeaguesPage => {
                leagues_page.draw(&mut terminal).unwrap();
            }
            NaviItem::LeagueSummary(league_id) => {
                if let Some(selected) = leagues_page.selected() {
                    if &selected.id() != league_id {
                        leagues_page.select_by_league_id(league_id.clone());
                    }
                } else {
                    leagues_page.select_by_league_id(league_id.clone());
                }
                LeagueSummaryPage::new(leagues_page.selected())
                    .draw(&mut terminal)
                    .unwrap();
            }
            NaviItem::LeagueMatchups(league_id) => {
                if let Some(selected) = leagues_page.selected() {
                    if &selected.id() != league_id {
                        leagues_page.select_by_league_id(league_id.clone());
                    }
                } else {
                    leagues_page.select_by_league_id(league_id.clone());
                }
                if let Some(selected_league) = leagues_page.selected() {
                    terminal
                        .draw(|f| {
                            let size = f.size();
                            league_matchups_page.draw(f, &size, selected_league);
                        })
                        .unwrap();
                } else {
                    page = NaviItem::LeaguesPage;
                    // navi_stack.push(NaviItem::LeaguesPage);
                }
            } // _ => {}
        }

        // Exit on 'q'
        if event::poll(std::time::Duration::from_millis(99))? {
            if let Event::Key(key) = event::read()? {
                let handled = match page.clone() {
                    NaviItem::WelcomePage => false,
                    NaviItem::LeaguesPage => match leagues_page.handle_key(&key.code) {
                        leagues_page::KeyHandleResult::NOOP => true,
                        leagues_page::KeyHandleResult::GoToLeagueSummary => {
                            // info!("Selected league: {:?}", leagues_page.selected().);
                            if let Some(selected_league) = leagues_page.selected() {
                                // Here you can handle the selected league, e.g., navigate to a summary page
                                // For example, you might want to set the league in a summary page struct
                                // league_summary_page.set_league(selected_league.clone());
                                info!("Navigating to summary for league: {}", selected_league);

                                // Here you can handle the selected league, e.g., navigate to a summary page
                                navi_stack.push(page);
                                page = NaviItem::LeagueSummary(selected_league.id());
                            } else {
                                error!("No league selected.");
                            }
                            // Example: go to a summary page
                            true
                        }
                        leagues_page::KeyHandleResult::Unhandled => false,
                    },
                    NaviItem::LeagueSummary(league_id) => {
                        if key.code == KeyCode::Char('m') {
                            navi_stack.push(page);
                            page = NaviItem::LeagueMatchups(league_id.to_string());
                            true
                        } else {
                            false
                        }
                    }
                    _ => false,
                };

                if !handled {
                    match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Char('l') => {
                            navi_stack.push(page);
                            page = NaviItem::LeaguesPage;
                        }
                        KeyCode::Char('w') => {
                            navi_stack.push(page);
                            page = NaviItem::WelcomePage;
                        }
                        KeyCode::Char('b') => match navi_stack.pop() {
                            Some(previous) => page = previous,
                            None => {}
                        },
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

#[derive(Debug, Clone)]
enum NaviItem {
    WelcomePage,
    LeaguesPage,
    LeagueSummary(String),
    LeagueMatchups(String),
}
