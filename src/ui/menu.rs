use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Flex, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::config::settings::{Config, GraphSymbols, LayoutPreset};
use crate::ui::theme::{self, Theme};

/// The kind of value a menu item controls.
#[derive(Debug, Clone)]
pub enum MenuItemKind {
    Numeric {
        min: u64,
        max: u64,
        step: u64,
        current: u64,
    },
    Cycle {
        options: Vec<String>,
        current_index: usize,
    },
    Toggle {
        enabled: bool,
    },
}

/// A single row in the settings menu.
#[derive(Debug, Clone)]
pub struct MenuItem {
    pub label: String,
    pub value: String,
    pub kind: MenuItemKind,
}

/// In-app settings overlay.
#[derive(Debug, Clone)]
pub struct SettingsMenu {
    pub selected_index: usize,
    pub items: Vec<MenuItem>,
}

impl SettingsMenu {
    /// Build the menu from the current application config.
    pub fn from_config(config: &Config) -> Self {
        let graph_options = vec![
            "Braille".to_string(),
            "Block".to_string(),
            "TTY".to_string(),
        ];
        let graph_index = match config.graph_symbols {
            GraphSymbols::Braille => 0,
            GraphSymbols::Block => 1,
            GraphSymbols::Tty => 2,
        };

        let theme_options: Vec<String> = theme::builtin_theme_names()
            .iter()
            .map(|s| s.to_string())
            .collect();
        let theme_index = theme_options
            .iter()
            .position(|t| t == &config.theme)
            .unwrap_or(0);

        let layout_options: Vec<String> = LayoutPreset::all_names()
            .iter()
            .map(|s| s.to_string())
            .collect();
        let layout_index = layout_options
            .iter()
            .position(|l| l == config.layout.display_name())
            .unwrap_or(0);

        let items = vec![
            MenuItem {
                label: "Update Interval".to_string(),
                value: format!("{}", config.update_interval_ms),
                kind: MenuItemKind::Numeric {
                    min: 100,
                    max: 10000,
                    step: 100,
                    current: config.update_interval_ms,
                },
            },
            MenuItem {
                label: "Graph Symbols".to_string(),
                value: graph_options[graph_index].clone(),
                kind: MenuItemKind::Cycle {
                    options: graph_options,
                    current_index: graph_index,
                },
            },
            MenuItem {
                label: "Theme".to_string(),
                value: theme_options[theme_index].clone(),
                kind: MenuItemKind::Cycle {
                    options: theme_options,
                    current_index: theme_index,
                },
            },
            MenuItem {
                label: "Layout".to_string(),
                value: layout_options[layout_index].clone(),
                kind: MenuItemKind::Cycle {
                    options: layout_options,
                    current_index: layout_index,
                },
            },
            MenuItem {
                label: "Show CPU".to_string(),
                value: toggle_display(config.panels.cpu),
                kind: MenuItemKind::Toggle {
                    enabled: config.panels.cpu,
                },
            },
            MenuItem {
                label: "Show Memory".to_string(),
                value: toggle_display(config.panels.memory),
                kind: MenuItemKind::Toggle {
                    enabled: config.panels.memory,
                },
            },
            MenuItem {
                label: "Show Disk".to_string(),
                value: toggle_display(config.panels.disk),
                kind: MenuItemKind::Toggle {
                    enabled: config.panels.disk,
                },
            },
            MenuItem {
                label: "Show Network".to_string(),
                value: toggle_display(config.panels.network),
                kind: MenuItemKind::Toggle {
                    enabled: config.panels.network,
                },
            },
            MenuItem {
                label: "Show Processes".to_string(),
                value: toggle_display(config.panels.processes),
                kind: MenuItemKind::Toggle {
                    enabled: config.panels.processes,
                },
            },
            MenuItem {
                label: "Show Docker".to_string(),
                value: toggle_display(config.panels.docker),
                kind: MenuItemKind::Toggle {
                    enabled: config.panels.docker,
                },
            },
            MenuItem {
                label: "Show Kubernetes".to_string(),
                value: toggle_display(config.panels.k8s),
                kind: MenuItemKind::Toggle {
                    enabled: config.panels.k8s,
                },
            },
            MenuItem {
                label: "Show GPU".to_string(),
                value: toggle_display(config.panels.gpu),
                kind: MenuItemKind::Toggle {
                    enabled: config.panels.gpu,
                },
            },
            MenuItem {
                label: "Show Sensors".to_string(),
                value: toggle_display(config.panels.sensors),
                kind: MenuItemKind::Toggle {
                    enabled: config.panels.sensors,
                },
            },
        ];

        Self {
            selected_index: 0,
            items,
        }
    }

    /// Select the previous item (wraps around).
    pub fn move_up(&mut self) {
        if self.items.is_empty() {
            return;
        }
        if self.selected_index == 0 {
            self.selected_index = self.items.len() - 1;
        } else {
            self.selected_index -= 1;
        }
    }

    /// Select the next item (wraps around).
    pub fn move_down(&mut self) {
        if self.items.is_empty() {
            return;
        }
        self.selected_index = (self.selected_index + 1) % self.items.len();
    }

