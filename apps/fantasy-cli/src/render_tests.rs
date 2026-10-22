//! Page and widget rendering against a real league fetched from recorded API responses.
use crate::{
    config::{HistoryConfig, OwnerAlias},
    pages,
    test_util::*,
    widgets,
};
use fantasy_sports_core as core;

#[tokio::test]
async fn the_summary_page_shows_the_league_its_settings_and_teams() {
    let league = sleeper_league().await;
    let mut page = pages::LeagueSummaryPage::new(Some(&league));
    let mut terminal = terminal(170, 50);
    page.draw(&mut terminal).unwrap();
    let screen = text(terminal.backend().buffer());

    assert!(screen.contains("Rapid League 17"), "title");
    assert!(screen.contains("League ID:    sleeper:164974877262287888"));
    assert!(screen.contains("Platform:     Sleeper"));
    assert!(screen.contains("Season:       2019"));
    assert!(screen.contains("Divisions:    North, South, West"));
    assert!(screen.contains("Teams (12)"));
    assert!(screen.contains("Scoring Settings"));
    assert!(screen.contains("pass_td"), "scoring rules are listed");
    assert!(screen.contains("10-3-0"), "team records are listed");
    assert!(screen.contains("Draft Rounds: 17"));
}

#[tokio::test]
async fn the_summary_page_adds_the_selected_owners_team_and_record() {
    let league = sleeper_league().await;
    let owner = league
        .standings()
        .entries
        .into_iter()
        .find(|e| e.roster_id == "1")
        .unwrap();
    let league = league.with_owner_id(core::ExternalId::new(
        core::Platform::Sleeper,
        owner.owner_id.clone(),
    ));
    let mut page = pages::LeagueSummaryPage::new(Some(&league));
    let mut terminal = terminal(170, 50);
    page.draw(&mut terminal).unwrap();
    let screen = text(terminal.backend().buffer());

    assert!(
        screen.contains(&format!("Name:         {}", owner.team_name)),
        "{screen}"
    );
    assert!(screen.contains("Record:       10-3-0"));
    assert!(screen.contains("Points:       1716.10"));
}

#[test]
fn the_summary_page_without_a_league_says_so() {
    let mut page = pages::LeagueSummaryPage::new(None);
    let mut terminal = terminal(100, 10);
    page.draw(&mut terminal).unwrap();
    let buffer = terminal.backend().buffer();
    assert!(contains(buffer, "No league selected."));
    assert!(contains(buffer, "N/A"), "the title has a placeholder name");
}

#[tokio::test]
async fn the_draft_board_lays_out_the_picks_round_by_round() {
    let league = sleeper_league().await;
    let board = pages::DraftBoardPage::from(&league);

    let buffer = render(170, 40, |f| board.draw(f, &f.area()));
    let screen = text(&buffer);

    assert!(screen.contains("Rapid League 17 (2019) - Draftboard"));
    assert!(
        screen.contains("1.1") && screen.contains("1.12"),
        "round one picks"
    );
    assert!(
        screen.contains("2.12"),
        "the second round runs the other way (snake)"
    );
    assert!(screen.contains("O. Beckham"), "the first pick");
}

#[tokio::test]
async fn the_transactions_page_has_its_columns_even_with_no_transactions() {
    let league = sleeper_league().await;
    let page = pages::TransactionsPage::from_league(&league);

    let buffer = render(120, 12, |f| page.draw(f, &f.area()));

    for column in [
        "Transactions",
        "TS",
        "Week",
        "Roster",
        "Type",
        "Adds",
        "Drops",
    ] {
        assert!(contains(&buffer, column), "{column}");
    }
}

#[tokio::test]
async fn the_matchups_page_shows_played_weeks_and_marks_the_rest() {
    let league = sleeper_league().await;
    let page = pages::LeagueMatchupsPage {};

    let buffer = render(140, 24, |f| page.draw(f, &f.area(), &league));
    let screen = text(&buffer);

    assert!(screen.contains("Rapid League 17 (2019) - Matchups"));
    assert!(screen.contains("Week 1"));
    assert!(
        screen.contains("86.50") && screen.contains("113.30"),
        "week one scores"
    );
    // unplayed weeks have empty scoreboards; this used to index out of bounds
    assert!(screen.contains("No matchups"), "{screen}");
}

