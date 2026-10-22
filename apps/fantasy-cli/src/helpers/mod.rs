use fantasy_core_rs as core;

use crate::constants;

pub fn color_from_position(pos: &core::PlayerPosition) -> tui::style::Color {
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

pub fn position_span(pos: &core::PlayerPosition) -> tui::text::Span {
    tui::text::Span::styled(
        format!("{pos}"),
        tui::style::Style::default().fg(color_from_position(pos)),
    )
}
