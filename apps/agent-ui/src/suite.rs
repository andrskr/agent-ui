use crate::{
    profile::SetupPlan,
    project::{Project, TaskSource},
    task::{TaskId, valid_part},
};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    os::unix::fs::PermissionsExt,
    path::Path,
};

// Change this version when runner instructions or verification semantics change.
const EXECUTION_CONTRACT: &str = "agent-ui-batch-1";

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Suite {
    schema_version: u32,
    scenarios: Vec<String>,
    variants: Vec<String>,
}

impl Suite {
    pub fn parse(text: &str) -> Result<Self> {
        let suite: Self = toml::from_str(text).context("Invalid suite file")?;
        suite.task_ids()?;
        Ok(suite)
    }
    pub fn task_ids(&self) -> Result<Vec<String>> {
        ensure!(self.schema_version == 1, "Unsupported suite schema");
        for parts in [&self.scenarios, &self.variants] {
            let mut unique = BTreeSet::new();
            ensure!(!parts.is_empty(), "Suite lists must not be empty");
            for part in parts {
                ensure!(
                    valid_part(part) && unique.insert(part),
                    "Invalid or duplicate suite entry: {part}"
                );
            }
        }
        self.scenarios
            .iter()
            .flat_map(|s| self.variants.iter().map(move |v| format!("{s}--{v}")))
            .map(|id| {
                TaskId::parse(&id)?;
                Ok(id)
            })
            .collect()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct InputFile {
    bytes: Vec<u8>,
    mode: u32,
}
type Files = BTreeMap<String, InputFile>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct TaskSnapshot {
    pub id: String,
    inputs: Files,
    pub plan: SetupPlan,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Snapshot {
    pub suite: String,
    pub manifest: String,
    contract: String,
    starter: Files,
    pub tasks: Vec<TaskSnapshot>,
}

impl Project {
    pub(crate) fn suite(&self, name: &str) -> Result<Snapshot> {
        ensure!(valid_part(name) && name.len() <= 120, "Invalid suite name");
        let path = self
            .root()
            .join("experiments/suites")
            .join(format!("{name}.toml"));
        ensure!(
            fs::symlink_metadata(&path)?.is_file(),
            "Suite must be a regular file"
        );
        let manifest = fs::read_to_string(path)?;
        let suite = Suite::parse(&manifest)?;
        let mut tasks = Vec::new();
        for id in suite.task_ids()? {
            let source = self.task(&id)?;
            tasks.push(TaskSnapshot {
                id,
                inputs: read_tree(&source.path, false)?,
                plan: source.plan,
            });
        }
        let snapshot = Snapshot {
            suite: name.into(),
            manifest,
            contract: EXECUTION_CONTRACT.into(),
            starter: read_tree(&self.root().join("experiments/starter"), true)?,
            tasks,
        };
        snapshot.validate()?;
        Ok(snapshot)
    }
}

impl Snapshot {
    pub fn fingerprint(&self) -> Result<String> {
        fingerprint(self)
    }
    pub fn task_fingerprint(&self, task: &TaskSnapshot) -> Result<String> {
        fingerprint(&(&self.contract, &self.starter, task))
    }
    pub fn ids(&self) -> Vec<String> {
        self.tasks.iter().map(|t| t.id.clone()).collect()
    }
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.contract == EXECUTION_CONTRACT,
            "Runner contract changed. Start a new batch"
        );
        ensure!(
            Suite::parse(&self.manifest)?.task_ids()? == self.ids(),
            "Saved suite selection is inconsistent"
        );
        for name in ["package.json", "pnpm-lock.yaml", "pnpm-workspace.yaml"] {
            ensure!(self.starter.contains_key(name), "Starter is missing {name}");
        }
        let manifest: serde_json::Value =
            serde_json::from_slice(&self.starter["package.json"].bytes)?;
        for task in &self.tasks {
            TaskId::parse(&task.id)?;
            task.plan.task.validate()?;
            task.plan.packages.validate()?;
            task.plan.packages.apply(&manifest)?;
            let prompt = task
                .inputs
                .get("task.md")
                .context("Saved task has no prompt")?;
            ensure!(
                !std::str::from_utf8(&prompt.bytes)?.trim().is_empty(),
                "Task prompt is empty"
            );
            let saved_config = task
                .inputs
                .get("task.toml")
                .map(|file| crate::task::TaskConfig::parse(std::str::from_utf8(&file.bytes)?))
                .transpose()?
                .unwrap_or_default();
            ensure!(
                serde_json::to_value(saved_config)? == serde_json::to_value(&task.plan.task)?,
                "Task configuration changed while taking the snapshot"
            );
            if let Some(repair) = &task.plan.task.repair {
                ensure!(
                    cfg!(target_os = "macos") && task.plan.checks.contains_key(&repair.check),
                    "Saved repair setup is unavailable on this machine"
                );
            } else {
                ensure!(
                    manifest["scripts"]["verify"]
                        .as_str()
                        .is_some_and(|s| !s.trim().is_empty()),
                    "Starter must define a verify script"
                );
            }
            for path in task.plan.files.keys() {
                crate::profile::relative(path)?;
            }
        }
        Ok(())
    }
    pub fn materialize(&self, root: &Path) -> Result<()> {
        self.validate()?;
        // The caller uses a new directory. Never merge with an older materialization.
        fs::create_dir(root)?;
        write_tree(&root.join("starter"), &self.starter)?;
        for task in &self.tasks {
            write_tree(&root.join(&task.id), &task.inputs)?;
        }
        Ok(())
    }
    pub fn source(&self, root: &Path, id: &str) -> Result<TaskSource> {
        let task = self
            .tasks
            .iter()
            .find(|t| t.id == id)
            .context("Task is not in this batch")?;
        Ok(TaskSource {
            id: id.into(),
            path: root.join(id),
            starter: root.join("starter"),
            plan: task.plan.clone(),
        })
    }
}

pub(crate) fn fingerprint(value: &impl Serialize) -> Result<String> {
    Ok(format!("{:x}", Sha256::digest(serde_json::to_vec(value)?)))
}

fn read_tree(root: &Path, exclude_builds: bool) -> Result<Files> {
    fn visit(root: &Path, at: &Path, exclude: bool, files: &mut Files) -> Result<()> {
        ensure!(
            fs::symlink_metadata(at)?.is_dir(),
            "Input folder must not be a link"
        );
        for entry in fs::read_dir(at)? {
            let entry = entry?;
            if exclude
                && matches!(
                    entry.file_name().to_str(),
                    Some("node_modules" | "dist" | ".git" | ".agent-ui")
                )
            {
                continue;
            }
            let kind = entry.file_type()?;
            if kind.is_dir() {
                visit(root, &entry.path(), exclude, files)?;
            } else {
                ensure!(
                    kind.is_file(),
                    "Input must be a regular file: {}",
                    entry.path().display()
                );
                let path = entry.path();
                let name = path
                    .strip_prefix(root)?
                    .to_str()
                    .context("Input path must be UTF-8")?
                    .to_owned();
                files.insert(
                    name,
                    InputFile {
                        bytes: fs::read(&path)?,
                        mode: entry.metadata()?.permissions().mode() & 0o777,
                    },
                );
            }
        }
        Ok(())
    }
    let mut files = BTreeMap::new();
    visit(root, root, exclude_builds, &mut files)?;
    Ok(files)
}

fn write_tree(root: &Path, files: &Files) -> Result<()> {
    fs::create_dir_all(root)?;
    for (name, file) in files {
        crate::profile::relative(name)?;
        let target = root.join(name);
        fs::create_dir_all(target.parent().context("Invalid input path")?)?;
        fs::write(&target, &file.bytes)?;
        fs::set_permissions(target, fs::Permissions::from_mode(file.mode & 0o777))?;
    }
    Ok(())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    #[test]
    fn selection_is_complete_ordered_and_strict() {
        let text =
            "schema_version = 1\nscenarios = ['a','b']\nvariants = ['baseline','context','repair']";
        let suite = Suite::parse(text).unwrap();
        assert_eq!(
            suite.task_ids().unwrap(),
            [
                "a--baseline",
                "a--context",
                "a--repair",
                "b--baseline",
                "b--context",
                "b--repair"
            ]
        );
        for bad in [
            text.replace("['a','b']", "[]"),
            text.replace("'b'", "'a'"),
            text.replace("'b'", "'../b'"),
            text.replace("= 1", "= 2"),
            format!("{text}\nextra = true"),
        ] {
            assert!(Suite::parse(&bad).is_err());
        }
        let suite = Suite {
            schema_version: 1,
            scenarios: (1..=12).map(|i| format!("scenario-{i}")).collect(),
            variants: vec!["baseline".into(), "context".into(), "repair".into()],
        };
        assert_eq!(suite.task_ids().unwrap().len(), 36);
    }
    pub(crate) fn fixture() -> Snapshot {
        Snapshot {
            suite: "test".into(),
            manifest: "schema_version=1\nscenarios=['a']\nvariants=['baseline','context']".into(),
            contract: EXECUTION_CONTRACT.into(),
            starter: [
                (
                    "package.json",
                    br#"{"scripts":{"verify":"true"}}"#.as_slice(),
                ),
                ("pnpm-lock.yaml", b"lockfileVersion: '9.0'\n".as_slice()),
                ("pnpm-workspace.yaml", b"packages: []\n".as_slice()),
            ]
            .into_iter()
            .map(|(name, bytes)| {
                (
                    name.into(),
                    InputFile {
                        bytes: bytes.to_vec(),
                        mode: 0o644,
                    },
                )
            })
            .collect(),
            tasks: ["a--baseline", "a--context"]
                .map(|id| TaskSnapshot {
                    id: id.into(),
                    inputs: BTreeMap::from([(
                        "task.md".into(),
                        InputFile {
                            bytes: b"Build a form".to_vec(),
                            mode: 0o644,
                        },
                    )]),
                    plan: SetupPlan::default(),
                })
                .to_vec(),
        }
    }
    #[test]
    fn snapshot_hash_tracks_content_and_setup() {
        let mut snapshot = fixture();
        let original = snapshot.fingerprint().unwrap();
        snapshot.tasks[0]
            .plan
            .files
            .insert("policy.ts".into(), b"strict".to_vec());
        assert_ne!(original, snapshot.fingerprint().unwrap());
        let restored: Snapshot =
            serde_json::from_slice(&serde_json::to_vec(&snapshot).unwrap()).unwrap();
        assert_eq!(
            restored.fingerprint().unwrap(),
            snapshot.fingerprint().unwrap()
        );
        restored.validate().unwrap();
    }
    #[test]
    fn missing_starter_and_changed_task_configuration_fail_preflight() {
        let mut missing = fixture();
        missing.starter.remove("pnpm-lock.yaml");
        assert!(missing.validate().is_err());
        let mut changed = fixture();
        changed.tasks[0].inputs.insert(
            "task.toml".into(),
            InputFile {
                bytes: b"[repair]\ncheck='quality'".to_vec(),
                mode: 0o644,
            },
        );
        assert!(changed.validate().is_err());
        let mut empty = fixture();
        empty.tasks[0]
            .inputs
            .get_mut("task.md")
            .unwrap()
            .bytes
            .clear();
        assert!(empty.validate().is_err());
    }
}
