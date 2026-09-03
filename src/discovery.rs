use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

const REGISTRY_URL: &str =
    "https://raw.githubusercontent.com/omacom/omarchy-plugin-marketplace/main/registry.json";
const CACHE_MAX_AGE_SECS: u64 = 3600; // 1 hour

#[derive(Debug, Deserialize, Clone)]
pub struct CatalogEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub kind: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Source {
    pub repo: String,
    pub catalog: CatalogEntry,
}

#[derive(Debug, Deserialize)]
struct Registry {
    sources: Vec<Source>,
}

fn cache_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("could not determine home directory")?;
    let dir = home.join(".cache/cyberplug");
    fs::create_dir_all(&dir)?;
    Ok(dir.join("registry.json"))
}

fn cache_is_fresh(path: &PathBuf) -> bool {
    let Ok(meta) = fs::metadata(path) else {
        return false;
    };
    let Ok(modified) = meta.modified() else {
        return false;
    };
    let age = SystemTime::now()
        .duration_since(modified)
        .unwrap_or(Duration::MAX);
    age.as_secs() < CACHE_MAX_AGE_SECS
}

/// Fetches the community registry, using a local cache when fresh.
/// Falls back to a stale cache on network failure rather than erroring
/// the whole app; returns an empty list only if there's truly nothing.
pub fn fetch(force_refresh: bool) -> Result<Vec<Source>> {
    let path = cache_path()?;

    if !force_refresh && cache_is_fresh(&path) {
        if let Ok(raw) = fs::read_to_string(&path) {
            if let Ok(reg) = serde_json::from_str::<Registry>(&raw) {
                return Ok(reg.sources);
            }
        }
    }

    match reqwest::blocking::get(REGISTRY_URL).and_then(|r| r.text()) {
        Ok(raw) => {
            let reg: Registry =
                serde_json::from_str(&raw).context("failed to parse registry.json")?;
            let _ = fs::write(&path, &raw);
            Ok(reg.sources)
        }
        Err(_) => {
            if let Ok(raw) = fs::read_to_string(&path) {
                if let Ok(reg) = serde_json::from_str::<Registry>(&raw) {
                    return Ok(reg.sources);
                }
            }
            Ok(vec![])
        }
    }
}
