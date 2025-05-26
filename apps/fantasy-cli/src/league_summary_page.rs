use std::io::{self, Stdout};

use tui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    terminal::CompletedFrame,
    text::Text,
    widgets::{Block, Borders, Paragraph},
};

use crate::leagues_page::League;

pub struct LeagueSummaryPage<'a> {
    league: Option<&'a League>,
}
impl<'a> LeagueSummaryPage<'a> {
    pub fn new(league: Option<&'a League>) -> Self {
        LeagueSummaryPage { league: league }
    }

    // pub fn set_league(&mut self, league: League) {
    //     self.league = Some(league);
    // }

    pub fn draw(
        &'a mut self,
        terminal: &'a mut Terminal<CrosstermBackend<Stdout>>,
    ) -> io::Result<CompletedFrame<'a>> {
        terminal.draw(|f| {
            let size = f.size();
            if let Some(league) = &self.league {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(
                        [
                            Constraint::Percentage(100),
                            // Constraint::Percentage(50),
                            // Constraint::Percentage(50),
                        ]
                        .as_ref(),
                    )
                    .split(size);

                // For each column, split into two rows
                for (i, col) in chunks.iter().enumerate() {
                    let rows = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints(
                            [
                                Constraint::Percentage(100),
                                // Constraint::Percentage(50),
                                // Constraint::Percentage(50),
                            ]
                            .as_ref(),
                        )
                        .split(*col);

                    for (j, row) in rows.iter().enumerate() {
                        let title = format!(
                            "{} League Summary (↑/↓, 'q' to quit, 'w' to welcome 'l' to leagues)",
                            league.name()
                        );
                        let text_1 = format!(
                            "Team Name: {}\nSeason:    {}\nRecord:    {}",
                            league.team_name(),
                            league.season(),
                            league.record(),
                        );
                        let text_2 = format!(
                            "Total Points: {}\nDetails: {}",
                            league.total_points(),
                            league.raw_standings(),
                        );
                        let text = Text::from(format!("{}\n{}", text_1, text_2));
                        let paragraph = Paragraph::new(text)
                            .block(Block::default().title(title).borders(Borders::ALL));

                        // let para = Paragraph::new(format!("Paragraph {},{}", i + 1, j + 1))
                        //     .block(Block::default().borders(Borders::ALL));
                        f.render_widget(paragraph, *row);
                    }
                }
            } else {
                let text = Text::from("No league selected.");
                let paragraph = Paragraph::new(text).block(Block::default().borders(Borders::NONE));
                f.render_widget(paragraph, size);
            }
        })
    }
}
