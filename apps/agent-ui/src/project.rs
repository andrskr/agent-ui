use crate::{storage::valid_id, task::TaskConfig};
use anyhow::{Context, Result, ensure};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Read-only task catalog and starter location.
pub struct Project {
    root: PathBuf,
}

impl Project {
    pub fn open(root: PathBuf) -> Result<Self> {
        let root = root
            .canonicalize()
            .context("Project folder does not exist")?;
        ensure!(
            root.join("experiments/starter/package.json").is_file(),
            "Project has no experiment starter"
        );
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn tasks(&self) -> Result<Vec<String>> {
        let mut tasks = Vec::new();
        for entry in fs::read_dir(self.root.join("experiments/tasks"))? {
            let entry = entry?;
            let id = entry.file_name().to_string_lossy().into_owned();
            if !id.starts_with('_')
                && entry.file_type()?.is_dir()
                && entry.path().join("task.md").is_file()
            {
                valid_id(&id)?;
                tasks.push(id);
            }
        }
        tasks.sort();
        Ok(tasks)
    }

    pub fn task(&self, id: &str) -> Result<TaskSource> {
        valid_id(id)?;
        let path = self.root.join("experiments/tasks").join(id);
        ensure!(
            !id.starts_with('_') && path.join("task.md").is_file(),
            "Task '{id}' does not exist"
        );
        ensure!(
            fs::symlink_metadata(&path)?.is_dir(),
            "Task folder must not be a link"
        );
        TaskInput::load(&path)?;
        Ok(TaskSource {
            id: id.into(),
            path,
            starter: self.root.join("experiments/starter"),
        })
    }
}

/// Validated source paths. The run takes its own snapshot before using their contents.
pub struct TaskSource {
    pub(crate) id: String,
    pub(crate) path: PathBuf,
    pub(crate) starter: PathBuf,
}

pub(crate) struct TaskInput {
    pub prompt: String,
    pub config: TaskConfig,
}

impl TaskInput {
    pub fn load(folder: &Path) -> Result<Self> {
        let path = folder.join("task.toml");
        let config = match fs::symlink_metadata(&path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => TaskConfig::default(),
            Err(error) => return Err(error).context("Cannot inspect task.toml"),
            Ok(metadata) => {
                ensure!(metadata.is_file(), "task.toml must be a regular file");
                TaskConfig::parse(&fs::read_to_string(path).context("Cannot read task.toml")?)?
            }
        };
        let prompt = fs::read_to_string(folder.join("task.md"))?;
        ensure!(!prompt.trim().is_empty(), "Task prompt is empty");
        Ok(Self { prompt, config })
    }
}
