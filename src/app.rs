use crate::config;
use crate::discovery;
use crate::models::Plugin;
use crate::omarchy;
use crate::settings;
use anyhow::Result;
use std::collections::HashSet;

pub const PLACEMENTS: [&str; 3] = ["left", "center", "right"];

pub struct PluginEntry {
    pub plugin: Plugin,
    pub enabled: bool,
}

#[derive(PartialEq)]
pub enum Mode {
    Normal,
    Filter,
    AddUrl,
    Settings,
    ConfirmRemove,
    Placement,
    Discovery,
}

pub struct App {
    pub plugins: Vec<PluginEntry>,
    pub filtered: Vec<usize>,
    pub selected: usize,
    pub should_quit: bool,
    pub mode: Mode,
    pub filter_text: String,
    pub input_buffer: String,
    pub status: Option<String>,
    pub settings_selected: usize,
    pub settings_editing: bool,
    pub settings_edit_buffer: String,
    pub local_settings: settings::LocalSettings,
    pub placement_selected: usize,
    pub discovery_sources: Vec<discovery::Source>,
    pub discovery_selected: usize,
}

impl App {
    pub fn new() -> Result<Self> {
        let plugins = load_plugins()?;
        let filtered = (0..plugins.len()).collect();
        let local_settings = settings::LocalSettings::load().unwrap_or_default();
        Ok(Self {
            plugins,
            filtered,
            selected: 0,
            should_quit: false,
            mode: Mode::Normal,
            filter_text: String::new(),
            input_buffer: String::new(),
            status: None,
            settings_selected: 0,
            settings_editing: false,
            settings_edit_buffer: String::new(),
            local_settings,
            placement_selected: 0,
            discovery_sources: vec![],
            discovery_selected: 0,
        })
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.plugins = load_plugins()?;
        self.apply_filter();
        Ok(())
    }

    pub fn apply_filter(&mut self) {
        let needle = self.filter_text.to_lowercase();
        self.filtered = self
            .plugins
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                needle.is_empty()
                    || e.plugin.name.to_lowercase().contains(&needle)
                    || e.plugin.id.to_lowercase().contains(&needle)
                    || e.plugin.description.to_lowercase().contains(&needle)
            })
            .map(|(i, _)| i)
            .collect();
        if self.selected >= self.filtered.len() {
            self.selected = self.filtered.len().saturating_sub(1);
        }
    }

    pub fn next(&mut self) {
        if !self.filtered.is_empty() {
            self.selected = (self.selected + 1) % self.filtered.len();
        }
    }

    pub fn previous(&mut self) {
        if !self.filtered.is_empty() {
            self.selected = if self.selected == 0 {
                self.filtered.len() - 1
            } else {
                self.selected - 1
            };
        }
    }

    pub fn selected_entry(&self) -> Option<&PluginEntry> {
        self.filtered
            .get(self.selected)
            .and_then(|&i| self.plugins.get(i))
    }

    pub fn selected_id(&self) -> Option<String> {
        self.selected_entry().map(|e| e.plugin.id.clone())
    }

    pub fn load_discovery(&mut self, force_refresh: bool) -> Result<()> {
        let installed: HashSet<&str> =
            self.plugins.iter().map(|e| e.plugin.id.as_str()).collect();
        let all = discovery::fetch(force_refresh)?;
        self.discovery_sources = all
            .into_iter()
            .filter(|s| !installed.contains(s.catalog.id.as_str()))
            .collect();
        self.discovery_selected = 0;
        Ok(())
    }

    pub fn discovery_next(&mut self) {
        if !self.discovery_sources.is_empty() {
            self.discovery_selected = (self.discovery_selected + 1) % self.discovery_sources.len();
        }
    }

    pub fn discovery_previous(&mut self) {
        if !self.discovery_sources.is_empty() {
            self.discovery_selected = if self.discovery_selected == 0 {
                self.discovery_sources.len() - 1
            } else {
                self.discovery_selected - 1
            };
        }
    }
}

fn load_plugins() -> Result<Vec<PluginEntry>> {
    let catalog = omarchy::catalog()?;
    let disabled = config::disabled_plugins().unwrap_or_default();

    Ok(catalog
        .into_iter()
        .map(|plugin| {
            let enabled = !disabled.contains(&plugin.id);
            PluginEntry { plugin, enabled }
        })
        .collect())
}
