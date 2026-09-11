use crate::{
    artifact::Artifact,
    preview::{self, ManagedPreview},
    report::Report,
    runner::{self, Active, Settings},
    storage::Store,
};
use anyhow::Result;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DetailTab {
    Overview,
    Activity,
    Report,
}
impl DetailTab {
    pub(super) const ALL: [Self; 3] = [Self::Overview, Self::Activity, Self::Report];
    pub(super) fn step(self, delta: isize) -> Self {
        let index = match self {
            Self::Overview => 0,
            Self::Activity => 1,
            Self::Report => 2,
        };
        Self::ALL[(index as isize + delta).rem_euclid(3) as usize]
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FormField {
    Task,
    Model,
    Effort,
}
impl FormField {
    pub(super) const ALL: [Self; 3] = [Self::Task, Self::Model, Self::Effort];
    pub(super) fn step(self, delta: isize) -> Self {
        let index = match self {
            Self::Task => 0,
            Self::Model => 1,
            Self::Effort => 2,
        };
        Self::ALL[(index as isize + delta).rem_euclid(3) as usize]
    }
}
#[derive(Clone, Copy)]
pub(super) enum ReviewAction {
    Preview,
    Open(Artifact),
}
impl ReviewAction {
    pub(super) const ALL: [Self; 4] = [
        Self::Preview,
        Self::Open(Artifact::Code),
        Self::Open(Artifact::Report),
        Self::Open(Artifact::Agent),
    ];
}
#[derive(PartialEq)]
pub(super) enum Modal {
    None,
    New,
    Delete,
    Help,
}
pub struct App {
    pub(super) store: Store,
    pub(super) settings: Settings,
    pub(super) runs: Vec<Report>,
    pub(super) tasks: Vec<String>,
    pub(super) selected: usize,
    pub(super) task: usize,
    pub(super) tab: DetailTab,
    pub(super) scroll: Option<u16>,
    pub(super) modal: Modal,
    pub(super) field: FormField,
    pub(super) notice: String,
    pub(super) active: Option<Active>,
    pub(super) preview: Option<ManagedPreview>,
}
impl App {
    pub fn new(store: Store, settings: Settings) -> Result<Self> {
        store.recover()?;
        let runs = store.list()?;
        let tasks = store.tasks()?;
        Ok(Self {
            store,
            settings,
            runs,
            tasks,
            selected: 0,
            task: 0,
            tab: DetailTab::Overview,
            scroll: None,
            modal: Modal::None,
            field: FormField::Task,
            notice: String::new(),
            active: None,
            preview: None,
        })
    }
    pub(super) fn current(&self) -> Option<&Report> {
        self.runs.get(self.selected)
    }
    pub(super) fn refresh(&mut self) -> Result<()> {
        let selected = self.current().map(|r| r.id.clone());
        self.runs = self.store.list()?;
        self.selected = selected
            .and_then(|id| self.runs.iter().position(|r| r.id == id))
            .unwrap_or(0);
        if self.active.as_ref().is_some_and(Active::finished)
            && let Some(active) = self.active.take()
        {
            let result = active.join()?;
            self.notice = format!(
                "{}: {}. Press e for code or b for a browser preview.",
                result.task,
                result.state.label()
            );
        }
        if let Some(preview) = &mut self.preview {
            match preview.poll() {
                Ok(true) => {
                    let url = preview.url().to_owned();
                    preview::open_url(&url)?;
                    self.notice = format!("Preview: {url}  Press x to stop it.");
                }
                Ok(false) => {}
                Err(error) => {
                    self.preview = None;
                    self.notice = error.to_string();
                }
            }
        }
        Ok(())
    }
    pub(super) fn start(&mut self) -> Result<()> {
        if let Some(task) = self.tasks.get(self.task) {
            let active = runner::start(self.store.clone(), task, self.settings.clone())?;
            let id = active.id.clone();
            self.active = Some(active);
            self.refresh()?;
            self.selected = self.runs.iter().position(|r| r.id == id).unwrap_or(0);
            self.tab = DetailTab::Activity;
            self.scroll = None;
            self.modal = Modal::None;
            self.notice = "Run started. Press c to cancel. Partial evidence is kept.".into();
        }
        Ok(())
    }
    pub(super) fn new_run(&mut self) -> Result<()> {
        if self.active.is_some() {
            self.notice = "Wait for the active run or press c to cancel it.".into();
        } else {
            self.tasks = self.store.tasks()?;
            self.task = self.task.min(self.tasks.len().saturating_sub(1));
            self.field = FormField::Task;
            self.modal = Modal::New;
        }
        Ok(())
    }
    pub(super) fn last_scroll_row(&self, visible_lines: u16) -> u16 {
        let rows = self
            .current()
            .map(|run| match self.tab {
                DetailTab::Activity => run.activity.len(),
                DetailTab::Report => serde_json::to_string_pretty(run)
                    .unwrap_or_default()
                    .lines()
                    .count(),
                _ => 0,
            })
            .unwrap_or(0);
        rows.saturating_sub(visible_lines as usize)
            .min(u16::MAX as usize) as u16
    }
    pub(super) fn scroll_page(&mut self, delta: i32, visible_lines: u16) {
        let last = self.last_scroll_row(visible_lines);
        let current = self.scroll.unwrap_or(if self.tab == DetailTab::Activity {
            last
        } else {
            0
        });
        self.scroll = Some((i32::from(current) + delta).clamp(0, i32::from(last)) as u16);
    }
    pub(super) fn review(&mut self, action: ReviewAction) -> Result<()> {
        let Some(id) = self.current().map(|r| r.id.clone()) else {
            return Ok(());
        };
        if let ReviewAction::Preview = action {
            if let Some(preview) = &self.preview
                && preview.id() == id
            {
                if preview.is_ready() {
                    preview::open_url(preview.url())?;
                }
                return Ok(());
            }
            self.preview = Some(ManagedPreview::start(
                &self.store,
                &id,
                &self.settings.tools.vp,
            )?);
            self.notice = "Starting the browser preview...".into();
        } else if let ReviewAction::Open(artifact) = action {
            let path = self.store.artifact(&id, artifact)?;
            preview::open_editor(&path)?;
            self.notice = match artifact {
                Artifact::Code => "Opened the app in your editor.",
                Artifact::Report => "Opened the JSON report in your editor.",
                Artifact::Agent => "Opened the agent report in your editor.",
                Artifact::Evidence => "Opened the evidence folder in your editor.",
            }
            .into();
        }
        Ok(())
    }
}
