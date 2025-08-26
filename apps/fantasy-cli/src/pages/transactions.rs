use fantasy_core_rs::{self as core};
use std::{
    collections::{HashMap, HashSet},
    io::Stdout,
};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Rect},
    style::Style,
    widgets::{Block, BorderType, Borders, Cell, Row, Table},
};

use crate::constants;

pub struct TransactionsPage {
    title_override: Option<String>,
    transactions: Vec<core::Transaction>,
    rosters: Vec<core::Roster>,
    players: HashMap<String, core::Player>,
}
impl TransactionsPage {
    pub fn new() -> Self {
        TransactionsPage {
            title_override: None,
            transactions: vec![],
            rosters: vec![],
            players: HashMap::new(),
        }
    }
    pub fn from_league(league: &core::League) -> Self {
        TransactionsPage {
            title_override: None,
            transactions: league.transactions(),
            rosters: league.rosters(),
            players: league.players().clone(), //TODO: i don't want to clone these
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
            .border_style(Style::default().fg(constants::COLOR_BORDER))
            .border_type(BorderType::Rounded)
            .style(Style::default().bg(constants::COLOR_BACKGROUND));
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
                let adds = player_moves
                    .iter()
                    .filter(|v| v.add())
                    .map(|p_move| {
                        let player_id = p_move.player_id();
                        let player = self.players.get(&player_id);
                        match player {
                            Some(p) => format!(
                                "{name} [{pos}]",
                                name = p.name,
                                pos = p
                                    .positions
                                    .iter()
                                    .map(String::from)
                                    .collect::<Vec<_>>()
                                    .join(",")
                                    .trim()
                                    .to_string()
                            ),
                            None => format!("Player({player_id})"),
                        }
                    })
                    .collect::<Vec<_>>();
                let drops = player_moves
                    .iter()
                    .filter(|v| v.drop())
                    .map(|p_move| {
                        let player_id = p_move.player_id();
                        let player = self.players.get(&player_id);
                        match player {
                            Some(p) => format!(
                                "{name} [{pos}]",
                                name = p.name,
                                pos = p
                                    .positions
                                    .iter()
                                    .map(String::from)
                                    .collect::<Vec<_>>()
                                    .join(",")
                                    .trim()
                                    .to_string()
                            ),
                            None => format!("Player({player_id})"),
                        }
                    })
                    .collect::<Vec<_>>();
                // let rosters = roster_ids
                //     .into_iter()
                //     // .filter_map(|s| s.as_str().parse::<u8>().ok())
                //     .filter_map(|roster_id| self.rosters.iter().find(|r| r.id == roster_id))
                //     .collect::<Vec<_>>();
                let roster_name = match roster_ids.len() {
                    0 => String::from("N/A"),
                    1 => roster_ids
                        .iter()
                        .next()
                        .map(|id| {
                            self.rosters
                                .iter()
                                .find(|r| &r.id == id)
                                .map_or(id.clone(), |r| r.team_name.clone())
                        })
                        .unwrap_or("N/A".to_string()),
                    _ => roster_ids
                        .iter()
                        .map(String::clone)
                        .collect::<Vec<_>>()
                        .join(", "),
                };

                Row::new(vec![
                    Cell::from(txn.ts().with_timezone(&constants::TZ).to_string()),
                    Cell::from("TBD"),
                    Cell::from(roster_name),
                    Cell::from(txn.r#type()),
                    Cell::from(
                        adds.iter()
                            .map(String::from)
                            .collect::<Vec<_>>()
                            .join("\n")
                            .trim()
                            .to_string(),
                    ),
                    Cell::from(
                        drops
                            .iter()
                            .map(String::from)
                            .collect::<Vec<_>>()
                            .join("\n")
                            .trim()
                            .to_string(),
                    ),
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
