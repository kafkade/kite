use std::collections::HashMap;

use serde::{Deserialize, Serialize};

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
            }
        }
    }
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
        }
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
}
