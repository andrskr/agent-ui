use crate::report::Report;
use anyhow::{Context, Result, ensure};
use fs2::FileExt;
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

#[derive(Clone)]
pub(crate) struct Store {
    root: PathBuf,
}

/// One owner for the public paths of a saved run.
#[derive(Clone)]
pub(crate) struct RunFiles {
    root: PathBuf,
}
impl RunFiles {
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn app(&self) -> PathBuf {
        self.root.join("app")
    }
    pub fn inputs(&self) -> PathBuf {
        self.root.join("inputs")
    }
    pub fn evidence(&self) -> PathBuf {
        self.root.join("evidence")
    }
    pub fn agent_report(&self) -> PathBuf {
        self.root.join("agent-report.md")
    }
}
pub fn valid_id(id: &str) -> Result<()> {
    ensure!(
        !id.is_empty()
            && id.len() <= 120
            && !id.starts_with('.')
            && id
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_'),
        "Use a task or run ID with letters, numbers, '-' or '_'"
    );
    Ok(())
}
pub fn private_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    Ok(())
}
pub fn write_json(path: &Path, value: &impl serde::Serialize) -> Result<()> {
    let temp = path.with_extension("json.tmp");
    let mut file = File::create(&temp)?;
    serde_json::to_writer_pretty(&mut file, value)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    fs::rename(temp, path)?;
    Ok(())
}
impl Store {
    pub fn files(&self, id: &str) -> Result<RunFiles> {
        Ok(RunFiles {
            root: self.dir(id)?,
        })
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn new(root: PathBuf) -> Result<Self> {
        private_dir(&root)?;
        let root = root.canonicalize()?;
        private_dir(&root.join("runs"))?;
        private_dir(&root.join("private"))?;
        Ok(Self { root })
    }
    pub fn dir(&self, id: &str) -> Result<PathBuf> {
        valid_id(id)?;
        let path = self.root.join("runs").join(id);
        if path.exists() {
            ensure!(
                !fs::symlink_metadata(&path)?.file_type().is_symlink(),
                "Run folder must not be a link"
            );
        }
        Ok(path)
    }
    pub fn load(&self, id: &str) -> Result<Report> {
        let report: Report = serde_json::from_slice(&fs::read(self.dir(id)?.join("report.json"))?)?;
        ensure!(
            report.schema_version == 1 && report.id == id,
            "Unknown report format or ID"
        );
        Ok(report)
    }
    pub fn save(&self, report: &Report) -> Result<()> {
        write_json(&self.dir(&report.id)?.join("report.json"), report)
    }
    pub fn list(&self) -> Result<Vec<Report>> {
        let mut reports = vec![];
        for entry in fs::read_dir(self.root.join("runs"))? {
            let entry = entry?;
            if entry.file_type()?.is_dir() && entry.path().join("report.json").exists() {
                reports.push(
                    self.load(&entry.file_name().to_string_lossy())
                        .with_context(|| format!("Cannot read run {}", entry.path().display()))?,
                );
            }
        }
        reports.sort_by_key(|r| std::cmp::Reverse(r.created_at_ms));
        Ok(reports)
    }
    fn try_run_lock(&self) -> Result<Option<File>> {
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(self.root.join("active.lock"))?;
        match file.try_lock_exclusive() {
            Ok(()) => Ok(Some(file)),
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
            Err(error) => Err(error).context("Cannot lock run storage"),
        }
    }
    pub fn lock(&self) -> Result<File> {
        self.try_run_lock()?
            .context("Another run is active. Wait for it to finish")
    }
    pub fn artifact(&self, id: &str, artifact: crate::artifact::Artifact) -> Result<PathBuf> {
        self.load(id)?;
        let path = self.dir(id)?.join(artifact.file_name());
        ensure!(path.exists(), "This output is not available");
        Ok(path)
    }
    pub fn recover(&self) -> Result<()> {
        let Some(_lock) = self.try_run_lock()? else {
            return Ok(());
        };
        for mut report in self.list()? {
            if report.state.active() {
                report.interrupt();
                self.save(&report)?;
            }
        }
        Ok(())
    }
    pub fn remove(&self, id: &str) -> Result<()> {
        let _lock = self.lock()?;
        let report = self.load(id)?;
        ensure!(!report.state.active(), "Cannot remove an active run");
        let _preview_lock = self.preview_lock(id)?;
        fs::remove_dir_all(self.dir(id)?)?;
        let private = self.root.join("private").join(id);
        if private.exists() {
            fs::remove_dir_all(private)?;
        }
        Ok(())
    }
    pub fn preview_lock(&self, id: &str) -> Result<File> {
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(self.dir(id)?.join("preview.lock"))?;
        file.try_lock_exclusive()
            .context("This run has an active preview. Stop it first")?;
        Ok(file)
    }
}
