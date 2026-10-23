use fantasy_sports_core::League;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::Text,
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::{constants, widgets};

pub struct LeagueMatchupsPage {}
impl LeagueMatchupsPage {
    pub fn draw<'a>(&self, f: &mut ratatui::Frame<'a>, rect: &Rect, league: &League) {
        let block = Block::default()
            .title(format!(
                "{name} ({season}) - Matchups",
                name = league.name(),
                season = league.season()
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(constants::COLOR_BORDER))
            .border_type(BorderType::Rounded)
            .style(Style::default().bg(constants::COLOR_BACKGROUND));
        f.render_widget(block.clone(), *rect);

        let area = block.inner(*rect);
        let scoreboards = league.scoreboards();
        let rows_needed_in_each_tile = scoreboards
            .iter()
            .map(|s| s.matchups.len())
            .max()
            .unwrap_or(6);
        let min_tile_height = rows_needed_in_each_tile + 2;
        let weeks = scoreboards.len();
        // never fewer than one row and one column, however small the terminal or short the season
        let rows = (area.height as usize / min_tile_height).max(1);
        let cols = weeks.div_ceil(rows).max(1);

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
            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    (1..=rows)
                        .into_iter()
                        .map(|_| Constraint::Percentage(100 / rows as u16))
                        .collect::<Vec<Constraint>>(),
                )
                .split(*col);

            for (i, row) in rows.iter().enumerate() {
                let week: usize = i * cols + j + 1;
                // weeks that have not been played have a scoreboard without matchups
                match scoreboards
                    .iter()
                    .find(|s| s.matchups.first().is_some_and(|m| m.week as usize == week))
                {
                    Some(scoreboard) => {
                        let matchup_widget = widgets::MatchupsList::from(
                            None,
                            &scoreboard.matchups,
                            &league.players(),
                        );
                        matchup_widget.draw(f, row, None);
                    }
                    None => {
                        // Render empty paragraph in other cells
                        let text = Text::from(format!("No matchups found for week {}", week));
                        let paragraph = Paragraph::new(text).block(
                            Block::default()
                                .borders(Borders::ALL)
                                .border_style(Style::default().fg(constants::COLOR_BORDER))
                                .border_type(BorderType::Rounded)
                                .style(Style::default().bg(constants::COLOR_BACKGROUND)),
                        );
                        f.render_widget(paragraph, *row);
                    }
                };
            }
        }
    }
}
