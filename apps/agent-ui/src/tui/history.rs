use crate::report::Report;
use chrono::{DateTime, Local};

#[derive(Default)]
pub(super) struct History {
    pub runs: Vec<Report>,
    pub query: String,
    selected: Option<String>,
}

pub(super) enum HistoryRow {
    Date(String),
    Run { index: usize, line: u16 },
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

impl History {
    pub fn replace(&mut self, mut runs: Vec<Report>) {
        runs.sort_by(|a, b| {
            b.created_at_ms
                .cmp(&a.created_at_ms)
                .then_with(|| b.id.cmp(&a.id))
        });
        self.runs = runs;
        self.reconcile();
    }
    pub fn visible(&self) -> Vec<usize> {
        let words: Vec<_> = self
            .query
            .split_whitespace()
            .map(str::to_lowercase)
            .collect();
        self.runs
            .iter()
            .enumerate()
            .filter_map(|(index, run)| {
                let text = format!(
                    "{} {} {} {} {}",
                    run.task,
                    run.id,
                    run.state.label(),
                    run.model_requested,
                    run.effort_requested
                )
                .to_lowercase();
                words
                    .iter()
                    .all(|word| text.contains(word))
                    .then_some(index)
            })
            .collect()
    }
    pub fn current(&self) -> Option<&Report> {
        self.selected
            .as_ref()
            .and_then(|id| self.runs.iter().find(|run| &run.id == id))
    }
    pub fn select(&mut self, index: usize) {
        if let Some(run) = self.runs.get(index) {
            self.selected = Some(run.id.clone());
        }
    }
    pub fn select_id(&mut self, id: &str) {
        if let Some(index) = self.runs.iter().position(|run| run.id == id) {
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
            .any(|&index| self.selected.as_deref() == Some(&self.runs[index].id))
        {
            self.selected = visible.first().map(|&index| self.runs[index].id.clone());
        }
    }
    pub fn navigate(&mut self, delta: isize) {
        let visible = self.visible();
        if visible.is_empty() {
            return;
        }
        let current = visible
            .iter()
            .position(|&index| self.selected.as_deref() == Some(&self.runs[index].id))
            .unwrap_or(0);
        let next = current.saturating_add_signed(delta).min(visible.len() - 1);
        self.select(visible[next]);
    }
    pub fn window(&self, height: u16) -> Vec<HistoryRow> {
        let visible = self.visible();
        let selected = visible
            .iter()
            .position(|&index| self.selected.as_deref() == Some(&self.runs[index].id))
            .unwrap_or(0);
        let dates: Vec<_> = visible
            .iter()
            .map(|&index| local_stamp(self.runs[index].created_at_ms).0)
            .collect();
        let mut start = selected;
        let mut used = 4usize;
        while start > 0 {
            let extra = 4 + usize::from(dates[start - 1] != dates[start]);
            if used + extra > usize::from(height) {
                break;
            }
            used += extra;
            start -= 1;
        }
        let mut rows = Vec::new();
        let mut previous = None;
        for (position, &index) in visible.iter().enumerate().skip(start) {
            if previous != Some(&dates[position]) {
                rows.push(HistoryRow::Date(dates[position].clone()));
                previous = Some(&dates[position]);
            }
            for line in 0..3 {
                rows.push(HistoryRow::Run { index, line });
            }
            rows.push(HistoryRow::Gap);
            if rows.len() >= usize::from(height) {
                break;
            }
        }
        rows.truncate(usize::from(height));
        rows
    }
}
