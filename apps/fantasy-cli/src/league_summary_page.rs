use std::io::{self, Stdout};

use fantasy_core_rs::League;
use tui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    terminal::CompletedFrame,
    text::Text,
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::{
    draft_picks_widget::DraftPicksWidget, roster_widget::RosterWidget,
    settings_list_widget::SettingsListWidget,
};

pub struct LeagueSummaryPage<'a> {
    league: Option<&'a League>,
    scoring_settings_widget: Option<SettingsListWidget>,
    draft_picks_widget: Option<DraftPicksWidget>,
}
impl<'a> LeagueSummaryPage<'a> {
    pub fn new(league: Option<&'a League>) -> Self {
        LeagueSummaryPage {
            league: league,
            scoring_settings_widget: match league {
                Some(l) => Some(SettingsListWidget::from_league(
                    Some(String::from("Scoring Settings")),
                    l,
                )),
                None => None,
            },
            draft_picks_widget: match league {
                Some(l) => Some(DraftPicksWidget::from_league(
                    None,
                    l,
                    RosterWidget::from_league(l).map(|r| r.roster_id.clone()),
                )),
                None => None,
            },
        }
    }

    // pub fn set_league(&mut self, league: League) {
    //     self.league = Some(league);
    // }

    pub fn draw(
        &'a mut self,
        terminal: &'a mut Terminal<CrosstermBackend<Stdout>>,
    ) -> io::Result<CompletedFrame<'a>> {
        let num_of_columns = 3; // Number of columns to split the screen into
        let num_of_rows = 2;

        terminal.draw(|f| {
            let size = f.size();
            let block = Block::default()
                .title(format!(
                    "{} ('q' to quit, 'w' to welcome, 'l' to leagues, 'b' to go back, 'd' to draftboard, 'm' to matchups, 't' to transactions)",
                    self.league.map_or("N/A".to_string(), |l| l.name())
                ))
                .borders(Borders::all());

            if let Some(league) = &self.league {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(
                        (1..=num_of_columns)
                            .into_iter()
                            .map(|_| Constraint::Percentage(100 / num_of_columns))
                            .collect::<Vec<Constraint>>(),
                    )
                    .split(block.inner(size));

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
                            if let Some(roster_widget) = RosterWidget::from_league(&league) {
                                // Render roster in the second column
                                roster_widget.draw(f, row);
                            } else {
                                // Render empty paragraph if no roster
                                let text = Text::from("No roster available.");
                                let paragraph = Paragraph::new(text)
                                    .block(Block::default().borders(Borders::ALL));
                                f.render_widget(paragraph, *row);
                            }
                        } else if i == 1 && j == 0 {
                            if let Some(scoring_settings_widget) = &self.scoring_settings_widget {
                                scoring_settings_widget.draw(f, row);
                            } else {
                                let paragraph = Paragraph::new("")
                                    .block(Block::default().borders(Borders::ALL));
                                f.render_widget(paragraph, *row);
                            }
                        } else if i == 2 && j == 0 {
                            // Render league rosters in the third column
                            render_league_teams(league, f, row);
                        } else if i == 1 && j == 1 {
                            if let Some(draft_picks_widget) = &self.draft_picks_widget {
                                draft_picks_widget.draw(f, row);
                            } else {
                                let paragraph = Paragraph::new("")
                                    .block(Block::default().borders(Borders::ALL));
                                f.render_widget(paragraph, *row);
                            }
                        } else {
                            let paragraph =
                                Paragraph::new("").block(Block::default().borders(Borders::ALL));
                            f.render_widget(paragraph, *row);
                        }
                    }
                }
            } else {
                let text = Text::from("No league selected.");
                let paragraph = Paragraph::new(text).block(Block::default().borders(Borders::NONE));
                f.render_widget(paragraph, block.inner(size));
            }
            f.render_widget(block, size);
        })
    }
}

fn render_league_details(
    league: &League,
    f: &mut tui::Frame<'_, CrosstermBackend<Stdout>>,
    row: &tui::layout::Rect,
) {
    let title = "Summary";

    let settings = league.settings();

    let mut lines = vec![
        format!("League ID:    {}", league.id()),
        format!("Platform:     {}", league.platform()),
        format!("Season:       {}", league.season()),
        format!("Team Name:    {}", league.team_name()),
        format!("Record:       {}", league.record()),
        format!("Total Points: {}", format!("{:.2}", league.total_points())),
        format!(
            "Divisions:    {:?}",
            match settings.divisions.is_empty() {
                true => String::from("None"),
                false => settings.divisions.join(","),
            }
        ),
        format!(
            "Total Txns:   {}",
            format!("{}", league.transactions().len())
        ),
    ];

    let fmt_settings = settings.formatted_settings();
    fmt_settings.iter().for_each(|(k, v)| {
        lines.push(format!("{key:<13} {v}", key = format!("{k}:")));
    });

    let text = Text::from(lines.join("\n"));
    let paragraph = Paragraph::new(text).block(Block::default().title(title).borders(Borders::ALL));

    // let para = Paragraph::new(format!("Paragraph {},{}", i + 1, j + 1))
    //     .block(Block::default().borders(Borders::ALL));
    f.render_widget(paragraph, *row);
}

fn render_league_teams(
    league: &League,
    f: &mut tui::Frame<'_, CrosstermBackend<Stdout>>,
    row: &tui::layout::Rect,
) {
    let teams = league.teams();
    let title = format!("Teams ({})", teams.len(),);
    let lines = teams
        .iter()
        .map(|team| {
            let team_name = &team.team_name;
            let record = league.record_for_roster(team.roster_id.clone());
            format!("{team_name:<20} {record}")
        })
        .collect::<Vec<_>>();
    let text = Text::from(lines.join("\n"));
    let paragraph = Paragraph::new(text).block(Block::default().title(title).borders(Borders::ALL));
    f.render_widget(paragraph, *row);
}
