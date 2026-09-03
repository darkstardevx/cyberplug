use crate::app::PluginEntry;
use crate::git;
use crate::omarchy;
use crate::settings::LocalSettings;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileEntry {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub repo: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    pub version: u32,
    pub plugins: Vec<ProfileEntry>,
    pub settings: HashMap<String, HashMap<String, String>>,
}

pub fn export(plugins: &[PluginEntry], local_settings: &LocalSettings, path: &Path) -> Result<()> {
    let entries = plugins
        .iter()
        .map(|e| ProfileEntry {
            id: e.plugin.id.clone(),
            name: e.plugin.name.clone(),
            repo: git::remote_url(&e.plugin.id),
            enabled: e.enabled,
        })
        .collect();

    let profile = Profile {
        version: 1,
        plugins: entries,
        settings: local_settings.values.clone(),
    };

    let raw = serde_json::to_string_pretty(&profile)?;
    fs::write(path, raw).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

pub struct ImportSummary {
    pub installed: usize,
    pub enabled: usize,
    pub disabled: usize,
    pub settings_applied: usize,
    pub errors: Vec<String>,
}

pub fn import(path: &Path, currently_installed: &[String]) -> Result<ImportSummary> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let profile: Profile = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse {}", path.display()))?;

    let mut summary = ImportSummary {
        installed: 0,
        enabled: 0,
        disabled: 0,
        settings_applied: 0,
        errors: vec![],
    };

    for entry in &profile.plugins {
        let already_installed = currently_installed.contains(&entry.id);

        if !already_installed {
            if let Some(repo) = &entry.repo {
                match omarchy::add(repo, false) {
                    Ok(_) => summary.installed += 1,
                    Err(e) => {
                        summary
                            .errors
                            .push(format!("{}: install failed: {}", entry.id, e));
                        continue;
                    }
                }
            } else {
                // First-party plugin missing on this system — nothing to install, skip.
                summary.errors.push(format!(
                    "{}: not present and no repo to install from",
                    entry.id
                ));
                continue;
            }
        }

        let result = if entry.enabled {
            omarchy::enable(&entry.id, None)
        } else {
            omarchy::disable(&entry.id)
        };

        match result {
            Ok(_) => {
                if entry.enabled {
                    summary.enabled += 1;
                } else {
                    summary.disabled += 1;
                }
            }
            Err(e) => summary.errors.push(format!("{}: {}", entry.id, e)),
        }
    }

    if !profile.settings.is_empty() {
        let mut local = LocalSettings::load().unwrap_or_default();
        for (plugin_id, fields) in &profile.settings {
            for (key, value) in fields {
                local.set(plugin_id, key, value.clone());
                summary.settings_applied += 1;
            }
        }
        local.save()?;
    }

    Ok(summary)
}
