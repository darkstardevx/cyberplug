mod app;
mod config;
mod discovery;
mod git;
mod models;
mod omarchy;
mod profile;
mod settings;
mod ui;

use anyhow::Result;
use app::Mode;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use std::path::PathBuf;

fn default_profile_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".config/cyberplug/profile.json")
}

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = app::App::new()?;
    let result = run(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut app::App) -> Result<()> {
    loop {
        terminal.draw(|frame| ui::draw(frame, app))?;

        if let Event::Key(key) = event::read()? {
            match app.mode {
                Mode::Normal => handle_normal(app, key.code)?,
                Mode::Filter => handle_filter(app, key.code),
                Mode::AddUrl => handle_add_url(app, key.code)?,
                Mode::ConfirmRemove => handle_confirm_remove(app, key.code)?,
                Mode::Settings => handle_settings(app, key.code)?,
                Mode::Placement => handle_placement(app, key.code)?,
                Mode::Discovery => handle_discovery(app, key.code)?,
                Mode::Profile => handle_profile(app, key.code),
                Mode::ProfilePath => handle_profile_path(app, key.code)?,
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn handle_normal(app: &mut app::App, code: KeyCode) -> Result<()> {
    match code {
        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
        KeyCode::Char('j') | KeyCode::Down => app.next(),
        KeyCode::Char('k') | KeyCode::Up => app.previous(),
        KeyCode::Char('/') => app.mode = Mode::Filter,
        KeyCode::Char('a') => {
            app.mode = Mode::AddUrl;
            app.input_buffer.clear();
        }
        KeyCode::Char('e') => {
            if app.selected_id().is_some() {
                app.mode = Mode::Placement;
                app.placement_selected = 1;
            }
        }
        KeyCode::Char('d') => {
            if let Some(id) = app.selected_id() {
                app.status = Some(match omarchy::disable(&id) {
                    Ok(_) => format!("disabled {}", id),
                    Err(e) => format!("error: {}", e),
                });
                app.refresh()?;
            }
        }
        KeyCode::Char('x') => {
            if app.selected_id().is_some() {
                app.mode = Mode::ConfirmRemove;
            }
        }
        KeyCode::Char('s') => {
            if let Some(entry) = app.selected_entry()
                && !entry.plugin.schema().is_empty() {
                    app.mode = Mode::Settings;
                    app.settings_selected = 0;
                    app.settings_editing = false;
                }
        }
        KeyCode::Char('u') => {
            if let Some(id) = app.selected_id() {
                app.status = Some(match omarchy::update(Some(&id)) {
                    Ok(_) => format!("updated {}", id),
                    Err(e) => format!("error: {}", e),
                });
                app.refresh()?;
            }
        }
        KeyCode::Char('U') => {
            app.status = Some(match omarchy::update(None) {
                Ok(_) => "updated all plugins".to_string(),
                Err(e) => format!("error: {}", e),
            });
            app.refresh()?;
        }
        KeyCode::Char('D') => {
            app.mode = Mode::Discovery;
            app.status = Some("loading registry...".to_string());
            app.load_discovery(false)?;
            app.status = Some(format!("{} plugins available", app.discovery_sources.len()));
        }
        KeyCode::Char('P') => {
            app.mode = Mode::Profile;
        }
        _ => {}
    }
    Ok(())
}

fn handle_filter(app: &mut app::App, code: KeyCode) {
    match code {
        KeyCode::Esc => {
            app.filter_text.clear();
            app.apply_filter();
            app.mode = Mode::Normal;
        }
        KeyCode::Enter => app.mode = Mode::Normal,
        KeyCode::Backspace => {
            app.filter_text.pop();
            app.apply_filter();
        }
        KeyCode::Char(c) => {
            app.filter_text.push(c);
            app.apply_filter();
        }
        _ => {}
    }
}

fn handle_add_url(app: &mut app::App, code: KeyCode) -> Result<()> {
    match code {
        KeyCode::Esc => {
            app.input_buffer.clear();
            app.mode = Mode::Normal;
        }
        KeyCode::Enter => {
            let url = app.input_buffer.trim().to_string();
            if !url.is_empty() {
                app.status = Some(match omarchy::add(&url, false) {
                    Ok(_) => format!("added {}", url),
                    Err(e) => format!("error: {}", e),
                });
                app.refresh()?;
            }
            app.input_buffer.clear();
            app.mode = Mode::Normal;
        }
        KeyCode::Backspace => {
            app.input_buffer.pop();
        }
        KeyCode::Char(c) => app.input_buffer.push(c),
        _ => {}
    }
    Ok(())
}

fn handle_confirm_remove(app: &mut app::App, code: KeyCode) -> Result<()> {
    match code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            if let Some(id) = app.selected_id() {
                app.status = Some(match omarchy::remove(&id) {
                    Ok(_) => format!("removed {}", id),
                    Err(e) => format!("error: {}", e),
                });
                app.refresh()?;
            }
            app.mode = Mode::Normal;
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => app.mode = Mode::Normal,
        _ => {}
    }
    Ok(())
}

fn handle_settings(app: &mut app::App, code: KeyCode) -> Result<()> {
    let Some(entry) = app.selected_entry() else {
        app.mode = Mode::Normal;
        return Ok(());
    };
    let schema_len = entry.plugin.schema().len();
    let plugin_id = entry.plugin.id.clone();

    if app.settings_editing {
        match code {
            KeyCode::Esc => {
                app.settings_editing = false;
                app.settings_edit_buffer.clear();
            }
            KeyCode::Enter => {
                if let Some(field) = entry.plugin.schema().get(app.settings_selected) {
                    let key = field.key.clone();
                    let mut value = app.settings_edit_buffer.trim().to_string();

                    if field.field_type == "integer"
                        && let Ok(mut n) = value.parse::<f64>() {
                            if let Some(min) = field.min {
                                n = n.max(min);
                            }
                            if let Some(max) = field.max {
                                n = n.min(max);
                            }
                            if let Some(step) = field.step
                                && step > 0.0 {
                                    let base = field.min.unwrap_or(0.0);
                                    n = base + ((n - base) / step).round() * step;
                                }
                            value = (n as i64).to_string();
                        }

                    app.local_settings.set(&plugin_id, &key, value.clone());
                    app.local_settings.save()?;
                    app.status = Some(format!("staged {}={} for {}", key, value, plugin_id));
                }
                app.settings_editing = false;
                app.settings_edit_buffer.clear();
            }
            KeyCode::Backspace => {
                app.settings_edit_buffer.pop();
            }
            KeyCode::Char(c) => app.settings_edit_buffer.push(c),
            _ => {}
        }
        return Ok(());
    }

    match code {
        KeyCode::Esc => app.mode = Mode::Normal,
        KeyCode::Char('j') | KeyCode::Down => {
            if schema_len > 0 {
                app.settings_selected = (app.settings_selected + 1) % schema_len;
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if schema_len > 0 {
                app.settings_selected = if app.settings_selected == 0 {
                    schema_len - 1
                } else {
                    app.settings_selected - 1
                };
            }
        }
        KeyCode::Enter => {
            if let Some(field) = entry.plugin.schema().get(app.settings_selected) {
                let current = app
                    .local_settings
                    .get(&plugin_id, &field.key)
                    .cloned()
                    .unwrap_or_else(|| {
                        field
                            .default_value
                            .as_ref()
                            .map(|v| v.to_string().trim_matches('"').to_string())
                            .unwrap_or_default()
                    });
                app.settings_edit_buffer = current;
                app.settings_editing = true;
            }
        }
        _ => {}
    }
    Ok(())
}

fn handle_placement(app: &mut app::App, code: KeyCode) -> Result<()> {
    use app::PLACEMENTS;
    match code {
        KeyCode::Esc => app.mode = Mode::Normal,
        KeyCode::Char('h') | KeyCode::Left | KeyCode::Char('k') | KeyCode::Up => {
            app.placement_selected = if app.placement_selected == 0 {
                PLACEMENTS.len() - 1
            } else {
                app.placement_selected - 1
            };
        }
        KeyCode::Char('l') | KeyCode::Right | KeyCode::Char('j') | KeyCode::Down => {
            app.placement_selected = (app.placement_selected + 1) % PLACEMENTS.len();
        }
        KeyCode::Enter => {
            if let Some(id) = app.selected_id() {
                let placement = PLACEMENTS[app.placement_selected];
                app.status = Some(match omarchy::enable(&id, Some(placement)) {
                    Ok(_) => format!("enabled {} at {}", id, placement),
                    Err(e) => format!("error: {}", e),
                });
                app.refresh()?;
            }
            app.mode = Mode::Normal;
        }
        _ => {}
    }
    Ok(())
}

fn handle_discovery(app: &mut app::App, code: KeyCode) -> Result<()> {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.mode = Mode::Normal,
        KeyCode::Char('j') | KeyCode::Down => app.discovery_next(),
        KeyCode::Char('k') | KeyCode::Up => app.discovery_previous(),
        KeyCode::Char('h') | KeyCode::Left => {
            let total = app.discovery_categories.len() + 1;
            app.discovery_category_index = (app.discovery_category_index + total - 1) % total;
            app.apply_discovery_filter();
        }
        KeyCode::Char('l') | KeyCode::Right => {
            app.discovery_cycle_category();
        }
        KeyCode::Char('r') => {
            app.status = Some("refreshing registry...".to_string());
            app.load_discovery(true)?;
            app.status = Some(format!("{} plugins available", app.discovery_sources.len()));
        }
        KeyCode::Enter => {
            if let Some(source) = app.discovery_sources.get(app.discovery_selected).cloned() {
                app.status = Some(format!("installing {}...", source.name));
                match omarchy::add(&source.repo, false) {
                    Ok(_) => {
                        app.refresh()?;
                        // Only route into placement if the plugin actually
                        // registered as a bar widget under this id — a repo
                        // can install without exposing that exact id (e.g.
                        // suite repos where the id we showed doesn't map
                        // 1:1 to what got installed).
                        if app.plugins.iter().any(|e| e.plugin.id == source.id) {
                            app.selected = app
                                .filtered
                                .iter()
                                .position(|&i| app.plugins[i].plugin.id == source.id)
                                .unwrap_or(0);
                            app.mode = Mode::Placement;
                            app.placement_selected = 1;
                            app.status =
                                Some(format!("{} installed — choose bar placement", source.name));
                        } else {
                            app.status = Some(format!(
                                "{} installed but not found as '{}' — check `omarchy plugin list` and enable manually",
                                source.name, source.id
                            ));
                        }
                        app.load_discovery(false)?;
                    }
                    Err(e) => {
                        app.status = Some(format!("error: {}", e));
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn handle_profile(app: &mut app::App, code: KeyCode) {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => app.mode = Mode::Normal,
        KeyCode::Char('e') => {
            app.profile_exporting = true;
            app.input_buffer = default_profile_path().to_string_lossy().to_string();
            app.mode = Mode::ProfilePath;
        }
        KeyCode::Char('i') => {
            app.profile_exporting = false;
            app.input_buffer = default_profile_path().to_string_lossy().to_string();
            app.mode = Mode::ProfilePath;
        }
        _ => {}
    }
}

fn handle_profile_path(app: &mut app::App, code: KeyCode) -> Result<()> {
    match code {
        KeyCode::Esc => {
            app.input_buffer.clear();
            app.mode = Mode::Normal;
        }
        KeyCode::Enter => {
            let path = PathBuf::from(app.input_buffer.trim());
            if app.profile_exporting {
                if let Some(parent) = path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                app.status = Some(
                    match profile::export(&app.plugins, &app.local_settings, &path) {
                        Ok(_) => format!("exported profile to {}", path.display()),
                        Err(e) => format!("error: {}", e),
                    },
                );
            } else {
                let installed: Vec<String> =
                    app.plugins.iter().map(|e| e.plugin.id.clone()).collect();
                match profile::import(&path, &installed) {
                    Ok(summary) => {
                        app.status = Some(format!(
                            "imported: {} installed, {} enabled, {} disabled, {} settings ({} errors)",
                            summary.installed,
                            summary.enabled,
                            summary.disabled,
                            summary.settings_applied,
                            summary.errors.len()
                        ));
                        app.refresh()?;
                    }
                    Err(e) => app.status = Some(format!("error: {}", e)),
                }
            }
            app.input_buffer.clear();
            app.mode = Mode::Normal;
        }
        KeyCode::Backspace => {
            app.input_buffer.pop();
        }
        KeyCode::Char(c) => app.input_buffer.push(c),
        _ => {}
    }
    Ok(())
}
