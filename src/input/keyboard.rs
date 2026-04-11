use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{App, AppState};

/// Handle a key event and update application state accordingly.
pub fn handle_key_event(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') => app.set_state(AppState::Quitting),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.set_state(AppState::Quitting);
        }
        _ => {}
    }
}
