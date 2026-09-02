use crate::app::{App, Mode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(area);

    match app.mode {
        Mode::Settings => draw_settings(frame, app, chunks[0]),
        _ => draw_main(frame, app, chunks[0]),
    }

    draw_status_line(frame, app, chunks[1]);
}

fn draw_main(frame: &mut Frame, app: &App, area: Rect) {
    let has_input = matches!(app.mode, Mode::Filter | Mode::AddUrl | Mode::ConfirmRemove);

    let chunks = if has_input {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(3)])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3)])
            .split(area)
    };

    let list_index = if has_input {
        draw_input_line(frame, app, chunks[0]);
        1
    } else {
        0
    };

    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(chunks[list_index]);

    draw_list(frame, app, panes[0]);
    draw_detail(frame, app, panes[1]);
}

fn draw_input_line(frame: &mut Frame, app: &App, area: Rect) {
    let (title, content) = match app.mode {
        Mode::Filter => (" filter (esc to cancel) ", app.filter_text.as_str()),
        Mode::AddUrl => (
            " add plugin — git url (enter to confirm, esc to cancel) ",
            app.input_buffer.as_str(),
        ),
        Mode::ConfirmRemove => (" remove plugin? (y/n) ", ""),
        _ => ("", ""),
    };
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    frame.render_widget(Paragraph::new(content).block(block), area);
}

fn draw_list(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .filtered
        .iter()
        .map(|&i| {
            let entry = &app.plugins[i];
            let (dot, color) = if entry.enabled {
                ("●", Color::Green)
            } else {
                ("○", Color::DarkGray)
            };
            let (badge, badge_color) = match entry.plugin.first_party {
                Some(true) => ("core", Color::Cyan),
                _ => ("community", Color::Magenta),
            };
            let line = Line::from(vec![
                Span::styled(format!("{} ", dot), Style::default().fg(color)),
                Span::raw(entry.plugin.name.clone()),
                Span::raw("  "),
                Span::styled(format!("[{}]", badge), Style::default().fg(badge_color)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let mut state = ListState::default();
    if !app.filtered.is_empty() {
        state.select(Some(app.selected));
    }

    let title = format!(" cyberplug — {} plugins ", app.filtered.len());
    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(Color::DarkGray))
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_detail(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" details ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let text = if let Some(entry) = app.selected_entry() {
        let status = if entry.enabled { "enabled" } else { "disabled" };
        let mut lines = vec![
            Line::from(Span::styled(
                entry.plugin.name.clone(),
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(format!("id: {}", entry.plugin.id)),
            Line::from(format!("status: {}", status)),
        ];
        if let Some(v) = &entry.plugin.version {
            lines.push(Line::from(format!("version: {}", v)));
        }
        if let Some(a) = &entry.plugin.author {
            lines.push(Line::from(format!("author: {}", a)));
        }
        let schema = entry.plugin.schema();
        if !schema.is_empty() {
            lines.push(Line::from(format!(
                "settings: {} option(s) — press s to edit",
                schema.len()
            )));
        }
        lines.push(Line::from(""));
        lines.push(Line::from(entry.plugin.description.clone()));
        lines.push(Line::from(""));
        lines.push(Line::from(
            "j/k nav   / filter   a add   e enable   d disable   x remove   s settings   q quit",
        ));
        lines
    } else {
        vec![Line::from("No plugins match.")]
    };

    frame.render_widget(Paragraph::new(text).block(block).wrap(Wrap { trim: true }), area);
}

fn draw_settings(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" settings — esc back, enter to edit/save ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let Some(entry) = app.selected_entry() else {
        frame.render_widget(Paragraph::new("No plugin selected.").block(block), area);
        return;
    };

    let schema = entry.plugin.schema();
    if schema.is_empty() {
        frame.render_widget(
            Paragraph::new("This plugin has no configurable settings.").block(block),
            area,
        );
        return;
    }

    let items: Vec<ListItem> = schema
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let stored = app.local_settings.get(&entry.plugin.id, &field.key).cloned();
            let current = stored.unwrap_or_else(|| {
                field
                    .default_value
                    .as_ref()
                    .map(|v| v.to_string().trim_matches('"').to_string())
                    .unwrap_or_else(|| "-".to_string())
            });
            let editing = app.settings_editing && app.settings_selected == i;
            let value_display = if editing {
                format!("{}_", app.settings_edit_buffer)
            } else {
                current
            };
            let line = Line::from(vec![
                Span::styled(format!("{:<28}", field.label), Style::default().fg(Color::White)),
                Span::styled(value_display, Style::default().fg(Color::Green)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.settings_selected));

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(Color::DarkGray))
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area, &mut state);
}

fn draw_status_line(frame: &mut Frame, app: &App, area: Rect) {
    let text = app.status.clone().unwrap_or_default();
    frame.render_widget(Paragraph::new(text).style(Style::default().fg(Color::Yellow)), area);
}
