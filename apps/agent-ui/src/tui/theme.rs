use crate::report::State;
use ratatui::style::Color;

pub(super) const BG: Color = Color::Rgb(24, 29, 33);
pub(super) const PANEL: Color = Color::Rgb(30, 36, 41);
pub(super) const SELECTED: Color = Color::Rgb(39, 50, 54);
pub(super) const BORDER: Color = Color::Rgb(60, 72, 79);
pub(super) const TEXT: Color = Color::Rgb(231, 236, 239);
pub(super) const MUTED: Color = Color::Rgb(166, 179, 191);
pub(super) const ACCENT: Color = Color::Rgb(156, 232, 207);
pub(super) const GREEN: Color = Color::Rgb(143, 221, 167);
pub(super) const GOLD: Color = Color::Rgb(230, 194, 139);
pub(super) const RED: Color = Color::Rgb(241, 139, 148);

pub(super) fn state_color(state: State) -> Color {
    match state {
        State::Ready => GREEN,
        State::Failed => RED,
        State::Cancelled | State::Interrupted => MUTED,
        _ => GOLD,
    }
}
pub(super) fn state_word(state: State) -> &'static str {
    match state {
        State::Preparing => "Preparing",
        State::Running => "Running",
        State::Verifying => "Verifying",
        State::Ready => "Ready",
        State::Failed => "Failed",
        State::Cancelled => "Cancelled",
        State::Interrupted => "Interrupted",
    }
}
pub(super) fn spinner_frame() -> char {
    const FRAMES: [char; 10] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    FRAMES[((crate::report::now() / 80) % 10) as usize]
}
