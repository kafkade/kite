use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::alert::rules::{AlertRule, Condition, Metric, Severity};

/// Graph symbol set for rendering charts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum GraphSymbols {
    #[default]
    Braille,
    Block,
    Tty,
}

/// Color mode for terminal rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ColorMode {
    #[default]
    Auto,
    TrueColor,
    Color256,
    Color16,
}

/// Which panels are visible.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanelVisibility {
    #[serde(default = "bool_true")]
    pub cpu: bool,
    #[serde(default = "bool_true")]
    pub memory: bool,
    #[serde(default = "bool_true")]
    pub disk: bool,
    #[serde(default = "bool_true")]
    pub network: bool,
    #[serde(default = "bool_true")]
    pub processes: bool,
    #[serde(default = "bool_true")]
    pub docker: bool,
    #[serde(default = "bool_true")]
    pub gpu: bool,
    #[serde(default)]
    pub k8s: bool,
    #[serde(default = "bool_true")]
    pub sensors: bool,
    #[serde(default)]
    pub remote: bool,
}

impl Default for PanelVisibility {
    fn default() -> Self {
        Self {
            cpu: true,
            memory: true,
            disk: true,
            network: true,
            processes: true,
            docker: true,
            gpu: true,
            k8s: false,
            sensors: true,
            remote: false,
        }
    }
}

/// Layout presets that configure panel visibility in one step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LayoutPreset {
    #[default]
    Default,
    Minimal,
    Full,
    Server,
    Laptop,
    GpuFocus,
}

impl LayoutPreset {
    /// Get all preset names for menu display.
    pub fn all_names() -> Vec<&'static str> {
        vec![
            "Default",
            "Minimal",
            "Full",
            "Server",
            "Laptop",
            "GPU Focus",
        ]
    }

    /// Convert from display name to enum.
    pub fn from_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "minimal" => Self::Minimal,
            "full" => Self::Full,
            "server" => Self::Server,
            "laptop" => Self::Laptop,
            "gpu focus" | "gpu-focus" | "gpufocus" | "gpu_focus" => Self::GpuFocus,
            _ => Self::Default,
        }
    }

    /// Convert to display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::Minimal => "Minimal",
            Self::Full => "Full",
            Self::Server => "Server",
            Self::Laptop => "Laptop",
            Self::GpuFocus => "GPU Focus",
        }
    }

    /// Apply this preset to PanelVisibility.
    pub fn apply_to_panels(&self, panels: &mut PanelVisibility) {
        match self {
            Self::Default => {
                panels.cpu = true;
                panels.memory = true;
                panels.disk = true;
                panels.network = true;
                panels.processes = true;
                panels.docker = true;
                panels.gpu = true;
                panels.k8s = false;
                panels.sensors = true;
                panels.remote = false;
            }
            Self::Full => {
                panels.cpu = true;
                panels.memory = true;
                panels.disk = true;
                panels.network = true;
                panels.processes = true;
                panels.docker = true;
                panels.gpu = true;
                panels.k8s = true;
                panels.sensors = true;
                panels.remote = true;
            }
            Self::Minimal => {
                panels.cpu = true;
                panels.memory = true;
                panels.disk = false;
                panels.network = false;
                panels.processes = true;
                panels.docker = false;
                panels.gpu = false;
                panels.k8s = false;
                panels.sensors = false;
                panels.remote = false;
            }
            Self::Server => {
                panels.cpu = true;
                panels.memory = true;
                panels.disk = true;
                panels.network = true;
                panels.processes = true;
                panels.docker = true;
                panels.gpu = false;
                panels.k8s = true;
                panels.sensors = false;
                panels.remote = true;
            }
            Self::Laptop => {
                panels.cpu = true;
                panels.memory = true;
                panels.disk = true;
                panels.network = true;
                panels.processes = true;
                panels.docker = false;
                panels.gpu = false;
                panels.k8s = false;
                panels.sensors = true;
                panels.remote = false;
            }
            Self::GpuFocus => {
                panels.cpu = true;
                panels.memory = true;
                panels.disk = false;
                panels.network = false;
                panels.processes = true;
                panels.docker = false;
                panels.gpu = true;
                panels.k8s = false;
                panels.sensors = true;
                panels.remote = false;
            }
        }
    }
}

