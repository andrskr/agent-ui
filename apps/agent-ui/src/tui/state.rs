use super::history::History;
use crate::{
    artifact::Artifact,
    preview::{self, ManagedPreview},
    report::Report,
    runner::{self, Active, Settings},
    storage::Store,
};
use anyhow::Result;
use std::{fs::File, io::Read};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DetailTab {
    Overview,
    Activity,
    Evidence,
}
impl DetailTab {
    pub(super) const ALL: [Self; 3] = [Self::Overview, Self::Activity, Self::Evidence];
    pub(super) fn step(self, delta: isize) -> Self {
        let index = match self {
            Self::Overview => 0,
            Self::Activity => 1,
            Self::Evidence => 2,
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
    pub(super) history: History,
    pub(super) searching: bool,
    pub(super) agent_note: String,
    note_id: Option<String>,
    pub(super) tasks: Vec<String>,
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
        let mut history = History::default();
        history.replace(runs);
        let mut app = Self {
            store,
            settings,
            history,
            searching: false,
            agent_note: String::new(),
            note_id: None,
            tasks,
            task: 0,
            tab: DetailTab::Overview,
            scroll: None,
            modal: Modal::None,
            field: FormField::Task,
            notice: String::new(),
            active: None,
            preview: None,
        };
        app.load_note();
        Ok(app)
    }
    pub(super) fn current(&self) -> Option<&Report> {
        self.history.current()
    }
    pub(super) fn refresh(&mut self) -> Result<()> {
        self.history.replace(self.store.list()?);
        self.load_note();
        if self.active.as_ref().is_some_and(Active::finished)
            && let Some(active) = self.active.take()
        {
            active.join()?;
            self.notice.clear();
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
            self.history.set_query(String::new());
            self.history.select_id(&id);
            self.load_note();
            self.tab = DetailTab::Activity;
            self.scroll = None;
            self.modal = Modal::None;
            self.notice.clear();
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
    pub(super) fn last_scroll_row(&self, visible_lines: u16, width: u16) -> u16 {
        let rows = self
            .current()
            .map(|run| {
                ratatui::widgets::Paragraph::new(super::details::content_lines(
                    run,
                    self.tab,
                    &self.agent_note,
                    width,
                ))
                .wrap(ratatui::widgets::Wrap { trim: false })
                .line_count(width)
            })
            .unwrap_or(0);
        rows.saturating_sub(visible_lines as usize)
            .min(u16::MAX as usize) as u16
    }
    pub(super) fn scroll_page(&mut self, delta: i32, visible_lines: u16, width: u16) {
        let last = self.last_scroll_row(visible_lines, width);
        let current = self.scroll.unwrap_or(if self.tab == DetailTab::Activity {
            last
        } else {
            0
        });
        self.scroll = Some((i32::from(current) + delta).clamp(0, i32::from(last)) as u16);
    }
    pub(super) fn load_note(&mut self) {
        let Some(run) = self.current() else {
            self.agent_note.clear();
            self.note_id = None;
            return;
        };
        if self.note_id.as_deref() == Some(&run.id) && !run.state.active() {
            return;
        }
        let id = run.id.clone();
        let active = run.state.active();
        let path = self.store.dir(&id).map(|dir| dir.join("agent-report.md"));
        let mut bytes = Vec::new();
        self.agent_note =
            match path.and_then(|path| Ok(File::open(path)?.take(4096).read_to_end(&mut bytes)?)) {
                Ok(_) => String::from_utf8_lossy(&bytes).into_owned(),
                Err(_) => String::new(),
            };
        self.note_id = (!active).then_some(id);
    }
    pub(super) fn navigate(&mut self, delta: isize) {
        self.history.navigate(delta);
        self.scroll = None;
        self.load_note();
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
                &crate::codex::which("vp")?,
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
