use fantasy_core_rs::{self as core, DraftType};
use std::io::Stdout;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::Text,
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::constants;

pub struct DraftBoardPage {
    league_name: Option<String>,
    league_season: Option<String>,
    league_settings: Option<core::LeagueSettings>,
    league_draft: Option<core::Draft>,
    league_rosters: Vec<core::Roster>,
}
impl DraftBoardPage {
    pub fn new() -> Self {
        DraftBoardPage {
            league_name: None,
            league_season: None,
            league_settings: None,
            league_draft: None,
            league_rosters: vec![],
        }
    }

    pub fn draw<'a>(&self, f: &mut tui::Frame<'a, CrosstermBackend<Stdout>>, rect: &Rect) {
        let league_name = if let Some(n) = &self.league_name {
            n
        } else {
            f.render_widget(block_with_title("No League Set - Draftboard"), *rect);
            return;
        };
        let league_season = if let Some(s) = &self.league_season {
            s
        } else {
            f.render_widget(block_with_title("No League Season - Draftboard"), *rect);
            return;
        };
        let league_settings = if let Some(s) = &self.league_settings {
            s
        } else {
            f.render_widget(block_with_title("No League Settings - Draftboard"), *rect);
            return;
        };
        let league_draft = if let Some(d) = &self.league_draft {
            d
        } else {
            f.render_widget(block_with_title("No League Draft - Draftboard"), *rect);
            return;
        };
        let league_rosters = &self.league_rosters;
        let pick_order = &league_draft.draft_order;

        let cols = league_rosters.len();
        let rows = league_settings.draft_rounds;
        let draft_type = &league_settings.draft_type;

        let block = block(league_name, league_season);
        f.render_widget(block.clone(), *rect);

        let area = block.inner(*rect);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                (1..=cols)
                    .into_iter()
                    .map(|_| Constraint::Percentage(100 / cols as u16))
                    .collect::<Vec<Constraint>>(),
            )
            .split(area);

        // For each column, split into two rows
        for (j, col) in chunks.iter().enumerate() {
            let mut row_constraints = vec![Constraint::Length(1)];
            row_constraints.extend(
                (1..=rows)
                    .into_iter()
                    .map(|_| Constraint::Percentage(100 / rows as u16)),
            );

            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints(row_constraints)
                .split(*col);

            for (i, row) in rows.iter().enumerate() {
                if i == 0 {
                    let team_name = if let Some(pick_order) = &pick_order {
                        let team_id = pick_order.get(j).map_or(format!("Slot {j}"), |v| {
                            v.clone().map_or(format!("Slot {j}"), |v| v.clone())
                        });
                        format!("Team {team_id}")
                    } else {
                        "Blerp".to_string()
                    };

                    let paragraph = Paragraph::new(team_name);
                    // .block(Block::default()
                    // .title("Blerp")
                    // .borders(Borders::ALL));
                    f.render_widget(paragraph, *row);
                    continue;
                }
                let round = i;
                let pick = if round % 2 == 0 {
                    match draft_type {
                        DraftType::Snake => cols - j,
                        DraftType::Auction => 0,
                        DraftType::Linear => j + 1,
                    }
                } else {
                    j + 1
                };

                let overall = (round - 1) * cols + pick;
                let overall_pick = format!("{round}.{pick}");
                let text = if let Some(pick_player) = league_draft.picks.get(overall - 1) {
                    Text::from(format!(
                        "{name}\n{pos} - {team}",
                        name = pick_player.player.name,
                        pos = pick_player
                            .player
                            .positions
                            .iter()
                            .map(String::from)
                            .collect::<Vec<_>>()
                            .join(","),
                        team = pick_player
                            .player
                            .irl_team
                            .as_ref()
                            .map_or("FA".to_string(), String::clone)
                    ))
                } else {
                    Text::from(format!("Overall\n{overall}"))
                };

                let paragraph = Paragraph::new(text).block(
                    Block::default()
                        .title(overall_pick)
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(constants::COLOR_BORDER))
                        .border_type(BorderType::Rounded)
                        .style(Style::default().bg(constants::COLOR_BACKGROUND)),
                );
                f.render_widget(paragraph, *row);
            }
        }
    }
}
impl From<&core::League> for DraftBoardPage {
    fn from(value: &core::League) -> Self {
        DraftBoardPage {
            league_name: Some(value.name()),
            league_season: Some(value.season()),
            league_settings: Some(value.settings()),
            league_draft: value.draft(),
            league_rosters: value.rosters(),
        }
    }
}

fn block<'a>(league_name: &'a str, league_season: &'a str) -> Block<'a> {
    block_with_title(format!("{league_name} ({league_season}) - Draftboard"))
}

fn block_with_title<'a, T: Into<String>>(title: T) -> Block<'a> {
    Block::default()
        .title(title.into())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(constants::COLOR_BORDER))
        .border_type(BorderType::Rounded)
        .style(Style::default().bg(constants::COLOR_BACKGROUND))
}
