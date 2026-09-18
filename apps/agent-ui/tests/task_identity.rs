use agent_ui::task::{TaskId, comparison_group};

#[test]
fn task_names_define_one_group_and_one_variant() {
    assert_eq!(
        TaskId::parse("smoke--baseline").unwrap(),
        TaskId {
            group: "smoke",
            variant: "baseline"
        }
    );
    assert_eq!(
        TaskId::parse("sales-dashboard-v2--extra-context-3").unwrap(),
        TaskId {
            group: "sales-dashboard-v2",
            variant: "extra-context-3"
        }
    );
    assert!(TaskId::parse(&format!("a--{}", "b".repeat(117))).is_ok());
    assert!(TaskId::parse(&format!("a--{}", "b".repeat(118))).is_err());
}

#[test]
fn invalid_names_cannot_be_task_ids() {
    for id in [
        "",
        "smoke",
        "_template",
        "--baseline",
        "smoke--",
        "smoke---baseline",
        "smoke--context--extra",
        "Smoke--baseline",
        "smoke--extra_context",
        "smoke--baseline-",
        "-smoke--baseline",
        "smoke.baseline",
        "../smoke--baseline",
        "smoke--base/line",
        "smoke--base\\line",
        "smoke--base line",
        "smoke--café",
    ] {
        assert!(TaskId::parse(id).is_err(), "{id:?}");
    }
}

#[test]
fn only_different_variants_in_the_exact_same_group_can_be_compared() {
    assert_eq!(
        comparison_group("smoke--baseline", "smoke--context").unwrap(),
        "smoke"
    );
    assert_eq!(
        comparison_group("smoke--context", "smoke--baseline").unwrap(),
        "smoke"
    );
    for (reference, other) in [
        ("smoke--baseline", "smoke--baseline"),
        ("smoke--baseline", "dashboard--baseline"),
        ("smoke--baseline", "smoke-extra--context"),
        ("smoke", "smoke--context"),
        ("smoke--baseline", "smoke---context"),
    ] {
        assert!(comparison_group(reference, other).is_err());
    }
}
