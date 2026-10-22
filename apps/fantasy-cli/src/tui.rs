use crate::{config, constants::SEASONS_TO_SCAN, data_fetch, events, pages};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use fantasy_sports_core::{self as core};
use std::{
    io::{self, Stdout},
    sync::Arc,
    time::Duration,
};
use tokio::time::sleep;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    style::{Color, Style},
    CompletedFrame,
    widgets::{Block, Borders},
};

const CHAN_BUF_LEN: usize = 100;

pub async fn exec(
    fantasy_league_accessor: core::LeagueAccessor,
    cfg: config::ConfigModel,
) -> Result<(), Box<dyn std::error::Error>> {
    // Set up terminal in raw mode
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let handle = tokio::spawn(async move {
        let _ = do_it(stdout, &Arc::new(fantasy_league_accessor), cfg).await;
    });

    match handle.await {
        Ok(_) => {}
        Err(err) if err.is_panic() => {
            // Handle panic
            log::error!("Caught a panic in the future: {err:?}");
            eprintln!("Caught a panic in the future: {}", err);
        }
        Err(err) => {
            // Handle cancellation or other errors
            log::error!("Future was cancelled or errored: {}", err);
            eprintln!("Future was cancelled or errored: {}", err);
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}

async fn do_it(
    stdout: Stdout,
    fantasy_league_accessor: &Arc<core::LeagueAccessor>,
    cfg: config::ConfigModel,
) -> Result<(), Box<dyn std::error::Error>> {
    let (events_tx, mut events_rx) = tokio::sync::mpsc::channel::<events::Event>(CHAN_BUF_LEN);
    let (actions_tx, mut actions_rx) =
        tokio::sync::mpsc::channel::<events::BackendAction>(CHAN_BUF_LEN);

    // let mut errors: Vec<String> = vec![];

    let league_accessor_clone = Arc::clone(&fantasy_league_accessor);
    let sleeper_user_id = cfg.sleeper_user_id.clone();
    let backend_action_handler = tokio::spawn(async move {
        loop {
            if let Some(action) = actions_rx.recv().await {
                match action {
                    events::BackendAction::Quit => {
                        log::warn!(
                            "Received Quit action, terminating event loop. <not yet implemented>"
                        );
                    }
                    events::BackendAction::RefreshLeague(external_id) => {
                        let force_update = true;
                        log::info!("Refreshing league with external_id: {}", external_id);

                        match league_accessor_clone
                            .fetch_league_context(&external_id, force_update)
                            .await
                        {
                            Ok(league) => {
                                if events_tx
                                    .send(events::Event::LeagueRefreshComplete(league))
                                    .await
                                    .is_err()
                                {
                                    log::error!("Failed to send league through channel.");
                                }
                            }
                            Err(err) => {
                                let error_msg =
                                    format!("Error refreshing league {external_id}: {err:?}");
                                log::error!("{error_msg}");
                                if events_tx
                                    .send(events::Event::Error(error_msg))
                                    .await
                                    .is_err()
                                {
                                    log::error!("Failed to send error through channel.");
                                }
                            }
                        }
                    }
                    events::BackendAction::FetchAllLeagues => {
                        log::info!("Fetching leagues...");

                        match data_fetch::get_all_leagues(
                            SEASONS_TO_SCAN
                                .into_iter()
                                .map(String::from)
                                .collect::<Vec<String>>(),
                            &league_accessor_clone,
                            sleeper_user_id.as_deref(),
                        )
                        .await
                        {
                            Ok(leagues) => {
                                for league in leagues {
                                    if events_tx
                                        .send(events::Event::LeagueRefreshComplete(league))
                                        .await
                                        .is_err()
                                    {
                                        log::error!("Failed to send leagues through channel.");
                                    }
                                }
                            }
                            Err(err) => {
                                let error_msg = format!("Error fetching leagues: {:?}", err);
                                log::error!("{error_msg}");
                                eprintln!("{error_msg}");
                                if events_tx
                                    .send(events::Event::Error(error_msg))
                                    .await
                                    .is_err()
                                {
                                    log::error!("Failed to send error through channel.");
                                }
                            }
                        };
                    }
                }
            } else {
                break; // Exit loop if channel is closed
            }
        }
    });

    // load initial data
    actions_tx
        .send(events::BackendAction::FetchAllLeagues)
        .await?;

    // Main loop
    render_loop(stdout, &mut events_rx, actions_tx, cfg).await?;
    backend_action_handler.await?;

    Ok(())
}

async fn render_loop(
    stdout: Stdout,
    rx: &mut tokio::sync::mpsc::Receiver<events::Event>,
    events_tx: tokio::sync::mpsc::Sender<events::BackendAction>,
    cfg: config::ConfigModel,
) -> Result<(), Box<dyn std::error::Error>> {
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut page: NaviItem = match cfg.landing_page.clone() {
        config::DefaultLandingPage::WelcomePage => NaviItem::WelcomePage,
        config::DefaultLandingPage::LeaguesPage => NaviItem::LeaguesPage,
        config::DefaultLandingPage::LeagueSummary(external_id) => {
            NaviItem::LeagueSummary(external_id.to_string())
        }
    };
    let mut leagues_page = pages::LeaguesPage::new();
    let mut league_history_owner_records_page = pages::LeagueHistoryOwnerRecordsPage::new();
    let league_matchups_page = pages::LeagueMatchupsPage {};
    // let mut league_transactions_page = TransactionsPage::new();
    let mut league_transactions_page;
    let mut league_draft_board_page;
    let mut navi_stack: Vec<NaviItem> = vec![];

    loop {
        // Poll channel for new value (non-blocking)
        while let Ok(Some(event)) = rx
            .try_recv()
            .map(Some)
            .or::<Option<events::Event>>(Ok(None))
        {
            match event {
                // events::Event::Input(_) => todo!(),
                events::Event::Tick => todo!(),
                events::Event::LeagueRefreshComplete(league) => {
                    leagues_page.add_league(league);
                    league_history_owner_records_page =
                        pages::LeagueHistoryOwnerRecordsPage::from_leagues(
                            &leagues_page.leagues,
                            &cfg.history,
                        );
                }
                events::Event::Error(err) => {
                    log::warn!("Error event received in render loop: {err}");
                }
            }
        }

        match &page {
            NaviItem::WelcomePage => {
                draw_blank_screen_with_q_exit(&mut terminal)
                    .map_err(|e| core::Error::new(format!("failed to render welcome page: {e}")))?;
            }
            NaviItem::LeaguesPage => {
                leagues_page.draw(&mut terminal).map_err(|e| {
                    core::Error::new(format!("failed to render leagues list page: {e}"))
                })?;
            }
            NaviItem::LeagueSummary(league_id) => {
                if let Some(selected) = leagues_page.selected() {
                    if &selected.id().to_string() != league_id {
                        leagues_page.select_by_league_id(league_id.clone());
                    }
                } else {
                    leagues_page.select_by_league_id(league_id.clone());
                }
                pages::LeagueSummaryPage::new(leagues_page.selected())
                    .draw(&mut terminal)
                    .map_err(|e| {
                        core::Error::new(format!("failed to render league summary page: {e}"))
                    })?;
            }
            NaviItem::LeagueMatchups(league_id) => {
                if let Some(selected) = leagues_page.selected() {
                    if &selected.id().to_string() != league_id {
                        leagues_page.select_by_league_id(league_id.clone());
                    }
                } else {
                    leagues_page.select_by_league_id(league_id.clone());
                }
                if let Some(selected_league) = leagues_page.selected() {
                    terminal
                        .draw(|f| {
                            let size = f.area();
                            league_matchups_page.draw(f, &size, selected_league);
                        })
                        .map_err(|e| {
                            core::Error::new(format!("failed to render league matchups page: {e}"))
                        })?;
                } else {
                    page = NaviItem::LeaguesPage;
                    // navi_stack.push(NaviItem::LeaguesPage);
                }
            }
            NaviItem::TransactionsPage(league_id) => {
                if let Some(selected) = leagues_page.selected() {
                    if &selected.id().to_string() != league_id {
                        leagues_page.select_by_league_id(league_id.clone());
                    }
                } else {
                    leagues_page.select_by_league_id(league_id.clone());
                }
                if let Some(selected_league) = leagues_page.selected() {
                    league_transactions_page =
                        pages::TransactionsPage::from_league(selected_league);
                    terminal
                        .draw(|f| {
                            let size = f.area();
                            league_transactions_page.draw(f, &size);
                        })
                        .map_err(|e| {
                            core::Error::new(format!(
                                "failed to render league transactions page: {e}"
                            ))
                        })?;
                } else {
                    page = NaviItem::LeaguesPage;
                    // navi_stack.push(NaviItem::LeaguesPage);
                }
            }
            NaviItem::LeagueHistoryOwnerRecords => {
                league_history_owner_records_page
                    .draw(&mut terminal)
                    .map_err(|e| {
                        core::Error::new(format!(
                            "failed to render league history owners records page: {e}"
                        ))
                    })?;
            }
            NaviItem::LeagueDraftBoardPage(league_id) => {
                if let Some(selected) = leagues_page.selected() {
                    if &selected.id().to_string() != league_id {
                        leagues_page.select_by_league_id(league_id.clone());
                    }
                } else {
                    leagues_page.select_by_league_id(league_id.clone());
                }
                if let Some(selected_league) = leagues_page.selected() {
                    league_draft_board_page = pages::DraftBoardPage::from(selected_league);
                    terminal
                        .draw(|f| {
                            let size = f.area();
                            league_draft_board_page.draw(f, &size);
                        })
                        .map_err(|e| {
                            core::Error::new(format!(
                                "failed to render league draftboard page: {e}"
                            ))
                        })?;
                } else {
                    page = NaviItem::LeaguesPage;
                    // navi_stack.push(NaviItem::LeaguesPage);
                }
            }
        }

        // Exit on 'q'
        if event::poll(std::time::Duration::from_millis(99))? {
            if let Event::Key(key) = event::read()? {
                let handled = match page.clone() {
                    NaviItem::WelcomePage => false,
                    NaviItem::LeaguesPage => match leagues_page.handle_key(&key.code) {
                        pages::KeyHandleResult::NOOP => true,
                        pages::KeyHandleResult::GoToLeagueSummary => {
                            // info!("Selected league: {:?}", leagues_page.selected().);
                            if let Some(selected_league) = leagues_page.selected() {
                                // Here you can handle the selected league, e.g., navigate to a summary page
                                // For example, you might want to set the league in a summary page struct
                                // league_summary_page.set_league(selected_league.clone());
                                log::info!("Navigating to summary for league: {}", selected_league);

                                // Here you can handle the selected league, e.g., navigate to a summary page
                                navi_stack.push(page);
                                page = NaviItem::LeagueSummary(selected_league.id().to_string());
                            } else {
                                log::error!("No league selected.");
                            }
                            // Example: go to a summary page
                            true
                        }

                        pages::KeyHandleResult::GoToLeagueHistoryOwnerRecords => {
                            navi_stack.push(page);
                            page = NaviItem::LeagueHistoryOwnerRecords;
                            true
                        }
                        pages::KeyHandleResult::Unhandled => false,
                    },
                    NaviItem::LeagueSummary(league_id) => {
                        if key.code == KeyCode::Char('m') {
                            // go to matchups
                            navi_stack.push(page);
                            page = NaviItem::LeagueMatchups(league_id.to_string());
                            true
                        } else if key.code == KeyCode::Char('t') {
                            // go to transactions
                            navi_stack.push(page);
                            page = NaviItem::TransactionsPage(league_id.to_string());
                            true
                        } else if key.code == KeyCode::Char('d') {
                            // go to draftboard
                            navi_stack.push(page);
                            page = NaviItem::LeagueDraftBoardPage(league_id.to_string());
                            true
                        } else if key.code == KeyCode::Char('r') {
                            // refresh league

                            // TODO: check to see if a league refresh is already in progress

                            if let Err(err) = events_tx
                                .send(events::BackendAction::RefreshLeague(core::ExternalId {
                                    platform: fantasy_sports_core::Platform::Sleeper,
                                    id: league_id.to_string(),
                                }))
                                .await
                            {
                                log::error!("Failed to send refresh league action: {err}");
                            }
                            // TODO: make sure to mark league as refresh in progress
                            true
                        } else {
                            false
                        }
                    }
                    _ => false,
                };

                if !handled {
                    match key.code {
                        KeyCode::Char('q') => break, // global quit
                        KeyCode::Char('l') => {
                            // go to leagues list page
                            navi_stack.push(page);
                            page = NaviItem::LeaguesPage;
                        }

                        KeyCode::Char('w') => {
                            // go to welcome page
                            navi_stack.push(page);
                            page = NaviItem::WelcomePage;
                        }
                        KeyCode::Char('b') => match navi_stack.pop() {
                            // go back
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
        let size = f.area();
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
    LeagueHistoryOwnerRecords,
    TransactionsPage(String),
    LeagueDraftBoardPage(String),
}
