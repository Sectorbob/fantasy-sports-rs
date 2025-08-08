use std::io::Stdout;

use fantasy_core_rs::{DraftPick, League};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Cell, Row, Table},
};

pub struct DraftPicksWidget {
    title_override: Option<String>,
    _league_id: String,
    picks: Vec<(String, String, String)>,
    _roster_id_filter: Option<String>,
}
impl DraftPicksWidget {
    pub fn from_league(
        title_override: Option<String>,
        league: &League,
        roster_id_filter: Option<String>,
    ) -> Self {
        DraftPicksWidget {
            title_override: title_override,
            _league_id: league.id(),
            picks: match &league.draft() {
                Some(draft) => draft
                    .picks
                    .iter()
                    .filter(|p| match &roster_id_filter {
                        Some(rid) => rid == &p.roster_id,
                        None => true,
                    })
                    .map(format_entry)
                    .collect(),
                None => vec![],
            },
            _roster_id_filter: roster_id_filter,
        }
    }
    pub fn draw<'a>(&self, f: &mut tui::Frame<'a, CrosstermBackend<Stdout>>, rect: &Rect) {
        let title: String = if let Some(title_override) = self.title_override.clone() {
            title_override.trim().to_string()
        } else {
            String::from("Draft Picks")
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
            .picks
            .iter()
            .map(|(r, p, pl)| {
                Row::new(vec![
                    Cell::from(r.as_str()),
                    Cell::from(p.as_str()),
                    Cell::from(pl.as_str()),
                ])
            })
            .collect::<Vec<Row>>();

        let table = Table::new(rows)
            .widths(&[
                Constraint::Percentage(12),
                Constraint::Percentage(12),
                Constraint::Percentage(76),
            ])
            .header(Row::new(vec!["Round", "Pick", "Player"]));

        f.render_widget(table, area);
    }
}

fn format_entry(draft_pick: &DraftPick) -> (String, String, String) {
    (
        draft_pick.round.to_string(),
        draft_pick.overall_pick.to_string(),
        draft_pick.player.to_string(),
    )
}
