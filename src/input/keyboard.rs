use crossterm::event::KeyEvent;

use crate::app::{App, AppState};
use crate::config::keybindings::Action;

/// Handle a key event by resolving it through the keybinding map.
pub fn handle_key_event(app: &mut App, key: KeyEvent) {
    let action = app.keybindings().resolve(key.code, key.modifiers);

    match action {
        Some(Action::Quit) => app.set_state(AppState::Quitting),
        // More actions will be handled as features are added
        Some(_) | None => {}
    }
}
