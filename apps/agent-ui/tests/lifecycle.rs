use agent_ui::{
    codex::event::decode,
    report::{Check, Phase, Report, State},
    settings::{Effort, Settings},
};

fn run() -> Report {
    Report::new(
        "run".into(),
        "task".into(),
        "unused/app".into(),
        &Settings {
            model: "test".into(),
            effort: Effort::Low,
            timeout: 10,
            codex: None,
        },
    )
}

fn complete_agent(report: &mut Report) {
    report.begin_phase(Phase::Agent).unwrap();
    report.observe(decode(br#"{"type":"turn.completed","usage":{"input_tokens":100,"cached_input_tokens":40,"output_tokens":8}}"#));
    report.agent_exit_code = Some(0);
}

#[test]
fn ready_requires_a_successful_agent_and_a_successful_verification() {
    let mut report = run();
    assert!(report.finish(None, false).is_err());
    assert!(report.begin_phase(Phase::Verification).is_err());
    assert_eq!(report.state, State::Preparing);
    assert!(report.finished_at_ms.is_none());

    report.begin_phase(Phase::Agent).unwrap();
    assert!(report.begin_phase(Phase::Verification).is_err());
    report.observe(decode(br#"{"type":"turn.completed"}"#));
    report.agent_exit_code = Some(7);
    assert!(report.begin_phase(Phase::Verification).is_err());
    report.agent_exit_code = Some(0);
    report.begin_phase(Phase::Verification).unwrap();
    assert!(report.finish(None, false).is_err());
    report.verification = Some(Check {
        exit_code: Some(1),
        seconds: 2.0,
    });
    assert!(report.finish(None, false).is_err());
    report.verification = Some(Check {
        exit_code: Some(0),
        seconds: 2.0,
    });
    report.finish(None, false).unwrap();
    assert_eq!(report.state, State::Ready);
    assert!(report.finished_at_ms.is_some());
}

#[test]
fn cancellation_wins_over_a_successful_final_process() {
    let mut report = run();
    complete_agent(&mut report);
    report.begin_phase(Phase::Verification).unwrap();
    report.verification = Some(Check {
        exit_code: Some(0),
        seconds: 1.0,
    });
    report.finish(None, true).unwrap();
    assert_eq!(report.state, State::Cancelled);
    assert_eq!(report.error.as_deref(), Some("Run cancelled"));
    assert_eq!(report.usage.as_ref().unwrap().input_tokens, 100);
}

#[test]
fn setup_failure_and_interruption_keep_partial_evidence() {
    let mut failed = run();
    failed.setup_seconds = 0.25;
    failed
        .finish(Some("Package install failed".into()), false)
        .unwrap();
    assert_eq!(failed.state, State::Failed);
    assert_eq!(failed.setup_seconds, 0.25);
    assert!(failed.usage.is_none());

    let mut interrupted = run();
    complete_agent(&mut interrupted);
    interrupted.interrupt();
    assert_eq!(interrupted.state, State::Interrupted);
    assert_eq!(interrupted.usage.as_ref().unwrap().cached_input_tokens, 40);
    assert_eq!(interrupted.completed_turns, 1);
}

#[test]
fn finished_runs_cannot_restart_or_change_their_result() {
    let mut report = run();
    report.finish(Some("Setup failed".into()), false).unwrap();
    let finished = report.finished_at_ms;
    for phase in [Phase::Setup, Phase::Agent, Phase::Verification] {
        assert!(report.begin_phase(phase).is_err());
    }
    assert!(report.finish(None, true).is_err());
    report.interrupt();
    assert_eq!(report.state, State::Failed);
    assert_eq!(report.error.as_deref(), Some("Setup failed"));
    assert_eq!(report.finished_at_ms, finished);
}
