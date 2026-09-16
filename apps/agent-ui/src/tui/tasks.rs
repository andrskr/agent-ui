use crate::task_result::TaskView;
use chrono::{DateTime, Local};

#[derive(Default)]
pub(super) struct Tasks {
    pub items: Vec<TaskView>,
    pub query: String,
    selected: Option<String>,
    level: Level,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Level {
    #[default]
    Flat,
    Group,
    Task,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum TaskRow {
    Group { index: usize },
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

pub(super) fn group(id: &str) -> &str {
    crate::task::TaskId::parse(id).map_or(id, |id| id.group)
}
pub(super) fn label(id: &str) -> String {
    id.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
pub(super) fn variant(id: &str) -> &str {
    crate::task::TaskId::parse(id).map_or(id, |id| id.variant)
}

impl Tasks {
    pub fn grouped() -> Self {
        Self {
            level: Level::Group,
            ..Self::default()
        }
    }
    pub fn is_group(&self) -> bool {
        self.level == Level::Group
    }
    pub fn is_grouped(&self) -> bool {
        self.level != Level::Flat
    }
    pub fn enter(&mut self) {
        if self.current().is_some() && self.is_grouped() {
            self.level = Level::Task;
        }
    }
    pub fn leave(&mut self) {
        if self.is_grouped() {
            self.level = Level::Group;
        }
    }
    pub fn group(&self) -> Option<&str> {
        self.current().map(|task| group(&task.id))
    }
    pub fn members(&self) -> Vec<&TaskView> {
        self.items
            .iter()
            .filter(|task| Some(group(&task.id)) == self.group())
            .collect()
    }
    pub fn reference_picker(&self) -> Self {
        let mut picker = Self::default();
        picker.replace(self.members().into_iter().cloned().collect());
        picker
    }
    pub fn select_row(&mut self, row: &TaskRow) -> bool {
        match *row {
            TaskRow::Group { index } => {
                self.select(index);
                self.leave();
            }
            TaskRow::Task { index, .. } => {
                self.select(index);
                self.enter();
            }
            TaskRow::Gap => return false,
        }
        true
    }

    pub fn comparison_picker(&self, reference: &str) -> anyhow::Result<Self> {
        crate::task::TaskId::parse(reference)?;
        let mut picker = Self::default();
        picker.replace(
            self.items
                .iter()
                .filter(|task| crate::task::comparison_group(reference, &task.id).is_ok())
                .cloned()
                .collect(),
        );
        Ok(picker)
    }

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
            let old_group = self.selected.as_deref().map(group).map(str::to_owned);
            self.selected = visible
                .iter()
                .find(|&&index| Some(group(&self.items[index].id)) == old_group.as_deref())
                .or(visible.first())
                .map(|&index| self.items[index].id.clone());
            if self.group() != old_group.as_deref() {
                self.leave();
            }
        }
    }
    pub fn navigate(&mut self, delta: isize) {
        let mut visible = self.visible();
        if self.is_group() {
            visible.dedup_by(|a, b| group(&self.items[*a].id) == group(&self.items[*b].id));
        } else if self.is_grouped() {
            visible.retain(|&index| Some(group(&self.items[index].id)) == self.group());
        }
        if visible.is_empty() {
            return;
        }
        let current = visible
            .iter()
            .position(|&index| {
                if self.is_group() {
                    Some(group(&self.items[index].id)) == self.group()
                } else {
                    self.selected.as_deref() == Some(&self.items[index].id)
                }
            })
            .unwrap_or(0);
        let next = current.saturating_add_signed(delta).min(visible.len() - 1);
        self.select(visible[next]);
    }
    pub fn window(&self, height: u16) -> Vec<TaskRow> {
        if self.is_grouped() {
            return self.group_window(height);
        }
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
    fn group_window(&self, height: u16) -> Vec<TaskRow> {
        if height == 0 {
            return Vec::new();
        }
        let mut rows = Vec::new();
        let mut previous = None;
        let mut selected = 0;
        let mut header = 0;
        let mut selected_header = 0;
        for index in self.visible() {
            let name = group(&self.items[index].id);
            if previous != Some(name) {
                if previous.is_some() {
                    rows.push(TaskRow::Gap);
                }
                header = rows.len();
                rows.push(TaskRow::Group { index });
                previous = Some(name);
            }
            if self.selected.as_deref() == Some(&self.items[index].id) {
                selected = if self.is_group() { header } else { rows.len() };
                selected_header = header;
            }
            rows.push(TaskRow::Task { index, line: 0 });
            rows.push(TaskRow::Task { index, line: 1 });
        }
        let height = usize::from(height);
        let end = (selected + if self.is_group() { 1 } else { 2 }).min(rows.len());
        let start = if self.is_group() {
            selected.saturating_sub(height.saturating_sub(5))
        } else {
            end.saturating_sub(height)
        };
        if start > selected_header && height > 2 {
            let mut result = vec![rows[selected_header].clone()];
            result.extend(
                rows.iter()
                    .skip(end.saturating_sub(height - 1))
                    .take(height - 1)
                    .cloned(),
            );
            result
        } else {
            rows.into_iter().skip(start).take(height).collect()
        }
    }
}
