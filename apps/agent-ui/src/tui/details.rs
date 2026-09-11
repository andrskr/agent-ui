use super::{state::DetailTab, theme::*};
use crate::report::{Report, State, now};
use ratatui::prelude::*;

fn number(n: Option<u64>) -> String {
    let Some(n) = n else {
        return "Not reported".into();
    };
    let raw = n.to_string();
    raw.chars()
        .enumerate()
        .fold(String::new(), |mut s, (i, c)| {
            if i > 0 && (raw.len() - i).is_multiple_of(3) {
                s.push(',');
            }
            s.push(c);
            s
        })
}
fn duration(seconds: Option<f64>) -> String {
    seconds
        .map(|s| {
            if s < 60.0 {
                format!("{s:.1}s")
            } else {
                format!("{}m {:02}s", s as u64 / 60, s as u64 % 60)
            }
        })
        .unwrap_or_else(|| "—".into())
}
fn pair(label: &str, value: &str, width: u16) -> Line<'static> {
    let gap = usize::from(width)
        .saturating_sub(label.len() + value.len())
        .max(1);
    Line::from(vec![
        Span::styled(label.to_owned(), Style::default().fg(MUTED)),
        Span::raw(" ".repeat(gap)),
        Span::styled(value.to_owned(), Style::default().fg(TEXT)),
    ])
}
pub(super) fn content_lines(
    run: &Report,
    tab: DetailTab,
    note: &str,
    width: u16,
) -> Vec<Line<'static>> {
    match tab {
        DetailTab::Overview => overview_lines(run, note, width),
        DetailTab::Activity => {
            let mut lines: Vec<_> = run
                .activity
                .iter()
                .map(|a| {
                    Line::from(vec![
                        Span::styled(
                            format!(
                                "{:>7.1}s  ",
                                a.at_ms.saturating_sub(run.created_at_ms) as f64 / 1000.0
                            ),
                            Style::default().fg(MUTED),
                        ),
                        Span::raw(safe_text(&a.text)),
                    ])
                })
                .collect();
            if lines.is_empty() {
                lines.push(Line::from("No activity recorded.").fg(MUTED));
            }
            lines
        }
        DetailTab::Evidence => {
            let mut lines = vec![
                Line::from("r JSON report   a Agent note   f Evidence").fg(ACCENT),
                Line::default(),
            ];
            lines.extend(
                serde_json::to_string_pretty(run)
                    .unwrap_or_default()
                    .lines()
                    .map(|line| Line::from(line.to_owned()).fg(MUTED)),
            );
            lines
        }
    }
}
fn safe_text(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_control() || *c == '\n')
        .collect()
}
pub(super) fn overview_lines(run: &Report, note: &str, width: u16) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    if let Some(error) = &run.error {
        lines.push(Line::from("Run failed").fg(RED).bold());
        lines.extend(
            safe_text(error)
                .lines()
                .map(|line| Line::from(line.to_owned()).fg(RED)),
        );
        lines.push(Line::default());
    } else if run.state.active() {
        let elapsed = now().saturating_sub(run.created_at_ms) as f64 / 1000.0;
        lines.push(
            Line::from(format!(
                "{} · {} elapsed",
                state_label(run.state),
                duration(Some(elapsed))
            ))
            .fg(GOLD),
        );
        lines.push(Line::default());
    }
    let usage = run.usage.as_ref();
    let times = [
        (
            "Setup",
            if run.state == State::Preparing {
                "In progress".into()
            } else {
                duration(Some(run.setup_seconds))
            },
        ),
        ("Agent", duration(run.agent_seconds)),
        (
            "Verification",
            duration(run.verification.as_ref().map(|c| c.seconds)),
        ),
    ];
    let tokens = [
        ("Input", number(usage.map(|u| u.input_tokens))),
        ("  Cached", number(usage.map(|u| u.cached_input_tokens))),
        ("Output", number(usage.map(|u| u.output_tokens))),
    ];
    if width >= 64 {
        let col = (width - 5) / 2;
        let mut heading = Line::from(format!("{:<width$}", "Duration", width = col as usize));
        heading.spans.push(Span::raw("     "));
        heading.spans.push(Span::raw("Tokens"));
        lines.push(heading.fg(TEXT).bold());
        lines.push(Line::default());
        for ((label, value), (token, count)) in times.iter().zip(&tokens) {
            let mut line = pair(label, value, col);
            line.spans.push(Span::raw("     "));
            line.spans.extend(pair(token, count, col).spans);
            lines.push(line);
        }
    } else {
        lines.push(Line::from("Duration").bold());
        for (label, value) in &times {
            lines.push(pair(label, value, width));
        }
        lines.push(Line::default());
        lines.push(Line::from("Tokens").bold());
        for (label, value) in &tokens {
            lines.push(pair(label, value, width));
        }
    }
    lines.push(Line::default());
    lines.push(
        Line::from(format!(
            "{}Cached tokens are part of input.",
            if width >= 64 {
                " ".repeat(usize::from((width - 5) / 2 + 5))
            } else {
                String::new()
            }
        ))
        .fg(MUTED),
    );
    section(&mut lines, width);
    let (check, color) = match &run.verification {
        Some(check) if check.exit_code == Some(0) => ("Passed", GREEN),
        Some(_) => ("Failed", RED),
        None if run.state == State::Verifying => ("In progress", GOLD),
        None => ("Not run", MUTED),
    };
    let mut verification = pair("Verification", check, width);
    verification.spans[0].style = Style::default().fg(TEXT).bold();
    if let Some(value) = verification.spans.last_mut() {
        value.style = Style::default().fg(color);
    }
    lines.push(verification);
    lines.push(
        Line::from(match check {
            "Passed" => "Project verify command passed.",
            "Failed" => "See the verification log in Evidence.",
            _ => "",
        })
        .fg(MUTED),
    );
    section(&mut lines, width);
    lines.push(Line::from("Changed files").bold());
    lines.push(Line::default());
    if run.changed_files.is_empty() {
        lines.push(
            Line::from(if run.state.active() {
                "Available after agent execution."
            } else {
                "No source changes recorded."
            })
            .fg(MUTED),
        );
    }
    for path in &run.changed_files {
        let (kind, color) = match (run.before.contains_key(path), run.after.contains_key(path)) {
            (false, true) => ("A", GREEN),
            (true, false) => ("D", RED),
            (true, true) => ("M", ACCENT),
            (false, false) => ("?", MUTED),
        };
        lines.push(Line::from(vec![
            Span::styled(format!("{kind}  "), Style::default().fg(color)),
            Span::raw(safe_text(path)),
        ]));
    }
    section(&mut lines, width);
    lines.push(Line::from("Agent note").bold());
    lines.push(Line::default());
    if note.trim().is_empty() {
        lines.push(Line::from("No agent note available.").fg(MUTED));
    } else {
        lines.extend(
            safe_text(&note.chars().take(600).collect::<String>())
                .lines()
                .filter(|line| !line.trim().is_empty())
                .take(4)
                .map(|line| Line::from(line.to_owned())),
        );
    }
    if !note.trim().is_empty() {
        lines.push(Line::default());
        lines.push(Line::from("a Open full note").fg(ACCENT));
    }
    lines
}
fn section(lines: &mut Vec<Line<'static>>, width: u16) {
    lines.push(Line::default());
    lines.push(Line::from("─".repeat(width as usize)).fg(BORDER));
    lines.push(Line::default());
}
