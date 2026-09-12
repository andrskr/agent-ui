use crate::settings::Settings;
use crate::{
    codex,
    journal::Journal,
    process::Cancel,
    project::TaskSource,
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
    let (workspace, session) = journal.measure(Phase::Setup, |journal| {
        let versions = tools.versions()?;
        journal.report.codex_version = Some(versions.codex);
        journal.report.node_version = Some(versions.node);
        journal.report.vp_version = Some(versions.vp);
        let workspace = PreparedWorkspace::prepare(source, journal, tools)?;
        let session = codex::prepare(journal, tools, &workspace.app)?;
        Ok((workspace, session))
    })?;
    journal.measure(Phase::Agent, |journal| {
        // Archive the session and source evidence even when the agent fails.
        let result = codex::execute(&session, settings, tools, &workspace, journal);
        let archived = session.archive(journal.store(), &journal.files.evidence());
        let source = inventory(&workspace.app);
        if let Ok(after) = source.as_ref() {
            journal.report.record_source_changes(after.clone());
        }
        result?;
        archived?;
        source?;
        Ok(())
    })?;
    drop(session);
    journal.measure(Phase::Verification, |journal| {
        workspace.verify(journal, tools)
    })
}
