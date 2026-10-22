use ratatui::{
    CompletedFrame, Terminal,
    backend::Backend,
    layout::Constraint,
    style::Style,
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
    pub fn from_leagues(leagues: &[core::League], history: &HistoryConfig) -> Self {
        let leagues_standings = leagues
            .iter()
            .filter(|l| {
                history
                    .league_name
                    .as_ref()
                    .is_none_or(|name| l.name() == name.as_str())
            })
            .map(|l| l.standings())
            .collect::<Vec<_>>();
        let owner_id_mapping = history
            .owners
            .iter()
            .map(|owner| {
                (
                    owner.name.as_str(),
                    owner.ids.iter().map(String::as_str).collect(),
                )
            })
            .collect::<Vec<(&str, Vec<&str>)>>();
        let owner_records =
            core::history::build_owner_records(&leagues_standings, &owner_id_mapping);
        LeagueHistoryOwnerRecordsPage {
            title_override: None,
            owner_records,
        }
    }

    pub fn draw<'a, B: Backend>(
        &'a mut self,
        terminal: &'a mut Terminal<B>,
    ) -> Result<CompletedFrame<'a>, B::Error> {
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
            for e in self.owner_records.iter() {
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
                .widths([
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::{contains, screen, terminal};

    fn entry(team: &str, owner: &str, wins: u64, losses: u64) -> core::StandingsEntry {
        core::StandingsEntry {
            roster_id: "1".into(),
            team_name: team.into(),
            owner_name: owner.into(),
            owner_id: "id".into(),
            wins,
            losses,
            ties: 0,
        }
    }

    #[test]
    fn renders_a_title_a_header_and_one_row_per_owner() {
        let mut page = LeagueHistoryOwnerRecordsPage {
            title_override: None,
            owner_records: vec![
                entry("Swift Falcons", "Sam Sample", 18, 10),
                entry("Iron Titans", "Alex Quote", 9, 19),
            ],
        };
        let mut terminal = terminal(80, 10);
        page.draw(&mut terminal).unwrap();

        let buffer = terminal.backend().buffer();
        assert!(contains(buffer, "League History Owner Records"));
        assert!(contains(buffer, "Team Name"), "{:#?}", screen(buffer));
        let row = screen(buffer)
            .into_iter()
            .find(|r| r.contains("Swift Falcons"))
            .expect("first owner is listed");
        assert!(
            row.contains("Sam Sample") && row.contains("18") && row.contains("10"),
            "{row}"
        );
        assert!(contains(buffer, "Iron Titans"));
    }

    #[test]
    fn a_title_override_replaces_the_default_title() {
        let mut page = LeagueHistoryOwnerRecordsPage::new();
        page.title_override = Some("  Loading...  ".into());
        let mut terminal = terminal(60, 6);
        page.draw(&mut terminal).unwrap();
        let buffer = terminal.backend().buffer();
        assert!(contains(buffer, "Loading..."));
        assert!(!contains(buffer, "League History Owner Records"));
    }
}
