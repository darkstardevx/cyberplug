use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct LocalSettings {
    pub values: HashMap<String, HashMap<String, String>>,
}

fn config_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("could not determine home directory")?;
    let dir = home.join(".config/cyberplug");
    fs::create_dir_all(&dir)?;
    Ok(dir.join("settings.json"))
}

impl LocalSettings {
    pub fn load() -> Result<Self> {
        let path = config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&raw).unwrap_or_default())
    }

    pub fn save(&self) -> Result<()> {
        let path = config_path()?;
        let raw = serde_json::to_string_pretty(self)?;
        fs::write(path, raw)?;
        Ok(())
    }

    pub fn set(&mut self, plugin_id: &str, key: &str, value: String) {
        self.values
            .entry(plugin_id.to_string())
            .or_default()
            .insert(key.to_string(), value);
    }

    pub fn get(&self, plugin_id: &str, key: &str) -> Option<&String> {
        self.values.get(plugin_id)?.get(key)
    }
}
