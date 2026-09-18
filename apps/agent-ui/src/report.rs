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
    pub task_config: Option<crate::task::TaskConfig>,
    pub state: State,
    #[serde(default)]
    pub step: Option<String>,
    pub created_at_ms: u64,
    pub finished_at_ms: Option<u64>,
    #[serde(default)]
    pub recovered_at_ms: Option<u64>,
    #[serde(default)]
    pub setup_started_at_ms: Option<u64>,
    #[serde(default)]
    pub setup_finished_at_ms: Option<u64>,
    #[serde(default)]
    pub verification_started_at_ms: Option<u64>,
    pub provider: String,
    pub model_requested: String,
    pub effort_requested: String,
    pub provider_version: Option<String>,
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
    pub warnings: Vec<String>,
    pub activity: Vec<Activity>,
    pub inputs: BTreeMap<String, String>,
    pub before: BTreeMap<String, String>,
    pub after: BTreeMap<String, String>,
    pub changed_files: Vec<String>,
    pub app: PathBuf,
    pub cost_usd: Option<f64>,
    pub cost_note: String,
    pub cost_basis: Option<crate::cost::Basis>,
    pub cost_models: Vec<String>,
    pub cost_source: Option<String>,
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
        if matches!(phase, Phase::Setup) {
            self.setup_started_at_ms = Some(now());
        }
        if matches!(phase, Phase::Verification) {
            self.verification_started_at_ms = Some(now());
        }
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
        self.step = None;
        self.finished_at_ms = Some(now());
        Ok(())
    }

    pub fn interrupt(&mut self) {
        if self.state.active() {
            self.state = State::Interrupted;
            self.step = None;
            self.finished_at_ms = None;
            self.recovered_at_ms = Some(now());
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
            schema_version: 2,
            id,
            task,
            task_config: None,
            state: State::Preparing,
            step: None,
            created_at_ms: now(),
            finished_at_ms: None,
            recovered_at_ms: None,
            setup_started_at_ms: None,
            setup_finished_at_ms: None,
            verification_started_at_ms: None,
            model_requested: settings.model.clone(),
            effort_requested: settings.effort.to_string(),
            provider_version: None,
            provider: settings.provider.clone(),
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
            cost_note: "API cost unavailable: token usage is not reported.".into(),
            cost_basis: None,
            cost_models: Vec::new(),
            cost_source: None,
            isolation: crate::providers::descriptor(&settings.provider)
                .map(|p| p.isolation.to_owned())
                .unwrap_or_else(|_| "Unknown provider isolation".into()),
        }
    }

    pub fn record(&mut self, text: impl Into<String>) {
        self.activity.push(Activity {
            at_ms: now(),
            text: text.into(),
        });
    }

    pub fn estimate_cost_from_totals(&mut self) {
        self.apply_cost(crate::providers::estimate(
            &self.provider,
            &self.cost_input(),
            None,
        ));
    }
    pub(crate) fn cost_input(&self) -> crate::cost::Input<'_> {
        crate::cost::Input {
            model: &self.model_requested,
            created_at_ms: self.created_at_ms,
            usage: self.usage.as_ref(),
            thread_id: self.thread_id.as_deref(),
            saved: self.saved_cost(),
        }
    }
    pub fn saved_cost(&self) -> Option<crate::cost::Estimate> {
        Some(crate::cost::Estimate {
            usd: self.cost_usd,
            basis: self.cost_basis?,
            models: self.cost_models.clone(),
            source: self.cost_source.clone()?,
            note: self.cost_note.clone(),
        })
    }

    pub fn apply_cost(&mut self, estimate: crate::cost::Estimate) {
        self.cost_usd = estimate.usd;
        self.cost_note = estimate.note;
        self.cost_basis = Some(estimate.basis);
        self.cost_models = estimate.models;
        self.cost_source = Some(estimate.source);
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
            "Agent exited without a completed turn"
        );
        ensure!(
            self.error.is_none(),
            "Agent reported an error; see the event stream"
        );
        ensure!(
            self.invalid_event_lines == 0,
            "Agent produced invalid JSON events"
        );
        Ok(())
    }

    pub fn observe(&mut self, observation: crate::evidence::AgentObservation) {
        use crate::evidence::AgentObservation;
        let AgentObservation::Event { kind, update } = observation else {
            self.invalid_event_lines += 1;
            return;
        };
        *self.event_counts.entry(kind).or_default() += 1;
        self.apply_update(update);
    }

    fn apply_update(&mut self, update: crate::evidence::AgentUpdate) {
        use crate::evidence::AgentUpdate;
        match update {
            AgentUpdate::Batch(updates) => {
                for update in updates {
                    self.apply_update(update);
                }
            }
            AgentUpdate::UsageSnapshot(usage) => self.usage = usage,
            AgentUpdate::Cost(estimate) => self.apply_cost(estimate),
            AgentUpdate::Thread(id) => self.thread_id = id,
            AgentUpdate::Turn(sample) => {
                self.completed_turns += 1;
                if let Some(sample) = sample {
                    self.usage.get_or_insert_default().add(&sample);
                }
                self.record("Agent completed a turn");
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
