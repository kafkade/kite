use std::collections::HashMap;
use std::time::Instant;

use crate::alert::{AlertEngine, Metric};
use crate::collector::Collector;
use crate::collector::battery::BatteryCollector;
use crate::collector::cpu::CpuCollector;
use crate::collector::disk::DiskCollector;
use crate::collector::docker::DockerCollector;
use crate::collector::gpu::GpuCollector;
use crate::collector::k8s::K8sCollector;
use crate::collector::memory::MemoryCollector;
use crate::collector::network::NetworkCollector;
use crate::collector::process::ProcessCollector;
use crate::collector::remote::RemoteCollector;
use crate::collector::sensor::SensorCollector;
use crate::config::keybindings::KeyBindings;
use crate::config::settings::Config;
use crate::ui::dialog::ConfirmDialog;
use crate::ui::menu::SettingsMenu;
use crate::ui::theme::{self, Theme};
use crate::ui::widgets::container_box::ContainerWidget;
use crate::ui::widgets::k8s_box::K8sWidget;
use crate::ui::widgets::proc_box::ProcessWidget;
use crate::ui::widgets::remote_box::RemoteWidget;

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
    pub docker: DockerCollector,
    pub container_widget: ContainerWidget,
    pub sensor: SensorCollector,
    pub gpu: GpuCollector,
    pub battery: BatteryCollector,
    pub k8s: K8sCollector,
    pub k8s_widget: K8sWidget,
    pub remote: RemoteCollector,
    pub remote_widget: RemoteWidget,
    pub dialog: Option<ConfirmDialog>,
    pub menu: Option<SettingsMenu>,
    pub theme: Theme,
    pub alerts: AlertEngine,
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
        let mut docker = DockerCollector::new();
        let mut sensor = SensorCollector::new(history_depth);
        let mut gpu = GpuCollector::new(history_depth);
        let mut battery = BatteryCollector::new();
        let mut k8s = K8sCollector::new();
        let mut remote = RemoteCollector::new(&config.remotes, history_depth);

        // Initial collection so first render has data
        let _ = cpu.collect();
        let _ = mem.collect();
        let _ = disk.collect();
        let _ = net.collect();
        let _ = proc_collector.collect();
        let _ = docker.collect();
        let _ = sensor.collect();
        let _ = gpu.collect();
        let _ = battery.collect();
        let _ = k8s.collect();
        let _ = remote.collect();

        let theme = theme::get_builtin_theme(&config.theme).unwrap_or_else(theme::default_theme);

        let alert_rules = if config.alerts.is_empty() {
            Config::default_alert_rules()
        } else {
            config.alerts.clone()
        };
        let alerts = AlertEngine::new(alert_rules, 100);

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
            docker,
            container_widget: ContainerWidget::new(),
            sensor,
            gpu,
            battery,
            k8s,
            k8s_widget: K8sWidget::new(),
            remote,
            remote_widget: RemoteWidget::new(),
            dialog: None,
            menu: None,
            theme,
            alerts,
        }
    }

    /// Collect fresh data from all collectors.
    pub fn collect_all(&mut self) {
        let _ = self.cpu.collect();
        let _ = self.mem.collect();
        let _ = self.disk.collect();
        let _ = self.net.collect();
        let _ = self.proc_collector.collect();
        let _ = self.docker.collect();
        let _ = self.sensor.collect();
        let _ = self.gpu.collect();
        let _ = self.battery.collect();
        let _ = self.k8s.collect();
        let _ = self.remote.collect();

        // Evaluate alert rules
        let metrics = self.collect_alert_metrics();
        self.alerts.evaluate(&metrics);
    }

    /// Collect current metric values for alert evaluation.
    fn collect_alert_metrics(&self) -> HashMap<Metric, f64> {
        let mut metrics = HashMap::new();
        metrics.insert(Metric::CpuTotal, self.cpu.total_usage());
        metrics.insert(Metric::MemoryPercent, self.mem.ram_usage_percent());
        metrics.insert(Metric::SwapPercent, self.mem.swap_usage_percent());

        if let Some(temp) = self.sensor.cpu_temp() {
            metrics.insert(Metric::CpuTemperature, temp as f64);
        }

        if let Some(dev) = self.gpu.devices().first() {
            if let Some(temp) = dev.temperature {
                metrics.insert(Metric::GpuTemperature, temp as f64);
            }
            if let Some(util) = dev.utilization_gpu {
                metrics.insert(Metric::GpuUtilization, util as f64);
            }
        }

        metrics
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
            if let Some(new_theme) = theme::get_builtin_theme(&self.config.theme) {
                self.theme = new_theme;
            }
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

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    #[allow(dead_code)]
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
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
