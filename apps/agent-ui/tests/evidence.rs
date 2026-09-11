use agent_ui::{
    report::Report,
    settings::{Effort, Settings},
    storage::valid_id,
};
use serde_json::json;
use std::collections::BTreeMap;

fn settings() -> Settings {
    Settings {
        model: "test-model".into(),
        effort: Effort::Low,
        timeout: 60,
        codex: None,
    }
}

fn report() -> Report {
    Report::new(
        "run-1".into(),
        "task-1".into(),
        "unused/app".into(),
        &settings(),
    )
}

#[test]
fn token_totals_add_turns_without_adding_cached_or_reasoning_tokens_twice() {
    let mut report = report();
    for event in [
        r#"{"type":"thread.started","thread_id":"thread-42"}"#,
        r#"{"type":"turn.completed","usage":{"input_tokens":100,"cached_input_tokens":60,"output_tokens":20,"reasoning_output_tokens":8}}"#,
        r#"{"type":"turn.completed","usage":{"input_tokens":40,"cached_input_tokens":10,"output_tokens":7,"reasoning_output_tokens":2}}"#,
    ] {
        report.event(event.as_bytes());
    }

    let saved = serde_json::to_value(&report).unwrap();
    assert_eq!(
        saved["usage"],
        json!({
            "input_tokens": 140,
            "cached_input_tokens": 70,
            "output_tokens": 27,
            "reasoning_output_tokens": 10,
        })
    );
    assert_eq!(report.thread_id.as_deref(), Some("thread-42"));
    assert_eq!(report.completed_turns, 2);
    assert!(report.check_agent_completion().is_ok());
}

#[test]
fn absent_or_invalid_usage_does_not_become_zero_in_the_report() {
    for event in [
        r#"{"type":"turn.completed"}"#,
        r#"{"type":"turn.completed","usage":{"input_tokens":12,"output_tokens":3}}"#,
        r#"{"type":"turn.completed","usage":{"input_tokens":-1,"cached_input_tokens":0,"output_tokens":3}}"#,
        r#"{"type":"turn.completed","usage":{"input_tokens":"12","cached_input_tokens":0,"output_tokens":3}}"#,
    ] {
        let mut report = report();
        report.event(event.as_bytes());
        let saved = serde_json::to_value(&report).unwrap();
        assert_eq!(saved["usage"], json!(null), "{event}");
        assert_eq!(saved["cost_usd"], json!(null));
        assert!(report.check_agent_completion().is_ok());
    }
}

#[test]
fn reported_zero_usage_remains_distinct_from_absent_usage() {
    let mut report = report();
    report.event(br#"{"type":"turn.completed","usage":{"input_tokens":0,"cached_input_tokens":0,"output_tokens":0}}"#);
    assert_eq!(
        serde_json::to_value(&report).unwrap()["usage"],
        json!({
            "input_tokens": 0,
            "cached_input_tokens": 0,
            "output_tokens": 0,
            "reasoning_output_tokens": null,
        })
    );
}

#[test]
fn malformed_events_block_completion_even_after_a_success_event() {
    let mut report = report();
    report.event(b"{broken");
    report.event(br#"{"type":"turn.completed"}"#);
    assert_eq!(report.invalid_event_lines, 1);
    assert!(report.check_agent_completion().is_err());
}

#[test]
fn an_empty_or_unfinished_stream_cannot_pass_completion() {
    let mut report = report();
    assert!(report.check_agent_completion().is_err());
    report.event(br#"{"type":"thread.started","thread_id":"unfinished"}"#);
    report.event(br#"{"type":"item.completed","item":{"type":"agent_message","text":"Done"}}"#);
    assert!(report.check_agent_completion().is_err());
}

#[test]
fn a_later_completion_does_not_erase_a_provider_failure() {
    for failure in [
        r#"{"type":"turn.failed","error":{"message":"Model unavailable"}}"#,
        r#"{"type":"error","message":"Model unavailable"}"#,
    ] {
        let mut report = report();
        report.event(failure.as_bytes());
        report.event(br#"{"type":"turn.completed"}"#);
        assert_eq!(report.error.as_deref(), Some("Model unavailable"));
        assert!(report.check_agent_completion().is_err());
    }
}

#[test]
fn warning_items_remain_visible_without_blocking_completion() {
    let mut report = report();
    report.event(br#"{"type":"item.completed","item":{"type":"error","message":"Feature under development"}}"#);
    report.event(br#"{"type":"turn.completed"}"#);
    assert_eq!(report.warnings, ["Feature under development"]);
    assert!(
        report
            .activity
            .iter()
            .any(|item| item.text.contains("Feature under development"))
    );
    assert!(report.check_agent_completion().is_ok());
}

#[test]
fn unknown_event_types_are_counted_without_discarding_known_events() {
    let mut report = report();
    report.event(br#"{"type":"future.event","payload":{"anything":true}}"#);
    report.event(br#"{"type":"turn.completed"}"#);
    report.event(br#"{"type":"future.event"}"#);
    assert_eq!(
        report.event_counts,
        BTreeMap::from([("future.event".into(), 2), ("turn.completed".into(), 1),])
    );
    assert_eq!(report.invalid_event_lines, 0);
    assert!(report.check_agent_completion().is_ok());
}

#[test]
fn source_changes_include_additions_edits_and_deletions_once() {
    let mut report = report();
    report.before = BTreeMap::from([
        ("removed.ts".into(), "hash-old".into()),
        ("edited.ts".into(), "hash-before".into()),
        ("same.ts".into(), "hash-same".into()),
    ]);
    report.record_source_changes(BTreeMap::from([
        ("added.ts".into(), "hash-new".into()),
        ("edited.ts".into(), "hash-after".into()),
        ("same.ts".into(), "hash-same".into()),
    ]));
    assert_eq!(
        report.changed_files,
        ["added.ts", "edited.ts", "removed.ts"]
    );

    report.record_source_changes(report.before.clone());
    assert!(
        report.changed_files.is_empty(),
        "Restored files must clear the previous changes"
    );
}

#[test]
fn task_ids_reject_paths_and_accept_normal_folder_names() {
    for id in ["workspace-settings", "run_42", "Smoke"] {
        assert!(valid_id(id).is_ok(), "{id}");
    }
    for id in [
        "",
        ".",
        "..",
        "../outside",
        "/tmp/task",
        "a/b",
        "a\\b",
        "a\0b",
        "a b",
    ] {
        assert!(valid_id(id).is_err(), "{id:?}");
    }
    assert!(valid_id(&"a".repeat(120)).is_ok());
    assert!(valid_id(&"a".repeat(121)).is_err());
}

#[test]
fn invalid_run_settings_fail_validation_without_starting_a_run() {
    let mut settings = settings();
    assert!(settings.validate().is_ok());
    settings.model = " \t\n".into();
    assert!(settings.validate().is_err());
    settings.model = "test-model".into();
    settings.timeout = 0;
    assert!(settings.validate().is_err());
}
