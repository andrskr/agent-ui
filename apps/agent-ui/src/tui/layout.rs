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
        Constraint::Length((area.width / 3).clamp(30, 44)),
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
pub(super) fn new_button(area: Rect) -> Rect {
    Rect::new(area.right().saturating_sub(19), area.y + 1, 17, 1)
}
pub(super) fn search_area(area: Rect) -> Rect {
    Rect::new(area.x + 1, area.y + 1, area.width.saturating_sub(3), 1)
}
pub(super) fn run_list_area(area: Rect) -> Rect {
    Rect::new(
        area.x + 1,
        area.y + 3,
        area.width.saturating_sub(3),
        area.height.saturating_sub(3),
    )
}
pub(super) fn detail_parts(area: Rect) -> [Rect; 3] {
    let area = area.inner(Margin::new(2, 0));
    let wide = area.width >= 84;
    let top = if wide { 3 } else { 5 };
    [
        Rect::new(area.x, area.y, 36.min(area.width), 1),
        Rect::new(
            if wide { area.right() - 44 } else { area.x },
            area.y + if wide { 0 } else { 2 },
            if wide { 44 } else { area.width },
            1,
        ),
        Rect::new(
            area.x,
            area.y + top,
            area.width,
            area.height.saturating_sub(top),
        ),
    ]
}
pub(super) fn action_areas(area: Rect) -> [Rect; 3] {
    Layout::horizontal([
        Constraint::Min(14),
        Constraint::Length(14),
        Constraint::Length(8),
    ])
    .spacing(1)
    .areas(area)
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
pub(super) fn content_area(area: Rect) -> Rect {
    detail_parts(regions(area).1[1])[2]
}
