use crate::report::Report;
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunRef {
    pub task: String,
    pub run: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pair {
    pub reference: RunRef,
    pub other: RunRef,
}
impl Pair {
    pub fn uses_task(&self, task: &str) -> bool {
        self.reference.task == task || self.other.task == task
    }
    pub fn is_current(&self, current: &BTreeMap<String, String>) -> bool {
        current.get(&self.reference.task) == Some(&self.reference.run)
            && current.get(&self.other.task) == Some(&self.other.run)
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct TimeDelta {
    pub reference: Option<f64>,
    pub other: Option<f64>,
    pub difference: Option<f64>,
}
impl TimeDelta {
    fn new(reference: Option<f64>, other: Option<f64>) -> Self {
        Self {
            reference,
            other,
            difference: reference.zip(other).map(|(a, b)| b - a),
        }
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct TokenDelta {
    pub reference: Option<u64>,
    pub other: Option<u64>,
    pub difference: Option<i128>,
}
impl TokenDelta {
    fn new(reference: Option<u64>, other: Option<u64>) -> Self {
        Self {
            reference,
            other,
            difference: reference
                .zip(other)
                .map(|(a, b)| i128::from(b) - i128::from(a)),
        }
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct Measurements {
    pub estimated_cost_usd: CostDelta,
    pub setup_seconds: TimeDelta,
    pub agent_seconds: TimeDelta,
    pub verification_seconds: TimeDelta,
    pub input_tokens: TokenDelta,
    pub cached_input_tokens: TokenDelta,
    pub output_tokens: TokenDelta,
}

#[derive(Clone, Debug, Serialize)]
pub struct CostDelta {
    pub reference: Option<f64>,
    pub other: Option<f64>,
    pub difference: Option<f64>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    Added,
    Removed,
    Modified,
}
#[derive(Clone, Debug, Serialize)]
pub struct FileChange {
    pub path: String,
    pub kind: ChangeKind,
}
pub fn file_changes(a: &BTreeMap<String, String>, b: &BTreeMap<String, String>) -> Vec<FileChange> {
    a.keys()
        .chain(b.keys())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter_map(|path| {
            let kind = match (a.get(path), b.get(path)) {
                (None, Some(_)) => ChangeKind::Added,
                (Some(_), None) => ChangeKind::Removed,
                (Some(a), Some(b)) if a != b => ChangeKind::Modified,
                _ => return None,
            };
            Some(FileChange {
                path: path.clone(),
                kind,
            })
        })
        .collect()
}
#[derive(Clone, Debug, Serialize)]
pub struct Comparison {
    pub pair: Pair,
    pub measurements: Measurements,
    pub input_changes: Option<Vec<FileChange>>,
    pub source_changes: Option<Vec<FileChange>>,
    pub setup_changes: Option<Vec<FileChange>>,
    pub reference: Report,
    pub other: Report,
}
impl Comparison {
    pub fn new(reference: Report, other: Report) -> Result<Self> {
        ensure!(reference.task != other.task, "Select two different tasks");
        let a = &reference;
        let b = &other;
        let pair = Pair {
            reference: RunRef {
                task: a.task.clone(),
                run: a.id.clone(),
            },
            other: RunRef {
                task: b.task.clone(),
                run: b.id.clone(),
            },
        };
        let measurements = Measurements {
            estimated_cost_usd: CostDelta {
                reference: a.cost_usd,
                other: b.cost_usd,
                difference: a.cost_usd.zip(b.cost_usd).map(|(a, b)| b - a),
            },
            setup_seconds: TimeDelta::new(
                (a.state != crate::report::State::Preparing).then_some(a.setup_seconds),
                (b.state != crate::report::State::Preparing).then_some(b.setup_seconds),
            ),
            agent_seconds: TimeDelta::new(a.agent_seconds, b.agent_seconds),
            verification_seconds: TimeDelta::new(
                a.verification.as_ref().map(|v| v.seconds),
                b.verification.as_ref().map(|v| v.seconds),
            ),
            input_tokens: TokenDelta::new(
                a.usage.as_ref().map(|u| u.input_tokens),
                b.usage.as_ref().map(|u| u.input_tokens),
            ),
            cached_input_tokens: TokenDelta::new(
                a.usage.as_ref().map(|u| u.cached_input_tokens),
                b.usage.as_ref().map(|u| u.cached_input_tokens),
            ),
            output_tokens: TokenDelta::new(
                a.usage.as_ref().map(|u| u.output_tokens),
                b.usage.as_ref().map(|u| u.output_tokens),
            ),
        };
        let input_changes = (!a.inputs.is_empty() && !b.inputs.is_empty())
            .then(|| file_changes(&a.inputs, &b.inputs));
        let source_changes =
            (!a.after.is_empty() && !b.after.is_empty()).then(|| file_changes(&a.after, &b.after));
        let setup_changes = (!a.before.is_empty() && !b.before.is_empty())
            .then(|| file_changes(&a.before, &b.before));
        Ok(Self {
            pair,
            measurements,
            input_changes,
            source_changes,
            setup_changes,
            reference,
            other,
        })
    }
    pub fn can_assess(&self) -> bool {
        !self.reference.state.active() && !self.other.state.active()
    }
}
