use super::{
    tasks::{label, variant},
    text::{duration, number, safe_text},
    theme::*,
};
use crate::comparison::Comparison;
use crossterm::event::KeyCode;
use ratatui::{prelude::*, widgets::*};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Action {
    Swap,
    SelectA,
    SelectB,
    Back,
    EnterTask,
    Navigate(isize),
    Scroll(i32),
    Home,
    End,
    Search,
    Help,
    Quit,
}

pub(super) fn key_action(key: KeyCode) -> Option<Action> {
    Some(match key {
        KeyCode::Char('s') => Action::Swap,
        KeyCode::Char('a') => Action::SelectA,
        KeyCode::Char('b') => Action::SelectB,
        KeyCode::Esc => Action::Back,
        KeyCode::Enter => Action::EnterTask,
        KeyCode::Up | KeyCode::Char('k') => Action::Navigate(-1),
        KeyCode::Down | KeyCode::Char('j') => Action::Navigate(1),
        KeyCode::PageUp => Action::Scroll(-10),
        KeyCode::PageDown => Action::Scroll(10),
        KeyCode::Home => Action::Home,
        KeyCode::End => Action::End,
        KeyCode::Char('/') => Action::Search,
        KeyCode::Char('?') => Action::Help,
        KeyCode::Char('q') => Action::Quit,
        _ => return None,
    })
}

pub(super) fn actions(area: Rect) -> [Rect; 3] {
    Layout::horizontal([
        Constraint::Ratio(1, 2),
        Constraint::Ratio(1, 2),
        Constraint::Length(8),
    ])
    .spacing(1)
    .areas(area)
}

pub(super) fn mouse_action(position: Position, area: Rect) -> Option<Action> {
    actions(super::layout::detail_parts(area)[2])
        .iter()
        .position(|r| r.contains(position))
        .map(|i| [Action::SelectA, Action::SelectB, Action::Swap][i])
}

pub(super) fn lines(comparison: Option<&Comparison>, width: u16) -> Vec<Line<'static>> {
    match comparison {
        Some(comparison) => measured_lines(comparison, width),
        None => vec![
            Line::from("No pair to compare").bold(),
            Line::default(),
            Line::from("This group needs two saved task results.").fg(MUTED),
            Line::from("Use Overview to run the group, or Enter to run one task.").fg(MUTED),
        ],
    }
}

