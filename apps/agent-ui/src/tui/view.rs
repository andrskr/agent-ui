use super::{
    details::screen_lines,
    layout::*,
    state::{App, DetailTab, FormField, Modal},
    tasks::{TaskRow, Tasks, local_stamp},
    theme::*,
};
use crate::{
    comparison::Comparison,
    report::Report,
    task_result::{Selection, TaskDetails},
};
use ratatui::{prelude::*, widgets::*};

fn block(title: &str) -> Block<'_> {
    Block::bordered()
        .border_type(BorderType::Plain)
        .border_style(Style::default().fg(BORDER))
        .title(Line::from(format!(" {title} ")).fg(MUTED))
        .padding(Padding::horizontal(2))
}
pub(super) fn fit(text: &str, width: u16) -> String {
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
pub(super) fn button(frame: &mut Frame, area: Rect, label: &str, primary: bool) {
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
    pub tasks: &'a Tasks,
    pub tab: DetailTab,
    pub scroll: Option<u16>,
    pub searching: bool,
    pub notice: &'a str,
    pub note: &'a str,
    pub preview: Option<PreviewInfo<'a>>,
    pub selection: &'a Selection,
    pub comparison: Option<&'a Comparison>,
    pub details: Option<&'a TaskDetails>,
    pub starting: Vec<String>,
    pub queued: Vec<String>,
    pub queue_errors: &'a std::collections::BTreeMap<String, String>,
}
pub(super) fn draw(frame: &mut Frame, app: &App) {
    draw_screen(
        frame,
        &Screen {
            tasks: &app.tasks,
            tab: app.tab,
            scroll: app.scroll,
            searching: app.searching,
            notice: &app.notice,
            note: &app.agent_note,
            preview: app
                .focused_task()
                .and_then(|task| app.runtime.preview(task))
                .map(|p| PreviewInfo {
                    id: p.id,
                    ready: p.ready,
                }),
            selection: &app.selection,
            comparison: app.comparison.as_ref(),
            details: app.details.as_ref(),
            starting: app.runtime.active_task_ids(),
            queued: app.runtime.queued_task_ids(),
            queue_errors: app.runtime.queue_errors(),
        },
    );
    if frame.area().width >= 76 && frame.area().height >= 24 && app.modal != Modal::None {
        modal(frame, app);
    }
}
fn focus_id<'a>(screen: &'a Screen<'a>) -> Option<&'a str> {
    if screen.selection.comparing || screen.tasks.is_group() {
        None
    } else {
        screen.tasks.current().map(|t| t.id.as_str())
    }
}
fn focused_run<'a>(screen: &'a Screen<'a>) -> Option<&'a Report> {
    focus_id(screen)
        .and_then(|id| screen.tasks.items.iter().find(|t| t.id == id))
        .and_then(|t| t.run.as_ref())
}
fn is_starting(screen: &Screen<'_>, id: &str, run: Option<&Report>) -> bool {
    screen.starting.iter().any(|s| s == id) && run.is_none_or(|r| !r.state.active())
}
fn elapsed_secs(run: &Report) -> f64 {
    crate::report::now().saturating_sub(run.created_at_ms) as f64 / 1000.0
}
fn active_status(run: &Report) -> String {
    let step = run
        .step
        .clone()
        .unwrap_or_else(|| state_word(run.state).to_string());
    format!(
        "{} {} · {} · {:.1}s elapsed",
        spinner_frame(),
        state_word(run.state),
        step,
        elapsed_secs(run)
    )
}
fn row_status(run: &Report) -> String {
    format!(
        "{} {} · {:.0}s",
        spinner_frame(),
        state_word(run.state),
        elapsed_secs(run)
    )
}
fn footer_status(screen: &Screen<'_>) -> Option<String> {
    let active: Vec<&Report> = screen
        .tasks
        .items
        .iter()
        .filter_map(|t| t.run.as_ref())
        .filter(|r| r.state.active())
        .collect();
    match active.as_slice() {
        [] => None,
        [run] => Some(active_status(run)),
        many => {
            let tasks = many
                .iter()
                .map(|r| format!("{} {:.0}s", r.task, elapsed_secs(r)))
                .collect::<Vec<_>>()
                .join(" · ");
            Some(format!(
                "{} {} running · {tasks}",
                spinner_frame(),
                many.len()
            ))
        }
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
    frame.render_widget(
        Block::default()
            .borders(Borders::RIGHT)
            .border_style(Style::default().fg(BORDER)),
        panels[0],
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" Groups", Style::default().bold()),
            Span::styled("", Style::default().fg(MUTED)),
        ])),
        Rect::new(panels[0].x, panels[0].y, panels[0].width, 1),
    );
    let search = search_area(panels[0]);
    let query = &screen.tasks.query;
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
    let list = task_list_area(panels[0]);
    let rows = screen.tasks.window(list.height);
    if rows.is_empty() {
        frame.render_widget(
            Paragraph::new(if query.is_empty() {
                "No tasks found"
            } else {
                "No matching tasks"
            })
            .fg(MUTED),
            list,
        );
    }
    for (offset, row) in rows.iter().enumerate() {
        let rect = Rect::new(list.x, list.y + offset as u16, list.width, 1);
        match row {
            TaskRow::Gap => {}
            TaskRow::Group { index } => {
                let name = super::tasks::group(&screen.tasks.items[*index].id);
                let selected = screen.tasks.is_group() && screen.tasks.group() == Some(name);
                frame.render_widget(
                    Paragraph::new(fit(
                        &format!(
                            "{} {}",
                            if selected { "▎" } else { "▾" },
                            super::tasks::label(name)
                        ),
                        rect.width,
                    ))
                    .bold()
                    .fg(if selected { ACCENT } else { TEXT })
                    .bg(if selected { SELECTED } else { BG }),
                    rect,
                );
            }
            TaskRow::Task { index, line } => {
                let run = &screen.tasks.items[*index];
                let selected = !screen.tasks.is_group()
                    && screen.tasks.current().is_some_and(|r| r.id == run.id);
                let style = Style::default().bg(if selected { SELECTED } else { BG });
                frame.render_widget(Block::default().style(style), rect);
                if selected {
                    frame.render_widget(
                        Paragraph::new("▎").fg(ACCENT),
                        Rect::new(rect.x, rect.y, 1, 1),
                    );
                }
                let indent = if screen.tasks.is_grouped() { 4 } else { 2 };
                let text = Rect::new(
                    rect.x + indent,
                    rect.y,
                    rect.width.saturating_sub(indent + 1),
                    1,
                );
                match line {
                    0 => frame.render_widget(
                        Paragraph::new(fit(
                            &format!(
                                "{}{}",
                                if screen.selection.comparing
                                    && screen
                                        .selection
                                        .pair
                                        .as_ref()
                                        .is_some_and(|p| p.reference == run.id)
                                {
                                    "A · "
                                } else if screen.selection.comparing
                                    && screen
                                        .selection
                                        .pair
                                        .as_ref()
                                        .is_some_and(|p| p.other == run.id)
                                {
                                    "B · "
                                } else {
                                    ""
                                },
                                if screen.tasks.is_grouped() {
                                    super::tasks::label(super::tasks::variant(&run.id))
                                } else {
                                    run.id.clone()
                                }
                            ),
                            text.width,
                        ))
                        .bold(),
                        text,
                    ),
                    1 => {
                        let (status, color) = if screen.queued.contains(&run.id) {
                            ("Queued".into(), GOLD)
                        } else if screen.queue_errors.contains_key(&run.id) {
                            ("Start failed".into(), GOLD)
                        } else if is_starting(screen, &run.id, run.run.as_ref()) {
                            (format!("{} Starting…", spinner_frame()), GOLD)
                        } else {
                            match run.run.as_ref() {
                                Some(r) if r.state.active() => (row_status(r), GOLD),
                                Some(r) => (run.status().to_owned(), state_color(r.state)),
                                None => (run.status().to_owned(), MUTED),
                            }
                        };
                        frame.render_widget(
                            Paragraph::new(fit(&status, text.width)).fg(color),
                            text,
                        );
                    }
                    _ => {
                        let stamp = run
                            .run
                            .as_ref()
                            .map(|r| {
                                let (date, time) = local_stamp(r.created_at_ms);
                                format!("{date} · {time}")
                            })
                            .unwrap_or_else(|| "No saved output".into());
                        frame
                            .render_widget(Paragraph::new(fit(&stamp, text.width)).fg(MUTED), text);
                    }
                }
            }
        }
    }
    if screen.tasks.current().is_some() {
        if screen.tasks.is_group() {
            let parts = detail_parts(panels[1]);
            frame.render_widget(
                Paragraph::new(super::tasks::label(
                    screen.tasks.group().unwrap_or_default(),
                ))
                .bold(),
                parts[0],
            );
            super::tabs::draw(
                frame,
                parts[1],
                true,
                usize::from(screen.selection.comparing),
            );
            if screen.selection.comparing {
                super::compare::draw(frame, panels[1], screen);
            } else {
                super::groups::overview(frame, panels[1], screen);
            }
        } else if screen.selection.comparing {
            super::compare::draw(frame, panels[1], screen);
        } else {
            details(frame, panels[1], screen);
        }
    } else {
        frame.render_widget(
            Paragraph::new(if query.is_empty() {
                "Add a task under experiments/tasks."
            } else {
                "Change the search to find a task."
            })
            .fg(MUTED),
            detail_parts(panels[1])[3],
        );
    }
    match footer_status(screen) {
        Some(status) => frame.render_widget(Paragraph::new(status).fg(GOLD), outer[2]),
        None => frame.render_widget(Paragraph::new(screen.notice).fg(MUTED), outer[2]),
    }
    let footer = if screen.searching {
        " Type to search  Enter Apply  Esc Clear"
    } else if screen.tasks.is_group() || screen.selection.comparing {
        " ↑↓ Group  Enter Tasks  ←→/Tab Views  ? Help"
    } else {
        " ↑↓ Task  Esc Group  ←→/Tab Views  ? Help"
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
fn details(frame: &mut Frame, area: Rect, screen: &Screen<'_>) {
    let parts = detail_parts(area);
    let run = focused_run(screen);
    let starting = focus_id(screen).is_some_and(|id| is_starting(screen, id, run));
    if let Some(task) = screen.tasks.current() {
        frame.render_widget(
            Paragraph::new(format!(
                "{} / {}",
                super::tasks::label(super::tasks::group(&task.id)),
                super::tasks::label(super::tasks::variant(&task.id))
            ))
            .bold(),
            parts[0],
        );
    }
    super::tabs::draw(frame, parts[1], false, screen.tab as usize);
    let actions = action_areas(parts[2]);
    button(
        frame,
        actions[0],
        if starting || run.is_some_and(|r| r.state.active()) {
            "C Cancel"
        } else {
            "n Run"
        },
        true,
    );
    if !starting && let Some(run) = run.filter(|r| !r.state.active()) {
        let preview = screen.preview.as_ref().filter(|p| p.id == run.id);
        button(
            frame,
            actions[1],
            match preview {
                Some(p) if p.ready => "b Open preview",
                Some(_) => "Starting…",
                None => "b Preview",
            },
            false,
        );
        button(frame, actions[2], "e Code", false);
        if preview.is_some() {
            button(frame, actions[3], "x Stop", false);
        }
    }
    let lines = if starting && screen.tab != DetailTab::Setup {
        vec![
            Line::default(),
            Line::from(format!("{} Starting the run…", spinner_frame())).fg(GOLD),
        ]
    } else {
        screen_lines(
            &super::details::Content {
                task: screen.tasks.current(),
                run,
                tab: screen.tab,
                details: screen.details,
                note: screen.note,
            },
            parts[3].width,
        )
    };
    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    let last = paragraph
        .line_count(parts[3].width)
        .saturating_sub(parts[3].height as usize)
        .min(u16::MAX as usize) as u16;
    let offset = screen
        .scroll
        .unwrap_or(if screen.tab == DetailTab::Activity {
            last
        } else {
            0
        })
        .min(last);
    frame.render_widget(paragraph.scroll((offset, 0)), parts[3]);
    if last > 0 {
        frame.render_widget(
            Paragraph::new(if offset < last { "↓" } else { "↑" }).fg(MUTED),
            Rect::new(area.right() - 1, parts[3].bottom() - 1, 1, 1),
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
            Modal::Run => "RUN TASK",
            Modal::RunGroup => "RUN ALL TASKS",
            Modal::Reference => "SELECT REFERENCE TASK",
            Modal::PairA => "SELECT A",
            Modal::PairB => "SELECT B",
            Modal::Picker => "COMPARE WITH",
            _ => "KEYBOARD",
        })
        .bg(PANEL),
        area,
    );
    match app.modal {
        Modal::Run | Modal::RunGroup => {
            frame.render_widget(
                Paragraph::new(if app.modal == Modal::RunGroup {
                    app.tasks.group().unwrap_or("No group").to_owned()
                } else {
                    app.focused_task().unwrap_or("No task").to_owned()
                })
                .fg(ACCENT)
                .bold(),
                Rect::new(area.x + 4, area.y + 2, area.width - 8, 1),
            );
            settings_fields(frame, area, &app.settings, app.field);
            let message = if app.modal == Modal::RunGroup {
                "Run all tasks with these settings. Active tasks are skipped.\nEach new run replaces its previous output. Esc cancels this form."
            } else if app.current().is_some() {
                "This run replaces the task's saved output.\nEsc cancels this form."
            } else {
                "Start from the React and Astryx starter.\nThe run copies the current task inputs."
            };
            frame.render_widget(
                Paragraph::new(message).fg(GOLD).wrap(Wrap { trim: false }),
                Rect::new(area.x + 4, area.y + 11, area.width - 8, 3),
            );
            frame.render_widget(
                Paragraph::new(format!(
                    "{}s limit · ↑↓ Field · ←→ Value · Tab Next",
                    app.settings.timeout
                ))
                .fg(MUTED),
                Rect::new(area.x + 4, area.y + 14, area.width - 8, 1),
            );
            button(
                frame,
                modal_submit(area),
                if app.modal == Modal::RunGroup {
                    "Run all tasks  ↵"
                } else {
                    "Start run  ↵"
                },
                true,
            );
        }
        Modal::Picker | Modal::Reference | Modal::PairA | Modal::PairB => {
            frame.render_widget(
                Paragraph::new(format!(
                    "Group: {}",
                    super::tasks::label(app.tasks.group().unwrap_or("—"))
                ))
                .fg(ACCENT),
                Rect::new(area.x + 4, area.y + 2, area.width - 8, 1),
            );
            frame.render_widget(
                Paragraph::new(format!("/ {}", app.picker.query)).fg(MUTED),
                Rect::new(area.x + 4, area.y + 3, area.width - 8, 1),
            );
            let rect = picker_list(area);
            if app.picker.visible().is_empty() {
                frame.render_widget(
                    Paragraph::new(if app.picker.items.is_empty() {
                        "No other variant in this group. Add a task with the same group name."
                    } else {
                        "No variants match this search."
                    })
                    .fg(MUTED)
                    .wrap(Wrap { trim: false }),
                    rect,
                );
            }
            for (offset, row) in app.picker.window(rect.height).iter().enumerate() {
                if let TaskRow::Task { index, line } = row {
                    let task = &app.picker.items[*index];
                    let enabled = task.run.is_some() && !task.cleanup_pending;
                    let chosen = app.picker.current().is_some_and(|t| t.id == task.id);
                    let value = match line {
                        0 => task.id.clone(),
                        1 => {
                            if enabled {
                                task.status().into()
                            } else if task.cleanup_pending {
                                "Cleanup blocked".into()
                            } else {
                                "Run this task first".into()
                            }
                        }
                        _ => String::new(),
                    };
                    frame.render_widget(
                        Paragraph::new(value)
                            .bg(if chosen { SELECTED } else { PANEL })
                            .fg(if enabled { TEXT } else { MUTED }),
                        Rect::new(rect.x, rect.y + offset as u16, rect.width, 1),
                    );
                }
            }
            frame.render_widget(
                Paragraph::new("Type to filter · ↑↓ Select · Enter Compare · Esc Back").fg(MUTED),
                Rect::new(area.x + 4, area.bottom() - 2, area.width - 8, 1),
            );
        }
        _ => {
            let shared = "↑↓ / j k     Select group or task\nEnter / Esc  Enter tasks / Back\n←→ / Tab     Change view (Shift+Tab reverses)\n1 / 2 / 3    Select a visible tab\n/            Search groups and tasks\nPgUp / PgDn  Scroll (Home / End for edges)\nq / Ctrl+C   Quit and stop owned processes\n";
            let specific = if app.selection.comparing {
                "\na / b        Select A / B from this group\ns            Swap A and B\n\nCompare shows saved configuration and metrics."
            } else if app.tasks.is_group() {
                "\nn            Run all tasks in this group\nC            Cancel this group's runs\n\nEach task keeps only its most recent run."
            } else {
                "\nn / C        Run / Cancel this task\nb / x        Open / Stop preview\ne / r / a / f Code / Report / Note / Files\nL            Log in to this provider\nc            Compare with another task in this group"
            };
            let text = format!("{shared}{specific}\n\nEsc Back");
            frame.render_widget(
                Paragraph::new(text).wrap(Wrap { trim: false }),
                area.inner(Margin::new(4, 2)),
            );
        }
    }
}

