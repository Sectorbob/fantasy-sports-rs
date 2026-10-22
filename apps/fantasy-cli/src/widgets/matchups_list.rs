use std::collections::{HashMap, HashSet};

use fantasy_sports_core as core;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Cell, Row, Table},
};

use crate::constants;

pub struct MatchupsList {
    title_override: Option<String>,
    matchups: Vec<Matchup>,
}
impl MatchupsList {
    pub fn from(
        title_override: Option<String>,
        matchups: &Vec<core::Matchup>,
        players: &HashMap<String, core::Player>,
    ) -> Self {
        MatchupsList {
            title_override,
            matchups: matchups
                .iter()
                .map(|m| Matchup {
                    // team_name: format!("Roster {}", m.roster_id),
                    week: m.week.to_string(),
                    side_1: MatchupSide {
                        team_name: m.left.team_name.clone(),
                        team_id: m.left.roster_id.clone(),
                        score: m.left.score,
                        starters: m
                            .left
                            .players
                            .iter()
                            .filter_map(|player| players.get(&player.player_id.id).cloned())
                            .collect(),
                    },
                    side_2: m.right.as_ref().map_or(
                        MatchupSide {
                            team_name: String::from(""),
                            team_id: String::from(""),
                            score: 0.0,
                            starters: vec![],
                        },
                        |right| MatchupSide {
                            team_name: right.team_name.clone(),
                            team_id: right.roster_id.clone(),
                            score: right.score,
                            starters: right
                                .players
                                .iter()
                                .filter_map(|player| players.get(&player.player_id.id).cloned())
                                .collect(),
                        },
                    ),
                })
                .collect(),
        }
    }
    pub fn draw<'a>(&self, f: &mut ratatui::Frame<'a>, rect: &Rect, for_team: Option<&String>) {
        let team_mode = for_team.is_some();
        let title: String = if let Some(title_override) = self.title_override.clone() {
            title_override.trim().to_string()
        } else if let Some(team) = for_team {
            team.to_string()
        } else {
            let week_set: HashSet<String> = self.matchups.iter().map(|m| m.week.clone()).collect();
            // Collect back to Vec and sort
            let mut weeks: Vec<String> = week_set.into_iter().collect();
            weeks.sort();
            if weeks.len() == 1 {
                format!("Week {}", weeks.first().unwrap_or(&"/N/A".to_string()))
            } else {
                format!("Matchup List {}", weeks.join(","))
            }
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(constants::COLOR_BORDER))
            .border_type(BorderType::Rounded)
            .style(Style::default().bg(constants::COLOR_BACKGROUND));
        f.render_widget(block.clone(), *rect);

        let area = block.inner(*rect);

        let score_col_width: usize = self
            .matchups
            .iter()
            .map(|m| m.side_2.score_str().len().max(m.side_1.score_str().len()))
            .max()
            .unwrap_or_default();
        let max_team_name_length: usize = self
            .matchups
            .iter()
            .map(|m| m.side_2.team_name.len().max(m.side_1.team_name.len()))
            .max()
            .unwrap_or_default();
        let width_left = rect.width as usize - (2 * score_col_width) - 5;
        let max_team_col_width = width_left / 2;
        let team_name_col_width = max_team_col_width
            .max(max_team_name_length)
            .min(max_team_col_width);

        let col_widths = if team_mode {
            [
                Constraint::Length(4),
                Constraint::Min(team_name_col_width as u16),
                Constraint::Min(team_name_col_width as u16),
                Constraint::Length(0),
            ]
        } else {
            [
                Constraint::Min(team_name_col_width as u16),
                Constraint::Length(score_col_width as u16),
                Constraint::Length(score_col_width as u16),
                Constraint::Min(team_name_col_width as u16),
            ]
        };

        let rows = self
            .matchups
            .iter()
            .filter_map(|m| {
                if let Some(team_id) = for_team {
                    if m.side_1.team_id == *team_id {
                        Some(
                            Row::new(vec![
                                m.week.clone(),
                                m.side_2.team_name.clone(),
                                m.side_2.score_str(),
                            ])
                            .style(Style::default().fg(Color::Green)),
                        )
                    } else if m.side_2.team_id == *team_id {
                        Some(
                            Row::new(vec![
                                m.week.clone(),
                                m.side_1.team_name.clone(),
                                m.side_1.score_str(),
                            ])
                            .style(Style::default().fg(Color::Red)),
                        )
                    } else {
                        None
                    }
                } else {
                    // Paragraph::new(m.week.as_str()).alignment(Alignment::Left);
                    // Team1 Score Score Team2
                    let mut team_1_cell = Cell::from(m.side_1.team_name.clone());
                    let mut team_2_cell =
                        Cell::from(format!("{:>team_name_col_width$}", m.side_2.team_name));
                    if let Some(winner) = m.winner() {
                        if winner.team_id == m.side_1.team_id {
                            team_1_cell = team_1_cell.style(Style::default().fg(Color::Yellow))
                        } else {
                            team_2_cell = team_2_cell.style(Style::default().fg(Color::Yellow))
                        }
                    }

                    Some(Row::new(vec![
                        team_1_cell, // Left Justified
                        Cell::from(format!(
                            "{:>score_col_width$}",
                            m.side_1.score_str().as_str()
                        )), // Right Justified
                        Cell::from(format!(
                            "{:>score_col_width$}",
                            m.side_2.score_str().as_str()
                        )), // Right Justified
                        team_2_cell, // Right Justified
                    ]))
                }
            })
            .collect::<Vec<Row>>();
        let header = if team_mode {
            Some(
                Row::new(vec!["Week", "Opponent", "Result"])
                    .style(
                        ratatui::style::Style::default()
                            .add_modifier(ratatui::style::Modifier::BOLD),
                    )
                    .style(Style::default().fg(Color::Yellow)),
            )
        } else {
            None
            // Row::new(vec!["Team 1", "", "", "Team 2"])
            //     .style(ratatui::style::Style::default().add_modifier(ratatui::style::Modifier::BOLD))
        };

        let mut table = Table::new(rows, Vec::<Constraint>::new())
            // .block(Block::default().borders(Borders::ALL).title("Students"))
            .widths(col_widths);
        if let Some(header) = header {
            table = table.header(header)
        }
        f.render_widget(table, area);
    }
}

#[derive(Debug, Clone)]
pub struct Matchup {
    // pub team_name: String,
    pub week: String,
    pub side_1: MatchupSide,
    pub side_2: MatchupSide,
}
impl Matchup {
    pub fn winner(&self) -> Option<&MatchupSide> {
        if self.side_1.score > self.side_2.score {
            Some(&self.side_1)
        } else if self.side_2.score > self.side_1.score {
            Some(&self.side_2)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct MatchupSide {
    pub team_name: String,
    pub team_id: String,
    pub score: f32,
    pub starters: Vec<core::Player>,
}
impl MatchupSide {
    pub fn score_str(&self) -> String {
        format!("{:.2}", self.score)
    }
}
