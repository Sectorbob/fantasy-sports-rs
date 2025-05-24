use std::io::{self, Stdout};

use crossterm::event::KeyCode;
use sleeper_fantasy_rs;
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
    Sleeper {
        league: sleeper_fantasy_rs::League,
        rosters: Vec<sleeper_fantasy_rs::Roster>,
        user_id: String,
    },
    Yahoo {
        team: yahoo_fantasy_rs::Team,
        league: yahoo_fantasy_rs::League,
        standings: yahoo_fantasy_rs::Standings,
    },
}
impl League {
    pub fn season(&self) -> String {
        match self {
            League::Sleeper {
                league,
                rosters,
                user_id: _,
            } => league.season.clone(),
            League::Yahoo {
                team: _,
                league,
                standings: _,
            } => league.season.clone(),
        }
    }
    pub fn record(&self) -> String {
        match self {
            League::Sleeper {
                league: _,
                rosters,
                user_id,
            } => match rosters.iter().find(|r| &r.owner_id == user_id) {
                Some(roster) => format!(
                    "{}-{}-{}",
                    match roster.settings.get("wins") {
                        Some(n) => n.to_string(),
                        None => "0".to_string(),
                    },
                    match roster.settings.get("losses") {
                        Some(n) => n.to_string(),
                        None => "0".to_string(),
                    },
                    match roster.settings.get("ties") {
                        Some(n) => n.to_string(),
                        None => "0".to_string(),
                    }
                ),
                None => "n/a".to_string(),
            },
            League::Yahoo {
                team,
                league: _,
                standings,
            } => match standings.teams().iter().find(|t| t.team_id == team.team_id) {
                Some(team_w_standings) => match &team_w_standings.team_standings {
                    Some(team_standings) => format!(
                        "{}-{}-{}",
                        team_standings.outcome_totals.wins,
                        team_standings.outcome_totals.losses,
                        team_standings.outcome_totals.ties
                    ),
                    None => "n/a".to_string(),
                },
                None => "n/a".to_string(),
            },
        }
    }
    pub fn team_name(&self) -> String {
        match self {
            League::Sleeper {
                league,
                rosters: _,
                user_id: _,
            } => "".to_string(),
            League::Yahoo {
                team,
                league: _,
                standings: _,
            } => team.name.clone(),
        }
    }
}
impl std::fmt::Display for League {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            League::Sleeper {
                league,
                rosters,
                user_id: _,
            } => write!(
                f,
                "[{}] {} (Sleeper) - {}    Record: {}",
                season = league.season,
                name = league.name,
                status = match league.status {
                    sleeper_fantasy_rs::LeagueStatus::PreDraft => "predraft",
                    sleeper_fantasy_rs::LeagueStatus::Drafting => "drafting",
                    sleeper_fantasy_rs::LeagueStatus::InSeason => "inseason",
                    sleeper_fantasy_rs::LeagueStatus::Complete => "complete",
                },
                record = self.record(),
            ),
            League::Yahoo {
                team: _,
                league,
                standings,
            } => write!(
                f,
                "[{}] {} (Yahoo) - {}       Record: {}",
                league.season,
                league.name,
                match league.is_finished {
                    Some(is_finished) => match is_finished {
                        true => "complete",
                        false => "in-progress",
                    },
                    None => "incomplete",
                },
                self.record(),
            ),
        }
    }
}
