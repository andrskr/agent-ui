use crate::settings::Settings;
use std::collections::VecDeque;

/// Pending group runs. Output is replaced only when a run starts.
#[derive(Default)]
pub(crate) struct RunQueue {
    pending: VecDeque<(String, Settings)>,
}

impl RunQueue {
    pub fn contains(&self, task: &str) -> bool {
        self.pending.iter().any(|(id, _)| id == task)
    }
    pub fn ids(&self) -> Vec<String> {
        self.pending.iter().map(|(id, _)| id.clone()).collect()
    }
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
    pub fn add(&mut self, tasks: Vec<String>, settings: Settings) {
        for task in tasks {
            if !self.contains(&task) {
                self.pending.push_back((task, settings.clone()));
            }
        }
    }
    pub fn next(&mut self, active: usize, limit: usize) -> Option<(String, Settings)> {
        if active < limit {
            self.pending.pop_front()
        } else {
            None
        }
    }
    pub fn cancel(&mut self, task: &str) -> bool {
        let before = self.pending.len();
        self.pending.retain(|(id, _)| id != task);
        before != self.pending.len()
    }
    pub fn clear(&mut self) {
        self.pending.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capacity_cancellation_and_settings_are_kept_until_dispatch() {
        let mut queue = RunQueue::default();
        let settings = Settings {
            timeout: 42,
            ..Settings::default()
        };
        queue.add(
            vec!["smoke--a".into(), "smoke--b".into(), "smoke--c".into()],
            settings,
        );
        queue.add(vec!["smoke--a".into()], Settings::default());
        assert!(queue.next(4, 4).is_none());
        assert_eq!(queue.ids(), ["smoke--a", "smoke--b", "smoke--c"]);
        assert!(queue.cancel("smoke--b"));
        let (task, settings) = queue.next(3, 4).unwrap();
        assert_eq!(task, "smoke--a");
        assert_eq!(settings.timeout, 42);
        assert_eq!(queue.next(0, 4).unwrap().0, "smoke--c");
        assert!(queue.is_empty());
        queue.add(vec!["smoke--d".into()], Settings::default());
        queue.clear();
        assert!(queue.next(0, 4).is_none());
    }
}
