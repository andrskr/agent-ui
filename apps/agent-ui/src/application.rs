use crate::{
    artifact::Artifact,
    codex,
    comparison::Comparison,
    preview::Preview,
    process::Cancel,
    project::Project,
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

/// Owns live runs and previews. Both user interfaces call this service.
pub struct Application {
    project: Project,
    store: Store,
    active: Option<Active>,
    previews: BTreeMap<String, Preview>,
    assessment: Option<crate::worker::Worker<crate::assessment::Assessment>>,
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
        Ok(Self {
            project,
            store,
            active: None,
            previews: BTreeMap::new(),
            assessment: None,
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
        self.active.is_some() || self.assessment.is_some()
    }
    pub fn compare(&self, reference: &str, other: &str) -> Result<Comparison> {
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
            "{} · {} · {} · {:.1}s\n{}\n{}\n{}\n{}",
            report.state.label(),
            report.model,
            report.effort,
            report.seconds,
            usage,
            report.visual_review,
            report.error.unwrap_or_default(),
            note
        ))
    }
    pub fn run_path(&self, id: &str) -> Result<PathBuf> {
        Ok(self.store.files(id)?.root().to_owned())
    }

    pub fn start(&mut self, task: &str, settings: Settings) -> Result<String> {
        ensure!(!self.is_busy(), "An operation is already active");
        settings.validate()?;
        let source = self.project.task(task)?;
        let tools = Tools::discover(settings.codex.as_deref())?;
        let lock = self.store.lock()?;
        self.stop_preview(task);
        let active = runner::start(self.store.clone(), source, settings, tools, lock)?;
        let id = active.id.clone();
        self.active = Some(active);
        Ok(id)
    }
    pub fn cancellation(&self) -> Option<Cancel> {
        self.active
            .as_ref()
            .map(Active::cancellation)
            .or_else(|| self.assessment.as_ref().map(|a| a.cancellation()))
    }
    pub fn cancel(&self) {
        if let Some(assessment) = &self.assessment {
            assessment.cancel();
        }
        if let Some(active) = &self.active {
            active.cancel();
        }
    }
    pub fn join(&mut self) -> Result<Report> {
        self.active.take().context("No active run")?.join()
    }
    pub fn poll_run(&mut self) -> Result<Option<Report>> {
        if self.active.as_ref().is_some_and(Active::finished) {
            self.join().map(Some)
        } else {
            Ok(None)
        }
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
    pub fn login(&self, codex: Option<&Path>) -> Result<()> {
        codex::login(&self.store, &Tools::discover(codex)?)
    }
    pub fn doctor(&self, codex: Option<&Path>) -> Result<String> {
        let tools = Tools::discover(codex)?;
        let mut lines = Vec::new();
        for (name, path) in [
            ("Codex", &tools.codex),
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
        lines.push(format!(
            "Credential file: {}",
            if self.data().join("private/auth.json").is_file() {
                "present"
            } else {
                "imported from Codex on first run, or use login"
            }
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
