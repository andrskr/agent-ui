use crate::{
    application::{Application, MAX_CONCURRENT_RUNS},
    ledger::{Completion, Ledger},
    process::Cancel,
    run_queue::RunQueue,
    runner,
    settings::Settings,
    storage::private_dir,
    suite::Snapshot,
    toolchain::Tools,
};
use anyhow::{Result, ensure};
use serde::Serialize;
use std::{fs, path::PathBuf, thread, time::Duration};

#[derive(Serialize)]
pub struct BatchOutcome {
    #[serde(flatten)]
    pub summary: crate::ledger::BatchSummary,
    pub recording: bool,
    pub recording_errors: Vec<String>,
    pub cancelled: bool,
}
impl BatchOutcome {
    pub fn exit_code(&self) -> u8 {
        if self.cancelled {
            130
        } else if self.recording_errors.is_empty() && self.summary.successful() {
            0
        } else {
            1
        }
    }
}

struct Batch {
    id: String,
    snapshot: Snapshot,
    settings: Settings,
    ledger: Ledger,
    recording: bool,
}

struct Materialization(PathBuf);
impl Drop for Materialization {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

impl Application {
    pub(crate) fn run_batch(
        &mut self,
        snapshot: Snapshot,
        settings: Settings,
        record: bool,
        stop: Cancel,
    ) -> Result<BatchOutcome> {
        ensure!(!self.is_busy(), "Wait for active work to finish");
        snapshot.validate()?;
        self.batch_preflight(&settings)?;
        let id = runner::new_id();
        let mut ledger = if record {
            Ledger::open(self.data(), true)?
        } else {
            Ledger::temporary()?
        };
        ledger.create_batch(&id, &snapshot, &settings, MAX_CONCURRENT_RUNS)?;
        self.execute_batch(
            Batch {
                id,
                snapshot,
                settings,
                ledger,
                recording: record,
            },
            false,
            stop,
        )
    }
    pub(crate) fn resume_batch(
        &mut self,
        id: &str,
        retry: bool,
        stop: Cancel,
    ) -> Result<BatchOutcome> {
        ensure!(!self.is_busy(), "Wait for active work to finish");
        crate::storage::valid_run_id(id)?;
        let ledger = Ledger::open(self.data(), false)?;
        let (snapshot, settings) = ledger.plan(id)?;
        if !ledger.selected(id, retry)?.is_empty() {
            self.batch_preflight(&settings)?;
        }
        self.execute_batch(
            Batch {
                id: id.into(),
                snapshot,
                settings,
                ledger,
                recording: true,
            },
            retry,
            stop,
        )
    }
    fn batch_preflight(&self, settings: &Settings) -> Result<()> {
        settings.validate()?;
        let tools = Tools::discover(settings)?;
        tools.versions()?;
        crate::providers::get(&settings.provider)?.preflight(&self.store, &tools.agent)
    }
    fn execute_batch(
        &mut self,
        mut batch: Batch,
        retry: bool,
        stop: Cancel,
    ) -> Result<BatchOutcome> {
        let mut queue = RunQueue::default();
        queue.add(
            batch.ledger.selected(&batch.id, retry)?,
            batch.settings.clone(),
        );
        let mut summary = batch.ledger.summary(&batch.id)?;
        let mut errors = Vec::new();
        eprintln!(
            "Batch {}: {} tasks; {} / {} / effort {}",
            batch.id,
            summary.planned,
            batch.settings.provider,
            batch.settings.model,
            batch.settings.effort
        );
        if batch.recording {
            eprintln!("Ledger: {}", crate::ledger::path(self.data()).display());
        }
        if queue.is_empty() {
            return Ok(BatchOutcome {
                summary,
                recording: batch.recording,
                recording_errors: errors,
                cancelled: stop.is_cancelled(),
            });
        }
        let parent = self.data().join("batch-work");
        private_dir(&parent)?;
        let workspace =
            Materialization(parent.join(format!("{}-{}", batch.id, uuid::Uuid::new_v4())));
        batch.snapshot.materialize(&workspace.0)?;
        batch.ledger.begin(&batch.id)?;
        while !queue.is_empty() || !self.active.is_empty() {
            if stop.is_cancelled() || !errors.is_empty() {
                queue.clear();
                self.cancel();
            }
            if batch.recording && errors.is_empty() {
                for active in self.active.values() {
                    if let Err(error) = batch.ledger.ingest_activity(self.data(), &active.id, false)
                    {
                        errors.push(format!("Cannot record activity: {error:#}"));
                        break;
                    }
                }
                if !errors.is_empty() {
                    queue.clear();
                    self.cancel();
                }
            }
            let finished: Vec<_> = self
                .active
                .iter()
                .filter(|(_, a)| a.finished())
                .map(|(task, _)| task.clone())
                .collect();
            for task in finished {
                let active = self
                    .active
                    .remove(&task)
                    .expect("Selected active task exists");
                let id = active.id.clone();
                let completion = match active.join() {
                    Ok(report) => Completion::report(report),
                    Err(error) => self.failed_batch_worker(
                        &id,
                        &task,
                        format!("{error:#}"),
                        stop.is_cancelled(),
                    ),
                };
                match completion.and_then(|c| {
                    eprintln!("{}: {} ({})", c.task, c.state, c.run_id);
                    if batch.recording {
                        batch.ledger.deliver(self.data(), &c)?;
                        if let Some(error) = batch.ledger.capture_error(&c.run_id)? {
                            anyhow::bail!("Incomplete activity capture: {error}");
                        }
                        Ok(())
                    } else {
                        batch.ledger.complete(&c)
                    }
                }) {
                    Ok(()) => {}
                    Err(error) => errors.push(format!("{task}: {error:#}")),
                }
            }
            if errors.is_empty() && !stop.is_cancelled() {
                while let Some((task, settings)) =
                    queue.next(self.active.len(), MAX_CONCURRENT_RUNS)
                {
                    if stop.is_cancelled() {
                        break;
                    }
                    let id = runner::new_id();
                    if let Err(error) = batch.ledger.reserve(&batch.id, &task, &id, &settings) {
                        errors.push(format!("Cannot reserve {task}: {error:#}"));
                        break;
                    }
                    if batch.recording
                        && let Err(error) = batch.ledger.enable_capture(self.data(), &id)
                    {
                        let c = Completion::no_report(
                            id,
                            task.clone(),
                            format!("Cannot start activity capture: {error:#}"),
                            false,
                        );
                        errors.push(format!(
                            "Cannot start activity capture for {task}: {error:#}"
                        ));
                        if let Err(error) = batch.ledger.deliver(self.data(), &c) {
                            errors.push(format!("Cannot record capture failure: {error:#}"));
                        }
                        break;
                    }
                    let start = (|| {
                        let source = batch.snapshot.source(&workspace.0, &task)?;
                        self.stop_preview(&task);
                        runner::start_with_id(self.store.clone(), source, settings, id.clone())
                    })();
                    match start {
                        Ok(active) => {
                            self.active.insert(task, active);
                        }
                        Err(error) => {
                            let c = Completion::no_report(id, task, format!("{error:#}"), false);
                            let result = if batch.recording {
                                batch.ledger.deliver(self.data(), &c)
                            } else {
                                batch.ledger.complete(&c)
                            };
                            if let Err(error) = result {
                                errors.push(format!("Cannot record start failure: {error:#}"));
                                break;
                            }
                        }
                    }
                }
            }
            if !self.active.is_empty() {
                thread::sleep(Duration::from_millis(100));
            }
        }
        if let Err(error) = batch.ledger.finish(&batch.id, false) {
            errors.push(format!("Cannot finish batch: {error:#}"));
        }
        match batch.ledger.summary(&batch.id) {
            Ok(current) => summary = current,
            Err(error) => errors.push(format!("Cannot read final state: {error:#}")),
        }
        Ok(BatchOutcome {
            summary,
            recording: batch.recording,
            recording_errors: errors,
            cancelled: stop.is_cancelled(),
        })
    }
    fn failed_batch_worker(
        &self,
        id: &str,
        task: &str,
        error: String,
        cancelled: bool,
    ) -> Result<Completion> {
        if self.store.dir(id)?.join("report.json").exists() {
            let mut report = self.store.load(id)?;
            if report.state.active() {
                report.finish(Some(error), cancelled)?;
                self.store.save(&report)?;
            }
            Completion::report(report)
        } else {
            Ok(Completion::no_report(id.into(), task.into(), error, false))
        }
    }
}
