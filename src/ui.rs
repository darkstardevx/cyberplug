use crate::app::{App, Mode, PLACEMENTS};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

fn category_color(category: &str) -> Color {
    match category {
        "Utilities" => Color::Yellow,
        "Status" => Color::Blue,
        "System" => Color::Red,
        "Info" => Color::Cyan,
        "Desktop" => Color::Magenta,
        _ => Color::White,
    }
}

pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(area);

    match app.mode {
        Mode::Settings => draw_settings(frame, app, chunks[0]),
        Mode::Placement => draw_placement(frame, app, chunks[0]),
        Mode::Discovery => draw_discovery(frame, app, chunks[0]),
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
            let category = entry
                .plugin
                .bar_widget
                .as_ref()
                .and_then(|bw| bw.category.clone())
                .unwrap_or_default();
            let cat_color = category_color(&category);

            let mut spans = vec![
                Span::styled(format!("{} ", dot), Style::default().fg(color)),
                Span::raw(entry.plugin.name.clone()),
                Span::raw("  "),
                Span::styled(format!("[{}]", badge), Style::default().fg(badge_color)),
            ];
            if !category.is_empty() {
                spans.push(Span::raw("  "));
                spans.push(Span::styled(category.clone(), Style::default().fg(cat_color)));
            }
            ListItem::new(Line::from(spans))
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
        let status_color = if entry.enabled { Color::Green } else { Color::DarkGray };
        let category = entry
            .plugin
            .bar_widget
            .as_ref()
            .and_then(|bw| bw.category.clone());

        let mut lines = vec![
            Line::from(Span::styled(
                entry.plugin.name.clone(),
                Style::default().add_modifier(Modifier::BOLD).fg(Color::White),
            )),
            Line::from(""),
            Line::from(vec![
                Span::raw("id: "),
                Span::styled(entry.plugin.id.clone(), Style::default().fg(Color::DarkGray)),
            ]),
            Line::from(vec![
                Span::raw("status: "),
                Span::styled(status, Style::default().fg(status_color)),
            ]),
        ];
        if let Some(cat) = category {
            let color = category_color(&cat);
            lines.push(Line::from(vec![
                Span::raw("category: "),
                Span::styled(cat, Style::default().fg(color)),
            ]));
        }
        if let Some(v) = &entry.plugin.version {
            lines.push(Line::from(format!("version: {}", v)));
        }
        if let Some(a) = &entry.plugin.author {
            lines.push(Line::from(format!("author: {}", a)));
        }
        if !entry.plugin.kinds.is_empty() {
            lines.push(Line::from(format!("kinds: {}", entry.plugin.kinds.join(", "))));
        }
        let schema = entry.plugin.schema();
        if !schema.is_empty() {
            lines.push(Line::from(Span::styled(
                format!("settings: {} option(s) — press s to edit", schema.len()),
                Style::default().fg(Color::Yellow),
            )));
        }
        lines.push(Line::from(""));
        lines.push(Line::from(entry.plugin.description.clone()));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "j/k nav   / filter   a add   e enable   d disable   x remove   s settings   D discover   u update   U update-all   q quit",
            Style::default().fg(Color::DarkGray),
        )));
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

    let bar_widget = entry.plugin.bar_widget.as_ref();
    let header_lines: Vec<Line> = {
        let mut lines = vec![];
        if let Some(bw) = bar_widget {
            if let Some(name) = &bw.display_name {
                lines.push(Line::from(Span::styled(
                    name.clone(),
                    Style::default().add_modifier(Modifier::BOLD),
                )));
            }
            if let Some(cat) = &bw.category {
                let color = category_color(cat);
                lines.push(Line::from(vec![
                    Span::raw("category: "),
                    Span::styled(cat.clone(), Style::default().fg(color)),
                ]));
            }
            if let Some(desc) = &bw.description {
                lines.push(Line::from(desc.clone()));
            }
        }
        lines
    };
    let header_height = header_lines.len() as u16 + if header_lines.is_empty() { 0 } else { 2 };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(header_height),
            Constraint::Length(schema.len() as u16 + 2),
            Constraint::Min(3),
        ])
        .split(area);

    if !header_lines.is_empty() {
        frame.render_widget(
            Paragraph::new(header_lines).wrap(Wrap { trim: true }),
            chunks[0],
        );
    }

    let items: Vec<ListItem> = schema
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let stored = app.local_settings.get(&entry.plugin.id, &field.key).cloned();
            let is_staged = stored.is_some();
            let current = stored.unwrap_or_else(|| {
                field
                    .default_value
                    .as_ref()
                    .map(|v| v.to_string().trim_matches('"').to_string())
                    .unwrap_or_else(|| "-".to_string())
            });
            let editing = app.settings_editing && app.settings_selected == i;
            let value_color = if editing {
                Color::Yellow
            } else if is_staged {
                Color::Green
            } else {
                Color::DarkGray
            };
            let value_display = if editing {
                format!("{}_", app.settings_edit_buffer)
            } else {
                current
            };

            let range_hint = match (field.min, field.max) {
                (Some(min), Some(max)) => format!("  ({}-{})", min, max),
                (Some(min), None) => format!("  (min {})", min),
                (None, Some(max)) => format!("  (max {})", max),
                _ => String::new(),
            };

            let line = Line::from(vec![
                Span::styled(format!("{:<28}", field.label), Style::default().fg(Color::White)),
                Span::styled(value_display, Style::default().fg(value_color)),
                Span::styled(range_hint, Style::default().fg(Color::DarkGray)),
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

    frame.render_stateful_widget(list, chunks[1], &mut state);

    let description = schema
        .get(app.settings_selected)
        .and_then(|f| f.description.clone())
        .or_else(|| {
            schema
                .get(app.settings_selected)
                .map(|f| format!("type: {}", f.field_type))
        })
        .unwrap_or_default();

    let desc_block = Block::default().borders(Borders::ALL).title(" info ");
    frame.render_widget(
        Paragraph::new(description).block(desc_block).wrap(Wrap { trim: true }),
        chunks[2],
    );
}

fn draw_placement(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" enable — choose placement (h/l or arrows, enter to confirm) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let name = app
        .selected_entry()
        .map(|e| e.plugin.name.clone())
        .unwrap_or_default();

    let options: Vec<Span> = PLACEMENTS
        .iter()
        .enumerate()
        .map(|(i, p)| {
            if i == app.placement_selected {
                Span::styled(
                    format!(" [{}] ", p),
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::styled(format!("  {}   ", p), Style::default().fg(Color::White))
            }
        })
        .collect();

    let lines = vec![
        Line::from(Span::styled(name, Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(options),
    ];

    frame.render_widget(Paragraph::new(lines).block(block), area);
}

fn draw_discovery(frame: &mut Frame, app: &App, area: Rect) {
    let panes = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(area);

    let items: Vec<ListItem> = app
        .discovery_sources
        .iter()
        .map(|s| {
            let category = s.catalog.category.clone().unwrap_or_default();
            let color = category_color(&category);
            let line = Line::from(vec![
                Span::raw(s.catalog.name.clone()),
                Span::raw("  "),
                Span::styled(category, Style::default().fg(color)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let mut state = ListState::default();
    if !app.discovery_sources.is_empty() {
        state.select(Some(app.discovery_selected));
    }

    let title = format!(
        " discover — {} available (r refresh, enter install, esc back) ",
        app.discovery_sources.len()
    );
    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Magenta)),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(Color::DarkGray))
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, panes[0], &mut state);

    let block = Block::default()
        .title(" details ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

    let text = if let Some(source) = app.discovery_sources.get(app.discovery_selected) {
        let mut lines = vec![
            Line::from(Span::styled(
                source.catalog.name.clone(),
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(format!("id: {}", source.catalog.id)),
        ];
        if let Some(v) = &source.catalog.version {
            lines.push(Line::from(format!("version: {}", v)));
        }
        if let Some(a) = &source.catalog.author {
            lines.push(Line::from(format!("author: {}", a)));
        }
        if !source.catalog.tags.is_empty() {
            lines.push(Line::from(format!("tags: {}", source.catalog.tags.join(", "))));
        }
        lines.push(Line::from(format!("repo: {}", source.repo)));
        lines.push(Line::from(""));
        lines.push(Line::from(source.catalog.description.clone()));
        lines
    } else {
        vec![Line::from("No plugins to discover — try r to refresh.")]
    };

    frame.render_widget(Paragraph::new(text).block(block).wrap(Wrap { trim: true }), panes[1]);
}

fn draw_status_line(frame: &mut Frame, app: &App, area: Rect) {
    let text = app.status.clone().unwrap_or_default();
    frame.render_widget(Paragraph::new(text).style(Style::default().fg(Color::Yellow)), area);
}
