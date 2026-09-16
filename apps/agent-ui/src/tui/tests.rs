static EMPTY_ERRORS: std::sync::LazyLock<std::collections::BTreeMap<String, String>> =
    std::sync::LazyLock::new(std::collections::BTreeMap::new);
use super::{
    details::content_lines,
    layout::{action_areas, detail_parts, regions},
    state::DetailTab,
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
            id: "smoke--baseline".into(),
            run: Some(report("a", "smoke--baseline", 1_789_160_000_000)),
            cleanup_pending: false,
        },
        TaskView {
            id: "smoke--context".into(),
            run: Some(report("b", "smoke--context", 1_789_159_000_000)),
            cleanup_pending: false,
        },
        TaskView {
            id: "smoke--check".into(),
            run: None,
            cleanup_pending: false,
        },
    ]);
    tasks
}

#[test]
fn comparison_picker_keeps_only_other_variants_and_includes_missing_runs() {
    let mut tasks = tasks();
    tasks.items.push(TaskView {
        id: "dashboard--baseline".into(),
        run: Some(report("unrelated", "dashboard--baseline", 0)),
        cleanup_pending: false,
    });
    tasks.set_query("baseline".into());
    let mut picker = tasks.comparison_picker("smoke--baseline").unwrap();
    assert_eq!(
        picker
            .items
            .iter()
            .map(|t| t.id.as_str())
            .collect::<Vec<_>>(),
        ["smoke--check", "smoke--context"]
    );
    assert!(picker.current().unwrap().run.is_none());
    picker.navigate(1);
    assert_eq!(picker.current().unwrap().id, "smoke--context");
    picker.set_query("dashboard".into());
    assert!(picker.current().is_none());
    assert!(
        tasks
            .comparison_picker("dashboard--baseline")
            .unwrap()
            .items
            .is_empty()
    );
}

