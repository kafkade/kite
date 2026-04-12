use crossterm::event::{KeyCode, KeyEvent};

use crate::app::{App, AppState};
use crate::config::keybindings::Action;
use crate::ui::dialog::{ConfirmDialog, DialogChoice};

/// Handle a key event by resolving it through the keybinding map.
pub fn handle_key_event(app: &mut App, key: KeyEvent) {
    // If a dialog is open, handle dialog input first
    if let Some(ref mut dialog) = app.dialog {
        match key.code {
            KeyCode::Left | KeyCode::Right | KeyCode::Tab => dialog.toggle_selection(),
            KeyCode::Enter => {
                if dialog.selected == DialogChoice::Confirm {
                    // Execute the kill
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
        return;
    }

    // If process filter input is active, handle text input
    if app.proc_widget.is_filtering {
        match key.code {
            KeyCode::Esc => {
                app.proc_widget.stop_filter();
                app.proc_collector.set_filter(None);
            }
            KeyCode::Enter => {
                app.proc_widget.is_filtering = false;
                // Filter stays active, just stop capturing input
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
        return;
    }

    // Normal mode: resolve through keybinding map
    let action = app.keybindings().resolve(key.code, key.modifiers);

    match action {
        Some(Action::Quit) => app.set_state(AppState::Quitting),
        Some(Action::ScrollUp) => {
            app.proc_widget.scroll_up();
        }
        Some(Action::ScrollDown) => {
            let total = app.proc_collector.processes().len();
            app.proc_widget.scroll_down(total);
        }
        Some(Action::SortNext) => {
            app.proc_collector.next_sort_column();
        }
        Some(Action::SortPrev) => {
            app.proc_collector.prev_sort_column();
        }
        Some(Action::TogglePause) => {
            app.proc_collector.toggle_pause();
        }
        Some(Action::Search) => {
            app.proc_widget.start_filter();
        }
        Some(Action::Escape) => {
            if app.proc_widget.filter_input.is_empty() {
                // Nothing to clear
            } else {
                app.proc_widget.stop_filter();
                app.proc_collector.set_filter(None);
            }
        }
        None => {
            // Handle keys not in the keybinding map
            match key.code {
                KeyCode::Char('t') | KeyCode::Char('T') => {
                    app.proc_widget.toggle_tree();
                }
                KeyCode::Char('k') | KeyCode::Char('K') => {
                    // Kill selected process (with confirmation)
                    if let Some(pid) = app.proc_widget.selected_pid(app.proc_collector.processes())
                    {
                        let name = app
                            .proc_collector
                            .processes()
                            .iter()
                            .find(|p| p.pid == pid)
                            .map(|p| p.name.clone())
                            .unwrap_or_default();
                        let dialog = ConfirmDialog::new(
                            "Kill Process",
                            format!("Kill process {} (PID {})?", name, pid),
                        )
                        .with_labels("Kill", "Cancel");
                        app.dialog = Some(dialog);
                    }
                }
                KeyCode::PageUp => {
                    app.proc_widget.page_up();
                }
                KeyCode::PageDown => {
                    let total = app.proc_collector.processes().len();
                    app.proc_widget.page_down(total, 20);
                }
                _ => {}
            }
        }
        Some(_) => {}
    }
}
