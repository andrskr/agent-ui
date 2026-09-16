use crate::{
    artifact::Artifact,
    comparison::Comparison,
    preview::Preview,
    process::Cancel,
    project::Project,
    providers,
    report::Report,
    runner::{self, Active},
    settings::Settings,
    storage::Store,
    task_result::{Selection, TaskDetails, TaskView},
    toolchain::{self, Tools},
};
use anyhow::{Context, Result, ensure};
use std::{
    collections::BTreeMap,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    process::Command,
};

const MAX_CONCURRENT_RUNS: usize = 4;

/// Owns live runs and previews. Both user interfaces call this service.
pub struct Application {
    project: Project,
    store: Store,
    active: BTreeMap<String, Active>,
    previews: BTreeMap<String, Preview>,
    queue: crate::run_queue::RunQueue,
    queue_errors: BTreeMap<String, String>,
    assessment: Option<crate::worker::Worker<crate::assessment::Assessment>>,
    _lock: File,
}

pub struct PreviewInfo<'a> {
    pub id: &'a str,
    pub url: &'a str,
    pub ready: bool,
}

impl Application {
    pub fn open(project: PathBuf, data: PathBuf) -> Result<Self> {
        let project = Project::open(project)?;
        let store = Store::new(data)?;
        ensure!(
            !store.root().starts_with(project.root()),
            "Run storage must be outside the repository"
        );
        store.recover()?;
        let lock = store
            .lock()
            .context("Another Agent UI instance is using this run storage")?;
        Ok(Self {
            project,
            store,
            active: BTreeMap::new(),
            previews: BTreeMap::new(),
            queue: Default::default(),
            queue_errors: BTreeMap::new(),
            assessment: None,
            _lock: lock,
        })
    }
    pub fn project(&self) -> &Path {
        self.project.root()
    }
    pub fn data(&self) -> &Path {
        self.store.root()
    }
    pub fn tasks(&self) -> Result<Vec<String>> {
        self.project.tasks()
    }
    pub fn task_views(&self) -> Result<Vec<TaskView>> {
        self.tasks()?
            .iter()
            .map(|task| self.store.task(task))
            .collect()
    }
    pub fn task_view(&self, task: &str) -> Result<TaskView> {
        self.store.task(task)
    }
    pub fn report(&self, task: &str) -> Result<Report> {
        self.store
            .task(task)?
            .run
            .context("This task has no current run")
    }
    pub fn task_details(&self, task: &str) -> Result<TaskDetails> {
        let source = self.project.task(task)?;
        let input = crate::project::TaskInput::load(&source.path)?;
        let inventory = crate::workspace::inventory(&source.path)?;
        let changed_since_run = self
            .store
            .task(task)?
            .run
            .is_some_and(|r| !r.inputs.is_empty() && r.inputs != inventory);
        Ok(TaskDetails {
            prompt: input.prompt,
            config: input.config,
            files: inventory.into_keys().collect(),
            changed_since_run,
        })
    }
    pub fn selection(&self) -> Result<Selection> {
        self.store.selection()
    }
    pub fn save_selection(&self, selection: &Selection) -> Result<()> {
        self.store.save_selection(selection)
    }
    pub fn is_busy(&self) -> bool {
        !self.active.is_empty() || !self.queue.is_empty() || self.assessment.is_some()
    }
    pub fn is_running(&self, task: &str) -> bool {
        self.active.contains_key(task) || self.queue.contains(task)
    }
    pub fn active_runs(&self) -> usize {
        self.active.len()
    }
    pub fn active_task_ids(&self) -> Vec<String> {
        self.active.keys().cloned().collect()
    }
    pub fn compare(&self, reference: &str, other: &str) -> Result<Comparison> {
        crate::task::comparison_group(reference, other)?;
        Comparison::new(self.report(reference)?, self.report(other)?)
    }
    pub fn start_assessment(
        &mut self,
        reference: &str,
        other: &str,
        settings: Settings,
    ) -> Result<()> {
        ensure!(!self.is_busy(), "An operation is already active");
        let comparison = self.compare(reference, other)?;
        self.assessment = Some(crate::assessment::start(
            self.store.clone(),
            comparison,
            settings,
        )?);
        Ok(())
    }
    pub fn join_assessment(&mut self) -> Result<crate::assessment::Assessment> {
        self.assessment
            .take()
            .context("No active assessment")?
            .join()
    }
    pub fn poll_assessment(&mut self) -> Result<Option<crate::assessment::Assessment>> {
        if self.assessment.as_ref().is_some_and(|a| a.finished()) {
            self.join_assessment().map(Some)
        } else {
            Ok(None)
        }
    }
    pub fn assessment_text(&self, comparison: &Comparison) -> Result<String> {
        comparison.pair.validate()?;
        let Some(report) = self.store.assessment()? else {
            return Ok(String::new());
        };
        if report.pair != comparison.pair {
            return Ok(String::new());
        }
        let path = self.store.assessment_dir().join("assessment.md");
        let note = if path.exists() {
            std::fs::read_to_string(path)?
        } else {
            String::new()
        };
        let usage = report
            .usage
            .as_ref()
            .map(|u| {
                format!(
                    "{} input · {} cached · {} output",
                    u.input_tokens, u.cached_input_tokens, u.output_tokens
                )
            })
            .unwrap_or_else(|| "Usage not reported".into());
        Ok(format!(
            "{} · {} · {} · {} · {:.1}s\n{}\nAPI cost (est): {}\n{}\n{}\n{}",
            report.state.label(),
            report.provider,
            report.model,
            report.effort,
            report.seconds,
            usage,
            crate::cost::format_usd(report.cost.as_ref().and_then(|cost| cost.usd)),
            report.visual_review,
            report.error.unwrap_or_default(),
            note
        ))
    }
    pub fn run_path(&self, id: &str) -> Result<PathBuf> {
        Ok(self.store.files(id)?.root().to_owned())
    }

