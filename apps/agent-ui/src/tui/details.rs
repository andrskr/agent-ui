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
fn active_status_line(run: &Report) -> Line<'static> {
    let elapsed = now().saturating_sub(run.created_at_ms) as f64 / 1000.0;
    let step = run
        .step
        .clone()
        .unwrap_or_else(|| state_word(run.state).to_string());
    Line::from(format!(
        "{} {} · {} · {} elapsed",
        spinner_frame(),
        state_word(run.state),
        step,
        duration(Some(elapsed)),
    ))
    .fg(GOLD)
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
            if run.state.active() {
                lines.push(Line::default());
                lines.push(active_status_line(run));
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
        lines.push(active_status_line(run));
        lines.push(Line::default());
    }
    let usage = run.usage.as_ref();
    lines.push(pair(
        "Estimated API cost (USD)",
        &crate::cost::format_usd(run.cost_usd),
        width,
    ));
    let cost_note = if run.cost_usd.is_some_and(|usd| usd > 0.0) {
        match run.cost_basis {
            Some(crate::cost::Basis::Requests) => {
                "API rates from saved requests. Subscription charge is separate."
            }
            Some(crate::cost::Basis::RunTotals) => {
                "Standard rates for the requested model. Request details unavailable."
            }
            Some(crate::cost::Basis::ProviderReported) => {
                "API price reported by the provider. Subscription charge is separate."
            }
            None => &run.cost_note,
        }
    } else {
        &run.cost_note
    };
    lines.push(Line::from(cost_note.to_owned()).fg(MUTED));
    lines.push(Line::default());
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
    if let Some(config) = &run.task_config
        && config.has_packages()
    {
        lines.push(Line::from("Task packages").bold());
        lines.push(Line::default());
        for (packages, kind) in [
            (&config.dependencies, "runtime"),
            (&config.dev_dependencies, "dev"),
        ] {
            for (name, version) in packages {
                lines.push(Line::from(format!("{name}  {version}  ({kind})")));
            }
        }
        for (package, allowed) in &config.allow_builds {
            let permission = if *allowed { "allowed" } else { "blocked" };
            lines.push(Line::from(format!(
                "{package}  (install scripts {permission})"
            )));
        }
        section(&mut lines, width);
    }
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

