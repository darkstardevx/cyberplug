use anyhow::Context;
use std::path::PathBuf;
use std::process::Command;

fn plugin_dir(id: &str) -> anyhow::Result<PathBuf> {
    let home = dirs::home_dir().context("could not determine home directory")?;
    Ok(home.join(".config/omarchy/plugins").join(id))
}

/// Reads the real `origin` remote URL for an installed plugin's git clone.
/// Returns None for first-party plugins (no separate repo) or anything
/// that isn't a git checkout — never guesses or fabricates a URL.
pub fn remote_url(id: &str) -> Option<String> {
    let dir = plugin_dir(id).ok()?;
    if !dir.join(".git").exists() {
        return None;
    }
    let output = Command::new("git")
        .args(["-C", dir.to_str()?, "remote", "get-url", "origin"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if url.is_empty() { None } else { Some(url) }
}
