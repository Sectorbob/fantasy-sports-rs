use std::io::Stdout;

use fantasy_core_rs::League;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::matchups_list_widget::MatchupsList;

pub struct LeagueMatchupsTilesPage {}
impl LeagueMatchupsTilesPage {
    pub fn draw<'a>(
        &self,
        f: &mut tui::Frame<'a, CrosstermBackend<Stdout>>,
        rect: &Rect,
        league: &League,
    ) {
        let block = Block::default()
            .title("League Matchups")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White))
            .border_type(BorderType::Rounded)
            .style(Style::default().bg(Color::Black));
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
        let rows = area.height as usize / min_tile_height;
        let cols = weeks / rows + (if weeks % rows > 0 { 1 } else { 0 });

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
                match scoreboards
                    .iter()
                    .find(|s| s.matchups[0].week as usize == week)
                {
                    Some(scoreboard) => {
                        let matchup_widget =
                            MatchupsList::from(None, &scoreboard.matchups, &league.players());
                        matchup_widget.draw(f, row, None);
                    }
                    None => {
                        // Render empty paragraph in other cells
                        let text = Text::from(format!("No matchups found for week {}", week));
                        let paragraph =
                            Paragraph::new(text).block(Block::default().borders(Borders::ALL));
                        f.render_widget(paragraph, *row);
                    }
                };
            }
        }
    }
}
