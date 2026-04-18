use serde::{Deserialize, Serialize};

/// Which metric to monitor.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Metric {
    CpuTotal,
    MemoryPercent,
    SwapPercent,
    DiskPercent,
    CpuTemperature,
    GpuTemperature,
    GpuUtilization,
}

/// Comparison condition.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Condition {
    Above,
    Below,
}

/// Alert severity level.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warning,
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Info => write!(f, "INFO"),
            Severity::Warning => write!(f, "WARN"),
            Severity::Critical => write!(f, "CRIT"),
        }
    }
}

/// A single alert rule from config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub name: String,
    pub metric: Metric,
    pub condition: Condition,
    pub threshold: f64,
    /// How many consecutive ticks the condition must hold before firing.
    #[serde(default = "default_duration_ticks")]
    pub duration_ticks: u32,
    #[serde(default = "default_severity")]
    pub severity: Severity,
    #[serde(default = "bool_true")]
    pub enabled: bool,
}

fn default_duration_ticks() -> u32 {
    3
}

fn default_severity() -> Severity {
    Severity::Warning
}

fn bool_true() -> bool {
    true
}