/// Configuration for a single SSH remote machine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteConfig {
    /// Display name for this remote (e.g., "prod-web-1")
    pub name: String,
    /// Hostname or IP address
    pub host: String,
    /// SSH port (default: 22)
    #[serde(default = "default_ssh_port")]
    pub port: u16,
    /// SSH username
    pub user: String,
    /// Path to private key file (optional — falls back to SSH agent)
    #[serde(default)]
    pub key: Option<String>,
    /// Enable SSH agent forwarding
    #[serde(default)]
    pub agent_forwarding: bool,
}

fn default_ssh_port() -> u16 {
    22
}

/// Log output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    #[default]
    Json,
    Csv,
}

/// Log rotation strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRotation {
    /// Rotation mode: "size" or "time"
    #[serde(default = "default_rotation_mode")]
    pub mode: String,
    /// Max file size in bytes before rotation (default: 50 MiB)
    #[serde(default = "default_max_size")]
    pub max_size_bytes: u64,
    /// Max age in seconds before rotation (default: 86400 = 1 day)
    #[serde(default = "default_max_age")]
    pub max_age_secs: u64,
    /// Max number of rotated files to keep (default: 10)
    #[serde(default = "default_max_files")]
    pub max_files: usize,
}

impl Default for LogRotation {
    fn default() -> Self {
        Self {
            mode: default_rotation_mode(),
            max_size_bytes: default_max_size(),
            max_age_secs: default_max_age(),
            max_files: default_max_files(),
        }
    }
}

fn default_rotation_mode() -> String {
    "size".to_string()
}
fn default_max_size() -> u64 {
    50 * 1024 * 1024 // 50 MiB
}
fn default_max_age() -> u64 {
    86400 // 1 day
}
fn default_max_files() -> usize {
    10
}

/// Configuration for file-based metrics logging.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Whether logging is enabled (default: false)
    #[serde(default)]
    pub enabled: bool,
    /// Output format: "json" or "csv"
    #[serde(default)]
    pub format: LogFormat,
    /// Log directory path (default: platform data dir / kite / logs)
    #[serde(default)]
    pub path: Option<String>,
    /// Log rotation settings
    #[serde(default)]
    pub rotation: LogRotation,
    /// Compress rotated log files with gzip
    #[serde(default)]
    pub compress: bool,
    /// Which metrics to log (empty = all). Values: "cpu", "memory", "disk", "network"
    #[serde(default)]
    pub metrics: Vec<String>,
}

/// Configuration for the Prometheus metrics exporter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrometheusConfig {
    /// Whether the Prometheus exporter is enabled (default: false)
    #[serde(default)]
    pub enabled: bool,
    /// Port for the HTTP server (default: 9898)
    #[serde(default = "default_prometheus_port")]
    pub port: u16,
    /// Bind address (default: "127.0.0.1")
    #[serde(default = "default_bind_address")]
    pub bind_address: String,
    /// Optional bearer token for authentication
    #[serde(default)]
    pub auth_token: Option<String>,
}

impl Default for PrometheusConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            port: default_prometheus_port(),
            bind_address: default_bind_address(),
            auth_token: None,
        }
    }
}

fn default_prometheus_port() -> u16 {
    9898
}

fn default_bind_address() -> String {
    "127.0.0.1".to_string()
}

