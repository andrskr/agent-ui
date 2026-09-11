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
}

impl Journal {
    pub fn new(store: Store, report: Report, cancel: Cancel) -> Result<Self> {
        let files = store.files(&report.id)?;
        Ok(Self {
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

    pub fn measure<T>(
        &mut self,
        phase: Phase,
        work: impl FnOnce(&mut Self) -> Result<T>,
    ) -> Result<T> {
        self.report.begin_phase(phase)?;
        self.phase = phase;
        self.phase_started = Instant::now();
        self.save()?;
        let result = work(self);
        self.update_time();
        self.save()?;
        result
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
        let result = process::execute(
            &mut command,
            &evidence.join(format!("{log}.log")),
            &evidence.join(format!("{log}.stderr.log")),
            None,
            timeout,
            &cancel,
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
        for observation in observations {
            self.report.observe(observation);
        }
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
            self.report
                .record("Verification passed. Ready for human review");
        }
        self.save()?;
        Ok(self.report)
    }
}
