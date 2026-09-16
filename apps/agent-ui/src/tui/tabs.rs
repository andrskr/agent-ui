use super::theme::*;
use crossterm::event::KeyCode;
use ratatui::{prelude::*, widgets::Paragraph};

pub(super) fn labels(group: bool) -> &'static [&'static str] {
    if group {
        &["Overview", "Compare"]
    } else {
        &["Overview", "Activity", "Setup"]
    }
}

pub(super) fn key_index(group: bool, current: usize, key: KeyCode) -> Option<usize> {
    let count = labels(group).len();
    match key {
        KeyCode::Right | KeyCode::Tab => Some((current + 1) % count),
        KeyCode::Left | KeyCode::BackTab => Some((current + count - 1) % count),
        KeyCode::Char(c @ '1'..='3') => {
            let index = c as usize - '1' as usize;
            (index < count).then_some(index)
        }
        _ => None,
    }
}

pub(super) fn areas(row: Rect, group: bool) -> Vec<Rect> {
    labels(group)
        .iter()
        .enumerate()
        .map(|(index, _)| Rect::new(row.x + index as u16 * 12, row.y, 11, 1))
        .collect()
}

pub(super) fn mouse_index(row: Rect, group: bool, position: Position) -> Option<usize> {
    areas(row, group)
        .iter()
        .position(|area| area.contains(position))
}

pub(super) fn draw(frame: &mut Frame, row: Rect, group: bool, current: usize) {
    for (index, (label, area)) in labels(group).iter().zip(areas(row, group)).enumerate() {
        frame.render_widget(
            Paragraph::new(*label)
                .alignment(Alignment::Center)
                .style(if index == current {
                    Style::default().bg(ACCENT).fg(BG).bold()
                } else {
                    Style::default().fg(MUTED)
                }),
            area,
        );
    }
}
