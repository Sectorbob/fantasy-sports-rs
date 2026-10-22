use fantasy_sports_core as core;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Row, Table},
};
use std::io::Stdout;

use crate::{constants, helpers};

pub struct DraftPicksWidget {
    title_override: Option<String>,
    _league_id: String,
    picks: Vec<core::DraftPick>,
    _roster_id_filter: Option<String>,
}
impl DraftPicksWidget {
    pub fn from_league(
        title_override: Option<String>,
        league: &core::League,
        roster_id_filter: Option<String>,
    ) -> Self {
        DraftPicksWidget {
            title_override: title_override,
            _league_id: league.id().to_string(),
            picks: league.draft().map_or(vec![], |d| {
                d.picks
                    .into_iter()
                    .filter(|p| match &roster_id_filter {
                        Some(rid) => rid == &p.roster_id,
                        None => true,
                    })
                    .collect::<Vec<_>>()
            }),
            _roster_id_filter: roster_id_filter,
        }
    }
    pub fn draw<'a>(&self, f: &mut ratatui::Frame<'a>, rect: &Rect) {
        let title: String = if let Some(title_override) = self.title_override.clone() {
            title_override.trim().to_string()
        } else {
            String::from("Draft Picks")
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
            .picks
            .iter()
            .map(|pick| {
                let (r, p, pl) = format_entry(pick);
                Row::new(vec![Cell::from(r), Cell::from(p), Cell::from(pl)])
            })
            .collect::<Vec<Row>>();

        let table = Table::new(rows, Vec::<Constraint>::new())
            .widths(&[
                Constraint::Percentage(12),
                Constraint::Percentage(12),
                Constraint::Percentage(76),
            ])
            .header(Row::new(vec!["Round", "Pick", "Player"]));

        f.render_widget(table, area);
    }

    // fn player_cell(&self, player_move: &fantasy_sports_core::PlayerMove) -> Line {
    //     let player_id = player_move.player_id();
    //     if let Some(player) = self.players.get(&player_id) {
    //         let mut spans = vec![Span::raw(format!("{} [", player.name))];
    //         spans.extend(
    //             player
    //                 .positions
    //                 .iter()
    //                 .map(|pos| helpers::position_span(pos)),
    //         );
    //         spans.push(Span::raw("]"));
    //         Line::from(spans)
    //     } else {
    //         Line::from(Span::raw(format!("Player({player_id})")))
    //     }
    // }
}

fn format_entry<'d>(draft_pick: &'d core::DraftPick) -> (String, String, Line<'d>) {
    let mut player_spans: Vec<Span> = vec![Span::raw("[")];
    player_spans.extend(
        draft_pick
            .player
            .positions
            .iter()
            .map(|pos| helpers::position_span(&pos).clone()),
    );
    player_spans.push(Span::raw("] "));
    player_spans.push(Span::raw(draft_pick.player.name.clone()));
    if let Some(irl_team) = &draft_pick.player.irl_team {
        player_spans.push(Span::raw(" - "));
        player_spans.push(Span::raw(irl_team.clone()));
    }

    let draft_pick_round = draft_pick.round.to_string();
    let draft_pick_overall = draft_pick.overall_pick.to_string();

    (
        draft_pick_round,
        draft_pick_overall,
        Line::from(player_spans),
    )
}