use crate::{
    comparison::{ChangeKind, Comparison, FileChange},
    task_result::{TaskDetails, TaskView},
};
pub(super) struct Content<'a> {
    pub task: Option<&'a TaskView>,
    pub run: Option<&'a Report>,
    pub comparison: Option<&'a Comparison>,
    pub comparing: bool,
    pub tab: DetailTab,
    pub details: Option<&'a TaskDetails>,
    pub note: &'a str,
    pub assessment: &'a str,
}
pub(super) fn screen_lines(content: &Content<'_>, width: u16) -> Vec<Line<'static>> {
    if content.tab != DetailTab::Overview {
        return content
            .run
            .map(|run| content_lines(run, content.tab, content.note, width))
            .unwrap_or_else(|| vec![Line::from("No current run for this task.").fg(MUTED)]);
    }
    if content.comparing {
        return content
            .comparison
            .map(|c| comparison_lines(c, content.assessment, width))
            .unwrap_or_else(|| {
                vec![
                    Line::from("Both tasks need a current run. Press Esc to return to the task.")
                        .fg(MUTED),
                ]
            });
    }
    let mut lines = Vec::new();
    if content.task.is_some_and(|t| t.cleanup_pending) {
        lines.push(
            Line::from("Previous output cleanup is blocked. Fix the reported error and run again.")
                .fg(RED),
        );
    }
    if let Some(details) = content.details
        && details.changed_since_run
    {
        lines.push(
            Line::from(
                "Task inputs changed since this run. The report describes the saved inputs.",
            )
            .fg(GOLD),
        );
        lines.push(Line::default());
    }
    if let Some(run) = content.run {
        lines.push(
            Line::from(format!(
                "{} · {} · {}",
                run.provider, run.model_requested, run.effort_requested
            ))
            .fg(MUTED),
        );
        lines.push(Line::default());
        lines.extend(overview_lines(run, content.note, width));
    } else if let Some(details) = content.details {
        lines.push(Line::from("n Run this task").fg(ACCENT));
        lines.push(Line::default());
        lines.push(Line::from("Prompt").bold());
        lines.extend(
            safe_text(&details.prompt)
                .lines()
                .map(|l| Line::from(l.to_owned())),
        );
        section(&mut lines, width);
        lines.push(Line::from("Task inputs").bold());
        lines.extend(details.files.iter().map(|p| Line::from(format!("  {p}"))));
        if details.config.has_packages() {
            section(&mut lines, width);
            lines.push(Line::from("Task package settings").bold());
            lines.extend(
                serde_json::to_string_pretty(&details.config)
                    .unwrap_or_default()
                    .lines()
                    .map(|l| Line::from(l.to_owned())),
            );
        }
    } else {
        lines.push(Line::from("No task inputs are available. Check the task folder.").fg(MUTED));
    }
    lines
}
fn change_lines(
    lines: &mut Vec<Line<'static>>,
    title: &str,
    changes: &Option<Vec<FileChange>>,
    width: u16,
) {
    section(lines, width);
    lines.push(Line::from(title.to_owned()).bold());
    match changes {
        None => lines.push(Line::from("Snapshot not available for both runs.").fg(MUTED)),
        Some(changes) if changes.is_empty() => lines.push(Line::from("No differences.").fg(MUTED)),
        Some(changes) => {
            for change in changes {
                let (label, color) = match change.kind {
                    ChangeKind::Added => ("A", GREEN),
                    ChangeKind::Removed => ("D", RED),
                    ChangeKind::Modified => ("M", ACCENT),
                };
                lines.push(Line::from(vec![
                    Span::styled(format!("{label}  "), Style::default().fg(color)),
                    Span::raw(safe_text(&change.path)),
                ]));
            }
        }
    }
}
fn comparison_lines(c: &Comparison, assessment: &str, width: u16) -> Vec<Line<'static>> {
    let a = &c.reference;
    let b = &c.other;
    let m = &c.measurements;
    let mut lines = vec![Line::from("Measured results").bold(), Line::default()];
    let mut rows = Vec::new();
    rows.push((
        "API cost (est)",
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
    if width >= 66 {
        let col = usize::from((width - 14) / 3);
        lines.push(
            Line::from(format!(
                "{:<14}{:<col$}{:<col$}B − A",
                "", "A · Reference", "B"
            ))
            .fg(ACCENT),
        );
        for (label, a, b, difference) in rows {
            lines.push(Line::from(format!(
                "{label:<14}{a:<col$}{b:<col$}{difference}"
            )));
        }
    } else {
        lines.push(Line::from("A → B  (difference)").fg(ACCENT));
        for (label, a, b, difference) in rows {
            lines.push(Line::from(format!("{label}: {a} → {b}  ({difference})")));
        }
    }
    lines.push(Line::default());
    lines.push(
        Line::from("Estimated API cost is in USD. Subscription charges are separate.").fg(MUTED),
    );
    for (side, run) in [("A", a), ("B", b)] {
        if !run.cost_note.is_empty() {
            lines.push(Line::from(format!("{side}: {}", run.cost_note)).fg(MUTED));
        }
    }
    lines.push(
        Line::from("Cached tokens are part of input. Lower usage does not prove better UI.")
            .fg(MUTED),
    );
    section(&mut lines, width);
    lines.push(Line::from("Verification and settings").bold());
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
        (
            "Agent CLI",
            a.provider_version.as_deref(),
            b.provider_version.as_deref(),
        ),
        ("Node", a.node_version.as_deref(), b.node_version.as_deref()),
        ("Vite+", a.vp_version.as_deref(), b.vp_version.as_deref()),
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
        lines.push(Line::from(value).fg(MUTED));
    }
    change_lines(&mut lines, "Saved task inputs", &c.input_changes, width);
    change_lines(
        &mut lines,
        "Setup files · includes manifests and lockfiles",
        &c.setup_changes,
        width,
    );
    change_lines(
        &mut lines,
        "Generated source files",
        &c.source_changes,
        width,
    );
    section(&mut lines, width);
    lines.push(Line::from("Agent assessment · separate time and usage").bold());
    if assessment.is_empty() {
        lines.push(Line::from("m Ask agent to inspect saved code and evidence.").fg(ACCENT));
        lines.push(Line::from("Visual review stays in the browser.").fg(MUTED));
    } else {
        lines.extend(
            safe_text(assessment)
                .lines()
                .map(|l| Line::from(l.to_owned())),
        );
    }
    lines
}
