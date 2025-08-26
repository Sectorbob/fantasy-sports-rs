use fantasy_core_rs::{League, Player};
use sleeper_fantasy_rs::User;
use std::{collections::HashMap, io::Stdout};
use tui::{
    backend::CrosstermBackend,
    layout::{self, Rect},
    style::Style,
    text::Text,
    widgets::{Block, BorderType, Borders, Paragraph},
};
use yahoo_fantasy_rs;

use crate::constants;

pub struct RosterWidget {
    pub team_name: String,
    pub roster_id: String,
    pub players: Vec<Player>,
}
impl RosterWidget {
    pub fn from_league(league: &League) -> Option<Self> {
        match league {
            League::Sleeper {
                owners,
                players,
                rosters,
                user_id,
                ..
            } => rosters
                .iter()
                .find(|(_, r)| &r.owner_id == user_id)
                .map(|(_, roster)| RosterWidget::from_sleeper(roster, &owners, &players)),
            League::Yahoo { rosters, team, .. } => team
                .as_ref()
                .map(|t| {
                    rosters
                        .get(&t.team_id)
                        .map(|r| RosterWidget::from_yahoo(t, r))
                })
                .flatten(),
        }
    }

    pub fn from_sleeper(
        roster: &sleeper_fantasy_rs::Roster,
        owners: &HashMap<String, User>,
        players: &HashMap<String, sleeper_fantasy_rs::Player>,
    ) -> Self {
        RosterWidget {
            roster_id: roster.roster_id.to_string(),
            team_name: roster
                .owner_id
                .as_ref()
                .map(|owner_id| owners.get(owner_id).map(|owner| owner.display_name.clone()))
                .flatten()
                .unwrap_or(format!("User({:?})", roster.owner_id)),
            players: roster
                .players
                .iter()
                .map(|pid| match players.get(pid) {
                    Some(player) => Player {
                        _id: pid.clone(),
                        name: format!(
                            "{} {}",
                            player.first_name.clone().unwrap_or_default(),
                            player.last_name.clone().unwrap_or_default()
                        )
                        .trim()
                        .to_string(),
                        positions: match player.position.clone() {
                            Some(pos) => vec![pos],
                            None => vec![],
                        },
                        irl_team: player.team.clone(),
                    },
                    None => Player {
                        _id: pid.clone(),
                        name: format!("Missing Player ({})", pid),
                        positions: vec![],
                        irl_team: None,
                    },
                })
                .collect::<Vec<Player>>(),
        }
    }
    pub fn from_yahoo(team: &yahoo_fantasy_rs::Team, roster: &yahoo_fantasy_rs::Roster) -> Self {
        RosterWidget {
            roster_id: team.team_key.to_string(),
            team_name: team.name.clone(),
            players: roster
                .players
                .iter()
                .map(|p| Player {
                    _id: p.player_key.to_string(),
                    name: p.name.full.clone(),
                    positions: vec![p.display_position.to_string()],
                    irl_team: Some(p.editorial_team_abbr.clone()),
                })
                .collect::<Vec<Player>>(),
        }
    }
    pub fn draw<'a>(&self, f: &mut tui::Frame<'a, CrosstermBackend<Stdout>>, rect: &Rect) {
        let title = format!("{}'s Roster", self.team_name);

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(constants::COLOR_BORDER))
            .border_type(BorderType::Rounded)
            .style(Style::default().bg(constants::COLOR_BACKGROUND));
        f.render_widget(block.clone(), *rect);
        let inside_area = block.inner(*rect);

        let pos_col_width = self
            .players
            .iter()
            .map(|player| player.positions.iter().map(String::len).sum::<usize>())
            .max()
            .unwrap_or(2);

        let layout = layout::Layout::default()
            .direction(tui::layout::Direction::Horizontal)
            .margin(1)
            .constraints(
                [
                    tui::layout::Constraint::Length(pos_col_width as u16),
                    tui::layout::Constraint::Min(25),
                ]
                .as_ref(),
            )
            .split(inside_area);

        let left_text = Text::from(
            self.players
                .iter()
                .map(|p| p.positions.join(","))
                .collect::<Vec<String>>()
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
