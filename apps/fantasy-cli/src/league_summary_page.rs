use std::io::{self, Stdout};

use tui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    terminal::CompletedFrame,
    text::Text,
    widgets::{Block, Borders, Paragraph},
};

use crate::{data::League, matchups_list_widget::MatchupsList};

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
        let num_of_columns = 5; // Number of columns to split the screen into
        let num_of_rows = 2;

        terminal.draw(|f| {
            let size = f.size();
            if let Some(league) = &self.league {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(
                        (1..=num_of_columns)
                            .into_iter()
                            .map(|_| Constraint::Percentage(100 / num_of_columns))
                            .collect::<Vec<Constraint>>(),
                    )
                    .split(size);

                // For each column, split into two rows
                for (i, col) in chunks.iter().enumerate() {
                    let rows = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints(
                            (1..=num_of_rows)
                                .into_iter()
                                .map(|_| Constraint::Percentage(100 / num_of_rows))
                                .collect::<Vec<Constraint>>(),
                        )
                        .split(*col);

                    for (j, row) in rows.iter().enumerate() {
                        if i == 0 && j == 0 {
                            // Render league details in the first column
                            render_league_details(league, f, row);
                        } else if i == 0 && j == 1 {
                            if let Some(roster_widget) = &league.roster_widget() {
                                // Render roster in the second column
                                roster_widget.draw(f, row);
                            } else {
                                // Render empty paragraph if no roster
                                let text = Text::from("No roster available.");
                                let paragraph = Paragraph::new(text)
                                    .block(Block::default().borders(Borders::ALL));
                                f.render_widget(paragraph, *row);
                            }
                        } else {
                            let week: usize = i * 2 + j; // Calculate week number based on column and row
                            let matchups = league.matchups();
                            let players = league.players();
                            match matchups.get(&week) {
                                Some(matchups) => {
                                    let matchup_widget =
                                        MatchupsList::from_sleeper(None, matchups, &players);
                                    matchup_widget.draw(f, row, None);
                                }
                                None => {
                                    // Render empty paragraph in other cells
                                    let text =
                                        Text::from(format!("No matchups found for week {}", week));
                                    let paragraph = Paragraph::new(text)
                                        .block(Block::default().borders(Borders::ALL));
                                    f.render_widget(paragraph, *row);
                                }
                            };
                        }
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

fn render_league_details(
    league: &League,
    f: &mut tui::Frame<'_, CrosstermBackend<Stdout>>,
    row: &tui::layout::Rect,
) {
    let title = format!(
        "{} ('q' to quit, 'w' to welcome 'l' to leagues)",
        league.name()
    );
    let text_1 = format!(
        "League ID: {}\nTeam Name: {}\nSeason:    {}\nRecord:    {}",
        league.id(),
        league.team_name(),
        league.season(),
        league.record(),
    );
    let text_2 = format!(
        "Total Points: {total_points}\n{settings:#?}\n{raw}",
        total_points = format!("{:.2}", league.total_points()),
        settings = league.settings(),
        raw = league.raw(),
    );
    let text = Text::from(format!("{}\n{}", text_1, text_2));
    let paragraph = Paragraph::new(text).block(Block::default().title(title).borders(Borders::ALL));

    // let para = Paragraph::new(format!("Paragraph {},{}", i + 1, j + 1))
    //     .block(Block::default().borders(Borders::ALL));
    f.render_widget(paragraph, *row);
}
