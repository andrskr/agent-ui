use crate::settings::Settings;
use crate::{
    journal::Journal,
    process::{self, Cancel},
    project::TaskSource,
    providers::{self, Purpose, Request},
    report::{Phase, Report, now},
    storage::Store,
    toolchain::Tools,
    workspace::{PreparedWorkspace, inventory},
};
use anyhow::Result;
use std::{fs::File, thread};

pub type Active = crate::worker::Worker<Report>;
pub fn start(
    store: Store,
    source: TaskSource,
    settings: Settings,
    tools: Tools,
    lock: File,
) -> Result<Active> {
    let id = format!("{}-{}", now(), &uuid::Uuid::new_v4().to_string()[..8]);
    let files = store.files(&id)?;
    let mut report = Report::new(id.clone(), source.id.clone(), files.app(), &settings);
    report.record("Run created");
    store.replace(&report)?;
    let cancel = Cancel::default();
    let worker_cancel = cancel.clone();
    let worker = thread::spawn(move || {
        let _lock = lock;
        let mut journal = Journal::new(store, report, worker_cancel)?;
        let result = execute(source, &settings, &tools, &mut journal);
        journal.finish(result)
    });
    Ok(Active {
        id,
        cancel,
        worker: Some(worker),
    })
}
fn execute(
    source: TaskSource,
    settings: &Settings,
    tools: &Tools,
    journal: &mut Journal,
) -> Result<()> {
    let provider = providers::get(&settings.provider)?;
    let store = journal.store().clone();
    let evidence = journal.files.evidence();
    let note = journal.files.agent_report();
    let (workspace, mut session) = journal.measure(Phase::Setup, |journal| {
        let versions = tools.versions()?;
        journal.report.provider_version = Some(versions.agent);
        journal.report.node_version = Some(versions.node);
        journal.report.vp_version = Some(versions.vp);
        let workspace = PreparedWorkspace::prepare(source, journal, tools)?;
        let request = Request {
            settings,
            cwd: &workspace.app,
            evidence: &evidence,
            note: &note,
            prompt: &workspace.prompt,
            purpose: Purpose::Task,
            read_roots: &[],
        };
        let session = provider.prepare(&store, &journal.report.id, tools, &request)?;
        Ok((workspace, session))
    })?;
    journal.measure(Phase::Agent, |journal| {
        journal.report.record("Agent started");
        let request = Request {
            settings,
            cwd: &workspace.app,
            evidence: &evidence,
            note: &note,
            prompt: &workspace.prompt,
            purpose: Purpose::Task,
            read_roots: &[],
        };
        let result = providers::execute(
            session.as_mut(),
            &store,
            &request,
            &journal.cancellation(),
            |observations, seconds| journal.agent_progress(seconds, observations),
        );
        let source = inventory(&workspace.app);
        if let Ok(after) = &source {
            journal.report.record_source_changes(after.clone());
        }
        journal
            .report
            .apply_cost(provider.estimate(&journal.report.cost_input(), Some(&evidence)));
        let result = result?;
        if let Ok(outcome) = &result.outcome {
            journal.agent_outcome(outcome);
        }
        process::checked(&result.outcome?, "Agent")?;
        result.artifacts?;
        source?;
        journal.report.check_agent_completion()
    })?;
    drop(session);
    journal.measure(Phase::Verification, |journal| {
        workspace.verify(journal, tools)
    })
}
