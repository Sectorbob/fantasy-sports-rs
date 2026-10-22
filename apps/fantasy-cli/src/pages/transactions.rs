use crate::{constants, helpers};
use fantasy_sports_core::{self as core, PlayerMove};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Row, Table},
};
use std::{
    collections::{HashMap, HashSet},
    io::Stdout,
};

const NA: &str = "N/A";

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
    pub fn draw<'a>(&self, f: &mut ratatui::Frame<'a>, rect: &Rect) {
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
                let adds: Vec<Line> = player_moves
                    .iter()
                    .filter(PlayerMove::is_add)
                    .map(|p_move| self.player_cell(p_move))
                    .collect::<Vec<_>>();
                let drops = player_moves
                    .iter()
                    .filter(PlayerMove::is_drop)
                    .map(|p_move| self.player_cell(p_move))
                    .collect::<Vec<_>>();
                // let rosters = roster_ids
                //     .into_iter()
                //     // .filter_map(|s| s.as_str().parse::<u8>().ok())
                //     .filter_map(|roster_id| self.rosters.iter().find(|r| r.id == roster_id))
                //     .collect::<Vec<_>>();
                let roster_name = match roster_ids.len() {
                    0 => String::from(NA),
                    1 => roster_ids
                        .iter()
                        .next()
                        .map(|id| {
                            self.rosters
                                .iter()
                                .find(|r| &r.id == id)
                                .map_or(id.clone(), |r| r.team_name.clone())
                        })
                        .unwrap_or(String::from(NA)),
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
                    Cell::from(adds),
                    Cell::from(drops),
                ])
            })
            .collect::<Vec<Row>>();

        let table = Table::new(rows, Vec::<Constraint>::new())
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

    fn player_cell(&self, player_move: &fantasy_sports_core::PlayerMove) -> Line {
        let player_id = player_move.player_id();
        if let Some(player) = self.players.get(&player_id) {
            let mut spans = vec![Span::raw(format!("{} [", player.name))];
            spans.extend(
                player
                    .positions
                    .iter()
                    .map(|pos| helpers::position_span(pos)),
            );
            spans.push(Span::raw("]"));
            Line::from(spans)
        } else {
            Line::from(Span::raw(format!("Player({player_id})")))
        }
    }
}
