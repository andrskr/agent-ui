use super::{
    layout::*,
    state::{App, DetailTab, FormField, Modal},
    theme::*,
};
use crate::report::{Report, State};
use ratatui::{prelude::*, widgets::*};

fn block(title: &str) -> Block<'_> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(BORDER))
        .title(Line::from(format!(" {title} ")).fg(MUTED))
        .padding(Padding::horizontal(2))
}
fn state_color(state: State) -> Color {
    match state {
        State::Ready => GREEN,
        State::Failed => RED,
        State::Cancelled | State::Interrupted => MUTED,
        _ => GOLD,
    }
}
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
fn kv(label: &str, value: impl Into<String>) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label:<19}"), Style::default().fg(MUTED)),
        Span::styled(value.into(), Style::default().fg(TEXT)),
    ])
}
fn button(frame: &mut Frame, area: Rect, label: &str, primary: bool) {
    frame.render_widget(
        Paragraph::new(label)
            .alignment(Alignment::Center)
            .style(if primary {
                Style::default().bg(ACCENT).fg(BG).bold()
            } else {
                Style::default().bg(SELECTED).fg(TEXT)
            }),
        area,
    );
}
pub(super) fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    frame.render_widget(Block::default().bg(BG).fg(TEXT), area);
    if area.width < 76 || area.height < 24 {
        frame.render_widget(
            Paragraph::new("AGENT UI\n\nUse at least 76 columns and 24 rows.\n\nPress q to quit.")
                .block(block("Terminal size")),
            area,
        );
        return;
    }
    let (outer, panels) = regions(area);
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("  AGENT UI", Style::default().fg(TEXT).bold()),
            Span::styled("   /   Experiments", Style::default().fg(MUTED)),
        ]))
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(BORDER)),
        ),
        outer[0],
    );
    button(frame, new_button(area), " + New run  n ", true);
    frame.render_widget(
        Block::default()
            .borders(Borders::RIGHT)
            .border_style(Style::default().fg(BORDER)),
        panels[0],
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" RUNS", Style::default().fg(MUTED).bold()),
            Span::styled(format!("  {}", app.runs.len()), Style::default().fg(MUTED)),
        ])),
        Rect::new(panels[0].x, panels[0].y, panels[0].width, 1),
    );
    let inner = run_list_area(panels[0]);
    let start = app
        .selected
        .saturating_sub((inner.height as usize / 4).saturating_sub(1));
    let rows: Vec<_> = app
        .runs
        .iter()
        .enumerate()
        .skip(start)
        .flat_map(|(i, r)| {
            let selected = i == app.selected;
            let style = Style::default()
                .fg(TEXT)
                .bg(if selected { SELECTED } else { BG });
            [
                ListItem::new(
                    Line::from(format!("{} {}", if selected { "▎" } else { " " }, r.task)).bold(),
                )
                .style(style),
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("  ● {}", r.state.label()),
                        Style::default().fg(state_color(r.state)),
                    ),
                    Span::styled(
                        format!("  {}", r.id.rsplit('-').next().unwrap_or(&r.id)),
                        Style::default().fg(MUTED),
                    ),
                ]))
                .style(style),
                ListItem::new(
                    Line::from(format!(
                        "  {} · {} out",
                        r.agent_seconds
                            .map(|s| format!("{s:.1}s"))
                            .unwrap_or("—".into()),
                        number(r.usage.as_ref().map(|u| u.output_tokens))
                    ))
                    .fg(MUTED),
                )
                .style(style),
                ListItem::new(""),
            ]
        })
        .collect();
    frame.render_widget(List::new(rows), inner);
    if let Some(run) = app.current() {
        details(frame, panels[1], app, run);
    } else {
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(""),
                Line::from("Your next experiment starts here.")
                    .fg(TEXT)
                    .bold(),
                Line::from(""),
                Line::from("Choose a task. Run Codex. Review what it builds.").fg(MUTED),
                Line::from(""),
                Line::from("  01   Start with a fresh copy of your app"),
                Line::from(""),
                Line::from("  02   Keep the code, timing, and token usage"),
                Line::from(""),
                Line::from("  03   Open the result in your browser or editor"),
                Line::from(""),
                Line::from("Press n or click New run to choose a task.").fg(ACCENT),
            ])
            .wrap(Wrap { trim: false })
            .block(Block::default().padding(Padding::horizontal(3))),
            panels[1],
        );
    }
    frame.render_widget(
        Paragraph::new(format!(" {}", app.notice))
            .fg(MUTED)
            .wrap(Wrap { trim: true }),
        outer[2],
    );
    let mut shortcuts = vec![
        ("↑↓", "runs"),
        ("Tab", "view"),
        ("n", "new"),
        ("?", "help"),
        ("q", "quit"),
    ];
    if app.active.is_some() {
        shortcuts.insert(3, ("c", "cancel run"));
    }
    if app.preview.is_some() {
        shortcuts.insert(3, ("x", "stop preview"));
    }
    let spans: Vec<_> = shortcuts
        .into_iter()
        .flat_map(|(key, label)| {
            [
                Span::styled(format!(" {key} "), Style::default().fg(TEXT).bg(PANEL)),
                Span::styled(format!(" {label}   "), Style::default().fg(MUTED)),
            ]
        })
        .collect();
    frame.render_widget(Paragraph::new(Line::from(spans)), outer[3]);
    if app.modal != Modal::None {
        modal(frame, app);
    }
}
fn details(frame: &mut Frame, area: Rect, app: &App, run: &Report) {
    let parts = detail_parts(area);
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(vec![
                Span::styled(run.task.clone(), Style::default().fg(TEXT).bold()),
                Span::styled(
                    format!("   ● {}", run.state.label()),
                    Style::default().fg(state_color(run.state)),
                ),
            ]),
            Line::from(format!(
                "{}  ·  {} effort  ·  Local Codex",
                run.model_requested, run.effort_requested
            ))
            .fg(MUTED),
            Line::from(format!("Run {}", run.id)).fg(MUTED),
        ]),
        parts[0],
    );
    let tabs = Layout::horizontal([
        Constraint::Length(15),
        Constraint::Length(15),
        Constraint::Length(15),
        Constraint::Min(0),
    ])
    .split(parts[1]);
    for (i, label) in ["1 Overview", "2 Activity", "3 Report"].iter().enumerate() {
        let row = Rect::new(tabs[i].x, tabs[i].y, tabs[i].width.saturating_sub(1), 1);
        frame.render_widget(
            Paragraph::new(*label).alignment(Alignment::Center).style(
                if app.tab == DetailTab::ALL[i] {
                    Style::default().bg(SELECTED).fg(ACCENT).bold()
                } else {
                    Style::default().fg(MUTED)
                },
            ),
            row,
        );
    }
    match app.tab {
        DetailTab::Overview => overview(frame, parts[2], run),
        DetailTab::Activity => {
            let lines: Vec<_> = run
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
                        Span::raw(a.text.clone()),
                    ])
                })
                .collect();
            let last = lines
                .len()
                .saturating_sub(parts[2].height.saturating_sub(2) as usize)
                .min(u16::MAX as usize) as u16;
            let offset = app.scroll.unwrap_or(last).min(last);
            frame.render_widget(
                Paragraph::new(lines)
                    .scroll((offset, 0))
                    .block(block("ACTIVITY · PgUp / PgDn")),
                parts[2],
            );
        }
        _ => {
            let json = serde_json::to_string_pretty(run).unwrap_or_default();
            let offset = app.scroll.unwrap_or_default().min(
                json.lines()
                    .count()
                    .saturating_sub(parts[2].height.saturating_sub(2) as usize)
                    .min(u16::MAX as usize) as u16,
            );
            frame.render_widget(
                Paragraph::new(
                    json.lines()
                        .map(|line| {
                            if let Some((key, value)) = line.split_once(": ") {
                                Line::from(vec![
                                    Span::styled(format!("{key}: "), Style::default().fg(ACCENT)),
                                    Span::styled(value.to_owned(), Style::default().fg(TEXT)),
                                ])
                            } else {
                                Line::from(line.to_owned()).fg(MUTED)
                            }
                        })
                        .collect::<Vec<_>>(),
                )
                .scroll((offset, 0))
                .block(block("SAVED REPORT · PgUp / PgDn").bg(PANEL)),
                parts[2],
            );
        }
    }
}
fn overview(frame: &mut Frame, area: Rect, run: &Report) {
    let usage = run.usage.as_ref();
    let seconds = run
        .agent_seconds
        .map(|s| format!("{s:.1}s"))
        .unwrap_or("—".into());
    let check = run
        .verification
        .as_ref()
        .map(|c| {
            if c.exit_code == Some(0) {
                "Passed"
            } else {
                "Failed"
            }
        })
        .unwrap_or("Not run");
    if area.width < 65 || area.height < 16 {
        frame.render_widget(
            Paragraph::new(vec![
                kv("Agent time", seconds),
                kv("Input tokens", number(usage.map(|u| u.input_tokens))),
                kv("Cached input", number(usage.map(|u| u.cached_input_tokens))),
                kv("Output tokens", number(usage.map(|u| u.output_tokens))),
                kv("Verification", check),
                kv("Human review", "Pending"),
                Line::from(run.error.clone().unwrap_or_default()).fg(RED),
            ]),
            Rect::new(area.x, area.y, area.width, area.height.saturating_sub(2)),
        );
    } else {
        let rows = Layout::vertical([
            Constraint::Length(4),
            Constraint::Min(8),
            Constraint::Length(2),
        ])
        .spacing(2)
        .split(area);
        let metrics = Layout::horizontal([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .spacing(1)
        .split(rows[0]);
        for (rect, title, value, hint) in [
            (metrics[0], "AGENT TIME", seconds, "Codex duration"),
            (
                metrics[1],
                "INPUT TOKENS",
                number(usage.map(|u| u.input_tokens)),
                "Includes cached input",
            ),
            (
                metrics[2],
                "OUTPUT TOKENS",
                number(usage.map(|u| u.output_tokens)),
                "Reported by Codex",
            ),
        ] {
            frame.render_widget(
                Paragraph::new(vec![
                    Line::from(title).fg(MUTED),
                    Line::from(value).fg(TEXT).bold(),
                    Line::from(hint).fg(MUTED),
                ])
                .block(Block::default().bg(PANEL).padding(Padding::new(2, 1, 1, 0))),
                rect,
            );
        }
        let cols = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .spacing(2)
            .split(rows[1]);
        frame.render_widget(
            Paragraph::new(vec![
                Line::from("RESULT").fg(MUTED).bold(),
                Line::from(""),
                kv("Verification", check),
                kv("Human review", "Pending"),
                kv("Changed files", run.changed_files.len().to_string()),
                kv("Setup time", format!("{:.1}s", run.setup_seconds)),
            ]),
            cols[0],
        );
        frame.render_widget(
            Paragraph::new(vec![
                Line::from("TOKEN USAGE").fg(MUTED).bold(),
                Line::from(""),
                kv("Cached input", number(usage.map(|u| u.cached_input_tokens))),
                kv(
                    "Uncached input",
                    number(usage.map(|u| u.input_tokens.saturating_sub(u.cached_input_tokens))),
                ),
                kv("Output", number(usage.map(|u| u.output_tokens))),
                Line::from("Subscription · no reported charge").fg(MUTED),
            ]),
            cols[1],
        );
        let note = run.error.as_deref().unwrap_or(match run.state {
            State::Ready => "Ready for your review. Open the app to check the result.",
            State::Preparing => "Preparing the app and task inputs...",
            State::Running => "Codex is working. Open Activity to follow the run.",
            State::Verifying => "Checking the app before review...",
            _ => "Partial code and evidence are available for review.",
        });
        frame.render_widget(
            Paragraph::new(note)
                .fg(if run.error.is_some() { RED } else { MUTED })
                .wrap(Wrap { trim: false }),
            rows[2],
        );
    }
    for (i, (rect, label)) in action_areas(area)
        .iter()
        .zip([
            "b Preview",
            "e Open code",
            "r JSON",
            if area.width >= 65 {
                "a Agent report"
            } else {
                "a Notes"
            },
        ])
        .enumerate()
    {
        button(frame, *rect, label, i == 0);
    }
}
fn modal(frame: &mut Frame, app: &App) {
    for cell in &mut frame.buffer_mut().content {
        cell.set_fg(BORDER).set_bg(BG);
    }
    let area = modal_rect(frame.area());
    frame.render_widget(Clear, area);
    frame.render_widget(
        block(match app.modal {
            Modal::New => "NEW EXPERIMENT",
            Modal::Delete => "REMOVE RUN",
            _ => "KEYBOARD",
        })
        .bg(PANEL)
        .border_style(Style::default().fg(BORDER)),
        area,
    );
    if app.modal == Modal::New {
        frame.render_widget(
            Paragraph::new("A fresh start. A result you can review.").fg(MUTED),
            Rect::new(area.x + 4, area.y + 2, area.width.saturating_sub(8), 1),
        );
        let task = app
            .tasks
            .get(app.task)
            .map(String::as_str)
            .unwrap_or("No tasks found");
        for (i, label, value) in [
            (0, "Task", task),
            (1, "Model", app.settings.model.as_str()),
            (2, "Effort", app.settings.effort.as_str()),
        ] {
            let rect = modal_field(area, i);
            frame.render_widget(
                Paragraph::new(format!(
                    "{} {label}",
                    if app.field == FormField::ALL[i] {
                        "›"
                    } else {
                        " "
                    }
                ))
                .fg(if app.field == FormField::ALL[i] {
                    TEXT
                } else {
                    MUTED
                }),
                Rect::new(area.x + 4, rect.y, 12, 1),
            );
            frame.render_widget(
                Paragraph::new(format!(" {}", value)).style(
                    Style::default()
                        .fg(if app.field == FormField::ALL[i] {
                            TEXT
                        } else {
                            MUTED
                        })
                        .bg(if app.field == FormField::ALL[i] {
                            SELECTED
                        } else {
                            BG
                        }),
                ),
                rect,
            );
            if i != 1 {
                frame.render_widget(
                    Paragraph::new("‹  ›").fg(ACCENT),
                    Rect::new(rect.right() - 4, rect.y, 4, 1),
                );
            }
            let hint = match i {
                0 => format!("{} available · use the arrow keys", app.tasks.len()),
                1 => "Type a model ID · Backspace to edit".into(),
                _ => "low / medium / high / xhigh".into(),
            };
            frame.render_widget(
                Paragraph::new(hint).fg(MUTED),
                Rect::new(rect.x, rect.y + 1, rect.width, 1),
            );
        }
        frame.render_widget(
            Paragraph::new(format!(
                "Local subscription  ·  {}s limit  ·  Saved evidence",
                app.settings.timeout
            ))
            .fg(MUTED),
            Rect::new(area.x + 4, area.y + 14, area.width.saturating_sub(8), 1),
        );
        button(frame, modal_submit(area), "Run experiment  ↵", true);
        frame.render_widget(
            Paragraph::new("Esc  Back").fg(MUTED),
            Rect::new(area.x + 32, area.y + 16, 18, 1),
        );
        if app.field == FormField::Model {
            let rect = modal_field(area, 1);
            frame.set_cursor_position((
                rect.x + 1 + (app.settings.model.len() as u16).min(rect.width.saturating_sub(2)),
                rect.y,
            ));
        }
    } else {
        let lines = if app.modal == Modal::Delete {
            vec![
                Line::from(""),
                Line::from("Remove this run and all its files?").bold(),
                Line::from(app.current().map(|r| r.task.as_str()).unwrap_or("")).fg(ACCENT),
                Line::from(""),
                Line::from("This includes its code, logs, and reports.").fg(MUTED),
                Line::from(""),
                Line::from("y  Remove run       Esc  Back").fg(RED),
            ]
        } else {
            "\n↑↓ / j k       Select a run\nTab / 1 2 3    Switch detail views\nPgUp / PgDn    Scroll activity or report\nHome / End     First / last page; End follows activity\nn              Select a task and run Codex\ne / r / a      Open code / JSON / agent report\nb              Start a browser preview\nx              Stop the preview\nc              Cancel the active run\nd              Remove the selected run\nq / Ctrl+C     Quit and stop owned processes\n\nClick tabs, buttons, or fields to select them.\nRun files remain on disk after you quit.\n\nEsc  Back".lines().map(|s| Line::from(s.to_owned())).collect()
        };
        frame.render_widget(
            Paragraph::new(lines).wrap(Wrap { trim: false }),
            area.inner(Margin::new(4, 1)),
        );
    }
}
