pub mod keyboard;

use crossterm::event::{KeyEvent, MouseEvent};

/// Application-level events.
#[derive(Debug)]
pub enum AppEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Tick,
    RenderTick,
}
