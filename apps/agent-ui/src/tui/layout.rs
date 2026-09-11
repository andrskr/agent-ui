use ratatui::layout::{Constraint, Layout, Rect};

pub(super) fn regions(area: Rect) -> ([Rect; 4], [Rect; 2]) {
    let outer: [Rect; 4] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(16),
        Constraint::Length(2),
        Constraint::Length(2),
    ])
    .margin(1)
    .areas(area);
    let panels = Layout::horizontal([
        Constraint::Length(if area.width >= 115 { 32 } else { 26 }),
        Constraint::Min(42),
    ])
    .spacing(1)
    .areas(outer[1]);
    (outer, panels)
}
pub(super) fn modal_rect(area: Rect) -> Rect {
    Rect::new(
        area.x + area.width.saturating_sub(76) / 2,
        area.y + area.height.saturating_sub(20) / 2,
        76.min(area.width),
        20.min(area.height),
    )
}
pub(super) fn new_button(area: Rect) -> Rect {
    Rect::new(area.width.saturating_sub(18), 1, 16, 1)
}
pub(super) fn run_list_area(area: Rect) -> Rect {
    Rect::new(
        area.x + 1,
        area.y + 2,
        area.width.saturating_sub(3),
        area.height.saturating_sub(2),
    )
}
pub(super) fn detail_parts(area: Rect) -> [Rect; 3] {
    Layout::vertical([
        Constraint::Length(4),
        Constraint::Length(3),
        Constraint::Min(5),
    ])
    .areas(area)
}
pub(super) fn action_areas(area: Rect) -> [Rect; 4] {
    let row = Rect::new(area.x, area.bottom().saturating_sub(1), area.width, 1);
    Layout::horizontal([
        Constraint::Percentage(26),
        Constraint::Percentage(27),
        Constraint::Percentage(23),
        Constraint::Percentage(24),
    ])
    .spacing(1)
    .areas(row)
}
pub(super) fn modal_field(area: Rect, index: usize) -> Rect {
    Rect::new(
        area.x + 16,
        area.y + 4 + index as u16 * 3,
        area.width.saturating_sub(21),
        1,
    )
}
pub(super) fn modal_submit(area: Rect) -> Rect {
    Rect::new(area.x + 4, area.y + 16, 24, 1)
}
pub(super) fn visible_lines(area: Rect) -> u16 {
    let (_, panels) = regions(area);
    detail_parts(panels[1])[2].height.saturating_sub(2)
}
