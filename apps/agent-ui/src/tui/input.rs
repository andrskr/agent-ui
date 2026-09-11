use super::history::HistoryRow;
use super::{
    layout::*,
    state::{App, DetailTab, FormField, Modal, ReviewAction},
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
        if self.searching {
            match key.code {
                KeyCode::Esc => {
                    self.searching = false;
                    self.history.set_query(String::new());
                }
                KeyCode::Enter => self.searching = false,
                KeyCode::Backspace => {
                    let mut query = self.history.query.clone();
                    query.pop();
                    self.history.set_query(query);
                }
                KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    let mut query = self.history.query.clone();
                    query.push(c);
                    self.history.set_query(query);
                }
                KeyCode::Down => self.history.navigate(1),
                KeyCode::Up => self.history.navigate(-1),
                _ => {}
            }
            self.scroll = None;
            self.load_note();
            return Ok(false);
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
                        self.runtime.remove(&id)?;
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
                KeyCode::PageDown => self.scroll_page(10, content.height, content.width),
                KeyCode::PageUp => self.scroll_page(-10, content.height, content.width),
                KeyCode::Home => self.scroll = Some(0),
                KeyCode::End => {
                    self.scroll = if self.tab == DetailTab::Activity {
                        None
                    } else {
                        Some(self.last_scroll_row(content.height, content.width))
                    };
                }
                KeyCode::Char('/') => self.searching = true,
                KeyCode::Esc => {
                    self.history.set_query(String::new());
                    self.load_note();
                }
                KeyCode::Char('n') => self.new_run()?,
                KeyCode::Char('c') => {
                    if self.runtime.has_active_run() {
                        self.runtime.cancel();
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
                    self.runtime.stop_preview();

                    self.notice = "Preview stopped.".into();
                }
                KeyCode::Char('b') => self.review(ReviewAction::Preview)?,
                KeyCode::Char('e') => self.review(ReviewAction::Open(Artifact::Code))?,
                KeyCode::Char('r') => self.review(ReviewAction::Open(Artifact::Report))?,
                KeyCode::Char('f') => self.review(ReviewAction::Open(Artifact::Evidence))?,
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
                let content = content_area(area);
                self.scroll_page(delta, content.height, content.width);
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
                self.searching = false;
                self.new_run()?;
            } else if search_area(panels[0]).contains(position) {
                self.searching = true;
            } else if run_list_area(panels[0]).contains(position) {
                let inner = run_list_area(panels[0]);
                if let Some(HistoryRow::Run { index, .. }) = self
                    .history
                    .window(inner.height)
                    .get((position.y - inner.y) as usize)
                {
                    self.history.select(*index);
                    self.scroll = None;
                    self.load_note();
                }
            } else if self.current().is_some() {
                let parts = detail_parts(panels[1]);
                if parts[0].contains(position) {
                    self.tab = DetailTab::ALL[((position.x - parts[0].x) / 12).min(2) as usize];
                    self.scroll = None;
                } else if parts[1].contains(position) {
                    let actions = action_areas(parts[1]);
                    if actions[0].contains(position) {
                        self.review(ReviewAction::Preview)?;
                    } else if actions[1].contains(position) {
                        self.review(ReviewAction::Open(Artifact::Code))?;
                    } else if actions[2].contains(position)
                        && self
                            .runtime
                            .preview()
                            .is_some_and(|p| self.current().is_some_and(|r| r.id == p.id))
                    {
                        self.runtime.stop_preview();
                        self.notice = "Preview stopped.".into();
                    }
                } else if self.tab == DetailTab::Overview && parts[2].contains(position) {
                    let run = self.current().expect("Selected run exists");
                    let lines =
                        super::details::overview_lines(run, &self.agent_note, parts[2].width);
                    let count = ratatui::widgets::Paragraph::new(lines)
                        .wrap(ratatui::widgets::Wrap { trim: false })
                        .line_count(parts[2].width);
                    let last = count
                        .saturating_sub(parts[2].height as usize)
                        .min(u16::MAX as usize) as u16;
                    let row =
                        usize::from(position.y - parts[2].y + self.scroll.unwrap_or(0).min(last));
                    if !self.agent_note.trim().is_empty()
                        && row + 1 == count
                        && position.x < parts[2].x + 16
                    {
                        self.review(ReviewAction::Open(Artifact::Agent))?;
                    }
                } else if self.tab == DetailTab::Evidence
                    && parts[2].contains(position)
                    && position.y == parts[2].y
                    && self.scroll.unwrap_or(0) == 0
                {
                    let x = position.x.saturating_sub(parts[2].x);
                    if x < 13 {
                        self.review(ReviewAction::Open(Artifact::Report))?;
                    } else if x < 28 {
                        self.review(ReviewAction::Open(Artifact::Agent))?;
                    } else {
                        self.review(ReviewAction::Open(Artifact::Evidence))?;
                    }
                }
            }
        }
        Ok(())
    }
}
