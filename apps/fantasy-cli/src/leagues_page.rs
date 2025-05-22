use std::io::{self, Stdout};

use crossterm::event::KeyCode;
use tui::{
    Terminal,
    backend::CrosstermBackend,
    style::{Color, Modifier, Style},
    terminal::CompletedFrame,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

pub struct LeaguesPage {
    leagues: Vec<League>,
    selected: usize,
    // terminal: &'a mut Terminal<CrosstermBackend<Stdout>>,
}
impl<'a> LeaguesPage {
    pub fn new() -> Self {
        LeaguesPage {
            leagues: vec![],
            selected: 0,
            // terminal: terminal,
        }
    }

    fn make_list(&self) -> Vec<ListItem<'a>> {
        self.leagues
            .iter()
            .map(|league| ListItem::new(league.to_string()))
            .collect()
    }

    pub fn draw(
        &'a mut self,
        terminal: &'a mut Terminal<CrosstermBackend<Stdout>>, // items: &Vec<ListItem<'a>>,
                                                              // selected: usize,
    ) -> io::Result<CompletedFrame<'a>> {
        let title = "Fantasy Leagues (↑/↓, 'q' to quit, 'w' to welcome)";

        if self.leagues.is_empty() {
            terminal.draw(|f| {
                let size = f.size();
                let paragraph =
                    Paragraph::new("You do not have any leagues loaded.\n\nPress c to configure.")
                        .block(Block::default().title(title).borders(Borders::ALL));
                f.render_widget(paragraph, size);
            })
        } else {
            terminal.draw(|f| {
                let size = f.size();
                let list = List::new(self.make_list())
                    .block(Block::default().title(title).borders(Borders::ALL))
                    .highlight_style(
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    )
                    .highlight_symbol(">> ");

                let mut state = tui::widgets::ListState::default();
                state.select(Some(self.selected));
                f.render_stateful_widget(list, size, &mut state);
            })
        }
    }

    pub fn add_league(&mut self, value: League) {
        self.leagues.push(value);
    }

    pub fn handle_key(&mut self, key_code: &KeyCode) -> bool {
        match key_code {
            KeyCode::Down => {
                if self.selected < self.leagues.len() - 1 {
                    self.selected += 1;
                }
                true
            }
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                true
            }
            KeyCode::Enter => {
                println!("Do Something");
                true
            }
            _ => false,
        }
    }
}

pub enum League {
    Sleeper(String),
    Yahoo(String),
}
impl std::fmt::Display for League {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            League::Sleeper(name) => write!(f, "{} (Sleeper)", name),
            League::Yahoo(name) => write!(f, "{} (Yahoo)", name),
        }
    }
}
