use fantasy_sports_core as core;

use crate::constants;

pub fn color_from_position(pos: &core::PlayerPosition) -> ratatui::style::Color {
    match pos {
        core::PlayerPosition::QB => constants::COLOR_QB,
        core::PlayerPosition::RB => constants::COLOR_RB,
        core::PlayerPosition::WR => constants::COLOR_WR,
        core::PlayerPosition::TE => constants::COLOR_TE,
        core::PlayerPosition::K => constants::COLOR_K,
        // defense, including the individual defensive positions (LB, DE, ...)
        _ => constants::COLOR_DEF,
    }
}

pub fn position_span(pos: &core::PlayerPosition) -> ratatui::text::Span {
    ratatui::text::Span::styled(
        format!("{pos}"),
        ratatui::style::Style::default().fg(color_from_position(pos)),
    )
}