#[tokio::test]
async fn the_matchups_page_survives_tiny_terminals_and_leagues_without_weeks() {
    let page = pages::LeagueMatchupsPage {};
    let mut league = sleeper_league().await;

    // far too small for even one tile: used to divide by zero
    for (width, height) in [(1, 1), (10, 3), (30, 6)] {
        render(width, height, |f| page.draw(f, &f.area(), &league));
    }

    // no weeks at all
    if let core::League::Sleeper { matchups, .. } = &mut league {
        matchups.clear();
    }
    let buffer = render(100, 20, |f| page.draw(f, &f.area(), &league));
    assert!(contains(&buffer, "Matchups"));
}

#[tokio::test]
async fn the_roster_and_settings_widgets_show_their_data() {
    let league = sleeper_league().await;

    // without a selected owner there is no roster to show
    assert!(widgets::RosterWidget::from_league(&league).is_none());

    let owner = league
        .standings()
        .entries
        .into_iter()
        .find(|e| e.roster_id == "1")
        .unwrap();
    let league = league.with_owner_id(core::ExternalId::new(
        core::Platform::Sleeper,
        owner.owner_id.clone(),
    ));
    let roster = widgets::RosterWidget::from_league(&league).expect("the owner has a roster");
    assert_eq!("1", roster.roster_id);
    assert!(!roster.players.is_empty());
    let buffer = render(80, 30, |f| roster.draw(f, &f.area()));
    assert!(
        contains(&buffer, "Roster"),
        "the widget is titled by its owner: {}",
        text(&buffer)
    );
    assert!(
        contains(&buffer, "Matt Ryan"),
        "rostered players are listed"
    );

    let settings = widgets::SettingsListWidget::from_league(Some("My Scoring".into()), &league);
    let buffer = render(60, 12, |f| settings.draw(f, &f.area()));
    assert!(contains(&buffer, "My Scoring"));
    assert!(contains(&buffer, "pass_yd"));
}

#[tokio::test]
async fn the_league_list_shows_leagues_and_moves_the_selection() {
    let league = sleeper_league().await;
    let mut page = pages::LeaguesPage::new();
    page.add_league(league.clone());
    page.add_league(league.clone());

    let mut terminal = terminal(100, 8);
    page.draw(&mut terminal).unwrap();
    assert!(contains(terminal.backend().buffer(), "Rapid League 17"));

    assert_eq!(
        Some("sleeper:164974877262287888".to_string()),
        page.selected().map(|l| l.id().to_string())
    );
    use crossterm::event::KeyCode;
    assert!(matches!(
        page.handle_key(&KeyCode::Down),
        pages::KeyHandleResult::Noop
    ));
    assert!(matches!(
        page.handle_key(&KeyCode::Down),
        pages::KeyHandleResult::Noop
    ));
    assert!(
        page.selected().is_some(),
        "the selection stays on the last league"
    );
    page.select_by_league_id("sleeper:164974877262287888".to_string());
    assert!(page.selected().is_some());
}

#[tokio::test]
async fn owner_records_merge_the_ids_one_person_used_and_can_filter_by_league_name() {
    let league = sleeper_league().await;
    let owner = league
        .standings()
        .entries
        .into_iter()
        .find(|e| e.roster_id == "1")
        .unwrap();
    let history = HistoryConfig {
        league_name: Some("Rapid League 17".into()),
        owners: vec![OwnerAlias {
            name: "Sam Sample".into(),
            ids: vec![owner.owner_id.clone(), "an-id-from-another-platform".into()],
        }],
    };

    let mut page =
        pages::LeagueHistoryOwnerRecordsPage::from_leagues(std::slice::from_ref(&league), &history);
    let mut terminal = terminal(100, 20);
    page.draw(&mut terminal).unwrap();
    let screen = screen_text(&terminal);
    let row = screen
        .lines()
        .find(|l| l.contains("Sam Sample"))
        .expect("the alias is listed");
    assert!(
        row.contains("10") && row.contains("3"),
        "10-3 record: {row}"
    );

    // a league name that matches nothing leaves an empty table
    let history = HistoryConfig {
        league_name: Some("No Such League".into()),
        ..history
    };
    let mut page = pages::LeagueHistoryOwnerRecordsPage::from_leagues(&[league], &history);
    let mut terminal = crate::test_util::terminal(100, 20);
    page.draw(&mut terminal).unwrap();
    assert!(!screen_text(&terminal).contains("Sam Sample"));
}

fn screen_text(terminal: &ratatui::Terminal<ratatui::backend::TestBackend>) -> String {
    text(terminal.backend().buffer())
}
