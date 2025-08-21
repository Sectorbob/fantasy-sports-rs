use std::io::Stdout;

use fantasy_core_rs::League;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Cell, Row, Table},
};

pub struct SettingsListWidget {
    title_override: Option<String>,
    settings: Vec<(String, String)>,
}
impl SettingsListWidget {
    pub fn from_league(title_override: Option<String>, league: &League) -> Self {
        SettingsListWidget {
            title_override: title_override,
            settings: match &league.settings().scoring {
                Some(scoring_settings) => scoring_settings.formatted_settings(),
                None => vec![],
            },
        }
    }
    pub fn draw<'a>(&self, f: &mut tui::Frame<'a, CrosstermBackend<Stdout>>, rect: &Rect) {
        let title: String = if let Some(title_override) = self.title_override.clone() {
            title_override.trim().to_string()
        } else {
            String::from("Settings")
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
            .settings
            .iter()
            .map(|(k, v)| Row::new(vec![Cell::from(k.as_str()), Cell::from(v.as_str())]))
            .collect::<Vec<Row>>();

        let table = Table::new(rows)
            // .block(Block::default().borders(Borders::ALL).title("Students"))
            .widths(&[Constraint::Percentage(60), Constraint::Percentage(40)]);

        f.render_widget(table, area);
    }
}
