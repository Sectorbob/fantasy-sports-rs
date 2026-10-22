use std::io::{self, Stdout};

use fantasy_sports_core::League;
use ratatui::{
    CompletedFrame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::Style,
    text::Text,
    widgets::{Block, BorderType, Borders, Paragraph},
};

use crate::{constants, widgets};

pub struct LeagueSummaryPage<'a> {
    league: Option<&'a League>,
    scoring_settings_widget: Option<widgets::SettingsListWidget>,
    draft_picks_widget: Option<widgets::DraftPicksWidget>,
}
impl<'a> LeagueSummaryPage<'a> {
    pub fn new(league: Option<&'a League>) -> Self {
        LeagueSummaryPage {
            league: league,
            scoring_settings_widget: match league {
                Some(l) => Some(widgets::SettingsListWidget::from_league(
                    Some(String::from("Scoring Settings")),
                    l,
                )),
                None => None,
            },
            draft_picks_widget: match league {
                Some(l) => Some(widgets::DraftPicksWidget::from_league(
                    None,
                    l,
                    widgets::RosterWidget::from_league(l).map(|r| r.roster_id.clone()),
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
            let size = f.area();
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
                            if let Some(roster_widget) = widgets::RosterWidget::from_league(&league) {
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
                            // Render league rosters in the third column, top row
                            render_league_teams(league, f, row);
                        } else if i == 2 && j == 1 {
                            // Render league playoff results in the third column, bottom row
                            render_league_playoff_results(league, f, row);
                        } else if i == 1 && j == 1 {
                            if let Some(draft_picks_widget) = &self.draft_picks_widget {
                                draft_picks_widget.draw(f, row);
                            } else {
                                let paragraph = Paragraph::new("")
                                    .block(Block::default()
                                        .borders(Borders::ALL)
                                        .border_style(Style::default().fg(constants::COLOR_BORDER))
                                        .border_type(BorderType::Rounded)
                                        .style(Style::default().bg(constants::COLOR_BACKGROUND))
                                    );
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

fn render_league_details(league: &League, f: &mut ratatui::Frame<'_>, row: &ratatui::layout::Rect) {
    let title = "Summary";

    let settings = league.settings();

    let mut lines = vec![
        format!("League ID:    {}", league.id()),
        format!("Platform:     {}", league.platform()),
        format!("Season:       {}", league.season()),
        format!(
            "Divisions:    {}",
            match settings.divisions.is_empty() {
                true => String::from("None"),
                false => settings.divisions.join(", "),
            }
        ),
        format!(
            "Total Txns:   {}",
            format!("{}", league.transactions().len())
        ),
    ];

    if let Some(team) = league.team() {
        lines.push("".to_string());
        lines.push(format!("Name:         {}", team.team_name));
        if let Some(owner_name) = &team.owner_name {
            lines.push(format!("Owner:        {}", owner_name));
        }
        lines.push(format!("Record:       {}", league.record()));
        lines.push(format!(
            "Points:       {pts:.2}",
            pts = league.total_points()
        ));
    }

    lines.push("".to_string());
    lines.push("Settings".to_string());
    let fmt_settings = settings.formatted_settings();
    fmt_settings.iter().for_each(|(k, v)| {
        lines.push(format!("{key:<13} {v}", key = format!("{k}:")));
    });

    let text = Text::from(lines.join("\n"));
    let paragraph = Paragraph::new(text).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(constants::COLOR_BORDER))
            .border_type(BorderType::Rounded)
            .style(Style::default().bg(constants::COLOR_BACKGROUND)),
    );

    f.render_widget(paragraph, *row);
}

fn render_league_teams(league: &League, f: &mut ratatui::Frame<'_>, row: &ratatui::layout::Rect) {
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
    let paragraph = Paragraph::new(text).block(
        Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(constants::COLOR_BORDER))
            .border_type(BorderType::Rounded)
            .style(Style::default().bg(constants::COLOR_BACKGROUND)),
    );
    f.render_widget(paragraph, *row);
}

fn render_league_playoff_results(
    league: &League,
    f: &mut ratatui::Frame<'_>,
    row: &ratatui::layout::Rect,
) {
    let teams = league.teams();
    if let Some(playoff_results) = league.playoff_results() {
        let mut entries = playoff_results.into_iter().collect::<Vec<_>>();
        entries.sort_by_key(|e| e.0);

        let lines = entries
            .iter()
            .map(|(place, team)| {
                format!(
                    "{place}: {team_id}",
                    team_id = teams
                        .iter()
                        .find(|t| { t.id == team.id })
                        .map_or(format!(""), |t| t.team_name.clone())
                )
            })
            .collect::<Vec<_>>();
        let text = Text::from(lines.join("\n"));
        let paragraph = Paragraph::new(text).block(
            Block::default()
                .title("Playoff Results")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(constants::COLOR_BORDER))
                .border_type(BorderType::Rounded)
                .style(Style::default().bg(constants::COLOR_BACKGROUND)),
        );
        f.render_widget(paragraph, *row);
    } else {
        let paragraph = Paragraph::new("").block(
            Block::default()
                .title("No Playoff Results")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(constants::COLOR_BORDER))
                .border_type(BorderType::Rounded)
                .style(Style::default().bg(constants::COLOR_BACKGROUND)),
        );
        f.render_widget(paragraph, *row);
    }
}