pub(super) fn draw(frame: &mut Frame, area: Rect, screen: &super::view::Screen<'_>) {
    let parts = super::layout::detail_parts(area);
    if !screen.tasks.is_group() {
        frame.render_widget(
            Paragraph::new(label(screen.tasks.group().unwrap_or_default())).bold(),
            parts[0],
        );
        super::tabs::draw(frame, parts[1], true, 1);
    }
    let buttons = actions(parts[2]);
    for (i, key, side, id) in [
        (
            0,
            "a",
            "A",
            screen.selection.pair.as_ref().map(|p| p.reference.as_str()),
        ),
        (
            1,
            "b",
            "B",
            screen.selection.pair.as_ref().map(|p| p.other.as_str()),
        ),
    ] {
        let value = format!(
            "{key} {side}: {}",
            id.map(|id| label(variant(id)))
                .unwrap_or_else(|| "Select".into())
        );
        super::view::button(
            frame,
            buttons[i],
            &super::view::fit(&value, buttons[i].width),
            false,
        );
    }
    if screen.selection.pair.is_some() {
        super::view::button(frame, buttons[2], "s Swap", false);
    }
    let paragraph =
        Paragraph::new(lines(screen.comparison, parts[3].width)).wrap(Wrap { trim: false });
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

fn measured_lines(c: &Comparison, width: u16) -> Vec<Line<'static>> {
    let a = &c.reference;
    let b = &c.other;
    let m = &c.measurements;
    let mut lines = vec![Line::from("Measured results").bold(), Line::default()];
    let mut rows = Vec::new();
    rows.push((
        "API cost (USD)",
        crate::cost::format_usd(m.estimated_cost_usd.reference),
        crate::cost::format_usd(m.estimated_cost_usd.other),
        crate::cost::format_difference(m.estimated_cost_usd.difference),
    ));
    for (label, times) in [
        ("Setup", &m.setup_seconds),
        ("Agent", &m.agent_seconds),
        ("Verify time", &m.verification_seconds),
    ] {
        rows.push((
            label,
            duration(times.reference),
            duration(times.other),
            times
                .difference
                .map(|d| format!("{d:+.1}s"))
                .unwrap_or_else(|| "—".into()),
        ));
    }
    for (label, tokens) in [
        ("Input", &m.input_tokens),
        ("Cached", &m.cached_input_tokens),
        ("Output", &m.output_tokens),
    ] {
        rows.push((
            label,
            number(tokens.reference),
            number(tokens.other),
            tokens
                .difference
                .map(|d| format!("{d:+}"))
                .unwrap_or_else(|| "—".into()),
        ));
    }
    if width >= 68 {
        let col = usize::from((width - 16) / 3);
        lines.push(
            Line::from(format!(
                "{:<16}{:<col$}{:<col$}B − A",
                "", "A · Reference", "B"
            ))
            .fg(ACCENT),
        );
        for (label, a, b, difference) in rows {
            lines.push(Line::from(format!(
                "{label:<16}{a:<col$}{b:<col$}{difference}"
            )));
        }
    } else {
        lines.push(Line::from("A → B  (difference)").fg(ACCENT));
        for (label, a, b, difference) in rows {
            lines.push(Line::from(format!("{label}: {a} → {b}  ({difference})")));
        }
    }
    lines.push(Line::default());
    lines.push(Line::from("USD estimates · cached tokens are part of input.").fg(MUTED));
    if a.cost_usd.is_none() || b.cost_usd.is_none() {
        for (side, run) in [("A", a), ("B", b)] {
            if run.cost_usd.is_none() && !run.cost_note.is_empty() {
                lines.push(Line::from(format!("{side}: {}", safe_text(&run.cost_note))).fg(MUTED));
            }
        }
    }
    lines.push(Line::default());
    lines.push(Line::from("Configuration and outcome").bold());
    for (label, run) in [("A", a), ("B", b)] {
        lines.push(Line::from(format!(
            "{label}  {} · {}",
            run.state.label(),
            match run.verification.as_ref() {
                Some(check) if check.exit_code == Some(0) => "Verify passed",
                Some(_) => "Verify failed",
                None => "Verify not complete",
            }
        )));
        if let Some(config) = &run.task_config
            && !config.setup.profiles.is_empty()
        {
            lines.push(
                Line::from(format!(
                    "{label} profiles: {}",
                    config.setup.profiles.join(", ")
                ))
                .fg(MUTED),
            );
        }
        if let Some(error) = &run.error {
            lines.push(Line::from(safe_text(error)).fg(RED));
        }
    }
    for (name, left, right) in [
        (
            "Provider",
            Some(a.provider.as_str()),
            Some(b.provider.as_str()),
        ),
        (
            "Model",
            Some(a.model_requested.as_str()),
            Some(b.model_requested.as_str()),
        ),
        (
            "Effort",
            Some(a.effort_requested.as_str()),
            Some(b.effort_requested.as_str()),
        ),
    ] {
        let short = |value: Option<&str>| {
            value
                .and_then(|v| v.lines().next())
                .unwrap_or("Not reported")
                .to_owned()
        };
        let value = if left == right {
            format!("{name}: {} · both", short(left))
        } else {
            format!("{name}: A {} → B {}", short(left), short(right))
        };
        lines.push(Line::from(value).fg(if left == right { MUTED } else { ACCENT }));
    }
    let timeout = |seconds| format!("{seconds}s");
    lines.push(Line::from(format!(
        "Time limit: A {} · B {}",
        timeout(a.timeout_seconds),
        timeout(b.timeout_seconds)
    )));
    let packages = |run: &crate::report::Report| match &run.task_config {
        None => "Not recorded".to_owned(),
        Some(config) if !config.has_packages() => "Starter defaults".to_owned(),
        Some(config) => config
            .dependencies
            .iter()
            .map(|(n, v)| format!("{n}@{v}"))
            .chain(
                config
                    .dev_dependencies
                    .iter()
                    .map(|(n, v)| format!("{n}@{v} (dev)")),
            )
            .chain(config.allow_builds.iter().map(|(n, allowed)| {
                format!(
                    "{n} scripts: {}",
                    if *allowed { "allowed" } else { "blocked" }
                )
            }))
            .collect::<Vec<_>>()
            .join(", "),
    };
    let left = packages(a);
    let right = packages(b);
    if left == right {
        lines.push(Line::from(format!("Packages: {left} · both")).fg(MUTED));
    } else {
        lines.push(Line::from(format!("Packages A: {left}")).fg(ACCENT));
        lines.push(Line::from(format!("Packages B: {right}")).fg(ACCENT));
    }
    lines
}
