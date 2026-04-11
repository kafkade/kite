use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyModifiers};

/// Actions that can be triggered by keybindings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    Quit,
    Help,
    Menu,
    Refresh,
    TogglePause,
    FocusNext,
    FocusPrev,
    ScrollUp,
    ScrollDown,
    SortNext,
    SortPrev,
    Search,
    Escape,
}

/// A key combination (key code + optional modifiers).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyBind {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyBind {
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }

    pub fn plain(code: KeyCode) -> Self {
        Self {
            code,
            modifiers: KeyModifiers::NONE,
        }
    }

    pub fn ctrl(code: KeyCode) -> Self {
        Self {
            code,
            modifiers: KeyModifiers::CONTROL,
        }
    }
}

/// Keybinding resolver: maps key combinations to actions.
pub struct KeyBindings {
    map: HashMap<KeyBind, Action>,
}

impl KeyBindings {
    /// Build keybindings from config overrides merged onto defaults.
    pub fn new(overrides: &HashMap<String, String>) -> Self {
        let mut map = Self::defaults();

        for (action_str, key_str) in overrides {
            if let (Some(action), Some(keybind)) = (
                parse_action(action_str),
                parse_keybind(key_str),
            ) {
                // Remove any existing binding for this action
                map.retain(|_, v| *v != action);
                map.insert(keybind, action);
            }
        }

        Self { map }
    }

    /// Resolve a key event to an action.
    pub fn resolve(&self, code: KeyCode, modifiers: KeyModifiers) -> Option<Action> {
        let key = KeyBind::new(code, modifiers);
        self.map.get(&key).copied()
    }

    fn defaults() -> HashMap<KeyBind, Action> {
        let mut map = HashMap::new();
        map.insert(KeyBind::plain(KeyCode::Char('q')), Action::Quit);
        map.insert(KeyBind::ctrl(KeyCode::Char('c')), Action::Quit);
        map.insert(KeyBind::plain(KeyCode::Char('?')), Action::Help);
        map.insert(KeyBind::plain(KeyCode::Char('m')), Action::Menu);
        map.insert(KeyBind::plain(KeyCode::Char('r')), Action::Refresh);
        map.insert(KeyBind::plain(KeyCode::Char(' ')), Action::TogglePause);
        map.insert(KeyBind::plain(KeyCode::Tab), Action::FocusNext);
        map.insert(KeyBind::plain(KeyCode::BackTab), Action::FocusPrev);
        map.insert(KeyBind::plain(KeyCode::Up), Action::ScrollUp);
        map.insert(KeyBind::plain(KeyCode::Down), Action::ScrollDown);
        map.insert(KeyBind::plain(KeyCode::Char('k')), Action::ScrollUp);
        map.insert(KeyBind::plain(KeyCode::Char('j')), Action::ScrollDown);
        map.insert(KeyBind::plain(KeyCode::Left), Action::SortPrev);
        map.insert(KeyBind::plain(KeyCode::Right), Action::SortNext);
        map.insert(KeyBind::plain(KeyCode::Char('/')), Action::Search);
        map.insert(KeyBind::plain(KeyCode::Esc), Action::Escape);
        map
    }
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            map: Self::defaults(),
        }
    }
}

fn parse_action(s: &str) -> Option<Action> {
    match s.to_lowercase().as_str() {
        "quit" => Some(Action::Quit),
        "help" => Some(Action::Help),
        "menu" => Some(Action::Menu),
        "refresh" => Some(Action::Refresh),
        "toggle_pause" | "pause" => Some(Action::TogglePause),
        "focus_next" => Some(Action::FocusNext),
        "focus_prev" => Some(Action::FocusPrev),
        "scroll_up" => Some(Action::ScrollUp),
        "scroll_down" => Some(Action::ScrollDown),
        "sort_next" => Some(Action::SortNext),
        "sort_prev" => Some(Action::SortPrev),
        "search" => Some(Action::Search),
        "escape" => Some(Action::Escape),
        _ => None,
    }
}

fn parse_keybind(s: &str) -> Option<KeyBind> {
    let s = s.trim();

    // Handle Ctrl+ prefix
    if let Some(rest) = s.strip_prefix("ctrl+") {
        let rest = rest.trim();
        if rest.len() == 1 {
            return Some(KeyBind::ctrl(KeyCode::Char(
                rest.chars().next().unwrap(),
            )));
        }
    }

    // Single character
    if s.len() == 1 {
        return Some(KeyBind::plain(KeyCode::Char(s.chars().next().unwrap())));
    }

    // Named keys
    match s.to_lowercase().as_str() {
        "space" => Some(KeyBind::plain(KeyCode::Char(' '))),
        "tab" => Some(KeyBind::plain(KeyCode::Tab)),
        "backtab" | "shift+tab" => Some(KeyBind::plain(KeyCode::BackTab)),
        "esc" | "escape" => Some(KeyBind::plain(KeyCode::Esc)),
        "enter" | "return" => Some(KeyBind::plain(KeyCode::Enter)),
        "up" => Some(KeyBind::plain(KeyCode::Up)),
        "down" => Some(KeyBind::plain(KeyCode::Down)),
        "left" => Some(KeyBind::plain(KeyCode::Left)),
        "right" => Some(KeyBind::plain(KeyCode::Right)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_bindings_resolve() {
        let kb = KeyBindings::default();
        assert_eq!(
            kb.resolve(KeyCode::Char('q'), KeyModifiers::NONE),
            Some(Action::Quit)
        );
        assert_eq!(
            kb.resolve(KeyCode::Char('c'), KeyModifiers::CONTROL),
            Some(Action::Quit)
        );
        assert_eq!(
            kb.resolve(KeyCode::Char('?'), KeyModifiers::NONE),
            Some(Action::Help)
        );
    }

    #[test]
    fn custom_override() {
        let mut overrides = HashMap::new();
        overrides.insert("quit".to_string(), "x".to_string());
        let kb = KeyBindings::new(&overrides);

        // 'x' should now quit
        assert_eq!(
            kb.resolve(KeyCode::Char('x'), KeyModifiers::NONE),
            Some(Action::Quit)
        );
        // 'q' should no longer quit (replaced)
        assert_eq!(kb.resolve(KeyCode::Char('q'), KeyModifiers::NONE), None);
    }

    #[test]
    fn parse_keybind_variants() {
        assert_eq!(parse_keybind("q"), Some(KeyBind::plain(KeyCode::Char('q'))));
        assert_eq!(
            parse_keybind("ctrl+c"),
            Some(KeyBind::ctrl(KeyCode::Char('c')))
        );
        assert_eq!(parse_keybind("space"), Some(KeyBind::plain(KeyCode::Char(' '))));
        assert_eq!(parse_keybind("tab"), Some(KeyBind::plain(KeyCode::Tab)));
        assert_eq!(parse_keybind("esc"), Some(KeyBind::plain(KeyCode::Esc)));
    }
}
