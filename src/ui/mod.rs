use std::io::{self, Stdout, stdout};

use crossterm::{
    ExecutableCommand, cursor,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{Terminal, prelude::CrosstermBackend};

pub mod dialog;
pub mod help;
pub mod layout;
pub mod menu;
pub mod theme;
pub mod widgets;

pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// RAII guard that restores the terminal on drop.
pub struct TerminalGuard {
    terminal: Tui,
}

impl TerminalGuard {
    /// Initialize the terminal: enable raw mode, enter alternate screen, hide cursor.
    pub fn new() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        stdout().execute(cursor::Hide)?;

        let backend = CrosstermBackend::new(stdout());
        let terminal = Terminal::new(backend)?;

        Ok(Self { terminal })
    }

    pub fn terminal_mut(&mut self) -> &mut Tui {
        &mut self.terminal
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        let _ = stdout().execute(LeaveAlternateScreen);
        let _ = stdout().execute(cursor::Show);
    }
}

/// Install a panic hook that restores the terminal before printing the panic message.
pub fn install_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = terminal::disable_raw_mode();
        let _ = stdout().execute(LeaveAlternateScreen);
        let _ = stdout().execute(cursor::Show);
        original_hook(panic_info);
    }));
}
