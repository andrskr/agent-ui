use super::{
    details::content_lines,
    history::{History, HistoryRow, local_stamp},
    layout::*,
    state::{App, DetailTab, FormField, Modal},
    theme::*,
};
use crate::report::Report;
use ratatui::{prelude::*, widgets::*};

fn block(title: &str) -> Block<'_> {
    Block::bordered()
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(BORDER))
        .title(Line::from(format!(" {title} ")).fg(MUTED))
        .padding(Padding::horizontal(2))
}
fn fit(text: &str, width: u16) -> String {
    if Line::from(text).width() <= usize::from(width) {
        return text.to_owned();
    }
    let mut value = text.to_owned();
    while Line::from(value.as_str()).width() >= usize::from(width) && !value.is_empty() {
        value.pop();
    }
    value.push('…');
    value
}
fn button(frame: &mut Frame, area: Rect, label: &str, primary: bool) {
    frame.render_widget(
        Paragraph::new(label)
            .alignment(Alignment::Center)
            .style(if primary {
                Style::default().bg(ACCENT).fg(BG).bold()
            } else {
                Style::default().bg(PANEL).fg(TEXT)
            }),
        area,
    );
}
pub(super) struct PreviewInfo<'a> {
    pub id: &'a str,
    pub ready: bool,
}
pub(super) struct Screen<'a> {
    pub history: &'a History,
    pub tab: DetailTab,
    pub scroll: Option<u16>,
    pub searching: bool,
    pub notice: &'a str,
    pub note: &'a str,
    pub preview: Option<PreviewInfo<'a>>,
    pub active: bool,
}
pub(super) fn draw(frame: &mut Frame, app: &App) {
    draw_screen(
        frame,
        &Screen {
            history: &app.history,
            tab: app.tab,
            scroll: app.scroll,
            searching: app.searching,
            notice: &app.notice,
            note: &app.agent_note,
            preview: app.runtime.preview().map(|p| PreviewInfo {
                id: p.id,
                ready: p.ready,
            }),
            active: app.runtime.has_active_run(),
        },
    );
    if frame.area().width >= 76 && frame.area().height >= 24 && app.modal != Modal::None {
        modal(frame, app);
    }
}
pub(super) fn draw_screen(frame: &mut Frame, screen: &Screen<'_>) {
    let area = frame.area();
    frame.render_widget(Block::default().bg(BG).fg(TEXT), area);
    if area.width < 76 || area.height < 24 {
        frame.render_widget(
            Paragraph::new("Use at least 76 columns and 24 rows.\nPress q to quit.")
                .wrap(Wrap { trim: false }),
            area,
        );
        return;
    }
    let (outer, panels) = regions(area);
    frame.render_widget(
        Paragraph::new(" AGENT UI").bold().block(
            Block::default()
                .border_type(BorderType::Plain)
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(BORDER)),
        ),
        outer[0],
    );
    button(frame, new_button(area), "+ New run  n", false);
    frame.render_widget(
        Block::default()
            .borders(Borders::RIGHT)
            .border_style(Style::default().fg(BORDER)),
        panels[0],
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" Runs", Style::default().bold()),
            Span::styled("  Local time", Style::default().fg(MUTED)),
        ])),
        Rect::new(panels[0].x, panels[0].y, panels[0].width, 1),
    );
    let search = search_area(panels[0]);
    let query = &screen.history.query;
    let search_text = if screen.searching || !query.is_empty() {
        format!("/ {query}")
    } else {
        "/ Search".into()
    };
    frame.render_widget(
        Paragraph::new(search_text)
            .fg(if screen.searching { ACCENT } else { MUTED })
            .scroll((
                0,
                (query.chars().count() as u16 + 3).saturating_sub(search.width),
            )),
        search,
    );
    if screen.searching {
        frame.set_cursor_position((
            search.x + (query.chars().count() as u16 + 2).min(search.width.saturating_sub(1)),
            search.y,
        ));
    }
    let list = run_list_area(panels[0]);
    let rows = screen.history.window(list.height);
    if rows.is_empty() {
        frame.render_widget(
            Paragraph::new(if query.is_empty() {
                "No runs yet"
            } else {
                "No matching runs"
            })
            .fg(MUTED),
            list,
        );
    }
    for (offset, row) in rows.iter().enumerate() {
        let rect = Rect::new(list.x, list.y + offset as u16, list.width, 1);
        match row {
            HistoryRow::Date(date) => {
                frame.render_widget(Paragraph::new(date.as_str()).fg(MUTED), rect)
            }
            HistoryRow::Gap => {}
            HistoryRow::Run { index, line } => {
                let run = &screen.history.runs[*index];
                let selected = screen.history.current().is_some_and(|r| r.id == run.id);
                let style = Style::default().bg(if selected { SELECTED } else { BG });
                frame.render_widget(Block::default().style(style), rect);
                if selected {
                    frame.render_widget(
                        Paragraph::new("▎").fg(ACCENT),
                        Rect::new(rect.x, rect.y, 1, 1),
                    );
                }
                let text = Rect::new(rect.x + 2, rect.y, rect.width.saturating_sub(3), 1);
                match line {
                    0 => {
                        frame.render_widget(Paragraph::new(fit(&run.task, text.width)).bold(), text)
                    }
                    1 => {
                        let clock = local_stamp(run.created_at_ms).1;
                        frame.render_widget(
                            Paragraph::new(state_label(run.state)).fg(state_color(run.state)),
                            Rect::new(text.x, text.y, text.width.saturating_sub(6), 1),
                        );
                        frame.render_widget(
                            Paragraph::new(clock).fg(MUTED).alignment(Alignment::Right),
                            Rect::new(text.right().saturating_sub(5), text.y, 5, 1),
                        );
                    }
                    _ => frame.render_widget(
                        Paragraph::new(fit(
                            &format!("{} · {}", run.model_requested, run.effort_requested),
                            text.width,
                        ))
                        .fg(MUTED),
                        text,
                    ),
                }
            }
        }
    }
    if let Some(run) = screen.history.current() {
        details(frame, panels[1], screen, run);
    } else {
        frame.render_widget(
            Paragraph::new(if query.is_empty() {
                "Press n to start a run."
            } else {
                "Change the search to find a run."
            })
            .fg(MUTED),
            detail_parts(panels[1])[2],
        );
    }
    frame.render_widget(Paragraph::new(screen.notice).fg(MUTED), outer[2]);
    let footer = if screen.searching {
        " Type to search   Enter Apply   Esc Clear".into()
    } else {
        format!(
            " ↑↓ Select   Tab View   / Search   ? Help{}",
            if screen.active { "   c Cancel" } else { "" }
        )
    };
    frame.render_widget(
        Paragraph::new(footer).fg(MUTED).block(
            Block::default()
                .border_type(BorderType::Plain)
                .borders(Borders::TOP)
                .border_style(Style::default().fg(BORDER)),
        ),
        outer[3],
    );
    if !screen.searching {
        frame.render_widget(
            Paragraph::new("q Quit ")
                .alignment(Alignment::Right)
                .fg(MUTED),
            Rect::new(outer[3].right() - 8, outer[3].y + 1, 8, 1),
        );
    }
}
fn details(frame: &mut Frame, area: Rect, screen: &Screen<'_>, run: &Report) {
    let parts = detail_parts(area);
    for (index, title) in ["Overview", "Activity", "Evidence"].iter().enumerate() {
        let rect = Rect::new(parts[0].x + index as u16 * 12, parts[0].y, 11, 1);
        frame.render_widget(
            Paragraph::new(*title).style(if screen.tab == DetailTab::ALL[index] {
                Style::default().fg(ACCENT).bold().underlined()
            } else {
                Style::default().fg(MUTED)
            }),
            rect,
        );
    }
    let selected_preview = screen.preview.as_ref().filter(|p| p.id == run.id);
    let actions = action_areas(parts[1]);
    let label = match selected_preview {
        Some(p) if p.ready => "b Open preview",
        Some(_) => "Starting…",
        None if actions[0].width >= 16 => "b Start preview",
        None => "b Preview",
    };
    button(
        frame,
        actions[0],
        label,
        selected_preview.is_none_or(|p| p.ready),
    );
    button(frame, actions[1], "e Open code", false);
    if selected_preview.is_some() {
        button(frame, actions[2], "x Stop", false);
    }
    let lines = content_lines(run, screen.tab, screen.note, parts[2].width);
    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    let last = paragraph
        .line_count(parts[2].width)
        .saturating_sub(parts[2].height as usize)
        .min(u16::MAX as usize) as u16;
    let offset = screen
        .scroll
        .unwrap_or(if screen.tab == DetailTab::Activity {
            last
        } else {
            0
        })
        .min(last);
    frame.render_widget(paragraph.scroll((offset, 0)), parts[2]);
    if last > 0 {
        frame.render_widget(
            Paragraph::new(if offset < last { "↓" } else { "↑" }).fg(MUTED),
            Rect::new(area.right() - 1, parts[2].bottom() - 1, 1, 1),
        );
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
            Paragraph::new("Select a task and run settings.").fg(MUTED),
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
            "↑↓ / j k       Select a run\nTab / 1 2 3    Switch views\n/              Search runs; Esc clears search\nPgUp / PgDn    Scroll the current view\nHome / End     First / last page; End follows activity\nn              Select a task and run Codex\ne / r / a / f  Code / JSON / agent note / evidence\nb / x          Start or open / stop preview\nc              Cancel the active run\nd              Remove the selected run\nq / Ctrl+C     Quit and stop owned processes\n\nClick tabs, buttons, or fields to select them.\nRun files remain on disk after you quit.\n\nEsc  Back".lines().map(|s| Line::from(s.to_owned())).collect()
        };
        frame.render_widget(
            Paragraph::new(lines).wrap(Wrap { trim: false }),
            area.inner(Margin::new(4, 1)),
        );
    }
}
