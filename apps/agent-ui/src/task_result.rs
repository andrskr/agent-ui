use crate::{report::Report, task::TaskConfig};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "state", content = "run_id", rename_all = "snake_case")]
pub(crate) enum Slot {
    Current(String),
    Removing(String),
}
#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Index {
    pub tasks: BTreeMap<String, Slot>,
}
impl Index {
    pub fn current(&self, task: &str) -> Option<&str> {
        match self.tasks.get(task) {
            Some(Slot::Current(id)) => Some(id),
            _ => None,
        }
    }
    pub fn keeps(&self, id: &str) -> bool {
        self.tasks.values().any(|slot| match slot {
            Slot::Current(run) | Slot::Removing(run) => run == id,
        })
    }
    pub fn begin_removal(&mut self, task: &str) -> Option<String> {
        let id = match self.tasks.get(task)? {
            Slot::Current(id) | Slot::Removing(id) => id.clone(),
        };
        self.tasks.insert(task.into(), Slot::Removing(id.clone()));
        Some(id)
    }
    pub fn finish_removal(&mut self, task: &str) {
        self.tasks.remove(task);
    }
    pub fn register(&mut self, task: &str, id: String) -> anyhow::Result<()> {
        anyhow::ensure!(
            !self.tasks.contains_key(task),
            "Previous output has not been removed"
        );
        self.tasks.insert(task.into(), Slot::Current(id));
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct TaskView {
    pub id: String,
    pub run: Option<Report>,
    pub cleanup_pending: bool,
}
impl TaskView {
    pub fn status(&self) -> &str {
        if self.cleanup_pending {
            "Cleanup blocked"
        } else {
            self.run
                .as_ref()
                .map(|r| r.state.label())
                .unwrap_or("Not run")
        }
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct TaskDetails {
    pub prompt: String,
    pub config: TaskConfig,
    pub files: Vec<String>,
    pub changed_since_run: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskPair {
    pub reference: String,
    pub other: String,
}
impl TaskPair {
    pub fn new(reference: String, other: String) -> anyhow::Result<Self> {
        crate::task::comparison_group(&reference, &other)?;
        Ok(Self { reference, other })
    }
    pub fn swap(&mut self) {
        std::mem::swap(&mut self.reference, &mut self.other);
    }
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Selection {
    pub task: Option<String>,
    pub pair: Option<TaskPair>,
    pub comparing: bool,
    #[serde(default)]
    pub task_level: bool,
}

impl Selection {
    /// Remove unavailable selections before restoring the task view.
    pub fn reconcile(&mut self, tasks: &[TaskView]) -> Option<String> {
        if !tasks.iter().any(|t| Some(&t.id) == self.task.as_ref()) {
            self.task = tasks.first().map(|t| t.id.clone());
        }
        let valid_pair = self.pair.as_ref().is_some_and(|pair| {
            crate::task::comparison_group(&pair.reference, &pair.other).is_ok()
                && [&pair.reference, &pair.other].iter().all(|id| {
                    tasks.iter().any(|task| {
                        &task.id == *id
                            && !task.cleanup_pending
                            && task.run.as_ref().is_some_and(|run| &run.task == *id)
                    })
                })
        });
        if !valid_pair && (self.pair.is_some() || self.comparing) {
            self.pair = None;
            self.comparing = false;
            return Some("Saved comparison is unavailable. Select two task variants from the same group with saved runs.".into());
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn incomplete_removal_blocks_replacement_and_survives_reload() {
        let mut index = Index::default();
        index.register("smoke--baseline", "old".into()).unwrap();
        index.register("smoke--context", "keep".into()).unwrap();
        assert_eq!(
            index.begin_removal("smoke--baseline").as_deref(),
            Some("old")
        );
        let bytes = serde_json::to_vec(&index).unwrap();
        let mut recovered: Index = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(recovered.current("smoke--baseline"), None);
        assert!(recovered.keeps("old"));
        assert!(recovered.register("smoke--baseline", "new".into()).is_err());
        assert_eq!(
            recovered.begin_removal("smoke--baseline").as_deref(),
            Some("old")
        );
        recovered.finish_removal("smoke--baseline");
        recovered.register("smoke--baseline", "new".into()).unwrap();
        assert_eq!(recovered.current("smoke--baseline"), Some("new"));
        assert_eq!(recovered.current("smoke--context"), Some("keep"));
        assert!(!recovered.keeps("old"));
    }
    #[test]
    fn saved_selection_keeps_swapped_pair_without_creating_approval_state() {
        let mut pair = TaskPair::new("smoke--baseline".into(), "smoke--context".into()).unwrap();
        pair.swap();
        let saved = serde_json::to_value(Selection {
            task: Some("smoke--context".into()),
            pair: Some(pair),
            comparing: true,
            task_level: true,
        })
        .unwrap();
        assert_eq!(
            saved,
            serde_json::json!({"task":"smoke--context","pair":{"reference":"smoke--context","other":"smoke--baseline"},"comparing":true,"task_level":true})
        );
    }
}
