pub mod keyboard;

use crossterm::event::{KeyEvent, MouseEvent};

/// Application-level events.
#[derive(Debug)]
#[allow(dead_code)]
pub enum AppEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Tick,
    RenderTick,
}
