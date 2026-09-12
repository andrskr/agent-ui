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
        crate::storage::valid_id(&reference)?;
        crate::storage::valid_id(&other)?;
        anyhow::ensure!(reference != other, "Select two different tasks");
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
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn incomplete_removal_blocks_replacement_and_survives_reload() {
        let mut index = Index::default();
        index.register("bare", "old".into()).unwrap();
        index.register("guided", "keep".into()).unwrap();
        assert_eq!(index.begin_removal("bare").as_deref(), Some("old"));
        let bytes = serde_json::to_vec(&index).unwrap();
        let mut recovered: Index = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(recovered.current("bare"), None);
        assert!(recovered.keeps("old"));
        assert!(recovered.register("bare", "new".into()).is_err());
        assert_eq!(recovered.begin_removal("bare").as_deref(), Some("old"));
        recovered.finish_removal("bare");
        recovered.register("bare", "new".into()).unwrap();
        assert_eq!(recovered.current("bare"), Some("new"));
        assert_eq!(recovered.current("guided"), Some("keep"));
        assert!(!recovered.keeps("old"));
    }
    #[test]
    fn saved_selection_keeps_swapped_pair_without_creating_approval_state() {
        let mut pair = TaskPair::new("bare".into(), "guided".into()).unwrap();
        pair.swap();
        let saved = serde_json::to_value(Selection {
            task: Some("guided".into()),
            pair: Some(pair),
            comparing: true,
        })
        .unwrap();
        assert_eq!(
            saved,
            serde_json::json!({"task":"guided","pair":{"reference":"guided","other":"bare"},"comparing":true})
        );
    }
}