    pub fn queued_task_ids(&self) -> Vec<String> {
        self.queue.ids()
    }
    pub fn queue_errors(&self) -> &BTreeMap<String, String> {
        &self.queue_errors
    }
    pub fn start_group(&mut self, group: &str, settings: Settings) -> Result<usize> {
        settings.validate()?;
        ensure!(
            self.assessment.is_none(),
            "Wait for the assessment to finish"
        );
        let members: Vec<_> = self
            .tasks()?
            .into_iter()
            .filter(|id| crate::task::TaskId::parse(id).is_ok_and(|id| id.group == group))
            .collect();
        ensure!(!members.is_empty(), "This group has no tasks");
        let tasks: Vec<_> = members
            .into_iter()
            .filter(|id| !self.is_running(id))
            .collect();
        ensure!(
            !tasks.is_empty(),
            "All tasks in this group are already running or queued"
        );
        for task in &tasks {
            self.project.task(task)?;
        }
        for task in &tasks {
            self.queue_errors.remove(task);
        }
        let count = tasks.len();
        self.queue.add(tasks, settings);
        self.dispatch_queue();
        Ok(count)
    }
    fn dispatch_queue(&mut self) {
        while let Some((task, settings)) = self.queue.next(self.active.len(), MAX_CONCURRENT_RUNS) {
            if let Err(error) = self.start(&task, settings) {
                self.queue_errors.insert(task, format!("{error:#}"));
            }
        }
    }
    pub fn cancel_assessment(&self) -> bool {
        if let Some(assessment) = &self.assessment {
            assessment.cancel();
            true
        } else {
            false
        }
    }
    pub fn cancel_group(&mut self, group: &str) {
        let tasks: Vec<_> = self
            .active_task_ids()
            .into_iter()
            .chain(self.queue.ids())
            .filter(|id| crate::task::TaskId::parse(id).is_ok_and(|id| id.group == group))
            .collect();
        for task in tasks {
            self.cancel_task(&task);
        }
    }
    pub fn start(&mut self, task: &str, settings: Settings) -> Result<String> {
        settings.validate()?;
        ensure!(
            self.assessment.is_none(),
            "Wait for the assessment to finish"
        );
        ensure!(!self.is_running(task), "This task is already running");
        ensure!(
            self.active.len() < MAX_CONCURRENT_RUNS,
            "{MAX_CONCURRENT_RUNS} runs are already active. Wait for one to finish"
        );
        let source = self.project.task(task)?;
        self.queue_errors.remove(task);
        self.stop_preview(task);
        let active = runner::start(self.store.clone(), source, settings)?;
        let id = active.id.clone();
        self.active.insert(task.to_string(), active);
        Ok(id)
    }
    pub fn cancellation(&self) -> Option<Cancel> {
        self.assessment
            .as_ref()
            .map(|a| a.cancellation())
            .or_else(|| {
                (self.active.len() == 1)
                    .then(|| self.active.values().next().map(Active::cancellation))
                    .flatten()
            })
    }
    pub fn cancel(&mut self) {
        self.queue.clear();
        if let Some(assessment) = &self.assessment {
            assessment.cancel();
        }
        for active in self.active.values() {
            active.cancel();
        }
    }
    pub fn cancel_task(&mut self, task: &str) -> bool {
        if self.queue.cancel(task) {
            return true;
        }
        match self.active.get(task) {
            Some(active) => {
                active.cancel();
                true
            }
            None => false,
        }
    }
    pub fn join(&mut self, task: &str) -> Result<Report> {
        self.active
            .remove(task)
            .context("No active run for this task")?
            .join()
    }
    pub fn poll_run(&mut self) -> Result<Vec<(String, Report)>> {
        let finished: Vec<String> = self
            .active
            .iter()
            .filter(|(_, active)| active.finished())
            .map(|(task, _)| task.clone())
            .collect();
        let result = finished
            .into_iter()
            .map(|task| Ok((task.clone(), self.join(&task)?)))
            .collect();
        self.dispatch_queue();
        result
    }

