use agent_ui::{
    comparison::{ChangeKind, Comparison, file_changes},
    report::{Report, State, Usage},
    settings::{Effort, Settings},
    task_result::TaskPair,
};
use std::collections::BTreeMap;

fn report(id: &str, task: &str) -> Report {
    Report::new(
        id.into(),
        task.into(),
        "unused/app".into(),
        &Settings {
            model: "test".into(),
            effort: Effort::Low,
            timeout: 10,
            codex: None,
        },
    )
}
#[test]
fn comparison_keeps_missing_values_distinct_from_zero_and_uses_signed_differences() {
    let mut a = report("one", "bare");
    let mut b = report("two", "guided");
    a.agent_seconds = Some(12.0);
    b.agent_seconds = Some(9.5);
    a.usage = Some(Usage {
        input_tokens: u64::MAX,
        cached_input_tokens: 100,
        output_tokens: 0,
        reasoning_output_tokens: None,
    });
    let missing = Comparison::new(a.clone(), b.clone()).unwrap();
    assert_eq!(missing.measurements.agent_seconds.difference, Some(-2.5));
    assert!(missing.measurements.output_tokens.difference.is_none());
    assert_eq!(missing.measurements.output_tokens.reference, Some(0));
    assert!(missing.measurements.output_tokens.other.is_none());
    assert!(missing.measurements.setup_seconds.reference.is_none());
    b.usage = Some(Usage {
        input_tokens: 0,
        cached_input_tokens: 50,
        output_tokens: 40,
        reasoning_output_tokens: None,
    });
    let measured = Comparison::new(a, b).unwrap();
    assert_eq!(
        measured.measurements.input_tokens.difference,
        Some(-i128::from(u64::MAX))
    );
    assert_eq!(
        measured.measurements.cached_input_tokens.difference,
        Some(-50)
    );
    assert_eq!(measured.measurements.output_tokens.difference, Some(40));
}
#[test]
fn incomplete_snapshots_do_not_claim_no_changes() {
    let mut a = report("one", "bare");
    let mut b = report("two", "guided");
    a.state = State::Ready;
    b.state = State::Failed;
    a.inputs.insert("task.md".into(), "prompt".into());
    let incomplete = Comparison::new(a.clone(), b.clone()).unwrap();
    assert!(incomplete.input_changes.is_none());
    assert!(incomplete.source_changes.is_none());
    assert!(incomplete.can_assess());
    b.inputs = a.inputs.clone();
    b.inputs.insert("AGENTS.md".into(), "instructions".into());
    let complete = Comparison::new(a, b).unwrap();
    let changes = complete.input_changes.unwrap();
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].path, "AGENTS.md");
    assert!(matches!(changes[0].kind, ChangeKind::Added));
}
#[test]
fn file_comparison_distinguishes_added_removed_changed_and_equal_files() {
    let a = BTreeMap::from([
        ("keep".into(), "1".into()),
        ("edit".into(), "2".into()),
        ("remove".into(), "3".into()),
    ]);
    let b = BTreeMap::from([
        ("keep".into(), "1".into()),
        ("edit".into(), "4".into()),
        ("add".into(), "5".into()),
    ]);
    let changes = file_changes(&a, &b);
    assert_eq!(
        changes.iter().map(|c| c.path.as_str()).collect::<Vec<_>>(),
        ["add", "edit", "remove"]
    );
    assert!(matches!(changes[0].kind, ChangeKind::Added));
    assert!(matches!(changes[1].kind, ChangeKind::Modified));
    assert!(matches!(changes[2].kind, ChangeKind::Removed));
}
#[test]
fn assessments_bind_two_different_tasks_and_the_exact_run_pair() {
    assert!(Comparison::new(report("one", "bare"), report("two", "bare")).is_err());
    assert!(TaskPair::new("bare".into(), "bare".into()).is_err());
    let pair = Comparison::new(report("one", "bare"), report("two", "guided"))
        .unwrap()
        .pair;
    let mut current = BTreeMap::from([
        ("bare".into(), "one".into()),
        ("guided".into(), "two".into()),
    ]);
    current.insert("unrelated".into(), "new".into());
    assert!(pair.is_current(&current));
    current.insert("bare".into(), "replacement".into());
    assert!(!pair.is_current(&current));
    current.remove("bare");
    assert!(!pair.is_current(&current));
}
#[test]
fn active_task_blocks_assessment_but_keeps_partial_measurements_available() {
    let mut a = report("one", "bare");
    a.state = State::Ready;
    let b = report("two", "guided");
    let comparison = Comparison::new(a, b).unwrap();
    assert!(!comparison.can_assess());
    assert!(comparison.measurements.setup_seconds.difference.is_none());
}
