use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
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
struct PluginStub {
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct RawSource {
    repo: String,
    #[serde(default)]
    catalog: Option<CatalogEntry>,
    #[serde(default)]
    plugins: Option<HashMap<String, PluginStub>>,
}

#[derive(Debug, Deserialize)]
struct Registry {
    sources: Vec<RawSource>,
}

#[derive(Debug, Clone)]
pub struct Source {
    pub repo: String,
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub version: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
}

fn display_name(id: &str) -> String {
    let last_segment = id.rsplit('.').next().unwrap_or(id);
    last_segment
        .split(|c| c == '-' || c == '_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn author_from_id(id: &str) -> Option<String> {
    let parts: Vec<&str> = id.split('.').collect();
    if parts.len() < 2 {
        return None;
    }
    if parts[0] == "io" && parts.get(1) == Some(&"github") && parts.len() >= 3 {
        return Some(parts[2].to_string());
    }
    if parts.len() >= 2 {
        return Some(parts[0].to_string());
    }
    None
}

fn flatten(raw: Vec<RawSource>) -> Vec<Source> {
    let mut out = Vec::new();
    for src in raw {
        if let Some(cat) = &src.catalog {
            out.push(Source {
                repo: src.repo.clone(),
                id: cat.id.clone(),
                name: cat.name.clone(),
                description: Some(cat.description.clone()),
                author: cat.author.clone(),
                version: cat.version.clone(),
                category: cat.category.clone(),
                tags: cat.tags.clone(),
            });
        }
        if let Some(plugins) = &src.plugins {
            for (key, stub) in plugins {
                out.push(Source {
                    repo: src.repo.clone(),
                    id: key.clone(),
                    name: display_name(key),
                    description: None,
                    author: author_from_id(key),
                    version: None,
                    category: stub.category.clone(),
                    tags: stub.tags.clone(),
                });
            }
        }
    }
    out
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

pub fn fetch(force_refresh: bool) -> Result<Vec<Source>> {
    let path = cache_path()?;

    if !force_refresh && cache_is_fresh(&path) {
        if let Ok(raw) = fs::read_to_string(&path) {
            if let Ok(reg) = serde_json::from_str::<Registry>(&raw) {
                return Ok(flatten(reg.sources));
            }
        }
    }

    match reqwest::blocking::get(REGISTRY_URL).and_then(|r| r.text()) {
        Ok(raw) => {
            let reg: Registry =
                serde_json::from_str(&raw).context("failed to parse registry.json")?;
            let _ = fs::write(&path, &raw);
            Ok(flatten(reg.sources))
        }
        Err(_) => {
            if let Ok(raw) = fs::read_to_string(&path) {
                if let Ok(reg) = serde_json::from_str::<Registry>(&raw) {
                    return Ok(flatten(reg.sources));
                }
            }
            Ok(vec![])
        }
    }
}

/// Best-effort fetch of a repo's own GitHub description (the tagline on
/// the repo page, not the plugin manifest) for entries the registry
/// doesn't carry a description for. Returns None on any failure —
/// network, rate limit, or a repo URL that doesn't parse — never errors
/// the UI over a missing description.
pub fn fetch_repo_description(repo_url: &str) -> Option<String> {
    let trimmed = repo_url.trim_end_matches('/');
    let parts: Vec<&str> = trimmed.rsplitn(3, '/').collect();
    if parts.len() < 2 {
        return None;
    }
    let repo = parts[0];
    let owner = parts[1];
    let api_url = format!("https://api.github.com/repos/{}/{}", owner, repo);

    let client = reqwest::blocking::Client::builder()
        .user_agent("cyberplug")
        .timeout(Duration::from_secs(5))
        .build()
        .ok()?;
    let resp = client.get(&api_url).send().ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let json: serde_json::Value = resp.json().ok()?;
    json.get("description")
        .and_then(|d| d.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}
