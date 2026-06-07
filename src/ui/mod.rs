// Terminal UI rendering
// Inspired by btop's modern TUI

pub mod theme;
pub mod widgets;
pub mod layout;
pub mod state;
pub mod detail_popup;

use crate::core::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;
use std::time::Duration;

pub use theme::Theme;
pub use widgets::{CpuWidget, MemoryWidget, ProcessWidget, NetworkWidget, DiskIoWidget, DiskWidget, GpuWidget, GpuProcessesWidget};
pub use layout::{MultiPanelLayout, PanelRect};
pub use state::{AppState, ViewMode, SortField, SortOrder, DetailSortField, DetailPopupType, DetailPopupState};
pub use detail_popup::render_detail_popup;

pub struct Ui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    theme: Theme,
}

impl Ui {
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self {
            terminal,
            theme: Theme::default(),
        })
    }

    pub fn render<F>(&mut self, render_fn: F) -> Result<()>
    where
        F: FnOnce(&mut ratatui::Frame),
    {
        self.terminal.draw(render_fn)?;
        Ok(())
    }

    pub fn size(&self) -> Result<(u16, u16)> {
        let size = self.terminal.size()?;
        Ok((size.width, size.height))
    }

    pub fn theme(&self) -> &Theme {
        &self.theme
    }

    /// Check for keyboard and mouse events with a timeout
    /// Returns Some(Event) if an event occurred, None if timeout
    pub fn poll_event(&self, timeout: Duration) -> Result<Option<Event>> {
        if event::poll(timeout)? {
            return Ok(Some(event::read()?));
        }
        Ok(None)
    }
}

impl Drop for Ui {
    fn drop(&mut self) {
        // Restore terminal state
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = self.terminal.show_cursor();
    }
}
