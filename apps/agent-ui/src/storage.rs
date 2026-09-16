use crate::{
    report::Report,
    task_result::{Index, Selection, Slot, TaskView},
};
use anyhow::{Context, Result, ensure};
use fs2::FileExt;
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub(crate) struct Store {
    root: PathBuf,
    index_lock: Arc<Mutex<()>>,
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
pub fn valid_run_id(id: &str) -> Result<()> {
    ensure!(
        !id.is_empty()
            && id.len() <= 120
            && !id.starts_with('.')
            && id
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_'),
        "Use a run ID with letters, numbers, '-' or '_'"
    );
    Ok(())
}
pub fn private_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    Ok(())
}
pub fn write_json(path: &Path, value: &impl serde::Serialize) -> Result<()> {
    let temp = path.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
    let mut file = File::create(&temp)?;
    serde_json::to_writer_pretty(&mut file, value)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    fs::rename(temp, path)?;
    File::open(path.parent().context("File has no parent")?)?.sync_all()?;
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
        Ok(Self {
            root,
            index_lock: Arc::new(Mutex::new(())),
        })
    }
    pub fn dir(&self, id: &str) -> Result<PathBuf> {
        valid_run_id(id)?;
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
            report.schema_version == 2 && report.id == id,
            "Unknown report format or ID"
        );
        Ok(report)
    }
    pub fn save(&self, report: &Report) -> Result<()> {
        write_json(&self.dir(&report.id)?.join("report.json"), report)
    }
    fn index(&self) -> Result<Index> {
        let path = self.root.join("current.json");
        if !path.exists() {
            return Ok(Index::default());
        }
        Ok(serde_json::from_slice(&fs::read(path)?)?)
    }
    fn save_index(&self, index: &Index) -> Result<()> {
        write_json(&self.root.join("current.json"), index)
    }
    pub fn task(&self, task: &str) -> Result<TaskView> {
        crate::task::TaskId::parse(task)?;
        let index = self.index()?;
        let run = index.current(task).map(|id| self.load(id)).transpose()?;
        if let Some(run) = &run {
            ensure!(run.task == task, "Saved run belongs to another task");
        }
        Ok(TaskView {
            id: task.into(),
            run,
            cleanup_pending: matches!(index.tasks.get(task), Some(Slot::Removing(_))),
        })
    }
    pub fn selection(&self) -> Result<Selection> {
        let path = self.root.join("selection.json");
        if !path.exists() {
            return Ok(Selection::default());
        }
        Ok(serde_json::from_slice(&fs::read(path)?)?)
    }
    pub fn save_selection(&self, selection: &Selection) -> Result<()> {
        write_json(&self.root.join("selection.json"), selection)
    }
    pub fn assessment_dir(&self) -> PathBuf {
        self.root.join("assessment")
    }
    pub fn assessment(&self) -> Result<Option<crate::assessment::Assessment>> {
        let path = self.assessment_dir().join("report.json");
        if !path.exists() {
            return Ok(None);
        }
        Ok(Some(serde_json::from_slice(&fs::read(path)?)?))
    }
    fn invalidate_assessment(&self, task: &str) -> Result<()> {
        let dir = self.assessment_dir();
        if dir.exists() && self.assessment()?.is_none_or(|a| a.pair.uses_task(task)) {
            fs::remove_dir_all(dir)?;
        }
        Ok(())
    }
    /// Concurrent runs write the shared index, so serialize it in this process. The lifetime file
    /// lock keeps other processes out.
    pub fn replace(&self, report: &Report) -> Result<()> {
        let _guard = self.index_lock.lock().unwrap_or_else(|e| e.into_inner());
        self.clear_task(&report.task)?;
        let mut index = self.index()?;
        fs::create_dir(self.dir(&report.id)?)?;
        self.save(report)?;
        index.register(&report.task, report.id.clone())?;
        self.save_index(&index)
    }
    fn clear_task(&self, task: &str) -> Result<()> {
        let mut index = self.index()?;
        let Some(id) = index.begin_removal(task) else {
            return Ok(());
        };
        // Take the preview lock before changing state or removing any files.
        let path = self.dir(&id)?;
        let _preview = path.exists().then(|| self.preview_lock(&id)).transpose()?;
        self.save_index(&index)?;
        self.invalidate_assessment(task)?;
        self.remove_files(&id)?;
        index.finish_removal(task);
        self.save_index(&index)
    }
    fn remove_files(&self, id: &str) -> Result<()> {
        let path = self.dir(id)?;
        if path.exists() {
            fs::remove_dir_all(path)?;
        }
        let private = self.root.join("private").join(id);
        if private.exists() {
            fs::remove_dir_all(private)?;
        }
        Ok(())
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
            .context("Another operation is active. Wait for it to finish or cancel it")
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
        if !self.root.join("current.json").exists() {
            self.save_selection(&Selection::default())?;
            self.save_index(&Index::default())?;
        }
        for (task, slot) in self.index()?.tasks {
            match slot {
                Slot::Removing(_) => {
                    if let Err(error) = self.clear_task(&task) {
                        eprintln!("Cleanup pending for {task}: {error:#}");
                    }
                }
                Slot::Current(id) => {
                    let mut report = self.load(&id)?;
                    if report.state.active() {
                        report.interrupt();
                        self.save(&report)?;
                    }
                }
            }
        }
        let index = self.index()?;
        // Unregistered folders are incomplete starts or obsolete output. Do not import them.
        for entry in fs::read_dir(self.root.join("runs"))? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let id = entry.file_name().to_string_lossy().into_owned();
            if !index.keeps(&id) {
                let result = (|| -> Result<()> {
                    let _preview = self.preview_lock(&id)?;
                    self.remove_files(&id)
                })();
                if let Err(error) = result {
                    eprintln!("Cannot remove obsolete output {id}: {error:#}");
                }
            }
        }
        if let Some(mut assessment) = self.assessment()? {
            if !assessment.pair.is_current(
                &index
                    .tasks
                    .iter()
                    .filter_map(|(task, slot)| match slot {
                        Slot::Current(id) => Some((task.clone(), id.clone())),
                        _ => None,
                    })
                    .collect(),
            ) {
                fs::remove_dir_all(self.assessment_dir())?;
            } else if assessment.state.active() {
                assessment.state = crate::report::State::Interrupted;
                assessment.error = Some("Comparison stopped before completion".into());
                write_json(&self.assessment_dir().join("report.json"), &assessment)?;
            }
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
