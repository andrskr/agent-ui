use agent_ui::task::TaskConfig;
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

#[test]
fn setup_profiles_and_repair_are_explicit_and_strict() {
    let config =
        TaskConfig::parse("[setup]\nprofiles = ['root-quality']\n[repair]\ncheck = 'quality'")
            .unwrap();
    assert_eq!(config.setup.profiles, ["root-quality"]);
    assert_eq!(config.repair.unwrap().check, "quality");
    for invalid in [
        "[setup]\nprofiles = ['../outside']",
        "[setup]\nprofiles = ['root-quality', 'root-quality']",
        "[setup]\nprofile = 'root-quality'",
        "[repair]\ncheck = ''",
        "[repair]\ncheck = 'quality'\noptional = true",
    ] {
        assert!(TaskConfig::parse(invalid).is_err(), "{invalid}");
    }
}
