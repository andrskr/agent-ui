use super::tasks::Tasks;
use crate::{
    application::Application,
    artifact::Artifact,
    comparison::Comparison,
    report::Report,
    settings::Settings,
    task_result::{Selection, TaskDetails, TaskPair},
};
use anyhow::{Context, Result, ensure};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DetailTab {
    Overview,
    Activity,
    Evidence,
}
impl DetailTab {
    pub const ALL: [Self; 3] = [Self::Overview, Self::Activity, Self::Evidence];
    pub fn step(self, delta: isize) -> Self {
        let i = Self::ALL.iter().position(|v| *v == self).unwrap();
        Self::ALL[(i as isize + delta).rem_euclid(3) as usize]
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FormField {
    Provider,
    Model,
    Effort,
}
impl FormField {
    pub fn step(self, delta: isize) -> Self {
        let fields = [Self::Provider, Self::Model, Self::Effort];
        fields[(self as isize + delta).rem_euclid(fields.len() as isize) as usize]
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Side {
    #[default]
    Reference,
    Other,
}
impl Side {
    pub fn toggle(self) -> Self {
        if self == Self::Reference {
            Self::Other
        } else {
            Self::Reference
        }
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
    Run,
    Assess,
    Picker,
    Help,
}
pub struct App {
    pub(super) runtime: Application,
    pub(super) settings: Settings,
    pub(super) tasks: Tasks,
    pub(super) picker: Tasks,
    pub(super) selection: Selection,
    pub(super) side: Side,
    pub(super) comparison: Option<Comparison>,
    pub(super) assessment: String,
    pub(super) details: Option<TaskDetails>,
    pub(super) agent_note: String,
    context_id: Option<String>,
    pub(super) searching: bool,
    pub(super) tab: DetailTab,
    pub(super) scroll: Option<u16>,
    pub(super) modal: Modal,
    pub(super) field: FormField,
    pub(super) notice: String,
}
impl App {
    pub fn new(runtime: Application, settings: Settings) -> Result<Self> {
        let mut tasks = Tasks::default();
        tasks.replace(runtime.task_views()?);
        let selection = runtime.selection()?;
        if let Some(task) = &selection.task {
            tasks.select_id(task);
        }
        let mut app = Self {
            runtime,
            settings,
            tasks,
            picker: Tasks::default(),
            selection,
            side: Side::Reference,
            comparison: None,
            assessment: String::new(),
            details: None,
            agent_note: String::new(),
            context_id: None,
            searching: false,
            tab: DetailTab::Overview,
            scroll: None,
            modal: Modal::None,
            field: FormField::Provider,
            notice: String::new(),
        };
        app.load_context()?;
        Ok(app)
    }
    pub(super) fn focused_task(&self) -> Option<&str> {
        if self.selection.comparing {
            self.selection.pair.as_ref().map(|p| match self.side {
                Side::Reference => p.reference.as_str(),
                Side::Other => p.other.as_str(),
            })
        } else {
            self.tasks.current().map(|t| t.id.as_str())
        }
    }
    pub(super) fn current(&self) -> Option<&Report> {
        let id = self.focused_task()?;
        self.tasks.items.iter().find(|t| t.id == id)?.run.as_ref()
    }
    fn persist(&mut self) -> Result<()> {
        self.selection.task = self.tasks.current().map(|t| t.id.clone());
        self.runtime.save_selection(&self.selection)
    }
    pub(super) fn load_context(&mut self) -> Result<()> {
        self.comparison = None;
        self.assessment.clear();
        if self.selection.comparing
            && let Some(pair) = &self.selection.pair
        {
            match self.runtime.compare(&pair.reference, &pair.other) {
                Ok(comparison) => {
                    self.assessment = self.runtime.assessment_text(&comparison)?;
                    self.comparison = Some(comparison);
                }
                Err(error) => self.notice = format!("Comparison unavailable: {error:#}"),
            }
        }
        if let Some(task) = self.focused_task().map(str::to_owned) {
            if self.context_id.as_ref() != Some(&task) {
                self.details = match self.runtime.task_details(&task) {
                    Ok(d) => Some(d),
                    Err(e) => {
                        self.notice = format!("Task input error: {e:#}");
                        None
                    }
                };
                self.context_id = Some(task.clone());
            }
            self.agent_note = self.runtime.agent_note(&task).unwrap_or_default();
        } else {
            self.details = None;
            self.agent_note.clear();
            self.context_id = None;
        }
        Ok(())
    }
    pub(super) fn refresh(&mut self) -> Result<()> {
        if self.runtime.poll_run()?.is_some() {
            self.context_id = None;
            self.notice = "Run finished. Open its code or preview to review it.".into();
        }
        if self.runtime.poll_assessment()?.is_some() {
            self.notice = "Agent assessment finished. Open Overview to read it.".into();
        }
        self.tasks.replace(self.runtime.task_views()?);
        self.picker.replace(self.tasks.items.clone());
        self.load_context()?;
        for (task, result) in self.runtime.poll_previews() {
            match result {
                Ok(true) => {
                    self.runtime.open_preview(&task)?;
                    self.notice = format!("Opened preview for {task}.");
                }
                Ok(false) => {}
                Err(error) => self.notice = format!("Preview {task}: {error:#}"),
            }
        }
        Ok(())
    }
    pub(super) fn navigate(&mut self, delta: isize) -> Result<()> {
        self.tasks.navigate(delta);
        self.selection.comparing = false;
        self.side = Side::Reference;
        self.scroll = None;
        self.persist()?;
        self.load_context()
    }
    pub(super) fn selected(&mut self) -> Result<()> {
        self.selection.comparing = false;
        self.side = Side::Reference;
        self.scroll = None;
        self.persist()?;
        self.load_context()
    }
    pub(super) fn new_run(&mut self) -> Result<()> {
        ensure!(
            !self.runtime.is_busy(),
            "Wait for the active operation or press C to cancel it"
        );
        self.focused_task().context("Select a task")?;
        self.context_id = None;
        self.load_context()?;
        self.modal = Modal::Run;
        self.field = FormField::Provider;
        Ok(())
    }
    pub(super) fn start(&mut self) -> Result<()> {
        if self.modal == Modal::Assess {
            self.start_assessment()?;
            self.modal = Modal::None;
            return Ok(());
        }
        let task = self.focused_task().context("Select a task")?.to_owned();
        self.runtime.start(&task, self.settings.clone())?;
        self.modal = Modal::None;
        self.tab = DetailTab::Activity;
        self.scroll = None;
        self.notice.clear();
        self.refresh()
    }
    pub(super) fn choose_comparison(&mut self) -> Result<()> {
        let task = self.tasks.current().context("Select the reference task")?;
        ensure!(
            task.run.is_some(),
            "Run the reference task before comparing it"
        );
        self.picker = Tasks::default();
        self.picker.replace(self.tasks.items.clone());
        if let Some(pair) = &self.selection.pair {
            self.picker.select_id(&pair.other);
        }
        self.notice.clear();
        self.modal = Modal::Picker;
        Ok(())
    }
    pub(super) fn confirm_comparison(&mut self) -> Result<()> {
        let reference = self
            .tasks
            .current()
            .context("Select the reference task")?
            .id
            .clone();
        let other = self.picker.current().context("Select another task")?;
        ensure!(other.run.is_some(), "This task has no run yet");
        let pair = TaskPair::new(reference, other.id.clone())?;
        self.runtime.compare(&pair.reference, &pair.other)?;
        self.notice.clear();
        self.selection.pair = Some(pair);
        self.selection.comparing = true;
        self.side = Side::Reference;
        self.tab = DetailTab::Overview;
        self.scroll = None;
        self.modal = Modal::None;
        self.persist()?;
        self.load_context()
    }
    pub(super) fn swap(&mut self) -> Result<()> {
        if self.selection.comparing
            && let Some(pair) = &mut self.selection.pair
        {
            pair.swap();
            self.tasks.select_id(&pair.reference);
            self.side = Side::Reference;
            self.scroll = None;
            self.persist()?;
            self.load_context()?;
        }
        Ok(())
    }
    pub(super) fn leave_comparison(&mut self) -> Result<()> {
        self.selection.comparing = false;
        self.scroll = None;
        self.persist()?;
        self.load_context()
    }
    pub(super) fn assess(&mut self) -> Result<()> {
        ensure!(self.selection.comparing, "Select two tasks first");
        ensure!(
            !self.runtime.is_busy(),
            "Wait for the active operation or press C to cancel it"
        );
        self.modal = Modal::Assess;
        self.field = FormField::Provider;
        Ok(())
    }
    fn start_assessment(&mut self) -> Result<()> {
        ensure!(self.selection.comparing, "Select two tasks first");
        let pair = self
            .selection
            .pair
            .as_ref()
            .context("Select two tasks first")?;
        self.runtime
            .start_assessment(&pair.reference, &pair.other, self.settings.clone())?;
        self.tab = DetailTab::Overview;
        self.notice = "The agent is assessing saved evidence. Usage is recorded separately.".into();
        Ok(())
    }
    pub(super) fn lines(&self, width: u16) -> Vec<ratatui::text::Line<'static>> {
        super::details::screen_lines(
            &super::details::Content {
                task: self.tasks.current(),
                run: self.current(),
                comparison: self.comparison.as_ref(),
                comparing: self.selection.comparing,
                tab: self.tab,
                details: self.details.as_ref(),
                note: &self.agent_note,
                assessment: &self.assessment,
            },
            width,
        )
    }
    pub(super) fn last_scroll_row(&self, height: u16, width: u16) -> u16 {
        ratatui::widgets::Paragraph::new(self.lines(width))
            .wrap(ratatui::widgets::Wrap { trim: false })
            .line_count(width)
            .saturating_sub(height as usize)
            .min(u16::MAX as usize) as u16
    }
    pub(super) fn scroll_page(&mut self, delta: i32, height: u16, width: u16) {
        let last = self.last_scroll_row(height, width);
        let current = self.scroll.unwrap_or(if self.tab == DetailTab::Activity {
            last
        } else {
            0
        });
        self.scroll = Some((i32::from(current) + delta).clamp(0, i32::from(last)) as u16);
    }
    pub(super) fn review(&mut self, action: ReviewAction) -> Result<()> {
        let task = self.focused_task().context("Select a task")?.to_owned();
        match action {
            ReviewAction::Preview => {
                if let Some(preview) = self.runtime.preview(&task) {
                    if preview.ready {
                        self.runtime.open_preview(&task)?;
                    }
                } else {
                    self.runtime.start_preview(&task)?;
                    self.notice = format!("Starting preview for {task}...");
                }
            }
            ReviewAction::Open(target) => {
                self.runtime.open_artifact(&task, target)?;
                self.notice = format!("Opened {task} in VS Code.");
            }
        }
        Ok(())
    }
    pub(super) fn stop_preview(&mut self) {
        if let Some(task) = self.focused_task().map(str::to_owned) {
            self.runtime.stop_preview(&task);
            self.notice = format!("Stopped preview for {task}.");
        }
    }
}
