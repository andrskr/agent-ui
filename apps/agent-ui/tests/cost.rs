use agent_ui::{
    cost::Basis,
    evidence::Usage,
    providers::codex::cost::{self, Request},
    report::Report,
    settings::Settings,
};

fn request(model: &str, input: u64, cached: u64, output: u64) -> Request {
    Request {
        model: model.into(),
        usage: Usage {
            input_tokens: input,
            cached_input_tokens: cached,
            output_tokens: output,
            reasoning_output_tokens: Some(output),
            cache_write_input_tokens: None,
        },
        cache_write_input_tokens: 0,
        timestamp_ms: 1_789_387_200_000,
        priority: false,
    }
}

fn close(actual: Option<f64>, expected: f64) {
    assert!(
        (actual.unwrap() - expected).abs() < 1e-10,
        "{actual:?} != {expected}"
    );
}

#[test]
fn same_tokens_have_different_model_costs_without_counting_cache_or_reasoning_twice() {
    close(
        cost::requests(&[request("gpt-5.6-luna", 100_000, 60_000, 10_000)]).usd,
        0.0212,
    );
    close(
        cost::requests(&[request("gpt-5.6-terra", 100_000, 60_000, 10_000)]).usd,
        0.212,
    );
    close(
        cost::requests(&[request("gpt-5.6-sol", 100_000, 60_000, 10_000)]).usd,
        0.53,
    );
    close(
        cost::requests(&[request("gpt-6-astra", 100_000, 60_000, 10_000)]).usd,
        0.96,
    );
}

#[test]
fn long_context_applies_to_each_request_and_uses_a_strict_threshold() {
    close(
        cost::requests(&[request("gpt-5.6-sol", 272_000, 72_000, 10_000)]).usd,
        1.336,
    );
    close(
        cost::requests(&[request("gpt-5.6-sol", 272_001, 72_000, 10_000)]).usd,
        2.52201,
    );
    close(
        cost::requests(&[
            request("gpt-5.6-sol", 200_000, 100_000, 10_000),
            request("gpt-5.6-sol", 200_000, 100_000, 10_000),
        ])
        .usd,
        1.7,
    );
    let total = request("gpt-5.6-sol", 400_000, 200_000, 20_000);
    let estimate = cost::run_totals(&total.model, Some(&total.usage), total.timestamp_ms);
    close(estimate.usd, 1.7);
    assert_eq!(estimate.basis, Basis::RunTotals);
}

#[test]
fn cache_writes_and_fast_rates_follow_codexbar_rules() {
    let mut row = request("gpt-6-astra", 300_000, 100_000, 10_000);
    row.cache_write_input_tokens = 50_000;
    close(cost::requests(&[row]).usd, 5.2);
    let mut row = request("gpt-6-astra", 300_000, 100_000, 10_000);
    row.priority = true;
    close(cost::requests(&[row]).usd, 9.9);
    let mut row = request("gpt-5.5", 100_000, 60_000, 10_000);
    row.priority = true;
    close(cost::requests(&[row]).usd, 1.325);
    let mut row = request("gpt-5.6-sol", 300_000, 100_000, 10_000);
    row.priority = true;
    close(cost::requests(&[row]).usd, 2.55);
}

#[test]
fn cached_subsets_are_clamped_and_missing_cache_rates_use_input_rates() {
    let mut row = request("gpt-5.6-luna", 100, 200, 0);
    row.cache_write_input_tokens = 200;
    close(cost::requests(&[row]).usd, 0.000002);
    let mut row = request("gpt-5.6-luna", 100, 20, 0);
    row.cache_write_input_tokens = 200;
    close(cost::requests(&[row]).usd, 0.0000204);
    close(
        cost::requests(&[request("gpt-5-pro", 100_000, 90_000, 1000)]).usd,
        1.62,
    );
}

#[test]
fn known_aliases_and_dated_models_keep_provider_boundaries() {
    for model in ["gpt-5.6", "openai/gpt-5.6-sol", "gpt-5.6-sol-2026-09-01"] {
        close(
            cost::requests(&[request(model, 100_000, 60_000, 10_000)]).usd,
            0.53,
        );
    }
    close(
        cost::requests(&[request("gpt-reserve", 100_000, 60_000, 10_000)]).usd,
        0.0212,
    );
    for model in [
        "unknown",
        "gpt-5.6-future",
        "other/gpt-5.6-sol",
        "anthropic/gpt-5.6-sol",
    ] {
        assert!(cost::requests(&[request(model, 100, 10, 10)]).usd.is_none());
    }
    assert!(
        cost::requests(&[
            request("gpt-5.6-sol", 100, 10, 10),
            request("unknown", 100, 10, 10)
        ])
        .usd
        .is_none()
    );
}

#[test]
fn missing_usage_and_known_zero_prices_remain_distinct() {
    assert!(cost::run_totals("gpt-5.6-luna", None, 0).usd.is_none());
    let preview = cost::requests(&[request("gpt-5.3-codex-spark", 1000, 500, 200)]);
    assert_eq!(preview.usd, Some(0.0));
    assert!(preview.note.contains("research preview"));
    assert_eq!(agent_ui::cost::format_usd(None), "Unavailable");
    assert_eq!(agent_ui::cost::format_usd(Some(0.0)), "$0.00");
    assert_eq!(agent_ui::cost::format_usd(Some(0.004)), "<$0.01");
    assert_eq!(agent_ui::cost::format_difference(Some(-0.004)), "−<$0.01");
    assert_eq!(agent_ui::cost::format_difference(Some(0.53)), "+$0.53");
}

#[test]
fn historical_prices_switch_at_the_codexbar_cutoff() {
    let mut row = request("gpt-5.6-luna", 100_000, 60_000, 10_000);
    row.timestamp_ms = 1_785_369_599_999;
    close(cost::requests(&[row]).usd, 0.106);
    let mut row = request("gpt-5.6-luna", 100_000, 60_000, 10_000);
    row.timestamp_ms = 1_785_369_600_000;
    close(cost::requests(&[row]).usd, 0.0212);
}

#[test]
fn saved_estimates_keep_their_source() {
    let settings = Settings {
        model: "gpt-5.6-luna".into(),
        effort: "low".into(),
        timeout: 60,
        provider: "codex".into(),
        binary: None,
    };
    let mut report = Report::new("run".into(), "task".into(), "app".into(), &settings);
    report.apply_cost(cost::requests(&[request(
        "gpt-5.6-sol",
        100_000,
        60_000,
        10_000,
    )]));
    let json = serde_json::to_value(&report).unwrap();
    let saved: Report = serde_json::from_value(json.clone()).unwrap();
    close(saved.cost_usd, 0.53);
    assert_eq!(saved.cost_basis, Some(Basis::Requests));
    assert_eq!(saved.cost_models, ["gpt-5.6-sol"]);
    assert_eq!(saved.cost_source.as_deref(), Some(cost::SOURCE));
}
