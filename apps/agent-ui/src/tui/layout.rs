use ratatui::layout::{Constraint, Layout, Margin, Rect};

pub(super) fn regions(area: Rect) -> ([Rect; 4], [Rect; 2]) {
    let outer = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(16),
        Constraint::Length(1),
        Constraint::Length(2),
    ])
    .margin(1)
    .areas(area);
    let panels = Layout::horizontal([
        Constraint::Length((area.width / 4).clamp(28, 38)),
        Constraint::Min(40),
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
pub(super) fn search_area(area: Rect) -> Rect {
    Rect::new(area.x + 1, area.y + 1, area.width.saturating_sub(3), 1)
}
pub(super) fn task_list_area(area: Rect) -> Rect {
    Rect::new(
        area.x + 1,
        area.y + 3,
        area.width.saturating_sub(3),
        area.height.saturating_sub(3),
    )
}
pub(super) fn detail_parts(area: Rect) -> [Rect; 4] {
    let area = area.inner(Margin::new(1, 0));
    [
        Rect::new(area.x, area.y, area.width, 1),
        Rect::new(area.x, area.y + 2, area.width, 1),
        Rect::new(area.x, area.y + 4, area.width, 1),
        Rect::new(
            area.x,
            area.y + 6,
            area.width,
            area.height.saturating_sub(6),
        ),
    ]
}
pub(super) fn picker_list(area: Rect) -> Rect {
    Rect::new(
        area.x + 4,
        area.y + 5,
        area.width.saturating_sub(8),
        area.height.saturating_sub(8),
    )
}
pub(super) fn action_areas(area: Rect) -> [Rect; 4] {
    Layout::horizontal([
        Constraint::Length(9),
        Constraint::Min(12),
        Constraint::Length(8),
        Constraint::Length(6),
    ])
    .spacing(1)
    .areas(area)
}
pub(super) fn modal_field(area: Rect, index: usize) -> Rect {
    Rect::new(
        area.x + 16,
        area.y + 4 + index as u16 * 2,
        area.width.saturating_sub(21),
        1,
    )
}
pub(super) fn modal_submit(area: Rect) -> Rect {
    Rect::new(area.x + 4, area.y + 16, 24, 1)
}

pub(super) fn group_actions(area: Rect) -> [Rect; 2] {
    Layout::horizontal([Constraint::Percentage(50); 2])
        .spacing(1)
        .areas(area)
}
pub(super) fn content_area(area: Rect, _group: bool, _comparing: bool) -> Rect {
    detail_parts(area)[3]
}