    /// Increase the selected item's value.
    pub fn increment(&mut self) {
        if let Some(item) = self.items.get_mut(self.selected_index) {
            match &mut item.kind {
                MenuItemKind::Numeric {
                    max, step, current, ..
                } => {
                    *current = (*current + *step).min(*max);
                    item.value = format!("{}", *current);
                }
                MenuItemKind::Cycle {
                    options,
                    current_index,
                } => {
                    *current_index = (*current_index + 1) % options.len();
                    item.value = options[*current_index].clone();
                }
                MenuItemKind::Toggle { enabled } => {
                    *enabled = !*enabled;
                    item.value = toggle_display(*enabled);
                }
            }
        }
    }

    /// Decrease the selected item's value.
    pub fn decrement(&mut self) {
        if let Some(item) = self.items.get_mut(self.selected_index) {
            match &mut item.kind {
                MenuItemKind::Numeric {
                    min, step, current, ..
                } => {
                    *current = current.saturating_sub(*step).max(*min);
                    item.value = format!("{}", *current);
                }
                MenuItemKind::Cycle {
                    options,
                    current_index,
                } => {
                    if *current_index == 0 {
                        *current_index = options.len() - 1;
                    } else {
                        *current_index -= 1;
                    }
                    item.value = options[*current_index].clone();
                }
                MenuItemKind::Toggle { enabled } => {
                    *enabled = !*enabled;
                    item.value = toggle_display(*enabled);
                }
            }
        }
    }

    /// Write current menu values back into a `Config`.
    pub fn apply_to_config(&self, config: &mut Config) {
        for item in &self.items {
            match item.label.as_str() {
                "Update Interval" => {
                    if let MenuItemKind::Numeric { current, .. } = &item.kind {
                        config.update_interval_ms = *current;
                    }
                }
                "Graph Symbols" => {
                    if let MenuItemKind::Cycle { current_index, .. } = &item.kind {
                        config.graph_symbols = match current_index {
                            0 => GraphSymbols::Braille,
                            1 => GraphSymbols::Block,
                            _ => GraphSymbols::Tty,
                        };
                    }
                }
                "Theme" => {
                    if let MenuItemKind::Cycle {
                        options,
                        current_index,
                        ..
                    } = &item.kind
                    {
                        config.theme = options[*current_index].to_lowercase();
                    }
                }
                "Layout" => {
                    if let MenuItemKind::Cycle {
                        options,
                        current_index,
                        ..
                    } = &item.kind
                    {
                        let preset = LayoutPreset::from_name(&options[*current_index]);
                        config.layout = preset;
                        preset.apply_to_panels(&mut config.panels);
                    }
                }
                "Show CPU" => {
                    if let MenuItemKind::Toggle { enabled } = &item.kind {
                        config.panels.cpu = *enabled;
                    }
                }
                "Show Memory" => {
                    if let MenuItemKind::Toggle { enabled } = &item.kind {
                        config.panels.memory = *enabled;
                    }
                }
                "Show Disk" => {
                    if let MenuItemKind::Toggle { enabled } = &item.kind {
                        config.panels.disk = *enabled;
                    }
                }
                "Show Network" => {
                    if let MenuItemKind::Toggle { enabled } = &item.kind {
                        config.panels.network = *enabled;
                    }
                }
                "Show Processes" => {
                    if let MenuItemKind::Toggle { enabled } = &item.kind {
                        config.panels.processes = *enabled;
                    }
                }
                "Show Docker" => {
                    if let MenuItemKind::Toggle { enabled } = &item.kind {
                        config.panels.docker = *enabled;
                    }
                }
                "Show Kubernetes" => {
                    if let MenuItemKind::Toggle { enabled } = &item.kind {
                        config.panels.k8s = *enabled;
                    }
                }
                "Show GPU" => {
                    if let MenuItemKind::Toggle { enabled } = &item.kind {
                        config.panels.gpu = *enabled;
                    }
                }
                "Show Sensors" => {
                    if let MenuItemKind::Toggle { enabled } = &item.kind {
                        config.panels.sensors = *enabled;
                    }
                }
                _ => {}
            }
        }
    }
}

fn toggle_display(enabled: bool) -> String {
    if enabled {
        "[✓]".to_string()
    } else {
        "[ ]".to_string()
    }
}

/// Format the display string for a menu item's value column.
fn format_value(kind: &MenuItemKind) -> String {
    match kind {
        MenuItemKind::Numeric { current, .. } => format!("◄ {} ►", current),
        MenuItemKind::Cycle {
            options,
            current_index,
        } => format!("◄ {} ►", options[*current_index]),
        MenuItemKind::Toggle { enabled } => toggle_display(*enabled),
    }
}

/// Center a rectangle of `width` × `height` within `area`.
fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .flex(Flex::Center)
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(width),
            Constraint::Min(0),
        ])
        .flex(Flex::Center)
        .split(vertical[1]);

    horizontal[1]
}

