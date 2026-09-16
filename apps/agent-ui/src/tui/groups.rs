use super::{
    tasks::{Tasks, label, variant},
    theme::*,
};
use ratatui::{prelude::*, widgets::*};
use std::collections::BTreeMap;

pub(super) fn lines(
    tasks: &Tasks,
    active: &[String],
    queued: &[String],
    errors: &BTreeMap<String, String>,
    width: u16,
) -> Vec<Line<'static>> {
    let members = tasks.members();
    let ready = members
        .iter()
        .filter(|t| {
            !active.contains(&t.id)
                && !queued.contains(&t.id)
                && !errors.contains_key(&t.id)
                && !t.cleanup_pending
                && t.run
                    .as_ref()
                    .is_some_and(|r| r.state == crate::report::State::Ready)
        })
        .count();
    let mut lines = vec![
        Line::from(format!("{} tasks · {ready} ready", members.len())).bold(),
        Line::default(),
    ];
    let name_width = usize::from(width.saturating_sub(36));
    if width >= 64 {
        lines.push(
            Line::from(format!(
                "{:<name_width$}{:<16}{:>9}{:>11}",
                "Task", "Status", "Agent", "Est. USD"
            ))
            .fg(MUTED),
        );
        lines.push(Line::default());
    }
    for task in members {
        let status = if queued.contains(&task.id) {
            "Queued"
        } else if active.contains(&task.id) {
            "Running"
        } else if errors.contains_key(&task.id) {
            "Start failed"
        } else {
            task.status()
        };
        if width >= 64 {
            let run = task.run.as_ref().filter(|_| {
                !active.contains(&task.id)
                    && !queued.contains(&task.id)
                    && !errors.contains_key(&task.id)
            });
            let name = super::view::fit(
                &label(variant(&task.id)),
                name_width.saturating_sub(1) as u16,
            );
            let time = super::text::duration(run.and_then(|r| r.agent_seconds));
            let cost = crate::cost::format_usd(run.and_then(|r| r.cost_usd));
            lines.push(Line::from(format!(
                "{name:<name_width$}{status:<16}{time:>9}{cost:>11}"
            )));
        } else {
            lines.push(Line::from(format!("{} · {status}", label(variant(&task.id)))).bold());
        }
        if let Some(error) = errors.get(&task.id) {
            lines.push(Line::from(error.clone()).fg(GOLD));
        } else if active.contains(&task.id) || queued.contains(&task.id) {
            lines.push(Line::from("Waiting for this run's result.").fg(MUTED));
        } else if let Some(error) = task.run.as_ref().and_then(|run| run.error.as_ref()) {
            lines.push(Line::from(error.clone()).fg(GOLD));
        } else if width >= 64 {
            continue;
        } else if let Some(run) = &task.run {
            let duration = run
                .agent_seconds
                .map(|v| format!("{v:.1}s"))
                .unwrap_or_else(|| "—".into());
            lines.push(
                Line::from(format!(
                    "Agent: {duration} · API cost (est): {}",
                    crate::cost::format_usd(run.cost_usd)
                ))
                .fg(MUTED),
            );
        } else {
            lines.push(Line::from("No saved output").fg(MUTED));
        }
        lines.push(Line::default());
    }
    lines
}

pub(super) fn overview(frame: &mut Frame, area: Rect, screen: &super::view::Screen<'_>) {
    let parts = super::layout::detail_parts(area);
    let actions = super::layout::group_actions(parts[2]);
    super::view::button(frame, actions[0], "n Run all tasks", true);
    if screen
        .tasks
        .members()
        .iter()
        .any(|t| screen.starting.contains(&t.id) || screen.queued.contains(&t.id))
    {
        super::view::button(frame, actions[1], "C Cancel group", false);
    }
    let paragraph = Paragraph::new(lines(
        screen.tasks,
        &screen.starting,
        &screen.queued,
        screen.queue_errors,
        parts[3].width,
    ))
    .wrap(Wrap { trim: false });
    let last = paragraph
        .line_count(parts[3].width)
        .saturating_sub(parts[3].height as usize)
        .min(u16::MAX as usize) as u16;
    let offset = screen.scroll.unwrap_or(0).min(last);
    frame.render_widget(paragraph.scroll((offset, 0)), parts[3]);
    if last > 0 && parts[3].height > 0 {
        frame.render_widget(
            Paragraph::new(if offset < last { "↓" } else { "↑" }).fg(MUTED),
            Rect::new(parts[3].right() - 1, parts[3].bottom() - 1, 1, 1),
        );
    }
}
