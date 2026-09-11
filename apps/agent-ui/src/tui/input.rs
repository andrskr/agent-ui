use super::{
    layout::*,
    state::{App, DetailTab, FormField, Modal, ReviewAction},
};
use crate::artifact::Artifact;
use anyhow::Result;
use crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers, MouseEventKind};
use ratatui::layout::{Position, Rect};

impl App {
    pub(super) fn key(&mut self, key: KeyEvent, visible_lines: u16) -> Result<bool> {
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            return Ok(true);
        }
        match self.modal {
            Modal::New => match key.code {
                KeyCode::Esc => self.modal = Modal::None,
                KeyCode::Tab => self.field = self.field.step(1),
                KeyCode::BackTab => self.field = self.field.step(-1),
                KeyCode::Enter => self.start()?,
                KeyCode::Up | KeyCode::Left => self.adjust(-1),
                KeyCode::Down | KeyCode::Right => self.adjust(1),
                KeyCode::Backspace if self.field == FormField::Model => {
                    self.settings.model.pop();
                }
                KeyCode::Char(c)
                    if self.field == FormField::Model
                        && (c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.')) =>
                {
                    self.settings.model.push(c)
                }
                _ => {}
            },
            Modal::Delete => match key.code {
                KeyCode::Esc => self.modal = Modal::None,
                KeyCode::Char('y') => {
                    if let Some(id) = self.current().map(|r| r.id.clone()) {
                        if self.preview.as_ref().is_some_and(|p| p.id() == id) {
                            self.preview = None;
                        }
                        self.store.remove(&id)?;
                        self.refresh()?;
                        self.notice = format!("Removed run {id} and its files.");
                    }
                    self.modal = Modal::None;
                }
                _ => {}
            },
            Modal::Help => {
                if matches!(key.code, KeyCode::Esc | KeyCode::Char('?')) {
                    self.modal = Modal::None;
                }
            }
            Modal::None => match key.code {
                KeyCode::Char('q') => return Ok(true),
                KeyCode::Down | KeyCode::Char('j') => self.navigate(1),
                KeyCode::Up | KeyCode::Char('k') => self.navigate(-1),
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
                KeyCode::PageDown => self.scroll_page(10, visible_lines),
                KeyCode::PageUp => self.scroll_page(-10, visible_lines),
                KeyCode::Home => self.scroll = Some(0),
                KeyCode::End => {
                    self.scroll = if self.tab == DetailTab::Activity {
                        None
                    } else {
                        Some(self.last_scroll_row(visible_lines))
                    };
                }
                KeyCode::Char('n') => self.new_run()?,
                KeyCode::Char('c') => {
                    if let Some(active) = &self.active {
                        active.cancel();
                        self.notice = "Stopping the active run...".into();
                    }
                }
                KeyCode::Char('d') => {
                    if self.current().is_some_and(|r| !r.state.active()) {
                        self.modal = Modal::Delete;
                    }
                }
                KeyCode::Char('?') => self.modal = Modal::Help,
                KeyCode::Char('x') => {
                    self.preview = None;

                    self.notice = "Preview stopped.".into();
                }
                KeyCode::Char('b') => self.review(ReviewAction::Preview)?,
                KeyCode::Char('e') => self.review(ReviewAction::Open(Artifact::Code))?,
                KeyCode::Char('r') => self.review(ReviewAction::Open(Artifact::Report))?,
                KeyCode::Char('a') => self.review(ReviewAction::Open(Artifact::Agent))?,
                _ => {}
            },
        }
        Ok(false)
    }
    fn adjust(&mut self, delta: isize) {
        if self.field == FormField::Task && !self.tasks.is_empty() {
            self.task = (self.task as isize + delta).rem_euclid(self.tasks.len() as isize) as usize;
        }
        if self.field == FormField::Effort {
            self.settings.effort = self.settings.effort.step(delta);
        }
    }
    fn navigate(&mut self, delta: isize) {
        if !self.runs.is_empty() {
            self.selected =
                (self.selected as isize + delta).rem_euclid(self.runs.len() as isize) as usize;
            self.scroll = None;
        }
    }
    pub(super) fn mouse(&mut self, mouse: event::MouseEvent, area: Rect) -> Result<()> {
        if area.width < 76 || area.height < 24 {
            return Ok(());
        }
        let position = Position::new(mouse.column, mouse.row);
        let (_, panels) = regions(area);
        if self.modal == Modal::None
            && matches!(
                mouse.kind,
                MouseEventKind::ScrollUp | MouseEventKind::ScrollDown
            )
        {
            let delta: i32 = if mouse.kind == MouseEventKind::ScrollUp {
                -3
            } else {
                3
            };
            if panels[0].contains(position) {
                self.navigate(delta.signum() as isize);
            } else if panels[1].contains(position) {
                self.scroll_page(delta, visible_lines(area));
            }
            return Ok(());
        }
        if mouse.kind != MouseEventKind::Down(event::MouseButton::Left) {
            return Ok(());
        }
        if self.modal == Modal::New {
            let area = modal_rect(area);
            if modal_submit(area).contains(position) {
                self.start()?;
            } else {
                for i in 0..3 {
                    let rect = modal_field(area, i);
                    if rect.contains(position) {
                        self.field = FormField::ALL[i];
                        if i != 1 && position.x >= rect.right() - 4 {
                            self.adjust(if position.x >= rect.right() - 2 {
                                1
                            } else {
                                -1
                            });
                        }
                    }
                }
            }
        } else if self.modal == Modal::None {
            if new_button(area).contains(position) {
                self.new_run()?;
            } else if run_list_area(panels[0]).contains(position) {
                let inner = run_list_area(panels[0]);
                let visible = (inner.height as usize / 4).saturating_sub(1);
                let start = self.selected.saturating_sub(visible);
                let index = start + (position.y - inner.y) as usize / 4;
                if index < self.runs.len() {
                    self.selected = index;
                    self.scroll = None;
                }
            } else if self.current().is_some() {
                let parts = detail_parts(panels[1]);
                if position.y == parts[1].y
                    && position.x >= parts[1].x
                    && position.x < parts[1].x + 45
                {
                    self.tab = DetailTab::ALL[((position.x - parts[1].x) / 15) as usize];
                    self.scroll = None;
                } else if self.tab == DetailTab::Overview {
                    for (rect, action) in action_areas(parts[2]).iter().zip(ReviewAction::ALL) {
                        if rect.contains(position) {
                            self.review(action)?;
                            break;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
