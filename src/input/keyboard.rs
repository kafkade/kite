use crossterm::event::{KeyCode, KeyEvent};

use crate::app::{App, AppState, InputMode};
use crate::config::keybindings::Action;
use crate::ui::dialog::{ConfirmDialog, DialogChoice};

/// Handle a key event based on the current input mode.
pub fn handle_key_event(app: &mut App, key: KeyEvent) {
    // If a dialog is open, it takes priority over everything
    if app.dialog.is_some() {
        handle_dialog(app, key);
        return;
    }

    match app.input_mode {
        InputMode::Help => handle_help(app, key),
        InputMode::Menu => handle_menu(app, key),
        InputMode::Filtering => handle_filter(app, key),
        InputMode::Normal => handle_normal(app, key),
    }
}

fn handle_dialog(app: &mut App, key: KeyEvent) {
    if let Some(ref mut dialog) = app.dialog {
        match key.code {
            KeyCode::Left | KeyCode::Right | KeyCode::Tab => dialog.toggle_selection(),
            KeyCode::Enter => {
                if dialog.selected == DialogChoice::Confirm {
                    if let Some(pid) = app.proc_widget.selected_pid(app.proc_collector.processes())
                    {
                        let _ = app.proc_collector.kill_process(pid);
                    }
                }
                app.dialog = None;
            }
            KeyCode::Esc => {
                app.dialog = None;
            }
            _ => {}
        }
    }
}

fn handle_help(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('?') | KeyCode::Esc | KeyCode::Char('q') => app.close_help(),
        _ => {}
    }
}

fn handle_menu(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('m') => app.close_menu(),
        KeyCode::Up | KeyCode::Char('k') => {
            if let Some(ref mut menu) = app.menu {
                menu.move_up();
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if let Some(ref mut menu) = app.menu {
                menu.move_down();
            }
        }
        KeyCode::Right | KeyCode::Enter => {
            if let Some(ref mut menu) = app.menu {
                menu.increment();
            }
        }
        KeyCode::Left => {
            if let Some(ref mut menu) = app.menu {
                menu.decrement();
            }
        }
        _ => {}
    }
}

fn handle_filter(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.proc_widget.stop_filter();
            app.proc_collector.set_filter(None);
            app.input_mode = InputMode::Normal;
        }
        KeyCode::Enter => {
            app.proc_widget.is_filtering = false;
            app.input_mode = InputMode::Normal;
        }
        KeyCode::Backspace => {
            app.proc_widget.backspace_filter();
            let f = if app.proc_widget.filter_input.is_empty() {
                None
            } else {
                Some(app.proc_widget.filter_input.clone())
            };
            app.proc_collector.set_filter(f);
        }
        KeyCode::Char(c) => {
            app.proc_widget.append_filter_char(c);
            app.proc_collector
                .set_filter(Some(app.proc_widget.filter_input.clone()));
        }
        _ => {}
    }
}

fn handle_normal(app: &mut App, key: KeyEvent) {
    let action = app.keybindings().resolve(key.code, key.modifiers);

    match action {
        Some(Action::Quit) => app.set_state(AppState::Quitting),
        Some(Action::Help) => app.toggle_help(),
        Some(Action::Menu) => app.toggle_menu(),
        Some(Action::ScrollUp) => app.proc_widget.scroll_up(),
        Some(Action::ScrollDown) => {
            let total = app.proc_collector.processes().len();
            app.proc_widget.scroll_down(total);
        }
        Some(Action::SortNext) => app.proc_collector.next_sort_column(),
        Some(Action::SortPrev) => app.proc_collector.prev_sort_column(),
        Some(Action::TogglePause) => app.proc_collector.toggle_pause(),
        Some(Action::Search) => {
            app.proc_widget.start_filter();
            app.input_mode = InputMode::Filtering;
        }
        Some(Action::Escape) => {
            if !app.proc_widget.filter_input.is_empty() {
                app.proc_widget.stop_filter();
                app.proc_collector.set_filter(None);
            }
        }
        Some(Action::Refresh) => app.collect_all(),
        None => match key.code {
            KeyCode::Char('t') | KeyCode::Char('T') => app.proc_widget.toggle_tree(),
            KeyCode::Char('K') => {
                if let Some(pid) = app.proc_widget.selected_pid(app.proc_collector.processes()) {
                    let name = app
                        .proc_collector
                        .processes()
                        .iter()
                        .find(|p| p.pid == pid)
                        .map(|p| p.name.clone())
                        .unwrap_or_default();
                    app.dialog = Some(
                        ConfirmDialog::new(
                            "Kill Process",
                            format!("Kill process {} (PID {})?", name, pid),
                        )
                        .with_labels("Kill", "Cancel"),
                    );
                }
            }
            KeyCode::PageUp => app.proc_widget.page_up(),
            KeyCode::PageDown => {
                let total = app.proc_collector.processes().len();
                app.proc_widget.page_down(total, 20);
            }
            _ => {}
        },
        Some(_) => {}
    }
}
