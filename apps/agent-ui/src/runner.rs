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
use std::thread;

pub type Active = crate::worker::Worker<Report>;
pub fn start(store: Store, source: TaskSource, settings: Settings) -> Result<Active> {
    start_with_id(store, source, settings, new_id())
}
pub(crate) fn new_id() -> String {
    format!("{}-{}", now(), uuid::Uuid::new_v4())
}
pub(crate) fn start_with_id(
    store: Store,
    source: TaskSource,
    settings: Settings,
    id: String,
) -> Result<Active> {
    let cancel = Cancel::default();
    let worker_cancel = cancel.clone();
    let worker_id = id.clone();
    let worker = thread::Builder::new()
        .name(format!("run-{id}"))
        .spawn(move || run(worker_id, store, source, settings, worker_cancel))?;
    Ok(Active {
        id,
        cancel,
        worker: Some(worker),
    })
}
fn run(
    id: String,
    store: Store,
    source: TaskSource,
    settings: Settings,
    cancel: Cancel,
) -> Result<Report> {
    let files = store.files(&id)?;
    let mut report = Report::new(id, source.id.clone(), files.app(), &settings);
    report.record("Run created");
    store.replace(&report)?;
    let mut journal = Journal::new(store, report, cancel)?;
    let result = execute(source, &settings, &mut journal);
    journal.finish(result)
}
fn execute(source: TaskSource, settings: &Settings, journal: &mut Journal) -> Result<()> {
    let provider = providers::get(&settings.provider)?;
    let store = journal.store().clone();
    let evidence = journal.files.evidence();
    let note = journal.files.agent_report();
    let (workspace, mut session, tools, mut repair) = journal.measure(Phase::Setup, |journal| {
        journal.step("Checking tools")?;
        let tools = Tools::discover(settings)?;
        journal.step("Checking sign-in")?;
        provider.preflight(&store, &tools.agent)?;
        journal.step("Checking tool versions")?;
        let versions = tools.versions()?;
        journal.report.provider_version = Some(versions.agent);
        journal.report.node_version = Some(versions.node);
        journal.report.vp_version = Some(versions.vp);
        let workspace = PreparedWorkspace::prepare(source, journal, &tools)?;
        let mut repair = workspace
            .repair
            .as_ref()
            .map(|(name, definition)| {
                crate::repair::Controller::prepare(
                    &workspace.app,
                    name,
                    definition.clone(),
                    &tools,
                    journal,
                )
            })
            .transpose()?;
        if let Some(repair) = &mut repair {
            repair.preflight(journal)?;
        }
        journal.step("Resolving the agent")?;
        let request = Request {
            settings,
            cwd: &workspace.app,
            evidence: &evidence,
            note: &note,
            prompt: &workspace.prompt,
            purpose: Purpose::Task,
            read_roots: &[],
        };
        let session = provider.prepare(&store, &journal.report.id, &tools, &request)?;
        Ok((workspace, session, tools, repair))
    })?;
    journal.measure(Phase::Agent, |journal| {
        journal.report.record("Agent started");
        journal.mark("Waiting for the agent to respond")?;
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
            journal.recorder.clone(),
            |observations, seconds| {
                journal.agent_progress(seconds, observations)?;
                if let Some(repair) = &mut repair {
                    repair.poll(
                        journal,
                        std::time::Duration::from_secs(settings.timeout)
                            .saturating_sub(std::time::Duration::from_secs_f64(seconds)),
                    )?;
                }
                Ok(())
            },
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
        if let Some(repair) = &mut repair {
            repair.verify(journal)
        } else {
            workspace.verify(journal, &tools)
        }
    })
}
