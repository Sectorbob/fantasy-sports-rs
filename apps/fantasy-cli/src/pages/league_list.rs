use std::io::{self, Stdout};

use crossterm::event::KeyCode;
use fantasy_sports_core::League;
use ratatui::{
    CompletedFrame, Terminal,
    backend::CrosstermBackend,
    style::{Modifier, Style},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
};

use crate::constants;

pub struct LeaguesPage {
    pub leagues: Vec<League>,
    selected: usize,
    _omit_incomplete: bool, //TODO: implement filtering
                            // terminal: &'a mut Terminal<CrosstermBackend<Stdout>>,
}
impl<'a> LeaguesPage {
    pub fn new() -> Self {
        LeaguesPage {
            leagues: vec![],
            selected: 0,
            _omit_incomplete: true, // terminal: terminal,
        }
    }

    // pub fn leagues(&self) -> &Vec<League> {
    //     &self.leagues
    //         .iter()
    //         .filter(|l| !self.omit_incomplete || l.status() != LeagueStatus::InComplete)
    // }

    fn make_list(&self) -> Vec<ListItem<'a>> {
        self.leagues
            .iter()
            // .filter(|l| !self.omit_incomplete || l.status() != LeagueStatus::InComplete)
            .map(|league| ListItem::new(league.to_string()))
            .collect()
    }

    pub fn select_by_league_id(&mut self, league_id: String) {
        if let Some(index) = self
            .leagues
            .iter()
            .position(|l| l.id().to_string() == league_id)
        {
            self.selected = index
        }
    }

    pub fn selected(&self) -> Option<&League> {
        if self.leagues.is_empty() || self.selected >= self.leagues.len() {
            None
        } else {
            Some(&self.leagues[self.selected])
        }
    }

    pub fn draw(
        &'a mut self,
        terminal: &'a mut Terminal<CrosstermBackend<Stdout>>, // items: &Vec<ListItem<'a>>,
                                                              // selected: usize,
    ) -> io::Result<CompletedFrame<'a>> {
        let title = "Fantasy Leagues (↑/↓, 'q' to quit, 'w' to welcome)";

        if self.leagues.is_empty() {
            terminal.draw(|f| {
                let size = f.area();
                let paragraph =
                    Paragraph::new("You do not have any leagues loaded.\n\nPress c to configure.")
                        .block(
                            Block::default()
                                .title(title)
                                .borders(Borders::ALL)
                                .border_style(Style::default().fg(constants::COLOR_BORDER))
                                .border_type(BorderType::Rounded)
                                .style(Style::default().bg(constants::COLOR_BACKGROUND)),
                        );
                f.render_widget(paragraph, size);
            })
        } else {
            terminal.draw(|f| {
                let size = f.area();
                let list = List::new(self.make_list())
                    .block(
                        Block::default()
                            .title(title)
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(constants::COLOR_BORDER))
                            .border_type(BorderType::Rounded)
                            .style(Style::default().bg(constants::COLOR_BACKGROUND)),
                    )
                    .highlight_style(
                        Style::default()
                            .fg(constants::COLOR_HIGHLIGHT)
                            .add_modifier(Modifier::BOLD),
                    )
                    .highlight_symbol(">> ");

                let mut state = ratatui::widgets::ListState::default();
                state.select(Some(self.selected));
                f.render_stateful_widget(list, size, &mut state);
            })
        }
    }

    pub fn add_league(&mut self, value: League) {
        self.leagues.push(value);
    }

    pub fn handle_key(&mut self, key_code: &KeyCode) -> KeyHandleResult {
        match key_code {
            KeyCode::Char('r') => KeyHandleResult::GoToLeagueHistoryOwnerRecords,
            KeyCode::Down => {
                if self.selected + 1 < self.leagues.len() {
                    self.selected += 1;
                }
                KeyHandleResult::Noop
            }
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                KeyHandleResult::Noop
            }
            KeyCode::Enter => {
                // Here you can handle the Enter key, e.g., show league details

                KeyHandleResult::GoToLeagueSummary
            }
            _ => KeyHandleResult::Unhandled,
        }
    }
}

pub enum KeyHandleResult {
    Unhandled,
    Noop,
    GoToLeagueSummary,
    GoToLeagueHistoryOwnerRecords,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigating_an_empty_list_does_not_panic_or_select_anything() {
        let mut page = LeaguesPage::new();
        // the list is empty until the first fetch finishes; this used to underflow `len() - 1`
        for key in [KeyCode::Down, KeyCode::Down, KeyCode::Up, KeyCode::Up] {
            assert!(matches!(page.handle_key(&key), KeyHandleResult::Noop));
        }
        assert!(page.selected().is_none());
    }

    #[test]
    fn keys_map_to_navigation_results() {
        let mut page = LeaguesPage::new();
        assert!(matches!(
            page.handle_key(&KeyCode::Enter),
            KeyHandleResult::GoToLeagueSummary
        ));
        assert!(matches!(
            page.handle_key(&KeyCode::Char('r')),
            KeyHandleResult::GoToLeagueHistoryOwnerRecords
        ));
        assert!(matches!(
            page.handle_key(&KeyCode::Char('x')),
            KeyHandleResult::Unhandled
        ));
    }
}
