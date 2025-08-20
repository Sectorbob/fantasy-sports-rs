use std::{collections::HashSet, io::Stdout};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Cell, Row, Table},
};

use fantasy_core_rs::{self as core};

pub struct TransactionsPage {
    title_override: Option<String>,
    transactions: Vec<core::Transaction>,
    rosters: Vec<core::Roster>,
}
impl TransactionsPage {
    pub fn new() -> Self {
        TransactionsPage {
            title_override: None,
            transactions: vec![],
            rosters: vec![],
        }
    }
    pub fn from_league(league: &core::League) -> Self {
        TransactionsPage {
            title_override: None,
            transactions: league.transactions(),
            rosters: league.rosters(),
        }
    }
    pub fn draw<'a>(&self, f: &mut tui::Frame<'a, CrosstermBackend<Stdout>>, rect: &Rect) {
        let title: String = if let Some(title_override) = self.title_override.clone() {
            title_override.trim().to_string()
        } else {
            String::from("Transactions")
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White))
            .border_type(BorderType::Rounded)
            .style(Style::default().bg(Color::Black));
        f.render_widget(block.clone(), *rect);

        let area = block.inner(*rect);

        let rows = self
            .transactions
            .iter()
            .map(|txn| {
                let player_moves = txn.player_moves();
                let mut roster_ids = HashSet::new();
                roster_ids.extend(player_moves.iter().flat_map(|m| {
                    let mut r = vec![];
                    if let core::RosterSpot::Roster(s) = &m.src() {
                        r.push(s.clone());
                    }
                    if let core::RosterSpot::Roster(s) = &m.dest() {
                        r.push(s.clone());
                    }
                    r
                }));
                roster_ids.extend(
                    txn.draft_pick_moves()
                        .into_iter()
                        .flat_map(|m| vec![m.dest_team_id.clone(), m.src_team_id.clone()]),
                );
                let adds = player_moves.iter().filter(|v| v.add()).count();
                let drops = player_moves.iter().filter(|v| v.drop()).count();
                // let rosters = roster_ids
                //     .into_iter()
                //     // .filter_map(|s| s.as_str().parse::<u8>().ok())
                //     .filter_map(|roster_id| self.rosters.iter().find(|r| r.id == roster_id))
                //     .collect::<Vec<_>>();
                let roster_name = match roster_ids.len() {
                    0 => String::from("N/A"),
                    1 => roster_ids.iter().next().map(String::clone).unwrap(),
                    _ => roster_ids
                        .iter()
                        .map(String::clone)
                        .collect::<Vec<_>>()
                        .join(", "),
                    // 1 => rosters.iter().next().map(|r| r.id.clone()).unwrap(),
                    // _ => rosters
                    //     .iter()
                    //     .map(|r| r.id.clone())
                    //     .collect::<Vec<_>>()
                    //     .join(", "),
                    // _ => String::from("multiple")
                };
                // let players =

                Row::new(vec![
                    Cell::from(txn.ts().to_string()),
                    Cell::from("TBD"),
                    Cell::from(roster_name),
                    Cell::from(txn.r#type()),
                    Cell::from(adds.to_string()),
                    Cell::from(drops.to_string()),
                ])
            })
            .collect::<Vec<Row>>();

        let table = Table::new(rows)
            .widths(&[
                Constraint::Percentage(20),
                Constraint::Percentage(10),
                Constraint::Percentage(20),
                Constraint::Percentage(10),
                Constraint::Percentage(20),
                Constraint::Percentage(20),
            ])
            .header(Row::new(vec![
                "TS", "Week", "Roster", "Type", "Adds", "Drops",
            ]));

        f.render_widget(table, area);
    }
}
