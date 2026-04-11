use std::time::Instant;

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
    update_interval_ms: u64,
    terminal_size: (u16, u16),
}

impl App {
    pub fn new(update_interval_ms: u64) -> Self {
        let hostname = hostname::get()
            .map(|h| h.to_string_lossy().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        Self {
            state: AppState::Running,
            hostname,
            start_time: Instant::now(),
            update_interval_ms,
            terminal_size: (80, 24),
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
        self.update_interval_ms
    }

    pub fn on_resize(&mut self, width: u16, height: u16) {
        self.terminal_size = (width, height);
    }

    pub fn is_running(&self) -> bool {
        self.state == AppState::Running
    }
}
