pub const APP_NAME: &str = "fantasy-cli";

pub const SEASONS_TO_SCAN: [&str; 11] = [
    "2015", "2016", "2017", "2018", "2019", "2020", "2021", "2022", "2023", "2024", "2025",
];

pub const COLOR_BACKGROUND: tui::style::Color = tui::style::Color::Black;
pub const COLOR_BORDER: tui::style::Color = tui::style::Color::Reset;
pub const COLOR_HIGHLIGHT: tui::style::Color = tui::style::Color::LightGreen;
pub const TZ: chrono::Local = chrono::Local;

pub const COLOR_RB: tui::style::Color = tui::style::Color::Rgb(114, 195, 166); // #72C3A6
pub const COLOR_TE: tui::style::Color = tui::style::Color::Rgb(203, 140, 74); // #CB8C4A
pub const COLOR_WR: tui::style::Color = tui::style::Color::Rgb(69, 162, 202); // #45A2CA
pub const COLOR_QB: tui::style::Color = tui::style::Color::Rgb(191, 94, 133); // #BF5E85
pub const COLOR_DEF: tui::style::Color = tui::style::Color::Rgb(107, 59, 45); // #6B3B2D
pub const COLOR_K: tui::style::Color = tui::style::Color::Rgb(146, 149, 206); // #9295D0
