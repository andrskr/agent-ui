use agent_ui::{
    cost::Basis,
    providers::{catalog, claude::event::Decoder},
    report::Report,
    settings::Settings,
};
use serde_json::{Value, json};

fn report() -> Report {
    Report::new(
        "run".into(),
        "task".into(),
        "app".into(),
        &Settings::for_provider("claude").unwrap(),
    )
}
fn result() -> Value {
    json!({"type":"result", "subtype":"success", "is_error":false, "result":"Done",
        "usage":{"input_tokens":100,"cache_read_input_tokens":600,"cache_creation_input_tokens":300,"output_tokens":40},
        "total_cost_usd":0.017, "modelUsage":{"claude-sonnet-5":{"costUSD":0.017}}, "permission_denials":[]})
}
fn feed(report: &mut Report, decoder: &mut Decoder, event: &Value) {
    report.observe(decoder.decode(&serde_json::to_vec(event).unwrap()));
}

#[test]
fn model_efforts_and_provider_changes_use_the_catalog() {
    let mut settings = Settings {
        binary: Some("/codex-only".into()),
        timeout: 90,
        ..Settings::default()
    };
    settings.select_model("gpt-6-astra").unwrap();
    settings.effort = "ultra".into();
    settings.validate().unwrap();
    settings.select_model("gpt-5.6-luna").unwrap();
    assert_eq!(settings.effort, "low");
    settings.effort = "ultra".into();
    assert!(settings.validate().is_err());
    settings.select_provider("claude").unwrap();
    assert_eq!(
        (&*settings.model, &*settings.effort),
        ("claude-sonnet-5", "high")
    );
    assert_eq!(settings.timeout, 90);
    assert!(settings.binary.is_none());
    settings.effort = "xhigh".into();
    settings.select_model("claude-sonnet-4-6").unwrap();
    assert_eq!(settings.effort, "high");
    settings.effort = "ultra".into();
    assert!(settings.validate().is_err());
    settings.select_model("org/custom-model:v2").unwrap();
    assert_eq!(settings.effort, "default");
    settings.validate().unwrap();
    assert!(settings.select_provider("missing").is_err());
    assert_eq!(settings.provider, "claude");
}

#[test]
fn every_catalog_choice_round_trips_through_selection_and_validation() {
    for provider in catalog() {
        let mut settings = Settings::for_provider(provider.id).unwrap();
        settings.validate().unwrap();
        for model in provider.models {
            for id in std::iter::once(model.id).chain(model.aliases.iter().copied()) {
                settings.select_model(id).unwrap();
                for effort in model.efforts {
                    settings.effort = (*effort).into();
                    settings.validate().unwrap();
                    settings.step_effort(1);
                    settings.validate().unwrap();
                }
            }
        }
        settings.step_provider(1);
        assert_ne!(settings.provider, provider.id);
        settings.step_provider(-1);
        assert_eq!(settings.provider, provider.id);
    }
}

#[test]
fn claude_final_usage_counts_each_token_once_and_keeps_native_cost() {
    let mut report = report();
    let mut decoder = Decoder::default();
    feed(
        &mut report,
        &mut decoder,
        &json!({"type":"system","subtype":"init","session_id":"session"}),
    );
    let message = json!({"type":"assistant","message":{"model":"claude-sonnet-5","content":[{"type":"text","text":"Working"}],"usage":{"input_tokens":100,"output_tokens":40}}});
    feed(&mut report, &mut decoder, &message);
    feed(&mut report, &mut decoder, &message);
    assert!(report.usage.is_none());
    assert_eq!(report.completed_turns, 0);
    feed(&mut report, &mut decoder, &result());
    report.estimate_cost_from_totals();
    let usage = report.usage.as_ref().unwrap();
    assert_eq!(
        (
            usage.input_tokens,
            usage.cached_input_tokens,
            usage.cache_write_input_tokens,
            usage.output_tokens
        ),
        (1000, 600, Some(300), 40)
    );
    assert!(usage.reasoning_output_tokens.is_none());
    assert_eq!(report.thread_id.as_deref(), Some("session"));
    assert_eq!(report.completed_turns, 1);
    assert_eq!(report.cost_usd, Some(0.017));
    assert_eq!(report.cost_basis, Some(Basis::ProviderReported));
    assert_eq!(report.cost_models, ["claude-sonnet-5"]);
    report.check_agent_completion().unwrap();
    feed(&mut report, &mut decoder, &result());
    assert_eq!(report.usage.unwrap().input_tokens, 1000);
    assert_eq!(report.completed_turns, 1);
    assert_eq!(report.invalid_event_lines, 1);
}

#[test]
fn failed_claude_results_keep_usage_and_cost_without_completing() {
    for patch in [
        json!({"subtype":"error_max_turns","is_error":true,"errors":["Turn limit"]}),
        json!({"permission_denials":[{"tool_name":"Bash"}]}),
        json!({"terminal_reason":"aborted_tools"}),
        json!({"terminal_reason":"context_exhausted"}),
    ] {
        let mut event = result();
        event
            .as_object_mut()
            .unwrap()
            .extend(patch.as_object().unwrap().clone());
        let mut report = report();
        feed(&mut report, &mut Decoder::default(), &event);
        assert!(report.check_agent_completion().is_err());
        assert_eq!(report.completed_turns, 0);
        assert!(report.error.is_some());
        assert_eq!(report.usage.unwrap().output_tokens, 40);
        assert_eq!(report.cost_usd, Some(0.017));
    }
}

