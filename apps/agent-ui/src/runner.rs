use crate::{
    codex::{self, Session, Tools},
    process::{self, Cancel},
    report::{Check, Report, State, now},
    storage::{CopyMode, Store, copy_tree, inventory, write_json},
};
use anyhow::{Context, Result, ensure};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

pub use crate::settings::Settings;
pub struct Active {
    pub id: String,
    cancel: Cancel,
    worker: Option<JoinHandle<Result<Report>>>,
}
impl Active {
    pub fn finished(&self) -> bool {
        self.worker.as_ref().is_none_or(JoinHandle::is_finished)
    }
    pub fn cancellation(&self) -> Cancel {
        self.cancel.clone()
    }
    pub fn cancel(&self) {
        self.cancel.cancel();
    }
    pub fn join(mut self) -> Result<Report> {
        self.worker
            .take()
            .context("Run was already joined")?
            .join()
            .map_err(|_| anyhow::anyhow!("Run worker panicked"))?
    }
}
impl Drop for Active {
    fn drop(&mut self) {
        self.cancel();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
pub fn start(store: Store, task: &str, settings: Settings) -> Result<Active> {
    settings.validate()?;
    let tools = Tools::discover(settings.codex.as_deref())?;
    let lock = store.lock()?;
    let task_path = store.task(task)?;
    let prompt = fs::read_to_string(task_path.join("task.md"))?;
    ensure!(!prompt.trim().is_empty(), "Task prompt is empty");
    let id = format!("{}-{}", now(), &uuid::Uuid::new_v4().to_string()[..8]);
    let dir = store.dir(&id)?;
    fs::create_dir(&dir)?;
    let mut report = Report::new(id.clone(), task.into(), dir.join("app"), &settings);
    report.record("Run created");
    store.save(&report)?;
    let cancel = Cancel::default();
    let worker_cancel = cancel.clone();
    let worker = thread::spawn(move || {
        let _lock = lock;
        let result = run(&store, &settings, &tools, &worker_cancel, &mut report);
        if let Err(error) = result {
            report.state = if worker_cancel.is_cancelled() {
                State::Cancelled
            } else {
                State::Failed
            };
            report.error = Some(format!("{error:#}"));
            report.record(format!("Run ended: {error:#}"));
        }
        report.finished_at_ms = Some(now());
        store.save(&report)?;
        Ok(report)
    });
    Ok(Active {
        id,
        cancel,
        worker: Some(worker),
    })
}
struct Execution<'a> {
    store: &'a Store,
    settings: &'a Settings,
    tools: &'a Tools,
    cancel: &'a Cancel,
    report: &'a mut Report,
    dir: PathBuf,
    evidence: PathBuf,
}
fn run(
    store: &Store,
    settings: &Settings,
    tools: &Tools,
    cancel: &Cancel,
    report: &mut Report,
) -> Result<()> {
    let dir = store.dir(&report.id)?;
    let mut execution = Execution {
        store,
        settings,
        tools,
        cancel,
        report,
        evidence: dir.join("evidence"),
        dir,
    };
    let session = execution.prepare()?;
    execution.agent(&session)?;
    execution.verify()
}
impl Execution<'_> {
    fn prepare(&mut self) -> Result<Session> {
        let Self {
            store,
            settings: _,
            tools,
            cancel,
            report,
            dir,
            evidence,
        } = self;
        let setup = Instant::now();
        let task_path = store.task(&report.task)?;
        fs::create_dir(&evidence)?;
        copy_tree(&task_path, &dir.join("inputs"), CopyMode::All)?;
        report.inputs = inventory(&dir.join("inputs"))?;
        copy_tree(
            &store.project().join("experiments/starter"),
            &report.app,
            CopyMode::Source,
        )?;
        for name in ["task.md", "AGENTS.md", "references"] {
            let source = dir.join("inputs").join(name);
            if source.is_dir() {
                copy_tree(&source, &report.app.join(name), CopyMode::All)?;
            } else if source.is_file() {
                fs::copy(&source, report.app.join(name))?;
            }
        }
        report.record("Copied the starter and task inputs");
        store.save(report)?;
        ensure!(!cancel.is_cancelled(), "Run cancelled during setup");
        report.codex_version = Some(codex::output(Command::new(&tools.codex).arg("--version"))?);
        report.node_version = Some(codex::output(Command::new(&tools.node).arg("--version"))?);
        report.vp_version = Some(codex::output(Command::new(&tools.vp).arg("--version"))?);
        let install = process::execute(
            Command::new(&tools.vp)
                .current_dir(&report.app)
                .args(["install", "--frozen-lockfile"])
                .env("CI", "1"),
            &evidence.join("install.log"),
            &evidence.join("install.stderr.log"),
            None,
            Duration::from_secs(300),
            cancel,
            |_, seconds| {
                report.setup_seconds = setup.elapsed().as_secs_f64();
                if seconds > 0.0 {
                    store.save(report)?;
                }
                Ok(())
            },
        )?;
        process::checked(&install, "Dependency setup")?;
        codex::output(
            Command::new("git")
                .current_dir(&report.app)
                .args(["init", "-q"]),
        )?;
        report.before = inventory(&report.app)?;
        let session = Session::new(store, &report.id, &report.app, tools)?;
        fs::write(evidence.join("codex-config.toml"), codex::CONFIG)?;
        write_json(&evidence.join("environment.json"), &session.environment())?;
        let login = process::execute(
            session
                .command(tools, &report.app)
                .args(["login", "status"]),
            &evidence.join("login.log"),
            &evidence.join("login.stderr.log"),
            None,
            Duration::from_secs(30),
            cancel,
            |_, _| Ok(()),
        )?;
        process::checked(&login, "Codex login")?;
        report.setup_seconds = setup.elapsed().as_secs_f64();
        Ok(session)
    }
    fn agent(&mut self, session: &Session) -> Result<()> {
        let Self {
            store,
            settings,
            tools,
            cancel,
            report,
            dir,
            evidence,
        } = self;
        let args = codex::exec_args(settings, &report.app, &dir.join("agent-report.md"))?;
        write_json(
            &evidence.join("command.json"),
            &serde_json::json!({"program": tools.codex, "args": args}),
        )?;
        let prompt = fs::read_to_string(dir.join("inputs/task.md"))?;
        fs::write(evidence.join("prompt.txt"), &prompt)?;
        report.state = State::Running;
        report.agent_started_at_ms = Some(now());
        report.record("Codex started");
        store.save(report)?;
        let result = process::execute(
            session.command(tools, &report.app).args(&args),
            &evidence.join("events.jsonl"),
            &evidence.join("codex.stderr.log"),
            Some(&prompt),
            Duration::from_secs(settings.timeout),
            cancel,
            |lines, seconds| {
                report.agent_seconds = Some(seconds);
                for line in lines {
                    report.event(line);
                }
                store.save(report)
            },
        );
        if let Ok(outcome) = &result {
            report.agent_exit_code = outcome.code;
            report.agent_seconds = Some(outcome.seconds);
        }
        let archived = session.archive(store, evidence);
        report.record_source_changes(inventory(&report.app)?);
        let result = result?;
        archived?;
        process::checked(&result, "Codex")?;
        report.check_agent_completion()
    }
    fn verify(&mut self) -> Result<()> {
        let Self {
            store,
            settings: _,
            tools,
            cancel,
            report,
            evidence,
            ..
        } = self;
        report.state = State::Verifying;
        report.record("Verification started");
        store.save(report)?;
        let check = process::execute(
            Command::new(&tools.vp)
                .current_dir(&report.app)
                .args(["run", "verify"])
                .env("CI", "1"),
            &evidence.join("verify.log"),
            &evidence.join("verify.stderr.log"),
            None,
            Duration::from_secs(300),
            cancel,
            |_, _| Ok(()),
        )?;
        report.verification = Some(Check {
            exit_code: check.code,
            seconds: check.seconds,
        });
        process::checked(&check, "Verification")?;
        report.state = State::Ready;
        report.record("Verification passed. Ready for human review");
        Ok(())
    }
}