/// Render the settings menu as a modal overlay.
pub fn render(frame: &mut Frame, menu: &SettingsMenu, theme: &Theme) {
    let area = frame.area();

    // Clear the entire terminal behind the menu
    frame.render_widget(Clear, area);

    // 2 border rows + 1 footer + 1 blank before footer + item rows
    let height = menu.items.len() as u16 + 4;
    let width: u16 = 45;

    let menu_area = centered_rect(width, height, area);

    let block = Block::default()
        .title(" Settings ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.dialog_border));

    let inner = block.inner(menu_area);
    frame.render_widget(block, menu_area);

    // Split inner into item rows + blank + footer
    let mut constraints: Vec<Constraint> =
        menu.items.iter().map(|_| Constraint::Length(1)).collect();
    constraints.push(Constraint::Min(0)); // spacer
    constraints.push(Constraint::Length(1)); // footer

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    let inner_width = inner.width as usize;

    for (i, item) in menu.items.iter().enumerate() {
        let value_str = format_value(&item.kind);
        let label = &item.label;

        // Pad so label is left-aligned and value is right-aligned
        let gap = inner_width.saturating_sub(label.len() + value_str.len());
        let line_text = format!("{}{:>gap$}", label, value_str, gap = gap + value_str.len());

        let style = if i == menu.selected_index {
            Style::default()
                .bg(theme.selected_bg)
                .fg(theme.selected_fg)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let line = Paragraph::new(Line::from(Span::styled(line_text, style)));
        frame.render_widget(line, chunks[i]);
    }

    // Footer
    let footer_idx = chunks.len() - 1;
    let footer = Paragraph::new(Line::from(Span::styled(
        "↑↓ Navigate  ←→ Change  Esc Close  m Close",
        Style::default().fg(theme.text_secondary),
    )))
    .alignment(Alignment::Center);
    frame.render_widget(footer, chunks[footer_idx]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::settings::Config;

    #[test]
    fn from_config_builds_correct_items() {
        let config = Config::default();
        let menu = SettingsMenu::from_config(&config);
        assert_eq!(menu.items.len(), 13);
        assert_eq!(menu.items[0].label, "Update Interval");
        assert_eq!(menu.items[1].label, "Graph Symbols");
        assert_eq!(menu.items[2].label, "Theme");
        assert_eq!(menu.items[3].label, "Layout");
        assert_eq!(menu.selected_index, 0);
    }

    #[test]
    fn move_up_wraps() {
        let config = Config::default();
        let mut menu = SettingsMenu::from_config(&config);
        menu.move_up();
        assert_eq!(menu.selected_index, 12);
    }

    #[test]
    fn move_down_wraps() {
        let config = Config::default();
        let mut menu = SettingsMenu::from_config(&config);
        menu.selected_index = 12;
        menu.move_down();
        assert_eq!(menu.selected_index, 0);
    }

    #[test]
    fn increment_numeric() {
        let config = Config::default();
        let mut menu = SettingsMenu::from_config(&config);
        // Item 0 is Update Interval, default 1000, step 100
        menu.increment();
        if let MenuItemKind::Numeric { current, .. } = &menu.items[0].kind {
            assert_eq!(*current, 1100);
        } else {
            panic!("expected Numeric");
        }
    }

    #[test]
    fn decrement_numeric_clamps_at_min() {
        let config = Config {
            update_interval_ms: 100,
            ..Config::default()
        };
        let mut menu = SettingsMenu::from_config(&config);
        menu.decrement();
        if let MenuItemKind::Numeric { current, .. } = &menu.items[0].kind {
            assert_eq!(*current, 100);
        } else {
            panic!("expected Numeric");
        }
    }

    #[test]
    fn increment_cycle_wraps() {
        let config = Config::default();
        let mut menu = SettingsMenu::from_config(&config);
        menu.selected_index = 1; // Graph Symbols
        menu.increment(); // Braille -> Block
        menu.increment(); // Block -> TTY
        menu.increment(); // TTY -> Braille (wrap)
        if let MenuItemKind::Cycle { current_index, .. } = &menu.items[1].kind {
            assert_eq!(*current_index, 0);
        } else {
            panic!("expected Cycle");
        }
    }

    #[test]
    fn toggle_flips() {
        let config = Config::default();
        let mut menu = SettingsMenu::from_config(&config);
        menu.selected_index = 4; // Show CPU
        menu.increment();
        if let MenuItemKind::Toggle { enabled } = &menu.items[4].kind {
            assert!(!enabled); // was true, now false
        } else {
            panic!("expected Toggle");
        }
    }

    #[test]
    fn apply_to_config_roundtrip() {
        let mut config = Config::default();
        let mut menu = SettingsMenu::from_config(&config);

        // Change update interval
        menu.increment(); // 1000 -> 1100

        // Change graph symbols to Block
        menu.selected_index = 1;
        menu.increment(); // Braille -> Block

        // Disable CPU panel (index 4 now)
        menu.selected_index = 4;
        menu.increment(); // true -> false

        menu.apply_to_config(&mut config);

        assert_eq!(config.update_interval_ms, 1100);
        assert_eq!(config.graph_symbols, GraphSymbols::Block);
        assert!(!config.panels.cpu);
        // Unchanged values stay the same
        assert!(config.panels.memory);
    }
}
