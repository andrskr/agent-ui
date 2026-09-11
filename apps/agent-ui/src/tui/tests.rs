use super::{
    details::content_lines,
    history::{History, HistoryRow},
    layout::{action_areas, detail_parts, regions},
    state::DetailTab,
    view::{PreviewInfo, Screen, draw_screen},
};
use crate::{
    report::{Check, Report, State, Usage},
    settings::{Effort, Settings},
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
        effort: Effort::Low,
        timeout: 60,
        codex: None,
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
    });
    run.before.insert("src/app.tsx".into(), "before".into());
    run.after.insert("src/app.tsx".into(), "after".into());
    run.after.insert("src/settings.tsx".into(), "new".into());
    run.changed_files = vec!["src/app.tsx".into(), "src/settings.tsx".into()];
    run
}
fn history() -> History {
    let mut history = History::default();
    let first = report("a", "workspace-settings", 1_789_160_000_000);
    let mut second = report("b", "workspace-settings", 1_789_159_000_000);
    second.state = State::Failed;
    second.effort_requested = "medium".into();
    history.replace(vec![second, first, report("c", "smoke", 1_789_000_000_000)]);
    history
}
fn screen(history: &History) -> Screen<'_> {
    Screen {
        history,
        tab: DetailTab::Overview,
        scroll: None,
        searching: false,
        notice: "",
        note: "Added workspace fields and a save confirmation.",
        preview: None,
        active: false,
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
fn new_runs_and_updates_do_not_move_the_selected_run() {
    let mut history = history();
    history.select_id("b");
    let mut changed = history.runs.clone();
    changed.push(report("new", "smoke", 1_800_000_000_000));
    history.replace(changed);
    assert_eq!(history.current().unwrap().id, "b");
    assert_eq!(history.runs[0].id, "new");
}
#[test]
fn search_filters_navigation_and_clears_a_stale_selection() {
    let mut history = history();
    history.set_query("WORKSPACE failed medium".into());
    assert_eq!(history.current().unwrap().id, "b");
    history.navigate(1);
    assert_eq!(history.current().unwrap().id, "b");
    history.set_query("no-such-task".into());
    assert!(history.current().is_none());
    assert!(history.window(20).is_empty());
    history.set_query(String::new());
    assert_eq!(history.current().unwrap().id, "a");
    history.navigate(100);
    assert_eq!(history.current().unwrap().id, "c");
}
#[test]
fn a_small_history_window_keeps_the_selected_item_and_its_date() {
    let mut history = history();
    history.select_id("c");
    let rows = history.window(4);
    assert!(matches!(&rows[0], HistoryRow::Date(_)));
    let visible: Vec<_> = rows
        .iter()
        .filter_map(|row| match row {
            HistoryRow::Run { index, line } => Some((history.runs[*index].id.as_str(), *line)),
            _ => None,
        })
        .collect();
    assert_eq!(visible, [("c", 0), ("c", 1), ("c", 2)]);
}
#[test]
fn overview_shows_measured_values_once_and_classifies_source_changes() {
    let history = history();
    let output = render(&screen(&history), 150, 44);
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
    let history = history();
    let mut screen = screen(&history);
    screen.scroll = Some(u16::MAX);
    let output = render(&screen, 76, 24);
    assert!(output.contains("a Open full note"));
    assert!(output.contains("e Open code"));
    assert!(output.contains("b Preview"));
    assert!(!output.contains("24,308"));
}
#[test]
fn a_preview_for_another_run_does_not_look_ready_for_the_selected_run() {
    let history = history();
    let mut screen = screen(&history);
    screen.preview = Some(PreviewInfo {
        id: "b",
        ready: true,
    });
    let output = render(&screen, 150, 44);
    assert!(output.contains("b Start preview"));
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
        let actions = action_areas(parts[1]);
        for rect in actions {
            assert!(parts[1].contains((rect.x, rect.y).into()));
            assert!(rect.right() <= parts[1].right());
            assert!(!rect.intersects(parts[0]));
            assert!(!rect.intersects(parts[2]));
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
fn timestamp_style_does_not_hide_state_colours() {
    let history = history();
    let mut terminal = Terminal::new(TestBackend::new(150, 44)).unwrap();
    terminal
        .draw(|frame| draw_screen(frame, &screen(&history)))
        .unwrap();
    let row = terminal
        .backend()
        .buffer()
        .content
        .chunks(150)
        .find(|row| {
            row.iter()
                .map(|cell| cell.symbol())
                .collect::<String>()
                .contains("● Ready")
        })
        .unwrap();
    let status = row.iter().position(|cell| cell.symbol() == "●").unwrap();
    let clock = row.iter().position(|cell| cell.symbol() == ":").unwrap();
    assert_ne!(row[status].fg, row[clock].fg);
}
