use fantasy_sports_core::{League, Player, PlayerPosition};
use fantasy_sports_sleeper::User;
use std::{collections::HashMap, io::Stdout};
use tui::{
    backend::CrosstermBackend,
    layout::{self, Rect},
    style::Style,
    text::{Span, Spans, Text},
    widgets::{Block, BorderType, Borders, Paragraph},
};
use fantasy_sports_yahoo as yahoo;

use crate::{constants, helpers};

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
            // the roster view is not implemented for FleaFlicker yet
            League::FleaFlicker { .. } => None,
        }
    }

    pub fn from_sleeper(
        roster: &fantasy_sports_sleeper::Roster,
        owners: &HashMap<String, User>,
        players: &HashMap<String, fantasy_sports_sleeper::Player>,
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
                        positions: match &player.position {
                            Some(pos) => vec![PlayerPosition::from(pos)],
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
    pub fn from_yahoo(team: &fantasy_sports_yahoo::Team, roster: &fantasy_sports_yahoo::Roster) -> Self {
        RosterWidget {
            roster_id: team.team_key.to_string(),
            team_name: team.name.clone(),
            players: roster
                .players
                .iter()
                .map(|p| Player {
                    _id: p.player_key.to_string(),
                    name: p.name.full.clone(),
                    positions: match &p.display_position {
                        yahoo::PlayerPositions::One(player_position) => {
                            vec![PlayerPosition::from(player_position)]
                        }
                        yahoo::PlayerPositions::Many(player_positions) => {
                            player_positions.iter().map(PlayerPosition::from).collect()
                        }
                    },
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
            .map(|player| {
                player
                    .positions
                    .iter()
                    .map(|p| p.to_string().len())
                    .sum::<usize>()
            })
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
                .map(|p| {
                    Spans::from(
                        p.positions
                            .iter()
                            .map(|pos| helpers::position_span(pos))
                            .collect::<Vec<_>>(),
                    )
                })
                .collect::<Vec<_>>(),
        );
        let right_text = Text::from(
            self.players
                .iter()
                .map(|p| Spans::from(Span::raw(p.name.as_str())))
                .collect::<Vec<_>>(),
        );

        f.render_widget(Paragraph::new(left_text), layout[0]);
        f.render_widget(Paragraph::new(right_text), layout[1]);
    }
}
