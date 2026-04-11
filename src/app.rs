use std::time::Instant;

use crate::collector::Collector;
use crate::collector::cpu::CpuCollector;
use crate::collector::memory::MemoryCollector;
use crate::config::keybindings::KeyBindings;
use crate::config::settings::Config;

/// Application state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    Running,
    Quitting,
}

/// Core application struct holding all shared state.
pub struct App {
    state: AppState,
    hostname: String,
    start_time: Instant,
    config: Config,
    keybindings: KeyBindings,
    terminal_size: (u16, u16),
    pub cpu: CpuCollector,
    pub mem: MemoryCollector,
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

        // Initial collection so first render has data
        let _ = cpu.collect();
        let _ = mem.collect();

        Self {
            state: AppState::Running,
            hostname,
            start_time: Instant::now(),
            config,
            keybindings,
            terminal_size: (80, 24),
            cpu,
            mem,
        }
    }

    /// Collect fresh data from all collectors.
    pub fn collect_all(&mut self) {
        let _ = self.cpu.collect();
        let _ = self.mem.collect();
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
