use crate::models::Plugin;
use anyhow::{Context, Result, bail};
use std::process::Command;

fn run(args: &[&str]) -> Result<String> {
    let output = Command::new("omarchy")
        .args(args)
        .output()
        .with_context(|| format!("failed to execute `omarchy {}`", args.join(" ")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("`omarchy {}` failed: {}", args.join(" "), stderr.trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn catalog() -> Result<Vec<Plugin>> {
    let raw = run(&["plugin", "catalog"])?;
    serde_json::from_str(&raw).context("failed to parse `omarchy plugin catalog` output")
}

pub fn enable(id: &str, placement: Option<&str>) -> Result<()> {
    match placement {
        Some(p) => run(&["plugin", "enable", id, p])?,
        None => run(&["plugin", "enable", id])?,
    };
    Ok(())
}

pub fn disable(id: &str) -> Result<()> {
    run(&["plugin", "disable", id])?;
    Ok(())
}

pub fn add(git_url: &str, enable: bool) -> Result<()> {
    let mut args = vec!["plugin", "add", git_url, "--yes"];
    if enable {
        args.push("--enable");
    }
    run(&args)?;
    Ok(())
}

pub fn remove(id: &str) -> Result<()> {
    run(&["plugin", "remove", id, "--yes"])?;
    Ok(())
}

pub fn update(id: Option<&str>) -> Result<()> {
    match id {
        Some(id) => run(&["plugin", "update", id, "--yes"])?,
        None => run(&["plugin", "update", "--yes"])?,
    };
    Ok(())
}
