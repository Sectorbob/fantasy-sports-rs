use std::io::{self, Stdout};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::Constraint,
    style::Style,
    CompletedFrame,
    widgets::{Block, BorderType, Borders, Cell, Row, Table},
};

use fantasy_sports_core::{self as core};

use crate::{config::HistoryConfig, constants};

pub struct LeagueHistoryOwnerRecordsPage {
    title_override: Option<String>,
    owner_records: Vec<core::StandingsEntry>,
}
impl LeagueHistoryOwnerRecordsPage {
    pub fn new() -> Self {
        LeagueHistoryOwnerRecordsPage {
            title_override: None,
            owner_records: vec![],
        }
    }
    pub fn from_leagues(leagues: &Vec<core::League>, history: &HistoryConfig) -> Self {
        let leagues_standings = leagues
            .iter()
            .filter(|l| {
                history
                    .league_name
                    .as_ref()
                    .map_or(true, |name| l.name() == name.as_str())
            })
            .map(|l| l.standings())
            .collect::<Vec<_>>();
        let owner_id_mapping = history
            .owners
            .iter()
            .map(|owner| (owner.name.as_str(), owner.ids.iter().map(String::as_str).collect()))
            .collect::<Vec<(&str, Vec<&str>)>>();
        let owner_records =
            core::history::build_owner_records(&leagues_standings, &owner_id_mapping);
        LeagueHistoryOwnerRecordsPage {
            title_override: None,
            owner_records,
        }
    }

    pub fn draw<'a>(
        &'a mut self,
        terminal: &'a mut Terminal<CrosstermBackend<Stdout>>,
    ) -> io::Result<CompletedFrame<'a>> {
        terminal.draw(|f| {
            let size = f.area();
            let title: String = if let Some(title_override) = self.title_override.clone() {
                title_override.trim().to_string()
            } else {
                String::from("League History Owner Records")
            };

            let block = Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(constants::COLOR_BORDER))
                .border_type(BorderType::Rounded)
                .style(Style::default().bg(constants::COLOR_BACKGROUND));

            let mut rows = vec![];
            for (_, e) in self.owner_records.iter().enumerate() {
                // let standings = &self.leagues_standings[i];
                rows.push(Row::new(vec![
                    // Cell::from("Rapid League 17"),
                    Cell::from(e.team_name.clone()),
                    Cell::from(e.roster_id.clone()),
                    Cell::from(e.owner_name.clone()),
                    // Cell::from(e.owner_id.clone()),
                    Cell::from(e.wins.to_string()),
                    Cell::from(e.losses.to_string()),
                    Cell::from(e.ties.to_string()),
                ]));
            }

            let table = Table::new(rows, Vec::<Constraint>::new())
                .widths(&[
                    // Constraint::Percentage(20),
                    Constraint::Percentage(30),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    // Constraint::Percentage(20),
                    Constraint::Percentage(10),
                    Constraint::Percentage(10),
                    Constraint::Percentage(10),
                ])
                .header(Row::new(vec![
                    // "League Name",
                    "Team Name",
                    "Roster",
                    "Owner Name",
                    // "OwnerId",
                    "W",
                    "L",
                    "T",
                ]));

            f.render_widget(table, block.inner(size));
            f.render_widget(block.clone(), size);
        })
    }
}
