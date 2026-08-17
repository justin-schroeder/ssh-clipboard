mod monitor;
mod setup;

pub use monitor::run_monitor;
pub use setup::run_setup;

use ratatui::style::Color;

pub const ACCENT: Color = Color::Rgb(167, 139, 250);
pub const CYAN: Color = Color::Rgb(34, 211, 238);
pub const GREEN: Color = Color::Rgb(52, 211, 153);
pub const RED: Color = Color::Rgb(251, 113, 133);
pub const YELLOW: Color = Color::Rgb(251, 191, 36);
pub const MUTED: Color = Color::Rgb(100, 116, 139);
pub const SOFT: Color = Color::Rgb(203, 213, 225);
pub const PANEL: Color = Color::Rgb(51, 65, 85);

fn clean_truncate(value: &str, width: usize) -> String {
    let clean = value
        .chars()
        .filter_map(|character| match character {
            '\n' | '\r' | '\t' => Some(' '),
            character if character.is_control() => None,
            character => Some(character),
        })
        .collect::<String>();
    if clean.chars().count() <= width {
        return clean;
    }
    let mut result = clean.chars().take(width.saturating_sub(1)).collect::<String>();
    result.push('…');
    result
}
