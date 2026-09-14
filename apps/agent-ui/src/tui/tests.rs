use super::{
    details::content_lines,
    layout::{action_areas, detail_parts, regions},
    state::{DetailTab, Side},
    tasks::{TaskRow, Tasks},
    view::{PreviewInfo, Screen, draw_screen},
};
use crate::{
    comparison::Comparison,
    report::{Check, Report, State, Usage},
    settings::Settings,
    task_result::{Selection, TaskPair, TaskView},
};
use ratatui::{
    Terminal,
    backend::TestBackend,
    layout::Rect,
    widgets::{Paragraph, Wrap},
};

fn report(id: &str, task: &str, at: u64) -> Report {
    let settings = Settings {
        model: "gpt-5.6-luna".into(),
        effort: "low".into(),
        timeout: 60,
        provider: "codex".into(),
        binary: None,
    };
    let mut run = Report::new(id.into(), task.into(), "unused/app".into(), &settings);
    run.created_at_ms = at;
    run.state = State::Ready;
    run.setup_seconds = 3.5;
    run.agent_seconds = Some(17.3);
    run.verification = Some(Check {
        exit_code: Some(0),
        seconds: 2.1,
    });
    run.usage = Some(Usage {
        input_tokens: 24_308,
        cached_input_tokens: 14_848,
        output_tokens: 418,
        reasoning_output_tokens: None,
        cache_write_input_tokens: None,
    });
    run.estimate_cost_from_totals();
    run.before.insert("src/app.tsx".into(), "before".into());
    run.after.insert("src/app.tsx".into(), "after".into());
    run.after.insert("src/settings.tsx".into(), "new".into());
    run.changed_files = vec!["src/app.tsx".into(), "src/settings.tsx".into()];
    run
}
fn tasks() -> Tasks {
    let mut tasks = Tasks::default();
    tasks.replace(vec![
        TaskView {
            id: "bare".into(),
            run: Some(report("a", "bare", 1_789_160_000_000)),
            cleanup_pending: false,
        },
        TaskView {
            id: "guided".into(),
            run: Some(report("b", "guided", 1_789_159_000_000)),
            cleanup_pending: false,
        },
        TaskView {
            id: "empty".into(),
            run: None,
            cleanup_pending: false,
        },
    ]);
    tasks
}
fn screen<'a>(tasks: &'a Tasks, selection: &'a Selection) -> Screen<'a> {
    Screen {
        tasks,
        tab: DetailTab::Overview,
        scroll: None,
        searching: false,
        notice: "",
        note: "Added workspace fields and a save confirmation.",
        preview: None,
        active: false,
        selection,
        side: Side::Reference,
        comparison: None,
        assessment: "",
        details: None,
    }
}
fn render(screen: &Screen<'_>, width: u16, height: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| draw_screen(frame, screen)).unwrap();
    terminal
        .backend()
        .buffer()
        .content
        .chunks(width as usize)
        .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn replacing_a_run_keeps_task_selection_and_includes_tasks_without_runs() {
    let mut tasks = tasks();
    tasks.select_id("guided");
    let mut changed = tasks.items.clone();
    changed.iter_mut().find(|t| t.id == "guided").unwrap().run =
        Some(report("replacement", "guided", 1_800_000_000_000));
    tasks.replace(changed);
    assert_eq!(tasks.current().unwrap().id, "guided");
    assert_eq!(
        tasks.current().unwrap().run.as_ref().unwrap().id,
        "replacement"
    );
    assert_eq!(
        tasks
            .items
            .iter()
            .map(|t| t.id.as_str())
            .collect::<Vec<_>>(),
        ["bare", "empty", "guided"]
    );
}
#[test]
fn active_run_shows_a_live_status_with_step_and_elapsed() {
    let settings = Settings {
        model: "gpt-5.6-luna".into(),
        effort: "low".into(),
        timeout: 60,
        provider: "codex".into(),
        binary: None,
    };
    let mut run = Report::new("live".into(), "bare".into(), "unused/app".into(), &settings);
    run.created_at_ms = crate::report::now().saturating_sub(5_000);
    run.step = Some("Installing packages".into());
    run.record("Copied the starter and task inputs");

    let text = content_lines(&run, DetailTab::Activity, "", 100)
        .iter()
        .map(|line| {
            line.spans
                .iter()
                .map(|s| s.content.as_ref())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(text.contains("Copied the starter and task inputs"));
    assert!(text.contains("Installing packages"));
    assert!(text.contains("Preparing"));
    assert!(text.contains("elapsed"));

    let mut items = tasks().items.clone();
    items.iter_mut().find(|t| t.id == "bare").unwrap().run = Some(run);
    let mut tasks = Tasks::default();
    tasks.replace(items);
    tasks.select_id("bare");
    let selection = Selection::default();
    let mut screen = screen(&tasks, &selection);
    screen.tab = DetailTab::Activity;
    screen.active = true;
    let output = render(&screen, 120, 40);
    assert!(output.contains("Installing packages"));
    assert!(!output.contains("b Preview"));
}
#[test]
fn two_active_runs_show_an_aggregate_footer_with_each_task() {
    let settings = Settings {
        model: "gpt-5.6-luna".into(),
        effort: "low".into(),
        timeout: 60,
        provider: "codex".into(),
        binary: None,
    };
    let mut items = tasks().items.clone();
    for id in ["bare", "guided"] {
        let mut run = Report::new(id.into(), id.into(), "unused/app".into(), &settings);
        run.state = State::Running;
        run.created_at_ms = crate::report::now().saturating_sub(3_000);
        run.step = Some("Running the agent".into());
        items.iter_mut().find(|t| t.id == id).unwrap().run = Some(run);
    }
    let mut tasks = Tasks::default();
    tasks.replace(items);
    tasks.select_id("bare");
    let selection = Selection::default();
    let mut screen = screen(&tasks, &selection);
    screen.active = true;
    let output = render(&screen, 120, 40);
    assert!(output.contains("2 running"));
    assert!(output.contains("bare"));
    assert!(output.contains("guided"));
}
#[test]
fn search_filters_tasks_and_handles_no_matches() {
    let mut tasks = tasks();
    tasks.set_query("EMPTY not run".into());
    assert_eq!(tasks.current().unwrap().id, "empty");
    tasks.navigate(1);
    assert_eq!(tasks.current().unwrap().id, "empty");
    tasks.set_query("no-such-task".into());
    assert!(tasks.current().is_none());
    assert!(tasks.window(20).is_empty());
    tasks.set_query(String::new());
    assert_eq!(tasks.current().unwrap().id, "bare");
    tasks.navigate(100);
    assert_eq!(tasks.current().unwrap().id, "guided");
}
#[test]
fn small_task_window_keeps_all_three_lines_of_the_selected_task() {
    let mut tasks = tasks();
    tasks.select_id("guided");
    let visible: Vec<_> = tasks
        .window(4)
        .iter()
        .filter_map(|row| match row {
            TaskRow::Task { index, line } => Some((tasks.items[*index].id.as_str(), *line)),
            TaskRow::Gap => None,
        })
        .collect();
    assert_eq!(visible, [("guided", 0), ("guided", 1), ("guided", 2)]);
}
#[test]
fn task_without_output_shows_prompt_and_no_preview_actions() {
    let mut tasks = tasks();
    tasks.select_id("empty");
    let selection = Selection::default();
    let mut screen = screen(&tasks, &selection);
    let details = crate::task_result::TaskDetails {
        prompt: "Build workspace settings.".into(),
        config: Default::default(),
        files: vec!["task.md".into()],
        changed_since_run: false,
    };
    screen.details = Some(&details);
    let output = render(&screen, 100, 32);
    assert!(output.contains("Build workspace settings."));
    assert!(output.contains("Not run"));
    assert!(!output.contains("e Open code"));
    assert!(!output.contains("b Preview"));
}
#[test]
fn overview_shows_measured_values_once_and_classifies_source_changes() {
    let tasks = tasks();
    let selection = Selection::default();
    let output = render(&screen(&tasks, &selection), 150, 44);
    for value in ["24,308", "14,848", "418", "17.3s", "3.5s", "2.1s"] {
        assert_eq!(output.matches(value).count(), 1, "{value}");
    }
    assert!(output.contains("M  src/app.tsx"));
    assert!(output.contains("A  src/settings.tsx"));
    assert!(output.contains("Added workspace fields and a save confirmation."));
    assert!(!output.contains("Human review"));
}
#[test]
fn narrow_overview_scrolls_to_the_note_while_actions_stay_visible() {
    let tasks = tasks();
    let selection = Selection::default();
    let mut screen = screen(&tasks, &selection);
    screen.scroll = Some(u16::MAX);
    let output = render(&screen, 76, 24);
    assert!(output.contains("a Open full note"));
    assert!(output.contains("e Open code"));
    assert!(output.contains("b Preview"));
    assert!(!output.contains("24,308"));
}
#[test]
fn a_preview_for_another_run_does_not_look_ready_for_the_selected_run() {
    let tasks = tasks();
    let selection = Selection::default();
    let mut screen = screen(&tasks, &selection);
    screen.preview = Some(PreviewInfo {
        id: "b",
        ready: true,
    });
    let output = render(&screen, 150, 44);
    assert!(output.contains("b Preview"));
    assert!(!output.contains("b Open preview"));
    screen.preview = Some(PreviewInfo {
        id: "a",
        ready: false,
    });
    let output = render(&screen, 150, 44);
    assert!(output.contains("Starting…"));
    assert!(output.contains("x Stop"));
    assert!(!output.contains("b Open preview"));
}
#[test]
fn action_targets_do_not_overlap_tabs_or_content_at_supported_sizes() {
    for (width, height) in [(76, 24), (100, 32), (150, 44)] {
        let parts = detail_parts(regions(Rect::new(0, 0, width, height)).1[1]);
        let actions = action_areas(parts[2]);
        for rect in actions {
            assert!(parts[2].contains((rect.x, rect.y).into()));
            assert!(rect.right() <= parts[2].right());
            assert!(!rect.intersects(parts[0]));
            assert!(!rect.intersects(parts[3]));
        }
        assert!(!actions[0].intersects(actions[1]));
        assert!(!actions[1].intersects(actions[2]));
    }
}
#[test]
fn long_errors_and_activity_are_included_in_the_scroll_extent() {
    let mut run = report("failure", "workspace-settings", 1_789_160_000_000);
    run.error = Some("A detailed failure message. ".repeat(15));
    run.record("A command with many arguments. ".repeat(20));
    for tab in [DetailTab::Overview, DetailTab::Activity] {
        let lines = content_lines(&run, tab, "", 38);
        let unwrapped = lines.len();
        let wrapped = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .line_count(38);
        assert!(wrapped > unwrapped);
    }
}

#[test]
fn comparison_side_selects_the_correct_preview_and_keeps_values_distinct() {
    let tasks = tasks();
    let selection = Selection {
        task: Some("bare".into()),
        pair: Some(TaskPair::new("bare".into(), "guided".into()).unwrap()),
        comparing: true,
    };
    let mut comparison = Comparison::new(
        tasks.items[0].run.clone().unwrap(),
        tasks.items[2].run.clone().unwrap(),
    )
    .unwrap();
    comparison.other.usage.as_mut().unwrap().input_tokens = 30_000;
    comparison = Comparison::new(comparison.reference, comparison.other).unwrap();
    let mut screen = screen(&tasks, &selection);
    screen.comparison = Some(&comparison);
    screen.preview = Some(PreviewInfo {
        id: "b",
        ready: true,
    });
    let output = render(&screen, 150, 44);
    assert!(output.contains("b Preview"));
    assert!(!output.contains("b Open preview"));
    assert_eq!(output.matches("24,308").count(), 1);
    assert_eq!(output.matches("30,000").count(), 1);
    screen.side = Side::Other;
    assert!(render(&screen, 150, 44).contains("b Open preview"));
    screen.tab = DetailTab::Evidence;
    let output = render(&screen, 150, 44);
    assert!(output.contains("\"id\": \"b\""));
    assert!(!output.contains("\"id\": \"a\""));
}

#[test]
fn cost_values_render_for_single_runs_and_comparisons_at_supported_widths() {
    let mut tasks = tasks();
    for task in &mut tasks.items {
        if let Some(run) = &mut task.run {
            run.model_requested = if task.id == "bare" {
                "gpt-5.6-sol"
            } else {
                "gpt-5.6-luna"
            }
            .into();
            run.usage = Some(Usage {
                input_tokens: 100_000,
                cached_input_tokens: 60_000,
                output_tokens: 10_000,
                reasoning_output_tokens: None,
                cache_write_input_tokens: None,
            });
            run.estimate_cost_from_totals();
        }
    }
    let mut selection = Selection {
        task: Some("bare".into()),
        pair: Some(TaskPair::new("bare".into(), "guided".into()).unwrap()),
        comparing: false,
    };
    for (width, height) in [(76, 24), (150, 44)] {
        let output = render(&screen(&tasks, &selection), width, height);
        assert!(output.contains("$0.53"), "{output}");
    }
    let comparison = Comparison::new(
        tasks.items[0].run.clone().unwrap(),
        tasks.items[2].run.clone().unwrap(),
    )
    .unwrap();
    selection.comparing = true;
    for (width, height) in [(76, 24), (150, 44)] {
        let mut view = screen(&tasks, &selection);
        view.comparison = Some(&comparison);
        let output = render(&view, width, height);
        for value in ["$0.53", "$0.02", "−$0.51"] {
            assert!(output.contains(value), "{output}");
        }
    }
}

#[test]
fn provider_form_keeps_choices_and_mouse_targets_in_sync() {
    use super::{
        layout::{modal_field, modal_rect, modal_submit},
        state::FormField,
        view::settings_fields,
    };
    let mut settings = Settings::default();
    settings.step_provider(1);
    settings.step_model(1);
    settings.step_effort(1);
    assert_eq!(
        (&*settings.provider, &*settings.model, &*settings.effort),
        ("claude", "claude-opus-5", "xhigh")
    );
    for (width, height) in [(76, 24), (120, 36)] {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        let area = modal_rect(Rect::new(0, 0, width, height));
        terminal
            .draw(|frame| settings_fields(frame, area, &settings, FormField::Effort))
            .unwrap();
        let buffer = terminal.backend().buffer();
        for (i, value) in ["Anthropic / Claude Code", "Opus 5", "xhigh"]
            .into_iter()
            .enumerate()
        {
            let field = modal_field(area, i);
            let text: String = (field.x..field.right())
                .map(|x| buffer[(x, field.y)].symbol())
                .collect();
            assert!(text.starts_with(value), "{text}");
            assert!(field.y < area.y + 11);
            assert!(!field.intersects(modal_submit(area)));
        }
        assert_eq!(FormField::Provider.step(1), FormField::Model);
        assert_eq!(FormField::Model.step(1), FormField::Effort);
        assert_eq!(FormField::Effort.step(1), FormField::Provider);
    }
}

#[test]
fn form_arrow_keys_separate_field_navigation_from_value_selection() {
    use super::state::FormField;
    use crossterm::event::KeyCode;
    let mut settings = Settings::default();
    let mut field = FormField::Provider;
    field.select(KeyCode::Down, &mut settings);
    assert_eq!(field, FormField::Model);
    assert_eq!(settings.model, "gpt-5.6-luna");
    field.select(KeyCode::Up, &mut settings);
    assert_eq!(field, FormField::Provider);
    field.select(KeyCode::Up, &mut settings);
    assert_eq!(field, FormField::Effort);
    assert_eq!(settings.effort, "low");
    field.select(KeyCode::Right, &mut settings);
    assert_eq!(field, FormField::Effort);
    assert_eq!(settings.effort, "medium");
    field.select(KeyCode::Tab, &mut settings);
    field.select(KeyCode::Right, &mut settings);
    assert_eq!(
        (&*settings.provider, &*settings.model, &*settings.effort),
        ("claude", "claude-sonnet-5", "high")
    );
    field.select(KeyCode::Down, &mut settings);
    field.select(KeyCode::Right, &mut settings);
    assert_eq!(field, FormField::Model);
    assert_eq!(settings.model, "claude-opus-5");
    field.select(KeyCode::Char('x'), &mut settings);
    field.select(KeyCode::Backspace, &mut settings);
    assert_eq!(settings.model, "claude-opus-5");
    field.select(KeyCode::Left, &mut settings);
    assert_eq!(settings.model, "claude-sonnet-5");
    field.select(KeyCode::BackTab, &mut settings);
    assert_eq!(field, FormField::Provider);
    field.select(KeyCode::Left, &mut settings);
    assert_eq!(
        (&*settings.provider, &*settings.model, &*settings.effort),
        ("codex", "gpt-5.6-luna", "low")
    );
}

#[test]
fn models_without_effort_have_no_value_control() {
    use super::{
        layout::{modal_field, modal_rect},
        state::FormField,
        view::settings_fields,
    };
    use crossterm::event::KeyCode;
    let mut settings = Settings::for_provider("claude").unwrap();
    settings.select_model("haiku").unwrap();
    let mut field = FormField::Effort;
    field.select(KeyCode::Right, &mut settings);
    assert_eq!(settings.effort, "default");
    for (width, height) in [(76, 24), (120, 36)] {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        let area = modal_rect(Rect::new(0, 0, width, height));
        terminal
            .draw(|frame| settings_fields(frame, area, &settings, field))
            .unwrap();
        let rect = modal_field(area, 2);
        let row: String = (rect.x..rect.right())
            .map(|x| terminal.backend().buffer()[(x, rect.y)].symbol())
            .collect();
        assert!(row.starts_with("Not supported"));
        assert!(!row.contains('‹') && !row.contains('›'));
    }
}
