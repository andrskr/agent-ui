use crate::{
    artifact::Artifact,
    codex,
    preview::Preview,
    process::Cancel,
    project::Project,
    report::Report,
    runner::{self, Active},
    settings::Settings,
    storage::Store,
    toolchain::{self, Tools},
};
use anyhow::{Context, Result, ensure};
use std::{
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
    preview: Option<Preview>,
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
            preview: None,
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
    pub fn runs(&self) -> Result<Vec<Report>> {
        self.store.list()
    }
    pub fn report(&self, id: &str) -> Result<Report> {
        self.store.load(id)
    }
    pub fn has_active_run(&self) -> bool {
        self.active.is_some()
    }
    pub fn run_path(&self, id: &str) -> Result<PathBuf> {
        Ok(self.store.files(id)?.root().to_owned())
    }

    pub fn start(&mut self, task: &str, settings: Settings) -> Result<String> {
        ensure!(self.active.is_none(), "A run is already active");
        settings.validate()?;
        let source = self.project.task(task)?;
        let active = runner::start(self.store.clone(), source, settings)?;
        let id = active.id.clone();
        self.active = Some(active);
        Ok(id)
    }
    pub fn cancellation(&self) -> Option<Cancel> {
        self.active.as_ref().map(Active::cancellation)
    }
    pub fn cancel(&self) {
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

    pub fn preview(&self) -> Option<PreviewInfo<'_>> {
        self.preview.as_ref().map(|preview| PreviewInfo {
            id: preview.id(),
            url: preview.url(),
            ready: preview.is_ready(),
        })
    }
    pub fn start_preview(&mut self, id: &str) -> Result<()> {
        if self.preview.as_ref().is_some_and(|p| p.id() == id) {
            return Ok(());
        }
        let preview = Preview::start(&self.store, id, &toolchain::which("vp")?)?;
        self.preview = Some(preview);
        Ok(())
    }
    pub fn poll_preview(&mut self) -> Result<bool> {
        let result = self.preview.as_mut().map(Preview::poll).transpose();
        match result {
            Ok(ready) => Ok(ready.unwrap_or(false)),
            Err(error) => {
                self.preview = None;
                Err(error)
            }
        }
    }
    pub fn stop_preview(&mut self) {
        self.preview = None;
    }
    pub fn open_preview(&self) -> Result<()> {
        let preview = self.preview().context("No preview is running")?;
        ensure!(preview.ready, "Preview is still starting");
        open_url(preview.url)
    }
    pub fn open_artifact(&self, id: &str, artifact: Artifact) -> Result<()> {
        open_editor(&self.store.artifact(id, artifact)?)
    }
    pub fn agent_note(&self, id: &str) -> Result<String> {
        let path = self.store.files(id)?.agent_report();
        let mut bytes = Vec::new();
        File::open(path)?.take(4096).read_to_end(&mut bytes)?;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }
    pub fn remove(&mut self, id: &str) -> Result<()> {
        if self.preview.as_ref().is_some_and(|p| p.id() == id) {
            self.stop_preview();
        }
        self.store.remove(id)
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
