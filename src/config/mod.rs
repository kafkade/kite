pub mod keybindings;
pub mod settings;

use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, Result};

use settings::Config;

/// Resolve the config file path using platform conventions.
pub fn config_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("kite");
    config_dir.join("config.toml")
}

/// Load configuration from the default file path.
/// Returns default config if the file doesn't exist.
pub fn load() -> Result<Config> {
    let path = config_path();

    if !path.exists() {
        return Ok(Config::default());
    }

    let contents =
        std::fs::read_to_string(&path).with_context(|| format!("reading config: {}", path.display()))?;

    let config: Config =
        toml::from_str(&contents).with_context(|| format!("parsing config: {}", path.display()))?;

    Ok(config)
}

/// Apply CLI overrides onto a loaded config.
pub fn apply_cli_overrides(config: &mut Config, interval: Option<u64>) {
    if let Some(ms) = interval {
        config.update_interval_ms = ms.max(100); // enforce minimum
    }
}

/// Generate a default config file and write it to the standard path.
pub fn generate_default() -> Result<PathBuf> {
    let path = config_path();

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating config directory: {}", parent.display()))?;
    }

    let config = Config::default();
    let toml_str = toml::to_string_pretty(&config).context("serializing default config")?;

    let mut file = std::fs::File::create(&path)
        .with_context(|| format!("creating config file: {}", path.display()))?;
    file.write_all(toml_str.as_bytes())?;

    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_path_is_reasonable() {
        let path = config_path();
        assert!(path.ends_with("kite/config.toml") || path.ends_with("kite\\config.toml"));
    }

    #[test]
    fn apply_overrides() {
        let mut config = Config::default();
        apply_cli_overrides(&mut config, Some(500));
        assert_eq!(config.update_interval_ms, 500);
    }

    #[test]
    fn apply_overrides_enforces_minimum() {
        let mut config = Config::default();
        apply_cli_overrides(&mut config, Some(50));
        assert_eq!(config.update_interval_ms, 100);
    }

    #[test]
    fn load_missing_file_returns_default() {
        // config_path() may not exist in test environments — should return default
        let config = load().unwrap();
        assert_eq!(config.update_interval_ms, 1000);
    }
}