/// Top-level application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_update_interval")]
    pub update_interval_ms: u64,

    #[serde(default)]
    pub graph_symbols: GraphSymbols,

    #[serde(default)]
    pub color_mode: ColorMode,

    #[serde(default = "default_graph_history")]
    pub graph_history_depth: usize,

    #[serde(default)]
    pub panels: PanelVisibility,

    #[serde(default)]
    pub keybindings: HashMap<String, String>,

    #[serde(default = "default_theme")]
    pub theme: String,

    #[serde(default)]
    pub layout: LayoutPreset,

    #[serde(default)]
    pub alerts: Vec<AlertRule>,

    #[serde(default)]
    pub remotes: Vec<RemoteConfig>,

    #[serde(default)]
    pub logging: LoggingConfig,

    #[serde(default)]
    pub prometheus: PrometheusConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            update_interval_ms: default_update_interval(),
            graph_symbols: GraphSymbols::default(),
            color_mode: ColorMode::default(),
            graph_history_depth: default_graph_history(),
            panels: PanelVisibility::default(),
            keybindings: HashMap::new(),
            theme: default_theme(),
            layout: LayoutPreset::default(),
            alerts: Vec::new(),
            remotes: Vec::new(),
            logging: LoggingConfig::default(),
            prometheus: PrometheusConfig::default(),
        }
    }
}

impl Config {
    /// Sensible default alert rules used when no custom alerts are configured.
    pub fn default_alert_rules() -> Vec<AlertRule> {
        vec![
            AlertRule {
                name: "High CPU".to_string(),
                metric: Metric::CpuTotal,
                condition: Condition::Above,
                threshold: 90.0,
                duration_ticks: 5,
                severity: Severity::Warning,
                enabled: true,
            },
            AlertRule {
                name: "Critical CPU".to_string(),
                metric: Metric::CpuTotal,
                condition: Condition::Above,
                threshold: 95.0,
                duration_ticks: 3,
                severity: Severity::Critical,
                enabled: true,
            },
            AlertRule {
                name: "High Memory".to_string(),
                metric: Metric::MemoryPercent,
                condition: Condition::Above,
                threshold: 90.0,
                duration_ticks: 5,
                severity: Severity::Warning,
                enabled: true,
            },
        ]
    }
}

fn default_update_interval() -> u64 {
    1000
}

fn default_graph_history() -> usize {
    300
}

fn default_theme() -> String {
    "default".to_string()
}

fn bool_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let config = Config::default();
        assert_eq!(config.update_interval_ms, 1000);
        assert_eq!(config.graph_symbols, GraphSymbols::Braille);
        assert_eq!(config.color_mode, ColorMode::Auto);
        assert_eq!(config.graph_history_depth, 300);
        assert!(config.panels.cpu);
        assert!(config.panels.processes);
        assert_eq!(config.theme, "default");
        assert_eq!(config.layout, LayoutPreset::Default);
    }

    #[test]
    fn deserialize_partial_toml() {
        let toml_str = r#"
            update_interval_ms = 500
            graph_symbols = "block"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.update_interval_ms, 500);
        assert_eq!(config.graph_symbols, GraphSymbols::Block);
        // Defaults should fill in the rest
        assert_eq!(config.color_mode, ColorMode::Auto);
        assert!(config.panels.cpu);
    }

    #[test]
    fn serialize_roundtrip() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.update_interval_ms, parsed.update_interval_ms);
    }

    #[test]
    fn deserialize_with_remotes() {
        let toml_str = r#"
            update_interval_ms = 1000

            [[remotes]]
            name = "prod-web-1"
            host = "10.0.1.5"
            port = 22
            user = "monitor"
            key = "~/.ssh/id_ed25519"

            [[remotes]]
            name = "prod-db-1"
            host = "10.0.1.10"
            user = "monitor"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.remotes.len(), 2);
        assert_eq!(config.remotes[0].name, "prod-web-1");
        assert_eq!(config.remotes[0].host, "10.0.1.5");
        assert_eq!(config.remotes[0].port, 22);
        assert_eq!(config.remotes[0].user, "monitor");
        assert_eq!(config.remotes[0].key, Some("~/.ssh/id_ed25519".to_string()));
        assert!(!config.remotes[0].agent_forwarding);

        assert_eq!(config.remotes[1].name, "prod-db-1");
        assert_eq!(config.remotes[1].port, 22); // default
        assert!(config.remotes[1].key.is_none());
    }

    #[test]
    fn default_config_has_no_remotes() {
        let config = Config::default();
        assert!(config.remotes.is_empty());
        assert!(!config.panels.remote);
    }
}