pub(super) fn settings_fields(
    frame: &mut Frame,
    area: Rect,
    settings: &crate::settings::Settings,
    field: FormField,
) {
    let provider = crate::providers::descriptor(&settings.provider).ok();
    let provider_name = provider.map_or(settings.provider.as_str(), |p| p.label);
    let model_name = provider
        .and_then(|p| p.model(&settings.model))
        .map_or(settings.model.as_str(), |m| m.label);
    let has_effort = provider.is_none_or(|p| !p.efforts(&settings.model).is_empty());
    let effort_name = if has_effort {
        settings.effort.as_str()
    } else {
        "Not supported"
    };
    for (i, label, value) in [
        (0, "Provider", provider_name),
        (1, "Model", model_name),
        (2, "Effort", effort_name),
    ] {
        let rect = modal_field(area, i);
        frame.render_widget(
            Paragraph::new(label).fg(MUTED),
            Rect::new(area.x + 4, rect.y, 10, 1),
        );
        let chosen = i == field as usize;
        frame.render_widget(
            Paragraph::new(value).bg(if chosen { SELECTED } else { BG }),
            rect,
        );
        if i != 2 || has_effort {
            frame.render_widget(
                Paragraph::new("‹  ›").fg(ACCENT),
                Rect::new(rect.right() - 4, rect.y, 4, 1),
            );
        }
    }
}