#[test]
fn missing_and_invalid_claude_measurements_do_not_become_zero() {
    for usage in [
        Value::Null,
        json!({"input_tokens":100,"output_tokens":40}),
        json!({"input_tokens":u64::MAX,"cache_read_input_tokens":1,"cache_creation_input_tokens":0,"output_tokens":1}),
    ] {
        let mut event = result();
        event["usage"] = usage;
        event["total_cost_usd"] = Value::Null;
        let mut report = report();
        feed(&mut report, &mut Decoder::default(), &event);
        assert!(report.usage.is_none());
        assert!(report.cost_usd.is_none());
    }
    let mut event = result();
    event["total_cost_usd"] = json!(-1);
    event["usage"] = json!({"input_tokens":0,"cache_read_input_tokens":0,"cache_creation_input_tokens":0,"output_tokens":0});
    let mut report = report();
    feed(&mut report, &mut Decoder::default(), &event);
    assert_eq!(report.usage.unwrap().input_tokens, 0);
    assert!(report.cost_usd.is_none());
}

#[test]
fn claude_message_fallback_replaces_duplicates_and_checks_final_totals() {
    let mut message = json!({"type":"assistant","requestId":"request","message":{"id":"message","model":"claude-sonnet-4-6","content":[],"usage":{"input_tokens":100,"cache_read_input_tokens":600,"cache_creation_input_tokens":300,"output_tokens":40,"cache_creation":{"ephemeral_1h_input_tokens":100}}}});
    let mut run = report();
    let mut decoder = Decoder::new(1_789_387_200_000);
    feed(&mut run, &mut decoder, &message);
    message["message"]["usage"]["output_tokens"] = json!(50);
    feed(&mut run, &mut decoder, &message);
    assert_eq!(run.usage.as_ref().unwrap().output_tokens, 50);
    assert!((run.cost_usd.unwrap() - 0.00258).abs() < 1e-10);
    assert_eq!(run.completed_turns, 0);
    let mut final_event = result();
    final_event["total_cost_usd"] = Value::Null;
    final_event["usage"]["output_tokens"] = json!(50);
    feed(&mut run, &mut decoder, &final_event);
    assert_eq!(run.cost_basis, Some(Basis::Requests));
    assert!((run.cost_usd.unwrap() - 0.00258).abs() < 1e-10);

    let mut mismatched = report();
    let mut decoder = Decoder::new(1_789_387_200_000);
    feed(&mut mismatched, &mut decoder, &message);
    final_event["usage"]["output_tokens"] = json!(60);
    feed(&mut mismatched, &mut decoder, &final_event);
    assert!(mismatched.cost_usd.is_none());
    assert_eq!(mismatched.usage.unwrap().output_tokens, 60);

    let mut missing = report();
    let mut decoder = Decoder::new(1_789_387_200_000);
    feed(&mut missing, &mut decoder, &message);
    message["message"].as_object_mut().unwrap().remove("id");
    feed(&mut missing, &mut decoder, &message);
    assert!(missing.cost_usd.is_none());
    assert!(missing.usage.is_none());
}

#[test]
fn added_models_use_their_own_efforts_and_aliases() {
    let mut settings = Settings::default();
    settings.select_model("gpt-5.6-terra").unwrap();
    settings.effort = "ultra".into();
    settings.validate().unwrap();
    settings.select_model("gpt-5.5").unwrap();
    assert_eq!(settings.effort, "medium");
    settings.effort = "max".into();
    assert!(settings.validate().is_err());
    settings.select_model("gpt-5.3-codex-spark").unwrap();
    assert_eq!(settings.effort, "high");
    settings.validate().unwrap();

    settings.select_provider("claude").unwrap();
    settings.select_model("claude-opus-4-8").unwrap();
    settings.effort = "xhigh".into();
    settings.validate().unwrap();
    settings.select_model("claude-opus-4-7").unwrap();
    settings.validate().unwrap();
    for model in [
        "haiku",
        "claude-haiku-4-5",
        "claude-opus-4-5-20251101",
        "claude-sonnet-4-5",
    ] {
        settings.effort = "high".into();
        settings.select_model(model).unwrap();
        assert_eq!(settings.effort, "default");
        settings.step_effort(1);
        assert_eq!(settings.effort, "default");
        settings.validate().unwrap();
        settings.effort = "low".into();
        assert!(settings.validate().is_err());
    }
    settings.select_model("haiku").unwrap();
    settings.step_model(1);
    assert_eq!(settings.model, "claude-opus-4-8");
    settings.step_model(-1);
    assert_eq!(settings.model, "claude-haiku-4-5-20251001");
    assert_eq!(settings.effort, "default");
}