    pub fn preview(&self, task: &str) -> Option<PreviewInfo<'_>> {
        self.previews.get(task).map(|p| PreviewInfo {
            id: p.id(),
            url: p.url(),
            ready: p.is_ready(),
        })
    }
    pub fn start_preview(&mut self, task: &str) -> Result<()> {
        let id = self.report(task)?.id;
        if self.previews.get(task).is_some_and(|p| p.id() == id) {
            return Ok(());
        }
        let preview = Preview::start(&self.store, &id, &toolchain::which("vp")?)?;
        self.previews.insert(task.into(), preview);
        Ok(())
    }
    pub fn poll_previews(&mut self) -> Vec<(String, Result<bool>)> {
        let results: Vec<_> = self
            .previews
            .iter_mut()
            .map(|(task, p)| (task.clone(), p.poll()))
            .collect();
        for (task, result) in &results {
            if result.is_err() {
                self.previews.remove(task);
            }
        }
        results
    }
    pub fn stop_preview(&mut self, task: &str) {
        self.previews.remove(task);
    }
    pub fn open_preview(&self, task: &str) -> Result<()> {
        let preview = self
            .preview(task)
            .context("No preview is running for this task")?;
        ensure!(preview.ready, "Preview is still starting");
        open_url(preview.url)
    }
    pub fn open_artifact(&self, task: &str, artifact: Artifact) -> Result<()> {
        open_editor(&self.store.artifact(&self.report(task)?.id, artifact)?)
    }
    pub fn agent_note(&self, task: &str) -> Result<String> {
        let path = self.store.files(&self.report(task)?.id)?.agent_report();
        let mut bytes = Vec::new();
        File::open(path)?.take(4096).read_to_end(&mut bytes)?;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }
    pub fn login(&self, settings: &Settings) -> Result<()> {
        let provider = providers::get(&settings.provider)?;
        provider.login(
            &self.store,
            &provider.resolve_binary(settings.binary.as_deref())?,
        )
    }
    pub fn doctor(&self, settings: &Settings) -> Result<String> {
        let tools = Tools::discover(settings)?;
        let mut lines = Vec::new();
        for (name, path) in [
            (
                providers::descriptor(&settings.provider)?.label,
                &tools.agent,
            ),
            ("Node", &tools.node),
            ("Vite+", &tools.vp),
            ("Ripgrep", &tools.rg),
        ] {
            lines.push(format!(
                "{name}: {}\nPath: {}",
                toolchain::output(Command::new(path).arg("--version"))?,
                path.display()
            ));
        }
        lines.push(format!(
            "Project: {}\nRun storage: {}\nTasks: {}",
            self.project().display(),
            self.data().display(),
            self.tasks()?.len()
        ));
        Ok(lines.join("\n"))
    }
}

fn open_editor(path: &Path) -> Result<()> {
    let result = Command::new("/usr/bin/open")
        .args(["-a", "Visual Studio Code"])
        .arg(path)
        .output()?;
    ensure!(
        result.status.success(),
        "Cannot open VS Code: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(())
}
fn open_url(url: &str) -> Result<()> {
    ensure!(
        Command::new("/usr/bin/open").arg(url).status()?.success(),
        "Cannot open the browser"
    );
    Ok(())
}
