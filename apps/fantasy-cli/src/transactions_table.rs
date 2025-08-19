use std::io::Stdout;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Cell, Row, Table},
};

use fantasy_core_rs as core;

pub struct TransactionsPage {
    title_override: Option<String>,
    transactions: Vec<core::Transaction>,
}
impl TransactionsPage {
    pub fn new() -> Self {
        TransactionsPage {
            title_override: None,
            transactions: vec![],
        }
    }
    pub fn from_league(league: &core::League) -> Self {
        TransactionsPage {
            title_override: None,
            transactions: league.transactions(),
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
                Row::new(vec![
                    Cell::from(txn.updated.to_string()),
                    Cell::from("TBD"),
                    Cell::from("TBD"),
                    Cell::from(format!(
                        "Type:{} Status: {}",
                        txn.transaction_type, txn.status
                    )),
                ])
            })
            .collect::<Vec<Row>>();

        let table = Table::new(rows)
            .widths(&[
                Constraint::Percentage(20),
                Constraint::Percentage(10),
                Constraint::Percentage(20),
                Constraint::Percentage(50),
            ])
            .header(Row::new(vec!["TS", "Week", "Roster", "Description"]));

        f.render_widget(table, area);
    }
}