#[test]
fn saved_comparison_restores_only_available_variants_from_the_same_group() {
    let tasks = tasks();
    let mut selection = Selection {
        task: Some("smoke--baseline".into()),
        pair: Some(TaskPair::new("smoke--context".into(), "smoke--baseline".into()).unwrap()),
        comparing: true,
        task_level: true,
    };
    assert!(selection.reconcile(&tasks.items).is_none());
    assert!(selection.comparing);
    assert_eq!(selection.pair.as_ref().unwrap().reference, "smoke--context");

    for other in [
        "dashboard--baseline",
        "smoke--missing",
        "smoke--check",
        "smoke--baseline",
    ] {
        let mut saved: Selection = serde_json::from_value(serde_json::json!({
            "task": "removed--task",
            "pair": {"reference": "smoke--baseline", "other": other},
            "comparing": true
        }))
        .unwrap();
        assert!(saved.reconcile(&tasks.items).is_some());
        assert!(!saved.comparing);
        assert!(saved.pair.is_none());
        assert_eq!(saved.task.as_deref(), Some("smoke--baseline"));
        let reloaded: Selection =
            serde_json::from_value(serde_json::to_value(saved).unwrap()).unwrap();
        assert!(!reloaded.comparing);
        assert!(reloaded.pair.is_none());
    }
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
        selection,
        comparison: None,
        details: None,
        starting: Vec::new(),
        queued: Vec::new(),
        queue_errors: std::sync::LazyLock::force(&EMPTY_ERRORS),
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
    tasks.select_id("smoke--context");
    let mut changed = tasks.items.clone();
    changed
        .iter_mut()
        .find(|t| t.id == "smoke--context")
        .unwrap()
        .run = Some(report("replacement", "smoke--context", 1_800_000_000_000));
    tasks.replace(changed);
    assert_eq!(tasks.current().unwrap().id, "smoke--context");
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
        ["smoke--baseline", "smoke--check", "smoke--context"]
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
    let mut run = Report::new(
        "live".into(),
        "smoke--baseline".into(),
        "unused/app".into(),
        &settings,
    );
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
    items
        .iter_mut()
        .find(|t| t.id == "smoke--baseline")
        .unwrap()
        .run = Some(run);
    let mut tasks = Tasks::default();
    tasks.replace(items);
    tasks.select_id("smoke--baseline");
    let selection = Selection::default();
    let mut screen = screen(&tasks, &selection);
    screen.tab = DetailTab::Activity;
    let output = render(&screen, 120, 40);
    assert!(output.contains("Installing packages"));
    assert!(!output.contains("b Preview"));
}
#[test]
fn a_starting_task_shows_a_starting_placeholder_before_the_report_exists() {
    let mut tasks = tasks();
    tasks.select_id("smoke--check");
    let selection = Selection::default();
    let mut screen = screen(&tasks, &selection);
    screen.starting = vec!["smoke--check".into()];
    let output = render(&screen, 120, 40);
    assert!(output.contains("Starting the run…"));
    assert!(!output.contains("No current run for this task."));
    assert!(!output.contains("n Run this task"));
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
    for id in ["smoke--baseline", "smoke--context"] {
        let mut run = Report::new(id.into(), id.into(), "unused/app".into(), &settings);
        run.state = State::Running;
        run.created_at_ms = crate::report::now().saturating_sub(3_000);
        run.step = Some("Running the agent".into());
        items.iter_mut().find(|t| t.id == id).unwrap().run = Some(run);
    }
    let mut tasks = Tasks::default();
    tasks.replace(items);
    tasks.select_id("smoke--baseline");
    let selection = Selection::default();
    let screen = screen(&tasks, &selection);
    let output = render(&screen, 120, 40);
    assert!(output.contains("2 running"));
    assert!(output.contains("smoke--baseline"));
    assert!(output.contains("smoke--context"));
}
#[test]
fn search_filters_tasks_and_handles_no_matches() {
    let mut tasks = tasks();
    tasks.set_query("CHECK not run".into());
    assert_eq!(tasks.current().unwrap().id, "smoke--check");
    tasks.navigate(1);
    assert_eq!(tasks.current().unwrap().id, "smoke--check");
    tasks.set_query("no-such-task".into());
    assert!(tasks.current().is_none());
    assert!(tasks.window(20).is_empty());
    tasks.set_query(String::new());
    assert_eq!(tasks.current().unwrap().id, "smoke--baseline");
    tasks.navigate(100);
    assert_eq!(tasks.current().unwrap().id, "smoke--context");
}
#[test]
fn small_task_window_keeps_all_three_lines_of_the_selected_task() {
    let mut tasks = tasks();
    tasks.select_id("smoke--context");
    let visible: Vec<_> = tasks
        .window(4)
        .iter()
        .filter_map(|row| match row {
            TaskRow::Task { index, line } => Some((tasks.items[*index].id.as_str(), *line)),
            TaskRow::Gap | TaskRow::Group { .. } | TaskRow::GroupInfo { .. } => None,
        })
        .collect();
    assert_eq!(
        visible,
        [
            ("smoke--context", 0),
            ("smoke--context", 1),
            ("smoke--context", 2)
        ]
    );
}
#[test]
fn task_without_output_exposes_prompt_in_setup_and_has_no_preview() {
    let mut tasks = tasks();
    tasks.select_id("smoke--check");
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
    assert!(output.contains("No saved result"));
    assert!(!output.contains("Build workspace settings."));
    screen.tab = DetailTab::Setup;
    let setup = render(&screen, 100, 32);
    assert!(setup.contains("Build workspace settings."));
    assert!(output.contains("Not run"));
    assert!(!output.contains("e Code"));
    assert!(!output.contains("b Preview"));
}
#[test]
fn overview_shows_measured_values_once_without_source_file_noise() {
    let tasks = tasks();
    let selection = Selection::default();
    let output = render(&screen(&tasks, &selection), 150, 44);
    for value in ["24,308", "14,848", "418", "17.3s", "3.5s", "2.1s"] {
        assert_eq!(output.matches(value).count(), 1, "{value}");
    }
    assert!(!output.contains("src/app.tsx"));
    assert!(!output.contains("src/settings.tsx"));
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
    assert!(output.contains("e Code"));
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
fn comparison_has_its_own_content_and_does_not_use_the_task_tab_or_preview() {
    let tasks = tasks();
    let selection = Selection {
        task: Some("smoke--baseline".into()),
        pair: Some(TaskPair::new("smoke--baseline".into(), "smoke--context".into()).unwrap()),
        comparing: true,
        task_level: true,
    };
    let mut other = tasks.items[2].run.clone().unwrap();
    other.usage.as_mut().unwrap().input_tokens = 30_000;
    let comparison = Comparison::new(tasks.items[0].run.clone().unwrap(), other).unwrap();
    let mut screen = screen(&tasks, &selection);
    screen.comparison = Some(&comparison);
    screen.preview = Some(PreviewInfo {
        id: "b",
        ready: true,
    });
    for tab in DetailTab::ALL {
        screen.tab = tab;
        let output = render(&screen, 150, 44);
        assert_eq!(output.matches("24,308").count(), 1);
        assert_eq!(output.matches("30,000").count(), 1);
        for text in [
            "b Preview",
            "Open preview",
            "Open code",
            "Activity",
            "Evidence",
            "n Run",
            "c Compare",
            "v Side",
            "Change pair",
        ] {
            assert!(!output.contains(text), "{text}: {output}");
        }
        assert!(output.contains("A: Baseline"));
        assert!(output.contains("B: Context"));
        assert!(output.contains("s Swap"));
        assert!(!output.contains("Assess pair"));
    }
}

#[test]
fn cost_values_render_for_single_runs_and_comparisons_at_supported_widths() {
    let mut tasks = tasks();
    for task in &mut tasks.items {
        if let Some(run) = &mut task.run {
            run.model_requested = if task.id == "smoke--baseline" {
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
        task: Some("smoke--baseline".into()),
        pair: Some(TaskPair::new("smoke--baseline".into(), "smoke--context".into()).unwrap()),
        comparing: false,
        task_level: true,
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

fn grouped_tasks() -> Tasks {
    let mut result = Tasks::grouped();
    let mut items = tasks().items;
    for id in ["alpha--first", "alpha--second", "zulu--only"] {
        items.push(TaskView {
            id: id.into(),
            run: None,
            cleanup_pending: false,
        });
    }
    result.replace(items);
    result
}

#[test]
fn group_navigation_requires_entry_and_stays_inside_the_group() {
    let mut tasks = grouped_tasks();
    assert!(tasks.is_group());
    assert_eq!(tasks.group(), Some("alpha"));
    tasks.navigate(1);
    assert_eq!(tasks.group(), Some("smoke"));
    tasks.enter();
    tasks.navigate(1);
    assert_eq!(tasks.current().unwrap().id, "smoke--check");
    tasks.navigate(100);
    assert_eq!(tasks.current().unwrap().id, "smoke--context");
    tasks.navigate(-100);
    assert_eq!(tasks.current().unwrap().id, "smoke--baseline");
    tasks.leave();
    tasks.navigate(1);
    assert_eq!(tasks.group(), Some("zulu"));
    assert!(tasks.is_group());
}

#[test]
fn group_search_and_refresh_keep_membership_and_selection() {
    let mut tasks = grouped_tasks();
    tasks.select_id("smoke--context");
    tasks.enter();
    tasks.set_query("smoke READY".into());
    assert_eq!(tasks.current().unwrap().id, "smoke--context");
    assert_eq!(tasks.members().len(), 3);
    let mut items = tasks.items.clone();
    items.retain(|t| t.id != "smoke--context");
    tasks.replace(items);
    assert_eq!(tasks.current().unwrap().id, "smoke--baseline");
    assert!(!tasks.is_group());
    tasks.set_query("zulu".into());
    assert!(tasks.is_group());
    assert_eq!(tasks.group(), Some("zulu"));
    tasks.set_query("no match".into());
    assert!(tasks.group().is_none());
    assert!(tasks.window(8).is_empty());
}

#[test]
fn mouse_rows_choose_group_or_task_and_small_windows_keep_parent_visible() {
    let mut tasks = grouped_tasks();
    tasks.select_id("smoke--context");
    tasks.enter();
    let rows = tasks.window(3);
    assert_eq!(
        rows,
        [
            TaskRow::Group { index: 2 },
            TaskRow::Task { index: 4, line: 0 },
            TaskRow::Task { index: 4, line: 1 }
        ]
    );
    assert!(tasks.select_row(&rows[0]));
    assert!(tasks.is_group());
    assert_eq!(tasks.group(), Some("smoke"));
    assert!(tasks.select_row(&rows[2]));
    assert!(!tasks.is_group());
    assert_eq!(tasks.current().unwrap().id, "smoke--context");
    assert!(!tasks.select_row(&TaskRow::Gap));
}

#[test]
fn reference_picker_ignores_search_but_stays_inside_selected_group() {
    let mut tasks = grouped_tasks();
    tasks.set_query("smoke context".into());
    let reference = tasks.reference_picker();
    assert_eq!(
        reference
            .items
            .iter()
            .map(|t| t.id.as_str())
            .collect::<Vec<_>>(),
        ["smoke--baseline", "smoke--check", "smoke--context"]
    );
    assert!(!reference.is_grouped());
    let other = tasks.comparison_picker("smoke--context").unwrap();
    assert_eq!(
        other
            .items
            .iter()
            .map(|t| t.id.as_str())
            .collect::<Vec<_>>(),
        ["smoke--baseline", "smoke--check"]
    );
}

#[test]
fn group_overview_shows_all_members_and_pending_runs_do_not_count_as_ready() {
    let mut tasks = grouped_tasks();
    tasks.set_query("smoke baseline".into());
    let selection = Selection::default();
    let mut view = screen(&tasks, &selection);
    view.queued = vec!["smoke--context".into()];
    view.starting = vec!["smoke--baseline".into()];
    for (width, height) in [(76, 24), (120, 36)] {
        let output = render(&view, width, height);
        assert!(output.contains("3 tasks · 0 ready"), "{output}");
        assert!(
            output
                .lines()
                .any(|row| row.contains("Baseline") && row.contains("Running")),
            "{output}"
        );
        assert!(
            output
                .lines()
                .any(|row| row.contains("Context") && row.contains("Queued")),
            "{output}"
        );
        assert!(!output.contains("24,308"));
        assert!(!output.contains("b Preview"));
    }
    let errors = std::collections::BTreeMap::from([(
        "smoke--baseline".into(),
        "Provider is unavailable".into(),
    )]);
    view.starting.clear();
    view.queue_errors = &errors;
    let output = render(&view, 120, 36);
    assert!(
        output
            .lines()
            .any(|row| row.contains("Baseline") && row.contains("Start failed"))
    );
    assert!(output.contains("Provider is unavailable"));
    assert!(output.contains("3 tasks · 0 ready"));
}

#[test]
fn group_comparison_keeps_both_results_and_its_navigation_at_supported_sizes() {
    let mut tasks = grouped_tasks();
    tasks.select_id("smoke--baseline");
    let selection = Selection {
        task: Some("smoke--baseline".into()),
        pair: Some(TaskPair::new("smoke--baseline".into(), "smoke--context".into()).unwrap()),
        comparing: true,
        task_level: false,
    };
    let comparison = Comparison::new(
        tasks.items[2].run.clone().unwrap(),
        tasks.items[4].run.clone().unwrap(),
    )
    .unwrap();
    let mut view = screen(&tasks, &selection);
    view.comparison = Some(&comparison);
    for (width, height) in [(76, 24), (120, 36)] {
        let output = render(&view, width, height);
        assert!(output.contains("Overview"));
        assert!(output.contains("Compare"));
        assert!(!output.contains("b Preview"));
        assert!(!output.contains("n Run task"));
        assert!(!output.contains("c Compare"));
        assert!(!output.contains("Activity"));
        assert!(!output.contains("Evidence"));
        let panel = regions(Rect::new(0, 0, width, height)).1[1];
        let parts = detail_parts(panel);
        for tab in super::tabs::areas(parts[1], true) {
            assert!(!tab.intersects(parts[0]));
            assert!(!tab.intersects(parts[2]));
            assert!(!tab.intersects(parts[3]));
            assert!(tab.right() <= panel.right());
        }
    }
}

#[test]
fn comparison_keys_cannot_trigger_task_or_agent_actions() {
    use super::compare::{Action, key_action};
    use crossterm::event::KeyCode;
    for key in ['n', 'c', 'e', 'r', 'f', 'x', 'v', 'L', 'm', 'C'] {
        assert_eq!(key_action(KeyCode::Char(key)), None, "{key}");
    }
    assert_eq!(key_action(KeyCode::Char('a')), Some(Action::SelectA));
    assert_eq!(key_action(KeyCode::Char('b')), Some(Action::SelectB));
    assert_eq!(key_action(KeyCode::Char('s')), Some(Action::Swap));
    assert_eq!(key_action(KeyCode::PageDown), Some(Action::Scroll(10)));
    assert_eq!(key_action(KeyCode::Esc), Some(Action::Back));
    assert_eq!(key_action(KeyCode::Tab), None);
}

#[test]
fn comparison_mouse_controls_do_not_cover_tabs_or_metrics() {
    use super::compare::{Action, actions, mouse_action};
    for (width, height) in [(76, 24), (120, 36), (180, 50)] {
        let area = regions(Rect::new(0, 0, width, height)).1[1];
        let [title, tabs, toolbar, content] = detail_parts(area);
        for rect in [title, tabs, content] {
            assert_eq!(mouse_action((rect.x, rect.y).into(), area), None);
        }
        let buttons = actions(toolbar);
        for (button, expected) in
            buttons
                .iter()
                .zip([Action::SelectA, Action::SelectB, Action::Swap])
        {
            assert_eq!(
                mouse_action((button.x, button.y).into(), area),
                Some(expected)
            );
            assert!(button.right() <= toolbar.right());
            assert!(!button.intersects(content));
        }
    }
}

#[test]
fn every_tab_uses_arrows_tab_shift_tab_and_numbers_consistently() {
    use super::tabs::key_index;
    use crossterm::event::KeyCode::*;
    for (group, current, forward, back) in [
        (true, 0, 1, 1),
        (true, 1, 0, 0),
        (false, 0, 1, 2),
        (false, 1, 2, 0),
        (false, 2, 0, 1),
    ] {
        for key in [Right, Tab] {
            assert_eq!(key_index(group, current, key), Some(forward));
        }
        for key in [Left, BackTab] {
            assert_eq!(key_index(group, current, key), Some(back));
        }
        assert_eq!(key_index(group, current, Char('1')), Some(0));
        assert_eq!(key_index(group, current, Char('2')), Some(1));
        assert_eq!(
            key_index(group, current, Char('3')),
            if group { None } else { Some(2) }
        );
        for key in [Up, Down, Enter, Esc, Char('n')] {
            assert_eq!(key_index(group, current, key), None);
        }
    }
}

#[test]
fn tab_mouse_targets_match_visible_cells_and_ignore_empty_space() {
    use super::tabs::mouse_index;
    let row = Rect::new(30, 6, 80, 1);
    for group in [true, false] {
        assert_eq!(mouse_index(row, group, (30, 6).into()), Some(0));
        assert_eq!(mouse_index(row, group, (42, 6).into()), Some(1));
        assert_eq!(
            mouse_index(row, group, (54, 6).into()),
            if group { None } else { Some(2) }
        );
        for point in [(41, 6), (80, 6), (30, 5), (30, 7)] {
            assert_eq!(mouse_index(row, group, point.into()), None);
        }
    }
}

fn plain(lines: Vec<ratatui::text::Line<'_>>) -> String {
    lines
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn comparison_keeps_saved_configuration_and_missing_metrics_without_file_inventories() {
    let mut a = report("a", "smoke--baseline", 100);
    let mut b = report("b", "smoke--context", 200);
    a.model_requested = "model-one".into();
    b.model_requested = "model-two".into();
    a.agent_seconds = Some(20.0);
    b.agent_seconds = Some(25.0);
    b.usage = None;
    b.cost_usd = None;
    b.cost_note = "Price not recorded".into();
    let comparison = Comparison::new(a, b).unwrap();
    for width in [40, 100] {
        let output = plain(super::compare::lines(Some(&comparison), width));
        for expected in [
            "model-one",
            "model-two",
            "20.0s",
            "25.0s",
            "+5.0s",
            "24,308",
            "Price not recorded",
            "—",
        ] {
            assert!(output.contains(expected), "{output}");
        }
        for removed in [
            "src/app.tsx",
            "Generated source",
            "Saved task inputs",
            "assessment",
            "Preview",
        ] {
            assert!(!output.contains(removed), "{output}");
        }
    }
}

#[test]
fn setup_separates_saved_settings_from_the_changed_current_prompt() {
    let mut run = report("a", "smoke--baseline", 100);
    run.vp_version = Some("vp v0.3.1\nTools: verbose output".into());
    let details = crate::task_result::TaskDetails {
        prompt: "A changed prompt".into(),
        config: Default::default(),
        files: vec![],
        changed_since_run: true,
    };
    let output = plain(super::details::screen_lines(
        &super::details::Content {
            task: None,
            run: Some(&run),
            tab: DetailTab::Setup,
            details: Some(&details),
            note: "",
        },
        100,
    ));
    for expected in [
        "Saved run configuration",
        "gpt-5.6-luna",
        "60s",
        "Task inputs changed",
        "Current task prompt",
        "A changed prompt",
    ] {
        assert!(output.contains(expected), "{output}");
    }
    assert!(output.contains("Vite+: vp v0.3.1"));
    assert!(!output.contains("verbose output"));
    assert!(!output.contains("created_at_ms"));
    assert!(!output.contains("src/app.tsx"));
}

#[test]
fn sidebar_group_summary_includes_hidden_members_and_uses_the_latest_run_start() {
    use super::sidebar::{latest_start, summary};
    let mut tasks = grouped_tasks();
    tasks.set_query("smoke baseline".into());
    let selection = Selection::default();
    let mut view = screen(&tasks, &selection);
    view.queued.push("smoke--baseline".into());
    view.starting.push("smoke--context".into());
    let (count, text, _) = summary(&view, "smoke");
    assert_eq!(count, 3);
    assert_eq!(text, "1 starting · 1 queued · 1 not run");
    assert_eq!(latest_start(&view, "smoke"), Some(1_789_160_000_000));
    assert_eq!(latest_start(&view, "alpha"), None);
    assert_eq!(summary(&view, "alpha").1, "2 not run");
}

#[test]
fn sidebar_status_uses_live_phases_and_replacement_state_before_saved_results() {
    use super::sidebar::{Status, status};
    let mut tasks = tasks();
    let selection = Selection::default();
    for (state, expected) in [
        (State::Preparing, Status::Preparing),
        (State::Running, Status::Running),
        (State::Verifying, Status::Verifying),
        (State::Failed, Status::Failed),
        (State::Cancelled, Status::Cancelled),
        (State::Interrupted, Status::Interrupted),
    ] {
        tasks.items[0].run.as_mut().unwrap().state = state;
        let view = screen(&tasks, &selection);
        assert_eq!(status(&view, &tasks.items[0]).label(), expected.label());
    }
    tasks.items[0].run.as_mut().unwrap().state = State::Ready;
    let mut view = screen(&tasks, &selection);
    view.starting.push("smoke--baseline".into());
    assert!(status(&view, &tasks.items[0]) == Status::Starting);
    view.queued.push("smoke--baseline".into());
    assert!(status(&view, &tasks.items[0]) == Status::Queued);
    view.starting.clear();
    view.queued.clear();
    let errors =
        std::collections::BTreeMap::from([("smoke--baseline".into(), "Cannot start".into())]);
    view.queue_errors = &errors;
    assert!(status(&view, &tasks.items[0]) == Status::StartFailed);
    tasks.items[0].cleanup_pending = true;
    let view = screen(&tasks, &selection);
    assert!(status(&view, &tasks.items[0]) == Status::Blocked);
}

#[test]
fn sidebar_dates_distinguish_saved_runs_from_queued_replacements() {
    use chrono::{Local, TimeZone};
    let mut tasks = Tasks::grouped();
    let mut run = report("a", "smoke--baseline", 100);
    run.created_at_ms = Local
        .with_ymd_and_hms(2026, 9, 15, 11, 30, 0)
        .unwrap()
        .timestamp_millis() as u64;
    tasks.replace(vec![TaskView {
        id: run.task.clone(),
        run: Some(run),
        cleanup_pending: false,
    }]);
    let selection = Selection::default();
    let mut view = screen(&tasks, &selection);
    for width in [76, 120, 180] {
        let output = render(&view, width, 36);
        let sidebar_width = regions(Rect::new(0, 0, width, 36)).1[0].right() as usize;
        let sidebar = output
            .lines()
            .map(|row| row.chars().take(sidebar_width).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n");
        for expected in [
            "SMOKE",
            "1 ready",
            "Last 15 Sep 26 11:30",
            "Run 15 Sep 26 11:30",
        ] {
            assert!(sidebar.contains(expected), "{sidebar}");
        }
    }
    view.queued.push("smoke--baseline".into());
    let output = render(&view, 76, 36);
    let sidebar = output
        .lines()
        .map(|row| row.chars().take(28).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(sidebar.contains("1 queued"));
    assert!(sidebar.contains("Prev 15 Sep 26 11:30"));
    assert!(!sidebar.contains("Ready"));
}

#[test]
fn sidebar_keeps_group_metadata_and_selected_task_date_visible_after_scrolling() {
    let mut tasks = grouped_tasks();
    tasks.select_id("smoke--context");
    tasks.enter();
    assert_eq!(
        tasks.window(6),
        [
            TaskRow::Group { index: 2 },
            TaskRow::GroupInfo { index: 2, line: 0 },
            TaskRow::GroupInfo { index: 2, line: 1 },
            TaskRow::GroupInfo { index: 2, line: 2 },
            TaskRow::Task { index: 4, line: 0 },
            TaskRow::Task { index: 4, line: 1 },
        ]
    );
    assert!(tasks.select_row(&TaskRow::GroupInfo { index: 2, line: 1 }));
    assert!(tasks.is_group());
    assert_eq!(tasks.group(), Some("smoke"));
}

#[test]
fn sidebar_typography_separates_group_task_metadata_and_selection() {
    use ratatui::style::Modifier;
    let mut tasks = Tasks::grouped();
    let mut items = vec![TaskView {
        id: "invite-member--baseline".into(),
        run: None,
        cleanup_pending: false,
    }];
    for name in ["baseline", "context", "repair"] {
        let id = format!("smoke--{name}");
        items.push(TaskView {
            id: id.clone(),
            run: Some(report(name, &id, 1_789_160_000_000)),
            cleanup_pending: false,
        });
    }
    tasks.replace(items);
    tasks.enter();
    let selection = Selection::default();
    let view = screen(&tasks, &selection);
    let mut terminal = Terminal::new(TestBackend::new(140, 40)).unwrap();
    terminal.draw(|frame| draw_screen(frame, &view)).unwrap();
    let buffer = terminal.backend().buffer();
    let find = |text: &str| {
        for y in 0..40 {
            let row: String = (0..35).map(|x| buffer[(x, y)].symbol()).collect();
            if let Some(byte) = row.find(text) {
                return (row[..byte].chars().count() as u16, y);
            }
        }
        panic!("Missing sidebar text: {text}");
    };
    let group = find("SMOKE");
    let task = find("Context");
    let selected = find("Baseline");
    let date = find("Last ");
    assert!(buffer[group].modifier.contains(Modifier::BOLD));
    assert!(!buffer[task].modifier.contains(Modifier::BOLD));
    assert!(buffer[selected].modifier.contains(Modifier::BOLD));
    assert_ne!(buffer[group].bg, buffer[task].bg);
    assert_ne!(buffer[selected].bg, buffer[task].bg);
    assert_ne!(buffer[date].fg, buffer[task].fg);
    assert_eq!(buffer[(task.0 - 3, task.1)].symbol(), "├");
    let row: String = (task.0..35).map(|x| buffer[(x, task.1)].symbol()).collect();
    assert!(row.contains("Ready"));
    assert_ne!(buffer[(selected.0 - 3, selected.1)].bg, buffer[selected].bg);
}
