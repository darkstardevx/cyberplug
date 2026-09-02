use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
struct ShellConfig {
    #[serde(default, rename = "disabledPlugins")]
    disabled_plugins: Vec<String>,
}

/// Reads ~/.config/omarchy/shell.json and returns the list of disabled plugin ids.
pub fn disabled_plugins() -> Result<Vec<String>> {
    let home = dirs::home_dir().context("could not determine home directory")?;
    let path = home.join(".config/omarchy/shell.json");

    let raw = fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;

    let config: ShellConfig = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse {}", path.display()))?;

    Ok(config.disabled_plugins)
}
