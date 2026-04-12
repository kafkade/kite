use std::time::Instant;

use crate::collector::Collector;
use crate::collector::cpu::CpuCollector;
use crate::collector::disk::DiskCollector;
use crate::collector::memory::MemoryCollector;
use crate::collector::network::NetworkCollector;
use crate::collector::process::ProcessCollector;
use crate::config::keybindings::KeyBindings;
use crate::config::settings::Config;
use crate::ui::dialog::ConfirmDialog;
use crate::ui::menu::SettingsMenu;
use crate::ui::widgets::proc_box::ProcessWidget;

/// Application state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    Running,
    Quitting,
}

/// Input mode determines how key events are interpreted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Filtering,
    Help,
    Menu,
}

/// Core application struct holding all shared state.
pub struct App {
    state: AppState,
    pub input_mode: InputMode,
    hostname: String,
    start_time: Instant,
    config: Config,
    keybindings: KeyBindings,
    terminal_size: (u16, u16),
    pub cpu: CpuCollector,
    pub mem: MemoryCollector,
    pub disk: DiskCollector,
    pub net: NetworkCollector,
    pub proc_collector: ProcessCollector,
    pub proc_widget: ProcessWidget,
    pub dialog: Option<ConfirmDialog>,
    pub menu: Option<SettingsMenu>,
}

impl App {
    pub fn new(config: Config) -> Self {
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        let keybindings = KeyBindings::new(&config.keybindings);
        let history_depth = config.graph_history_depth;

        let mut cpu = CpuCollector::new(history_depth);
        let mut mem = MemoryCollector::new(history_depth);
        let mut disk = DiskCollector::new(history_depth);
        let mut net = NetworkCollector::new(history_depth);
        let mut proc_collector = ProcessCollector::new();

        // Initial collection so first render has data
        let _ = cpu.collect();
        let _ = mem.collect();
        let _ = disk.collect();
        let _ = net.collect();
        let _ = proc_collector.collect();

        Self {
            state: AppState::Running,
            input_mode: InputMode::Normal,
            hostname,
            start_time: Instant::now(),
            config,
            keybindings,
            terminal_size: (80, 24),
            cpu,
            mem,
            disk,
            net,
            proc_collector,
            proc_widget: ProcessWidget::new(),
            dialog: None,
            menu: None,
        }
    }

    /// Collect fresh data from all collectors.
    pub fn collect_all(&mut self) {
        let _ = self.cpu.collect();
        let _ = self.mem.collect();
        let _ = self.disk.collect();
        let _ = self.net.collect();
        let _ = self.proc_collector.collect();
    }

    /// Open the help overlay.
    pub fn open_help(&mut self) {
        self.input_mode = InputMode::Help;
    }

    /// Close the help overlay.
    pub fn close_help(&mut self) {
        self.input_mode = InputMode::Normal;
    }

    /// Toggle help overlay.
    pub fn toggle_help(&mut self) {
        if self.input_mode == InputMode::Help {
            self.close_help();
        } else {
            self.open_help();
        }
    }

    /// Open the settings menu.
    pub fn open_menu(&mut self) {
        self.menu = Some(SettingsMenu::from_config(&self.config));
        self.input_mode = InputMode::Menu;
    }

    /// Close the settings menu, applying changes.
    pub fn close_menu(&mut self) {
        if let Some(ref menu) = self.menu {
            menu.apply_to_config(&mut self.config);
        }
        self.menu = None;
        self.input_mode = InputMode::Normal;
    }

    /// Toggle the settings menu.
    pub fn toggle_menu(&mut self) {
        if self.input_mode == InputMode::Menu {
            self.close_menu();
        } else {
            self.open_menu();
        }
    }

    pub fn state(&self) -> AppState {
        self.state
    }

    pub fn set_state(&mut self, state: AppState) {
        self.state = state;
    }

    pub fn hostname(&self) -> &str {
        &self.hostname
    }

    pub fn uptime_secs(&self) -> u64 {
        self.start_time.elapsed().as_secs()
    }

    pub fn update_interval_ms(&self) -> u64 {
        self.config.update_interval_ms
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn keybindings(&self) -> &KeyBindings {
        &self.keybindings
    }

    pub fn on_resize(&mut self, width: u16, height: u16) {
        self.terminal_size = (width, height);
    }

    pub fn is_running(&self) -> bool {
        self.state == AppState::Running
    }
}
