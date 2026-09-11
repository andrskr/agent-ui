use crate::report::{Report, State, now};
use anyhow::{Context, Result, bail, ensure};
use fs2::FileExt;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::Write,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

#[derive(Clone)]
pub struct Store {
    project: PathBuf,
    root: PathBuf,
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
#[derive(Clone, Copy)]
pub enum CopyMode {
    All,
    Source,
}
pub fn copy_tree(source: &Path, target: &Path, mode: CopyMode) -> Result<()> {
    ensure!(
        !fs::symlink_metadata(source)?.file_type().is_symlink(),
        "Source must not be a link: {}",
        source.display()
    );
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let name = entry.file_name();
        if matches!(mode, CopyMode::Source)
            && matches!(name.to_str(), Some("node_modules" | "dist" | ".git"))
        {
            continue;
        }
        let kind = entry.file_type()?;
        ensure!(
            !kind.is_symlink(),
            "Input must not be a link: {}",
            entry.path().display()
        );
        if kind.is_dir() {
            copy_tree(&entry.path(), &target.join(name), mode)?;
        } else if kind.is_file() {
            fs::copy(entry.path(), target.join(name))?;
        } else {
            bail!("Input must be a regular file: {}", entry.path().display());
        }
    }
    Ok(())
}
pub fn inventory(root: &Path) -> Result<BTreeMap<String, String>> {
    fn visit(root: &Path, path: &Path, result: &mut BTreeMap<String, String>) -> Result<()> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if matches!(
                entry.file_name().to_str(),
                Some("node_modules" | "dist" | ".git")
            ) {
                continue;
            }
            let kind = entry.file_type()?;
            if kind.is_dir() {
                visit(root, &entry.path(), result)?;
            } else if kind.is_file() {
                result.insert(
                    entry
                        .path()
                        .strip_prefix(root)?
                        .to_string_lossy()
                        .into_owned(),
                    format!("{:x}", Sha256::digest(fs::read(entry.path())?)),
                );
            } else if kind.is_symlink() {
                result.insert(
                    entry
                        .path()
                        .strip_prefix(root)?
                        .to_string_lossy()
                        .into_owned(),
                    format!("symlink:{}", fs::read_link(entry.path())?.display()),
                );
            }
        }
        Ok(())
    }
    let mut result = BTreeMap::new();
    visit(root, root, &mut result)?;
    Ok(result)
}
impl Store {
    pub fn project(&self) -> &Path {
        &self.project
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn new(project: PathBuf, root: PathBuf) -> Result<Self> {
        let project = project
            .canonicalize()
            .context("Project folder does not exist")?;
        ensure!(
            project.join("experiments/starter/package.json").is_file(),
            "Project has no experiment starter"
        );
        private_dir(&root)?;
        let root = root.canonicalize()?;
        ensure!(
            !root.starts_with(&project),
            "Run storage must be outside the repository"
        );
        private_dir(&root.join("runs"))?;
        private_dir(&root.join("private"))?;
        Ok(Self { project, root })
    }
    pub fn tasks(&self) -> Result<Vec<String>> {
        let mut tasks = vec![];
        for entry in fs::read_dir(self.project.join("experiments/tasks"))? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().into_owned();
            if !name.starts_with('_')
                && entry.file_type()?.is_dir()
                && entry.path().join("task.md").is_file()
            {
                valid_id(&name)?;
                tasks.push(name);
            }
        }
        tasks.sort();
        Ok(tasks)
    }
    pub fn task(&self, id: &str) -> Result<PathBuf> {
        valid_id(id)?;
        let path = self.project.join("experiments/tasks").join(id);
        ensure!(
            !id.starts_with('_') && path.join("task.md").is_file(),
            "Task '{id}' does not exist"
        );
        ensure!(
            !fs::symlink_metadata(&path)?.file_type().is_symlink(),
            "Task folder must not be a link"
        );
        ensure!(
            !path.join("task.toml").exists(),
            "task.toml is not supported in this version"
        );
        Ok(path)
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
                report.state = State::Interrupted;
                report.finished_at_ms = Some(now());
                report.error = Some("The runner stopped before it saved a final result".into());
                report.record("Recovered an interrupted run");
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
