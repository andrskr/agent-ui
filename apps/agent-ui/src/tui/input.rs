use super::{
    layout::*,
    state::{App, DetailTab, FormField, Modal, ReviewAction},
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
        let content = content_area(area);
        if self.modal == Modal::Picker {
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
        match self.modal {
            Modal::Run | Modal::Assess => match key.code {
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
                KeyCode::Tab | KeyCode::Right => {
                    self.tab = self.tab.step(1);
                    self.scroll = None;
                }
                KeyCode::BackTab | KeyCode::Left => {
                    self.tab = self.tab.step(-1);
                    self.scroll = None;
                }
                KeyCode::Char(c @ '1'..='3') => {
                    self.tab = DetailTab::ALL[c as usize - '1' as usize];
                    self.scroll = None;
                }
                KeyCode::PageDown => self.scroll_page(10, content.height, content.width),
                KeyCode::PageUp => self.scroll_page(-10, content.height, content.width),
                KeyCode::Home => self.scroll = Some(0),
                KeyCode::End => {
                    self.scroll = Some(self.last_scroll_row(content.height, content.width))
                }
                KeyCode::Char('/') => self.searching = true,
                KeyCode::Esc => {
                    self.tasks.set_query(String::new());
                    self.leave_comparison()?;
                }
                KeyCode::Char('n') => self.new_run()?,
                KeyCode::Char('c') => self.choose_comparison()?,
                KeyCode::Char('s') => self.swap()?,
                KeyCode::Char('v') if self.selection.comparing => {
                    self.side = self.side.toggle();
                    self.scroll = None;
                    self.load_context()?;
                }
                KeyCode::Char('m') => self.assess()?,
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
                KeyCode::Char('x') => self.stop_preview(),
                KeyCode::Char('b') => self.review(ReviewAction::Preview)?,
                KeyCode::Char('e') => self.review(ReviewAction::Open(Artifact::Code))?,
                KeyCode::Char('r') => self.review(ReviewAction::Open(Artifact::Report))?,
                KeyCode::Char('a') => self.review(ReviewAction::Open(Artifact::Agent))?,
                KeyCode::Char('f') => self.review(ReviewAction::Open(Artifact::Evidence))?,
                _ => {}
            },
            Modal::Picker => {}
        }
        Ok(false)
    }
    pub(super) fn mouse(&mut self, mouse: event::MouseEvent, area: Rect) -> Result<()> {
        let position = Position::new(mouse.column, mouse.row);
        let panels = regions(area).1;
        let content = content_area(area);
        match mouse.kind {
            MouseEventKind::ScrollDown | MouseEventKind::ScrollUp => {
                let delta = if mouse.kind == MouseEventKind::ScrollDown {
                    1
                } else {
                    -1
                };
                if self.modal == Modal::Picker {
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
        if matches!(self.modal, Modal::Run | Modal::Assess) {
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
        } else if self.modal == Modal::Picker {
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
            if new_button(area).contains(position) {
                self.new_run()?;
            } else if compare_button(area).contains(position) {
                self.choose_comparison()?;
            } else if search_area(panels[0]).contains(position) {
                self.searching = true;
            } else if task_list_area(panels[0]).contains(position) {
                let rect = task_list_area(panels[0]);
                if let Some(TaskRow::Task { index, .. }) = self
                    .tasks
                    .window(rect.height)
                    .get((position.y - rect.y) as usize)
                {
                    self.tasks.select(*index);
                    self.selected()?;
                }
            } else if parts[0].contains(position) && self.selection.comparing {
                self.side = if position.x < parts[0].x + parts[0].width / 2 {
                    super::state::Side::Reference
                } else {
                    super::state::Side::Other
                };
                self.scroll = None;
                self.load_context()?;
            } else if parts[1].contains(position) {
                self.tab = DetailTab::ALL[((position.x - parts[1].x) / 12).min(2) as usize];
                self.scroll = None;
            } else if parts[2].contains(position) && self.current().is_some() {
                let actions = action_areas(parts[2]);
                if actions[0].contains(position) {
                    self.review(ReviewAction::Preview)?;
                } else if actions[1].contains(position) {
                    self.review(ReviewAction::Open(Artifact::Code))?;
                } else {
                    self.stop_preview();
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
