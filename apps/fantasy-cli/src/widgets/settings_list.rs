use fantasy_sports_core::League;
use ratatui::{
    layout::{Constraint, Rect},
    style::Style,
    widgets::{Block, BorderType, Borders, Cell, Row, Table},
};

use crate::constants;

pub struct SettingsListWidget {
    title_override: Option<String>,
    settings: Vec<(&'static str, String)>,
}
impl SettingsListWidget {
    pub fn from_league(title_override: Option<String>, league: &League) -> Self {
        SettingsListWidget {
            title_override,
            settings: match &league.settings().scoring {
                Some(scoring_settings) => scoring_settings.formatted_settings(),
                None => vec![],
            },
        }
    }
    pub fn draw<'a>(&self, f: &mut ratatui::Frame<'a>, rect: &Rect) {
        let title: String = if let Some(title_override) = self.title_override.clone() {
            title_override.trim().to_string()
        } else {
            String::from("Settings")
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
            .settings
            .iter()
            .map(|(k, v)| Row::new(vec![Cell::from(*k), Cell::from(v.as_str())]))
            .collect::<Vec<Row>>();

        let table = Table::new(rows, Vec::<Constraint>::new())
            // .block(Block::default().borders(Borders::ALL).title("Students"))
            .widths([Constraint::Percentage(60), Constraint::Percentage(40)]);

        f.render_widget(table, area);
    }
}
