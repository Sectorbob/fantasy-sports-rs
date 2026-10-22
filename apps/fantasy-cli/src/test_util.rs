//! Helpers for rendering tests: draw into an in-memory terminal and read back the text.
use ratatui::{Terminal, backend::TestBackend, buffer::Buffer};

pub fn terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    Terminal::new(TestBackend::new(width, height)).expect("test backend never fails")
}

/// The buffer as lines of text, one per row, trailing spaces trimmed.
pub fn screen(buffer: &Buffer) -> Vec<String> {
    (0..buffer.area.height)
        .map(|y| {
            (0..buffer.area.width)
                .map(|x| buffer[(x, y)].symbol().to_string())
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect()
}

/// Whether any row of the screen contains `needle`.
pub fn contains(buffer: &Buffer, needle: &str) -> bool {
    screen(buffer).iter().any(|row| row.contains(needle))
}
