use crate::task_result::TaskView;
use chrono::{DateTime, Local};

#[derive(Default)]
pub(super) struct Tasks {
    pub items: Vec<TaskView>,
    pub query: String,
    selected: Option<String>,
}

pub(super) enum TaskRow {
    Task { index: usize, line: u16 },
    Gap,
}

pub(super) fn local_stamp(ms: u64) -> (String, String) {
    i64::try_from(ms)
        .ok()
        .and_then(DateTime::from_timestamp_millis)
        .map(|date| {
            let date = date.with_timezone(&Local);
            (
                date.format("%d %b %Y").to_string(),
                date.format("%H:%M").to_string(),
            )
        })
        .unwrap_or_else(|| ("Unknown date".into(), "—".into()))
}

impl Tasks {
    pub fn replace(&mut self, mut items: Vec<TaskView>) {
        items.sort_by(|a, b| a.id.cmp(&b.id));
        self.items = items;
        self.reconcile();
    }
    pub fn visible(&self) -> Vec<usize> {
        let words: Vec<_> = self
            .query
            .split_whitespace()
            .map(str::to_lowercase)
            .collect();
        self.items
            .iter()
            .enumerate()
            .filter_map(|(index, run)| {
                let text = format!("{} {}", run.id, run.status()).to_lowercase();
                words
                    .iter()
                    .all(|word| text.contains(word))
                    .then_some(index)
            })
            .collect()
    }
    pub fn current(&self) -> Option<&TaskView> {
        self.selected
            .as_ref()
            .and_then(|id| self.items.iter().find(|run| &run.id == id))
    }
    pub fn select(&mut self, index: usize) {
        if let Some(run) = self.items.get(index) {
            self.selected = Some(run.id.clone());
        }
    }
    pub fn select_id(&mut self, id: &str) {
        if let Some(index) = self.items.iter().position(|run| run.id == id) {
            self.select(index);
        }
    }
    pub fn set_query(&mut self, query: String) {
        self.query = query;
        self.reconcile();
    }
    fn reconcile(&mut self) {
        let visible = self.visible();
        if !visible
            .iter()
            .any(|&index| self.selected.as_deref() == Some(&self.items[index].id))
        {
            self.selected = visible.first().map(|&index| self.items[index].id.clone());
        }
    }
    pub fn navigate(&mut self, delta: isize) {
        let visible = self.visible();
        if visible.is_empty() {
            return;
        }
        let current = visible
            .iter()
            .position(|&index| self.selected.as_deref() == Some(&self.items[index].id))
            .unwrap_or(0);
        let next = current.saturating_add_signed(delta).min(visible.len() - 1);
        self.select(visible[next]);
    }
    pub fn window(&self, height: u16) -> Vec<TaskRow> {
        let visible = self.visible();
        let selected = visible
            .iter()
            .position(|&index| self.selected.as_deref() == Some(&self.items[index].id))
            .unwrap_or(0);
        let capacity = (usize::from(height) / 4).max(1);
        let start = selected.saturating_sub(capacity - 1);
        let mut rows = Vec::new();
        for &index in visible.iter().skip(start).take(capacity) {
            for line in 0..3 {
                rows.push(TaskRow::Task { index, line });
            }
            rows.push(TaskRow::Gap);
        }
        rows.truncate(usize::from(height));
        rows
    }
}
