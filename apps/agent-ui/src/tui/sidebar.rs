use super::{
    tasks::{TaskRow, group, label, variant},
    theme::*,
    view::{Screen, fit},
};
use crate::{report::State, task_result::TaskView};
use chrono::{DateTime, Local};
use ratatui::{
    prelude::*,
    widgets::{Block, Paragraph},
};
use std::collections::BTreeMap;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Status {
    Blocked,
    StartFailed,
    Failed,
    Starting,
    Preparing,
    Running,
    Verifying,
    Queued,
    Interrupted,
    Cancelled,
    Ready,
    NotRun,
}
impl Status {
    pub fn label(self) -> &'static str {
        match self {
            Self::Blocked => "Cleanup blocked",
            Self::StartFailed => "Start failed",
            Self::Failed => "Failed",
            Self::Starting => "Starting",
            Self::Preparing => "Preparing",
            Self::Running => "Running",
            Self::Verifying => "Verifying",
            Self::Queued => "Queued",
            Self::Interrupted => "Interrupted",
            Self::Cancelled => "Cancelled",
            Self::Ready => "Ready",
            Self::NotRun => "Not run",
        }
    }
    pub fn color(self) -> Color {
        match self {
            Self::Blocked | Self::StartFailed | Self::Failed => RED,
            Self::Ready => GREEN,
            Self::Interrupted | Self::Cancelled | Self::NotRun => MUTED,
            _ => GOLD,
        }
    }
}

pub(super) fn status(screen: &Screen<'_>, task: &TaskView) -> Status {
    if task.cleanup_pending {
        return Status::Blocked;
    }
    if screen.queued.contains(&task.id) {
        return Status::Queued;
    }
    if screen.queue_errors.contains_key(&task.id) {
        return Status::StartFailed;
    }
    if screen.starting.contains(&task.id) && task.run.as_ref().is_none_or(|r| !r.state.active()) {
        return Status::Starting;
    }
    match task.run.as_ref().map(|r| r.state) {
        None => Status::NotRun,
        Some(State::Preparing) => Status::Preparing,
        Some(State::Running) => Status::Running,
        Some(State::Verifying) => Status::Verifying,
        Some(State::Ready) => Status::Ready,
        Some(State::Failed) => Status::Failed,
        Some(State::Cancelled) => Status::Cancelled,
        Some(State::Interrupted) => Status::Interrupted,
    }
}

pub(super) fn summary(screen: &Screen<'_>, name: &str) -> (usize, String, Color) {
    let mut counts = BTreeMap::new();
    for task in screen.tasks.items.iter().filter(|t| group(&t.id) == name) {
        *counts.entry(status(screen, task)).or_insert(0usize) += 1;
    }
    let count = counts.values().sum();
    let color = counts.keys().next().map_or(MUTED, |s| s.color());
    let text = counts
        .iter()
        .map(|(status, n)| format!("{n} {}", status.label().to_lowercase()))
        .collect::<Vec<_>>()
        .join(" · ");
    (count, text, color)
}

pub(super) fn stamp(ms: u64) -> String {
    i64::try_from(ms)
        .ok()
        .and_then(DateTime::from_timestamp_millis)
        .map(|date| {
            date.with_timezone(&Local)
                .format("%d %b %y %H:%M")
                .to_string()
        })
        .unwrap_or_else(|| "Unknown date".into())
}

pub(super) fn latest_start(screen: &Screen<'_>, name: &str) -> Option<u64> {
    screen
        .tasks
        .items
        .iter()
        .filter(|t| group(&t.id) == name && !t.cleanup_pending)
        .filter_map(|t| t.run.as_ref().map(|r| r.created_at_ms))
        .max()
}

const META: Color = Color::Rgb(132, 149, 158);
const GUIDE: Color = Color::Rgb(83, 101, 109);

