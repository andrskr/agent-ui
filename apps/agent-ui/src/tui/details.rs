use super::{
    state::DetailTab,
    text::{duration, number, safe_text, section},
    theme::*,
};
use crate::report::{Report, State, now};
use ratatui::prelude::*;

const DURATION_LABEL: usize = 13;
const DURATION_VALUE: usize = 7;
const TOKEN_LABEL: usize = 8;
const TOKEN_VALUE: usize = 12;
const COLUMN_GAP: usize = 3;

fn pad(text: &str, width: usize) -> String {
    format!("{text:<width$}")
}
fn rpad(text: &str, width: usize) -> String {
    format!("{text:>width$}")
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
        DetailTab::Setup => setup_lines(Some(run), None, width),
    }
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
        if error.to_lowercase().contains("login") {
            lines.push(Line::from(format!("L Log in to {}", run.provider)).fg(ACCENT));
        }
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
        ("Setup", duration(Some(run.setup_seconds))),
        ("Agent", duration(run.agent_seconds)),
        (
            "Verification",
            duration(run.verification.as_ref().map(|c| c.seconds)),
        ),
    ];
    let tokens = [
        ("Input", number(usage.map(|u| u.input_tokens))),
        ("Cached", number(usage.map(|u| u.cached_input_tokens))),
        ("Output", number(usage.map(|u| u.output_tokens))),
    ];
    let indent = " ".repeat(DURATION_LABEL + DURATION_VALUE + COLUMN_GAP);
    if usize::from(width)
        >= DURATION_LABEL + DURATION_VALUE + COLUMN_GAP + TOKEN_LABEL + TOKEN_VALUE
    {
        lines.push(
            Line::from(format!("{}Tokens", pad("Duration", indent.len())))
                .fg(TEXT)
                .bold(),
        );
        lines.push(Line::default());
        for ((label, value), (token, count)) in times.iter().zip(&tokens) {
            lines.push(Line::from(format!(
                "{}{}{}{}{}",
                pad(label, DURATION_LABEL),
                rpad(value, DURATION_VALUE),
                " ".repeat(COLUMN_GAP),
                pad(token, TOKEN_LABEL),
                rpad(count, TOKEN_VALUE),
            )));
        }
        lines.push(Line::default());
        lines.push(Line::from(format!("{indent}Cached tokens are part of input.")).fg(MUTED));
    } else {
        lines.push(Line::from("Duration").bold());
        for (label, value) in &times {
            lines.push(Line::from(format!(
                "{}{}",
                pad(label, DURATION_LABEL),
                rpad(value, DURATION_VALUE)
            )));
        }
        lines.push(Line::default());
        lines.push(Line::from("Tokens").bold());
        for (label, count) in &tokens {
            lines.push(Line::from(format!(
                "{}{}",
                pad(label, TOKEN_LABEL),
                rpad(count, TOKEN_VALUE)
            )));
        }
        lines.push(Line::default());
        lines.push(Line::from("Cached tokens are part of input.").fg(MUTED));
    }
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
            "Failed" => "Press f to open the verification log folder.",
            _ => "",
        })
        .fg(MUTED),
    );
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

use crate::task_result::{TaskDetails, TaskView};
pub(super) struct Content<'a> {
    pub task: Option<&'a TaskView>,
    pub run: Option<&'a Report>,
    pub tab: DetailTab,
    pub details: Option<&'a TaskDetails>,
    pub note: &'a str,
}
pub(super) fn screen_lines(content: &Content<'_>, width: u16) -> Vec<Line<'static>> {
    if content.tab == DetailTab::Setup {
        return setup_lines(content.run, content.details, width);
    }
    if content.tab != DetailTab::Overview {
        return content
            .run
            .map(|run| content_lines(run, content.tab, content.note, width))
            .unwrap_or_else(|| vec![Line::from("No current run for this task.").fg(MUTED)]);
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
                "{} · {} · {} · {}",
                run.state.label(),
                run.provider,
                run.model_requested,
                run.effort_requested
            ))
            .fg(MUTED),
        );
        lines.push(Line::default());
        lines.extend(overview_lines(run, content.note, width));
    } else if content.details.is_some() {
        lines.push(Line::from("No saved result").bold());
        lines.push(Line::default());
        lines.push(Line::from("Run this task to collect metrics.").fg(MUTED));
        lines.push(Line::from("Open Setup to read its prompt and configuration.").fg(MUTED));
    } else {
        lines.push(Line::from("No task inputs are available. Check the task folder.").fg(MUTED));
    }
    lines
}

fn setup_lines(
    run: Option<&Report>,
    details: Option<&TaskDetails>,
    width: u16,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    if let Some(run) = run {
        lines.push(Line::from("Saved run configuration").bold());
        for (name, value) in [
            ("Provider", run.provider.clone()),
            ("Model", run.model_requested.clone()),
            ("Effort", run.effort_requested.clone()),
            ("Time limit", format!("{}s", run.timeout_seconds)),
            (
                "Agent CLI",
                run.provider_version
                    .clone()
                    .unwrap_or_else(|| "Not recorded".into()),
            ),
            (
                "Node",
                run.node_version
                    .clone()
                    .unwrap_or_else(|| "Not recorded".into()),
            ),
            (
                "Vite+",
                run.vp_version
                    .clone()
                    .unwrap_or_else(|| "Not recorded".into()),
            ),
        ] {
            lines.push(Line::from(format!(
                "{name}: {}",
                safe_text(value.lines().next().unwrap_or("Not recorded"))
            )));
        }
        packages(&mut lines, run.task_config.as_ref(), width);
        section(&mut lines, width);
        lines.push(Line::from("r Report  a Agent note  f Run files").fg(ACCENT));
    }
    if let Some(details) = details {
        if run.is_some() {
            section(&mut lines, width);
        }
        if details.changed_since_run {
            lines.push(Line::from("Task inputs changed since this run.").fg(GOLD));
        }
        lines.push(Line::from("Current task prompt").bold());
        lines.extend(
            safe_text(&details.prompt)
                .lines()
                .map(|line| Line::from(line.to_owned())),
        );
        if run.is_none() || details.changed_since_run {
            packages(&mut lines, Some(&details.config), width);
        }
    } else if run.is_none() {
        lines.push(Line::from("No task configuration is available.").fg(MUTED));
    }
    lines
}
fn packages(lines: &mut Vec<Line<'static>>, config: Option<&crate::task::TaskConfig>, width: u16) {
    section(lines, width);
    lines.push(Line::from("Package configuration").bold());
    let Some(config) = config else {
        lines.push(Line::from("Not recorded.").fg(MUTED));
        return;
    };
    if !config.has_packages() {
        lines.push(Line::from("Starter defaults.").fg(MUTED));
    }
    for (packages, kind) in [
        (&config.dependencies, "runtime"),
        (&config.dev_dependencies, "dev"),
    ] {
        for (name, version) in packages {
            lines.push(Line::from(format!("{name}@{version} ({kind})")));
        }
    }
    for (name, allowed) in &config.allow_builds {
        lines.push(Line::from(format!(
            "{name} install scripts: {}",
            if *allowed { "allowed" } else { "blocked" }
        )));
    }
}
