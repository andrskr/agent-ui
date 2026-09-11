use agent_ui::{report::Report, task::TaskConfig};
use serde_json::json;

#[test]
fn package_overrides_keep_the_rest_of_the_starter_and_do_not_mutate_the_input() {
    let config = TaskConfig::parse(
        r#"
[dependencies]
zod = "4.1.0"
react = "19.3.0"
[dev-dependencies]
"@astryxdesign/cli" = "0.5.4"
"@types/react" = "19.3.0"
"#,
    )
    .unwrap();
    let original = json!({
        "name": "starter", "scripts": {"dev": "vp dev"}, "packageManager": "pnpm@12.4.1",
        "dependencies": {"react": "19.2.0", "@astryxdesign/core": "0.5.4", "@types/react": "19.2.0"},
        "devDependencies": {"typescript": "7.0.2"}
    });
    let result = config.apply(&original).unwrap();
    assert_eq!(
        result,
        json!({
            "name": "starter", "scripts": {"dev": "vp dev"}, "packageManager": "pnpm@12.4.1",
            "dependencies": {"react": "19.3.0", "@astryxdesign/core": "0.5.4", "zod": "4.1.0"},
            "devDependencies": {"typescript": "7.0.2", "@astryxdesign/cli": "0.5.4", "@types/react": "19.3.0"}
        })
    );
    assert_eq!(original["dependencies"]["react"], "19.2.0");
    assert!(
        original["devDependencies"]
            .get("@astryxdesign/cli")
            .is_none()
    );
}

#[test]
fn an_empty_config_keeps_the_bare_starter() {
    let config = TaskConfig::parse("# Use the bare starter\n").unwrap();
    let manifest = json!({"dependencies": {"react": "19.3.0"}, "scripts": {"verify": "vp check"}});
    assert_eq!(config.apply(&manifest).unwrap(), manifest);
    assert!(!config.has_packages());
}

#[test]
fn invalid_package_settings_fail_instead_of_silently_changing_the_experiment() {
    for source in [
        "[packages]\nzod = '4.1.0'",
        "[dependencies]\nzod = 4",
        "[dependencies]\nzod = '^4.1.0'",
        "[dependencies]\nzod = 'latest'",
        "[dependencies]\nzod = 'file:../outside'",
        "[dependencies]\nzod = ''",
        "[dependencies]\n'../outside' = '1.0.0'",
        "[dependencies]\n'@scope' = '1.0.0'",
        "[dependencies]\n'-flag' = '1.0.0'",
        "[dependencies]\nzod = '4.1.0'\n[dev-dependencies]\nzod = '4.1.0'",
    ] {
        assert!(TaskConfig::parse(source).is_err(), "{source}");
    }
}

#[test]
fn a_malformed_starter_is_not_replaced_with_an_empty_manifest() {
    let config = TaskConfig::parse("[dependencies]\nzod = '4.1.0'").unwrap();
    assert!(config.apply(&json!([])).is_err());
    assert!(config.apply(&json!({"dependencies": []})).is_err());
    assert!(config.apply(&json!({"devDependencies": "bad"})).is_err());
}

#[test]
fn older_reports_load_without_claiming_a_recorded_task_config() {
    let legacy = r#"{
        "schema_version": 1, "id": "legacy-run", "task": "smoke",
        "state": "ready", "created_at_ms": 1789157900000, "finished_at_ms": 1789157920000,
        "model_requested": "gpt-5.6-luna", "effort_requested": "low",
        "codex_version": "codex-cli 0.154.0", "node_version": "v24.21.0",
        "vp_version": "vp v0.3.0", "runner_version": "0.1.0", "thread_id": "legacy-thread",
        "timeout_seconds": 300, "setup_seconds": 3.0, "agent_seconds": 15.0,
        "agent_started_at_ms": 1789157903000, "agent_exit_code": 0,
        "verification": {"exit_code": 0, "seconds": 2.0},
        "usage": {"input_tokens": 100, "cached_input_tokens": 60, "output_tokens": 8},
        "completed_turns": 1, "invalid_event_lines": 0,
        "event_counts": {"turn.completed": 1}, "error": null, "activity": [],
        "inputs": {}, "before": {}, "after": {}, "changed_files": ["src/app.tsx"],
        "app": "/saved/legacy-run/app", "cost_usd": null, "cost_note": "Subscription use.",
        "human_review": "pending", "isolation": "Fresh HOME and CODEX_HOME"
    }"#;
    let loaded: Report = serde_json::from_str(legacy).unwrap();
    assert!(loaded.task_config.is_none());
    assert!(loaded.warnings.is_empty());
    assert_eq!(loaded.state, agent_ui::report::State::Ready);
    assert_eq!(loaded.usage.unwrap().cached_input_tokens, 60);
    assert_eq!(loaded.changed_files, ["src/app.tsx"]);
}

#[test]
fn build_permissions_preserve_other_workspace_settings() {
    let config = TaskConfig::parse(
        r#"
[dev-dependencies]
"@astryxdesign/cli" = "0.5.4"
[allow-builds]
"@astryxdesign/cli@0.5.4" = true
"#,
    )
    .unwrap();
    let source = "packages: [.]\nallowBuilds:\n  '@astryxdesign/core': true\n  other: false\noverrides:\n  vite: npm:@voidzero-dev/vite-plus-core@0.3.1\n";
    let output = config.apply_workspace(source).unwrap();
    let value: serde_json::Value = serde_saphyr::from_str(&output).unwrap();
    assert_eq!(
        value,
        json!({
            "packages": ["."],
            "allowBuilds": {"@astryxdesign/core": true, "other": false, "@astryxdesign/cli@0.5.4": true},
            "overrides": {"vite": "npm:@voidzero-dev/vite-plus-core@0.3.1"}
        })
    );
    assert_eq!(
        TaskConfig::default().apply_workspace(source).unwrap(),
        source
    );
    assert!(config.apply_workspace("allowBuilds: []").is_err());
    assert!(config.apply_workspace("[]").is_err());
}

#[test]
fn build_permissions_require_a_matching_declared_package_version() {
    for permission in [
        "'@astryxdesign/cli' = true",
        "'@astryxdesign/cli@*' = true",
        "'@astryxdesign/cli@0.5.3' = true",
        "'other@0.5.4' = true",
        "'@astryxdesign/cli@0.5.4' = 'true'",
    ] {
        let input = format!(
            "[dev-dependencies]\n'@astryxdesign/cli' = '0.5.4'\n[allow-builds]\n{permission}"
        );
        assert!(TaskConfig::parse(&input).is_err(), "{input}");
    }
    assert!(TaskConfig::parse("[allow-builds]\n'other@1.0.0' = true").is_err());
    let blocked =
        TaskConfig::parse("[dependencies]\nother = '1.0.0'\n[allow-builds]\n'other@1.0.0' = false")
            .unwrap();
    let output = blocked.apply_workspace("packages: [.]").unwrap();
    let value: serde_json::Value = serde_saphyr::from_str(&output).unwrap();
    assert_eq!(
        value,
        json!({"packages": ["."], "allowBuilds": {"other@1.0.0": false}})
    );
}