pub(super) fn draw_row(frame: &mut Frame, rect: Rect, screen: &Screen<'_>, row: &TaskRow) {
    match *row {
        TaskRow::Gap => {}
        TaskRow::Group { index } => {
            let name = group(&screen.tasks.items[index].id);
            let selected = screen.tasks.is_group() && screen.tasks.group() == Some(name);
            let parent = screen.tasks.group() == Some(name);
            let count = summary(screen, name).0.to_string();
            let bg = if selected { ACCENT } else { PANEL };
            frame.render_widget(Block::default().bg(bg), rect);
            let title = format!("{} {}", "▾", label(name).to_uppercase());
            frame.render_widget(
                Paragraph::new(fit(
                    &title,
                    rect.width.saturating_sub(count.len() as u16 + 2),
                ))
                .fg(if selected {
                    BG
                } else if parent {
                    ACCENT
                } else {
                    TEXT
                })
                .bold(),
                rect,
            );
            frame.render_widget(
                Paragraph::new(count)
                    .alignment(Alignment::Right)
                    .fg(if selected { BG } else { META }),
                Rect::new(rect.right().saturating_sub(4), rect.y, 3, 1),
            );
        }
        TaskRow::GroupInfo { index, line } => {
            if line == 2 {
                return;
            }
            let name = group(&screen.tasks.items[index].id);
            let (value, color) = if line == 0 {
                let (_, summary, color) = summary(screen, name);
                (summary, if color == RED { RED } else { MUTED })
            } else {
                (
                    latest_start(screen, name)
                        .map(|ms| format!("Last {}", stamp(ms)))
                        .unwrap_or_default(),
                    META,
                )
            };
            let area = Rect::new(rect.x + 2, rect.y, rect.width.saturating_sub(2), 1);
            frame.render_widget(Paragraph::new(fit(&value, area.width)).fg(color), area);
        }
        TaskRow::Task { index, line } => draw_task(frame, rect, screen, index, line),
    }
}

fn draw_task(frame: &mut Frame, rect: Rect, screen: &Screen<'_>, index: usize, line: u16) {
    let task = &screen.tasks.items[index];
    let selected =
        !screen.tasks.is_group() && screen.tasks.current().is_some_and(|t| t.id == task.id);
    let last = screen
        .tasks
        .visible()
        .into_iter()
        .rfind(|&i| group(&screen.tasks.items[i].id) == group(&task.id))
        == Some(index);
    if screen.tasks.is_grouped() {
        let guide = match (line, last) {
            (0, true) => "└─",
            (0, false) => "├─",
            (_, true) => "  ",
            _ => "│ ",
        };
        frame.render_widget(
            Paragraph::new(guide).fg(GUIDE),
            Rect::new(rect.x + 1, rect.y, 2, 1),
        );
    }
    if line == 2 {
        return;
    }
    let indent = if screen.tasks.is_grouped() { 4 } else { 2 };
    let area = Rect::new(
        rect.x + indent,
        rect.y,
        rect.width.saturating_sub(indent),
        1,
    );
    if selected {
        frame.render_widget(
            Block::default().bg(SELECTED),
            Rect::new(area.x - 1, rect.y, area.width + 1, 1),
        );
        frame.render_widget(
            Paragraph::new("▎").fg(ACCENT),
            Rect::new(area.x - 1, rect.y, 1, 1),
        );
    }
    let state = status(screen, task);
    if line == 0 {
        let value = match state {
            Status::Blocked => "Blocked",
            Status::StartFailed => "Start fail",
            _ => state.label(),
        };
        let status_width = value.len() as u16;
        let name_width = area.width.saturating_sub(status_width + 1);
        let side = screen
            .selection
            .pair
            .as_ref()
            .filter(|_| screen.selection.comparing)
            .map(|p| {
                if p.reference == task.id {
                    "A "
                } else if p.other == task.id {
                    "B "
                } else {
                    ""
                }
            })
            .unwrap_or("");
        let name = if screen.tasks.is_grouped() {
            label(variant(&task.id))
        } else {
            task.id.clone()
        };
        frame.render_widget(
            Paragraph::new(fit(&format!("{side}{name}"), name_width)).style(if selected {
                Style::default().fg(TEXT).bold()
            } else {
                Style::default().fg(TEXT)
            }),
            Rect::new(area.x, area.y, name_width, 1),
        );
        frame.render_widget(
            Paragraph::new(value)
                .fg(state.color())
                .alignment(Alignment::Right),
            Rect::new(
                area.right().saturating_sub(status_width),
                area.y,
                status_width,
                1,
            ),
        );
    } else {
        let text = task
            .run
            .as_ref()
            .filter(|_| !task.cleanup_pending)
            .map(|run| {
                if matches!(
                    state,
                    Status::Preparing | Status::Running | Status::Verifying
                ) {
                    let elapsed =
                        crate::report::now().saturating_sub(run.created_at_ms) as f64 / 1000.0;
                    format!(
                        "{} · {}",
                        stamp(run.created_at_ms),
                        super::text::duration(Some(elapsed))
                    )
                } else {
                    let prefix = if matches!(
                        state,
                        Status::Queued | Status::Starting | Status::StartFailed
                    ) {
                        "Prev"
                    } else {
                        "Run"
                    };
                    format!("{prefix} {}", stamp(run.created_at_ms))
                }
            })
            .unwrap_or_default();
        frame.render_widget(Paragraph::new(fit(&text, area.width)).fg(META), area);
    }
}
