mod compare;
mod details;
mod groups;
mod input;
mod layout;
mod state;
mod tabs;
mod tasks;
mod text;
mod theme;
mod view;

#[cfg(test)]
mod tests;

use anyhow::Result;
use crossterm::{
    cursor::Show,
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

use ratatui::prelude::*;
pub use state::App;
use std::{
    io,
    time::{Duration, Instant},
};
use view::draw;

pub fn snapshot(app: &App, width: u16, height: u16) -> Result<String> {
    let mut terminal = Terminal::new(ratatui::backend::TestBackend::new(width, height))?;
    terminal.draw(|frame| draw(frame, app))?;
    Ok(format!("{}", terminal.backend()))
}
pub fn run(mut app: App) -> Result<()> {
    let stop = crate::process::Cancel::default();
    stop.install_signal_handler()?;
    struct Mouse;
    impl Drop for Mouse {
        fn drop(&mut self) {
            let _ = execute!(io::stdout(), DisableMouseCapture);
        }
    }
    ratatui::run(|terminal| -> Result<()> {
        execute!(io::stdout(), EnableMouseCapture)?;
        let _mouse = Mouse;
        let mut refresh = Instant::now();
        loop {
            if stop.is_cancelled() {
                return Ok(());
            }
            if refresh.elapsed() >= Duration::from_millis(300) {
                if let Err(e) = app.refresh() {
                    app.notice = format!("{e:#}");
                }
                refresh = Instant::now();
            }
            terminal.draw(|frame| draw(frame, &app))?;
            if !event::poll(Duration::from_millis(100))? {
                continue;
            }
            let result = match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    let size = terminal.size()?;
                    app.key(key, Rect::new(0, 0, size.width, size.height))
                }
                Event::Mouse(mouse) => {
                    let size = terminal.size()?;
                    app.mouse(mouse, Rect::new(0, 0, size.width, size.height))
                        .map(|_| false)
                }
                _ => Ok(false),
            };
            match result {
                Ok(true) => return Ok(()),
                Ok(false) => {}
                Err(e) => app.notice = format!("{e:#}"),
            }
            if let Some(settings) = app.take_login_request() {
                disable_raw_mode()?;
                execute!(
                    io::stdout(),
                    LeaveAlternateScreen,
                    DisableMouseCapture,
                    Show
                )?;
                println!(
                    "\nStarting {} login. Follow the prompts.\n",
                    settings.provider
                );
                let outcome = app.runtime.login(&settings);
                enable_raw_mode()?;
                execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
                terminal.clear()?;
                app.notice = match outcome {
                    Ok(()) => format!("{} login complete. Run the task again.", settings.provider),
                    Err(error) => format!("Login failed: {error:#}"),
                };
            }
        }
    })
}
