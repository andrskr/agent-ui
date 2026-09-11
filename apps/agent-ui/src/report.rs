use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Preparing,
    Running,
    Verifying,
    Ready,
    Failed,
    Cancelled,
    Interrupted,
}
impl State {
    pub fn active(self) -> bool {
        matches!(self, Self::Preparing | Self::Running | Self::Verifying)
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Preparing => "PREPARING",
            Self::Running => "RUNNING",
            Self::Verifying => "VERIFYING",
            Self::Ready => "READY",
            Self::Failed => "FAILED",
            Self::Cancelled => "CANCELLED",
            Self::Interrupted => "INTERRUPTED",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Phase {
    Setup,
    Agent,
    Verification,
}
pub use crate::evidence::Usage;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Check {
    pub exit_code: Option<i32>,
    pub seconds: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Activity {
    pub at_ms: u64,
    pub text: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Report {
    pub schema_version: u32,
    pub id: String,
    pub task: String,
    #[serde(default)]
    pub task_config: Option<crate::task::TaskConfig>,
    pub state: State,
    pub created_at_ms: u64,
    pub finished_at_ms: Option<u64>,
    pub model_requested: String,
    pub effort_requested: String,
    pub codex_version: Option<String>,
    pub node_version: Option<String>,
    pub vp_version: Option<String>,
    pub runner_version: String,
    pub thread_id: Option<String>,
    pub timeout_seconds: u64,
    pub setup_seconds: f64,
    pub agent_seconds: Option<f64>,
    pub agent_started_at_ms: Option<u64>,
    pub agent_exit_code: Option<i32>,
    pub verification: Option<Check>,
    pub usage: Option<Usage>,
    pub completed_turns: u64,
    pub invalid_event_lines: u64,
    pub event_counts: BTreeMap<String, u64>,
    pub error: Option<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
    pub activity: Vec<Activity>,
    pub inputs: BTreeMap<String, String>,
    pub before: BTreeMap<String, String>,
    pub after: BTreeMap<String, String>,
    pub changed_files: Vec<String>,
    pub app: PathBuf,
    pub cost_usd: Option<f64>,
    pub cost_note: String,
    pub human_review: String,
    pub isolation: String,
}
impl Report {
    pub fn begin_phase(&mut self, phase: Phase) -> Result<()> {
        let (previous, next) = match phase {
            Phase::Setup => (State::Preparing, State::Preparing),
            Phase::Agent => (State::Preparing, State::Running),
            Phase::Verification => (State::Running, State::Verifying),
        };
        ensure!(
            self.state == previous,
            "Cannot start {phase:?} from {}",
            self.state.label()
        );
        if matches!(phase, Phase::Verification) {
            self.check_agent_completion()?;
            ensure!(
                self.agent_exit_code == Some(0),
                "Agent process did not succeed"
            );
        }
        self.state = next;
        if matches!(phase, Phase::Agent) {
            self.agent_started_at_ms = Some(now());
        }
        Ok(())
    }

    pub fn finish(&mut self, failure: Option<String>, cancelled: bool) -> Result<()> {
        ensure!(self.state.active(), "Run already finished");
        if cancelled {
            self.state = State::Cancelled;
            self.error = Some(failure.unwrap_or_else(|| "Run cancelled".into()));
        } else if let Some(error) = failure {
            self.state = State::Failed;
            self.error = Some(error);
        } else {
            ensure!(
                self.state == State::Verifying,
                "Run did not reach verification"
            );
            self.check_agent_completion()?;
            ensure!(
                self.verification
                    .as_ref()
                    .is_some_and(|check| check.exit_code == Some(0)),
                "Verification did not pass"
            );
            self.state = State::Ready;
        }
        self.finished_at_ms = Some(now());
        Ok(())
    }

    pub fn interrupt(&mut self) {
        if self.state.active() {
            self.state = State::Interrupted;
            self.finished_at_ms = Some(now());
            self.error = Some("The runner stopped before it saved a final result".into());
            self.record("Recovered an interrupted run");
        }
    }
    pub fn new(
        id: String,
        task: String,
        app: PathBuf,
        settings: &crate::settings::Settings,
    ) -> Self {
        Self {
            schema_version: 1,
            id,
            task,
            task_config: None,
            state: State::Preparing,
            created_at_ms: now(),
            finished_at_ms: None,
            model_requested: settings.model.clone(),
            effort_requested: settings.effort.to_string(),
            codex_version: None,
            node_version: None,
            vp_version: None,
            runner_version: env!("CARGO_PKG_VERSION").into(),
            thread_id: None,
            timeout_seconds: settings.timeout,
            setup_seconds: 0.0,
            agent_seconds: None,
            agent_started_at_ms: None,
            agent_exit_code: None,
            verification: None,
            usage: None,
            completed_turns: 0,
            invalid_event_lines: 0,
            event_counts: BTreeMap::new(),
            error: None,
            warnings: Vec::new(),
            activity: Vec::new(),
            inputs: BTreeMap::new(),
            before: BTreeMap::new(),
            after: BTreeMap::new(),
            changed_files: Vec::new(),
            app,
            cost_usd: None,
            cost_note: "Subscription use. Codex does not report a per-run currency charge.".into(),
            human_review: "pending".into(),
            isolation: concat!(
                "Fresh HOME and CODEX_HOME; explicit environment; workspace-write sandbox. ",
                "This does not isolate host reads or shared account limits. ",
                "Setup, verification, and preview run on the host."
            )
            .into(),
        }
    }

    pub fn record(&mut self, text: impl Into<String>) {
        self.activity.push(Activity {
            at_ms: now(),
            text: text.into(),
        });
    }

    pub fn record_source_changes(&mut self, after: BTreeMap<String, String>) {
        self.after = after;
        self.changed_files = self
            .before
            .keys()
            .chain(self.after.keys())
            .filter(|path| self.before.get(*path) != self.after.get(*path))
            .cloned()
            .collect();
        self.changed_files.sort();
        self.changed_files.dedup();
    }

    pub fn check_agent_completion(&self) -> Result<()> {
        ensure!(
            self.completed_turns > 0,
            "Codex exited without a completed turn"
        );
        ensure!(
            self.error.is_none(),
            "Codex reported an error; see the event stream"
        );
        ensure!(
            self.invalid_event_lines == 0,
            "Codex produced invalid JSON events"
        );
        Ok(())
    }

    pub fn observe(&mut self, observation: crate::evidence::AgentObservation) {
        use crate::evidence::{AgentObservation, AgentUpdate};
        let AgentObservation::Event { kind, update } = observation else {
            self.invalid_event_lines += 1;
            return;
        };
        *self.event_counts.entry(kind).or_default() += 1;
        match update {
            AgentUpdate::Thread(id) => self.thread_id = id,
            AgentUpdate::Turn(sample) => {
                self.completed_turns += 1;
                if let Some(sample) = sample {
                    let usage = self.usage.get_or_insert_default();
                    usage.input_tokens += sample.input_tokens;
                    usage.cached_input_tokens += sample.cached_input_tokens;
                    usage.output_tokens += sample.output_tokens;
                    if let Some(n) = sample.reasoning_output_tokens {
                        *usage.reasoning_output_tokens.get_or_insert(0) += n;
                    }
                }
                self.record("Codex completed a turn");
            }
            AgentUpdate::Failure(message) => {
                self.error = Some(message.clone());
                self.record(message);
            }
            AgentUpdate::Activity { text, warning } => {
                if let Some(warning) = warning {
                    self.warnings.push(warning);
                }
                self.record(text);
            }
            AgentUpdate::Unknown => {}
        }
    }
}
