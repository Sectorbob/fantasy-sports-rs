use std::io::Stdout;

use tui::{
    backend::CrosstermBackend,
    layout::{self, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, BorderType, Borders, Paragraph},
};
use yahoo_fantasy_rs;

pub struct Roster {
    pub team_name: String,
    pub players: Vec<Player>,
}
impl Roster {
    pub fn from_sleeper(roster: &sleeper_fantasy_rs::Roster) -> Self {
        Roster {
            team_name: "TODO".to_string(),
            players: roster
                .players
                .iter()
                .map(|p| Player {
                    name: format!("Id: {}", p),
                    position: "TBD".to_string(),
                })
                .collect(),
        }
    }
    pub fn from_yahoo(team: &yahoo_fantasy_rs::Team, roster: &yahoo_fantasy_rs::Roster) -> Self {
        Roster {
            team_name: team.name.clone(),
            players: roster
                .players
                .players
                .iter()
                .map(|p| Player {
                    name: p.name.full.clone(),
                    position: p.display_position.to_string(),
                })
                .collect(),
        }
    }

    pub fn draw<'a>(&self, f: &mut tui::Frame<'a, CrosstermBackend<Stdout>>, rect: &Rect) {
        let title = format!("{}'s Roster", self.team_name);

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White))
            .border_type(BorderType::Rounded)
            .style(Style::default().bg(Color::Black));
        f.render_widget(block.clone(), *rect);
        let inside_area = block.inner(*rect);

        let layout = layout::Layout::default()
            .direction(tui::layout::Direction::Horizontal)
            .margin(1)
            .constraints(
                [
                    tui::layout::Constraint::Length(3),
                    tui::layout::Constraint::Min(0),
                ]
                .as_ref(),
            )
            .split(inside_area);

        let left_text = Text::from(
            self.players
                .iter()
                .map(|p| p.position.as_str())
                .collect::<Vec<&str>>()
                .join("\n"),
        );
        let right_text = Text::from(
            self.players
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<&str>>()
                .join("\n"),
        );

        f.render_widget(Paragraph::new(left_text), layout[0]);
        f.render_widget(Paragraph::new(right_text), layout[1]);
    }
}

pub struct Player {
    pub name: String,
    pub position: String,
}
