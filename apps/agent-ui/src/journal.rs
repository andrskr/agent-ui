use crate::{
    process::{self, Cancel, Outcome},
    report::{Check, Phase, Report},
    storage::{RunFiles, Store},
};
use anyhow::Result;
use std::{
    process::Command,
    time::{Duration, Instant},
};

/// Owns phase measurements, report writes, and command evidence for one run.
pub(crate) struct Journal {
    pub report: Report,
    pub files: RunFiles,
    store: Store,
    cancel: Cancel,
    phase: Phase,
    phase_started: Instant,
    pub(crate) recorder: crate::activity::Recorder,
}

impl Journal {
    pub fn new(store: Store, report: Report, cancel: Cancel) -> Result<Self> {
        let files = store.files(&report.id)?;
        let recorder = crate::activity::Recorder::open(store.root(), &report.id)?;
        recorder.note(
            "run.started",
            "run",
            serde_json::json!({"task":report.task,"provider":report.provider,"model_requested":report.model_requested,"effort_requested":report.effort_requested}),
        )?;
        Ok(Self {
            recorder,
            report,
            files,
            store,
            cancel,
            phase: Phase::Setup,
            phase_started: Instant::now(),
        })
    }

    pub fn store(&self) -> &Store {
        &self.store
    }
    pub fn save(&self) -> Result<()> {
        self.store.save(&self.report)
    }

    pub fn step(&mut self, text: &str) -> Result<()> {
        self.recorder.note(
            "activity",
            self.phase_name(),
            serde_json::json!({"text":text}),
        )?;
        self.report.record(text);
        self.report.step = Some(text.to_string());
        self.update_time();
        self.save()
    }

    pub fn mark(&mut self, text: &str) -> Result<()> {
        self.report.step = Some(text.to_string());
        self.update_time();
        self.save()
    }

    pub fn measure<T>(
        &mut self,
        phase: Phase,
        work: impl FnOnce(&mut Self) -> Result<T>,
    ) -> Result<T> {
        self.report.begin_phase(phase)?;
        self.phase = phase;
        self.phase_started = Instant::now();
        self.save()?;
        self.recorder
            .note("phase.started", self.phase_name(), serde_json::json!({}))?;
        let result = work(self);
        self.recorder.note("phase.finished", self.phase_name(), serde_json::json!({"seconds":self.phase_started.elapsed().as_secs_f64(),"error":result.as_ref().err().map(|e|format!("{e:#}"))}))?;
        self.update_time();
        if matches!(phase, Phase::Setup) && result.is_ok() {
            self.report.setup_finished_at_ms = Some(crate::report::now());
        }
        self.save()?;
        result
    }

    pub(crate) fn phase_name(&self) -> &'static str {
        match self.phase {
            Phase::Setup => "setup",
            Phase::Agent => "agent",
            Phase::Verification => "verification",
        }
    }

    fn update_time(&mut self) {
        let seconds = self.phase_started.elapsed().as_secs_f64();
        if matches!(self.phase, Phase::Setup) {
            self.report.setup_seconds = seconds;
        }
    }

    pub fn command(
        &mut self,
        mut command: Command,
        label: &str,
        log: &str,
        timeout: Duration,
    ) -> Result<Outcome> {
        let evidence = self.files.evidence();
        let cancel = self.cancel.clone();
        let phase = self.phase;
        let trace = crate::activity::Trace::new(self.recorder.clone(), self.phase_name());
        let result = process::execute_traced(
            &mut command,
            (
                &evidence.join(format!("{log}.log")),
                &evidence.join(format!("{log}.stderr.log")),
            ),
            None,
            timeout,
            &cancel,
            trace,
            |_, _| {
                self.update_time();
                self.save()
            },
        )?;
        if matches!(phase, Phase::Verification) {
            self.report.verification = Some(Check {
                exit_code: result.code,
                seconds: result.seconds,
            });
        }
        self.save()?;
        process::checked(&result, label)?;
        Ok(result)
    }

    pub fn cancellation(&self) -> Cancel {
        self.cancel.clone()
    }

    pub fn agent_progress(
        &mut self,
        seconds: f64,
        observations: impl IntoIterator<Item = crate::evidence::AgentObservation>,
    ) -> Result<()> {
        self.report.agent_seconds = Some(seconds);
        let mut responded = false;
        for observation in observations {
            self.report.observe(observation);
            responded = true;
        }
        if responded {
            self.report.step = Some("Running the agent".to_string());
        }
        self.report.estimate_cost_from_totals();
        self.save()
    }

    pub fn agent_outcome(&mut self, result: &Outcome) {
        self.report.agent_exit_code = result.code;
        self.report.agent_seconds = Some(result.seconds);
    }

    pub fn finish(mut self, result: Result<()>) -> Result<Report> {
        let failure = result.err().map(|error| format!("{error:#}"));
        self.report.finish(failure, self.cancel.is_cancelled())?;
        if let Some(error) = &self.report.error {
            self.report.record(format!("Run ended: {error}"));
        } else {
            self.report.record("Verification passed. Run complete");
        }
        self.save()?;
        self.recorder.finish(
            serde_json::to_value(self.report.state)?
                .as_str()
                .unwrap_or("unknown"),
        )?;
        Ok(self.report)
    }
}
