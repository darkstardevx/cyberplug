mod app;
mod config;
mod models;
mod omarchy;
mod settings;
mod ui;

use anyhow::Result;
use app::Mode;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = app::App::new()?;
    let result = run(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
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
            if let Some(id) = app.selected_id() {
                app.status = Some(match omarchy::enable(&id, None) {
                    Ok(_) => format!("enabled {}", id),
                    Err(e) => format!("error: {}", e),
                });
                app.refresh()?;
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
            if let Some(entry) = app.selected_entry() {
                if !entry.plugin.schema().is_empty() {
                    app.mode = Mode::Settings;
                    app.settings_selected = 0;
                    app.settings_editing = false;
                }
            }
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
                    let value = app.settings_edit_buffer.trim().to_string();
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
