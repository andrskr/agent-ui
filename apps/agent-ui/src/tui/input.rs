use super::{
    layout::*,
    state::{App, FormField, Modal, ReviewAction},
    tasks::TaskRow,
};
use crate::artifact::Artifact;
use anyhow::Result;
use crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers, MouseEventKind};
use ratatui::layout::{Position, Rect};
impl App {
    pub(super) fn key(&mut self, key: KeyEvent, area: Rect) -> Result<bool> {
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            return Ok(true);
        }
        let content = content_area(
            regions(area).1[1],
            self.tasks.is_group(),
            self.selection.comparing,
        );
        if matches!(
            self.modal,
            Modal::Picker | Modal::Reference | Modal::PairA | Modal::PairB
        ) {
            match key.code {
                KeyCode::Esc => self.modal = Modal::None,
                KeyCode::Enter => self.confirm_comparison()?,
                KeyCode::Up => self.picker.navigate(-1),
                KeyCode::Down => self.picker.navigate(1),
                KeyCode::Backspace => {
                    let mut query = self.picker.query.clone();
                    query.pop();
                    self.picker.set_query(query);
                }
                KeyCode::Char(c) => {
                    let mut query = self.picker.query.clone();
                    query.push(c);
                    self.picker.set_query(query);
                }
                _ => {}
            }
            return Ok(false);
        }
        if self.searching {
            let mut query = self.tasks.query.clone();
            match key.code {
                KeyCode::Esc => {
                    self.searching = false;
                    query.clear();
                }
                KeyCode::Enter => self.searching = false,
                KeyCode::Backspace => {
                    query.pop();
                }
                KeyCode::Char(c) => query.push(c),
                KeyCode::Up => self.tasks.navigate(-1),
                KeyCode::Down => self.tasks.navigate(1),
                _ => {}
            }
            self.tasks.set_query(query);
            self.selected()?;
            return Ok(false);
        }
        if self.modal == Modal::None
            && let Some(index) =
                super::tabs::key_index(self.tasks.is_group(), self.tab_index(), key.code)
        {
            self.switch_tab(index)?;
            return Ok(false);
        }
        if self.modal == Modal::None && self.selection.comparing {
            if let Some(action) = super::compare::key_action(key.code) {
                return self.comparison_action(action, content);
            }
            return Ok(false);
        }
        match self.modal {
            Modal::Run | Modal::RunGroup => match key.code {
                KeyCode::Esc => self.modal = Modal::None,
                KeyCode::Enter => self.start()?,
                key => self.field.select(key, &mut self.settings),
            },
            Modal::Help => {
                if matches!(key.code, KeyCode::Esc | KeyCode::Char('?')) {
                    self.modal = Modal::None;
                }
            }
            Modal::None => match key.code {
                KeyCode::Char('q') => return Ok(true),
                KeyCode::Down | KeyCode::Char('j') => self.navigate(1)?,
                KeyCode::Up | KeyCode::Char('k') => self.navigate(-1)?,
                KeyCode::Enter => self.enter_group()?,
                KeyCode::PageDown => self.scroll_page(10, content.height, content.width),
                KeyCode::PageUp => self.scroll_page(-10, content.height, content.width),
                KeyCode::Home => self.scroll = Some(0),
                KeyCode::End => {
                    self.scroll = Some(self.last_scroll_row(content.height, content.width))
                }
                KeyCode::Char('/') => self.searching = true,
                KeyCode::Esc => {
                    self.back()?;
                }
                KeyCode::Char('n') => self.new_run()?,
                KeyCode::Char('c') if self.tasks.is_group() => self.group_tab(true)?,
                KeyCode::Char('c') => self.choose_comparison()?,
                KeyCode::Char('s') => self.swap()?,
                KeyCode::Char('C') if self.tasks.is_group() => {
                    if let Some(group) = self.tasks.group().map(str::to_owned) {
                        self.runtime.cancel_group(&group);
                        self.notice =
                            format!("Stopped queued tasks and requested cancellation for {group}.");
                    }
                }
                KeyCode::Char('C') => match self.focused_task().map(str::to_owned) {
                    Some(task) if self.runtime.cancel_task(&task) => {
                        self.notice = format!("Stopping {task}…");
                    }
                    _ => {
                        self.runtime.cancel();
                        self.notice = "Stopping the active operation…".into();
                    }
                },
                KeyCode::Char('?') => self.modal = Modal::Help,
                KeyCode::Char('L') => self.request_login(),
                KeyCode::Char('x') => self.stop_preview(),
                KeyCode::Char('b') => self.review(ReviewAction::Preview)?,
                KeyCode::Char('e') => self.review(ReviewAction::Open(Artifact::Code))?,
                KeyCode::Char('r') => self.review(ReviewAction::Open(Artifact::Report))?,
                KeyCode::Char('a') => self.review(ReviewAction::Open(Artifact::Agent))?,
                KeyCode::Char('f') => self.review(ReviewAction::Open(Artifact::Evidence))?,
                _ => {}
            },
            Modal::Picker | Modal::Reference | Modal::PairA | Modal::PairB => {}
        }
        Ok(false)
    }
    fn comparison_action(&mut self, action: super::compare::Action, content: Rect) -> Result<bool> {
        use super::compare::Action;
        match action {
            Action::Quit => return Ok(true),
            Action::Swap => self.swap()?,
            Action::SelectA => self.edit_pair(true)?,
            Action::SelectB => self.edit_pair(false)?,
            Action::Back => self.leave_comparison()?,
            Action::EnterTask => self.enter_group()?,
            Action::Navigate(delta) => self.navigate(delta)?,
            Action::Scroll(delta) => self.scroll_page(delta, content.height, content.width),
            Action::Home => self.scroll = Some(0),
            Action::End => self.scroll = Some(self.last_scroll_row(content.height, content.width)),
            Action::Search => self.searching = true,
            Action::Help => self.modal = Modal::Help,
        }
        Ok(false)
    }
    pub(super) fn mouse(&mut self, mouse: event::MouseEvent, area: Rect) -> Result<()> {
        let position = Position::new(mouse.column, mouse.row);
        let panels = regions(area).1;
        let content = content_area(
            regions(area).1[1],
            self.tasks.is_group(),
            self.selection.comparing,
        );
        match mouse.kind {
            MouseEventKind::ScrollDown | MouseEventKind::ScrollUp => {
                let delta = if mouse.kind == MouseEventKind::ScrollDown {
                    1
                } else {
                    -1
                };
                if matches!(
                    self.modal,
                    Modal::Picker | Modal::Reference | Modal::PairA | Modal::PairB
                ) {
                    self.picker.navigate(delta);
                } else if self.modal == Modal::None {
                    if panels[0].contains(position) {
                        self.navigate(delta)?;
                    } else {
                        self.scroll_page(delta as i32 * 3, content.height, content.width);
                    }
                }
                return Ok(());
            }
            MouseEventKind::Down(event::MouseButton::Left) => {}
            _ => return Ok(()),
        }
        if matches!(self.modal, Modal::Run | Modal::RunGroup) {
            let rect = modal_rect(area);
            if modal_submit(rect).contains(position) {
                self.start()?;
            }
            for i in 0..3 {
                let field = modal_field(rect, i);
                if field.contains(position) {
                    self.field = match i {
                        0 => FormField::Provider,
                        1 => FormField::Model,
                        _ => FormField::Effort,
                    };
                    if position.x >= field.right() - 4 {
                        self.field.adjust(
                            &mut self.settings,
                            if position.x >= field.right() - 2 {
                                1
                            } else {
                                -1
                            },
                        );
                    }
                }
            }
        } else if matches!(
            self.modal,
            Modal::Picker | Modal::Reference | Modal::PairA | Modal::PairB
        ) {
            let rect = picker_list(modal_rect(area));
            if rect.contains(position)
                && let Some(TaskRow::Task { index, .. }) = self
                    .picker
                    .window(rect.height)
                    .get((position.y - rect.y) as usize)
            {
                self.picker.select(*index);
                self.confirm_comparison()?;
            }
        } else if self.modal == Modal::None {
            let parts = detail_parts(panels[1]);
            if search_area(panels[0]).contains(position) {
                self.searching = true;
            } else if task_list_area(panels[0]).contains(position) {
                let rect = task_list_area(panels[0]);
                if let Some(row) = self
                    .tasks
                    .window(rect.height)
                    .get((position.y - rect.y) as usize)
                    && self.tasks.select_row(row)
                {
                    self.selected()?;
                }
            } else if let Some(index) =
                super::tabs::mouse_index(parts[1], self.tasks.is_group(), position)
            {
                self.switch_tab(index)?;
            } else if self.tasks.is_group()
                && !self.selection.comparing
                && parts[2].contains(position)
            {
                if group_actions(parts[2])[0].contains(position) {
                    self.new_run()?;
                } else if group_actions(parts[2])[1].contains(position)
                    && let Some(group) = self.tasks.group().map(str::to_owned)
                {
                    self.runtime.cancel_group(&group);
                }
            } else if self.selection.comparing {
                if let Some(action) = super::compare::mouse_action(position, panels[1]) {
                    self.comparison_action(action, content)?;
                }
            } else if parts[2].contains(position) {
                let actions = action_areas(parts[2]);
                if actions[0].contains(position) {
                    if self
                        .focused_task()
                        .is_some_and(|id| self.runtime.is_running(id))
                    {
                        if let Some(task) = self.focused_task().map(str::to_owned) {
                            self.runtime.cancel_task(&task);
                        }
                    } else {
                        self.new_run()?;
                    }
                } else if self.current().is_some_and(|r| !r.state.active()) {
                    if actions[1].contains(position) {
                        self.review(ReviewAction::Preview)?;
                    } else if actions[2].contains(position) {
                        self.review(ReviewAction::Open(Artifact::Code))?;
                    } else if actions[3].contains(position) {
                        self.stop_preview();
                    }
                }
            }
        }
        Ok(())
    }
}

impl FormField {
    pub(super) fn select(&mut self, key: KeyCode, settings: &mut crate::settings::Settings) {
        match key {
            KeyCode::Up | KeyCode::BackTab => *self = self.step(-1),
            KeyCode::Down | KeyCode::Tab => *self = self.step(1),
            KeyCode::Left => self.adjust(settings, -1),
            KeyCode::Right => self.adjust(settings, 1),
            _ => {}
        }
    }
    fn adjust(self, settings: &mut crate::settings::Settings, delta: isize) {
        match self {
            Self::Provider => settings.step_provider(delta),
            Self::Model => settings.step_model(delta),
            Self::Effort => settings.step_effort(delta),
        }
    }
}
