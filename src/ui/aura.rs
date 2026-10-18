use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::app::AppState;
use crate::backend::Backend;

pub fn render_aura<B: Backend>(f: &mut Frame, app: &AppState<B>, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Min(12),
            Constraint::Length(4),
        ])
        .split(area);

    // Color Swatch & Preview Banner
    let c1 = app.aura_data.color1;
    let c2 = app.aura_data.color2;
    let c1_style = Style::default().fg(Color::Rgb(c1.r, c1.g, c1.b));
    let c2_style = Style::default().fg(Color::Rgb(c2.r, c2.g, c2.b));

    let preview_lines = vec![
        Line::from(vec![
            Span::styled("  Color 1 Swatch: ", Style::default().fg(Color::DarkGray)),
            Span::styled("████████████████", c1_style),
            Span::styled(format!("  {} (R:{} G:{} B:{})", c1.to_hex(), c1.r, c1.g, c1.b), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  Color 2 Swatch: ", Style::default().fg(Color::DarkGray)),
            Span::styled("████████████████", c2_style),
            Span::styled(format!("  {} (R:{} G:{} B:{})", c2.to_hex(), c2.r, c2.g, c2.b), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
    ];

    let preview_widget = Paragraph::new(preview_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(Span::styled(" Live Color Swatch ", Style::default().add_modifier(Modifier::BOLD))),
    );
    f.render_widget(preview_widget, chunks[0]);

    // Aura Settings List
    let brightness_str = match app.aura_brightness {
        0 => "Off",
        1 => "Low",
        2 => "Medium",
        3 => "High",
        _ => "Custom",
    };

    let mode_str = match app.aura_mode {
        0 => "Static",
        1 => "Breathe",
        2 => "Rainbow",
        10 => "Pulse",
        _ => "Custom",
    };

    let fields = [
        ("Backlight Brightness", format!("< {brightness_str} >")),
        ("Aura Effect Mode", format!("< {mode_str} >")),
        ("Primary Color (Hex)", c1.to_hex()),
        ("Color 1 Red Channel", format!("< {} >", c1.r)),
        ("Color 1 Green Channel", format!("< {} >", c1.g)),
        ("Color 1 Blue Channel", format!("< {} >", c1.b)),
        ("Secondary Color (Hex)", c2.to_hex()),
        ("Effect Animation Speed", format!("< {} >", app.aura_data.speed)),
        ("Animation Direction", format!("< {} >", app.aura_data.direction)),
        ("Power State: Boot", if app.aura_power.boot { "[ ON ]".to_string() } else { "[ OFF ]".to_string() }),
        ("Power State: Awake", if app.aura_power.awake { "[ ON ]".to_string() } else { "[ OFF ]".to_string() }),
        ("Power State: Sleep", if app.aura_power.sleep { "[ ON ]".to_string() } else { "[ OFF ]".to_string() }),
        ("Power State: Shutdown", if app.aura_power.shutdown { "[ ON ]".to_string() } else { "[ OFF ]".to_string() }),
    ];

    let mut list_lines = Vec::new();
    list_lines.push(Line::from(""));

    for (idx, (label, val)) in fields.iter().enumerate() {
        let is_selected = app.aura_selected_field == idx;
        let prefix = if is_selected { "  > " } else { "    " };
        let label_style = if is_selected {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let val_style = if is_selected {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        list_lines.push(Line::from(vec![
            Span::styled(format!("{prefix}{label: <26}: "), label_style),
            Span::styled(val, val_style),
        ]));
    }

    let list_widget = Paragraph::new(list_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(Span::styled(" Lighting Parameters & Power States ", Style::default().add_modifier(Modifier::BOLD))),
    );
    f.render_widget(list_widget, chunks[1]);

    // Footer
    let guide_line = Line::from(vec![
        Span::styled("  j/k or Up/Down: ", Style::default().fg(Color::Yellow)),
        Span::raw("Navigate field   |   "),
        Span::styled("h/l or Left/Right: ", Style::default().fg(Color::Yellow)),
        Span::raw("Adjust / toggle   |   "),
        Span::styled("w: ", Style::default().fg(Color::Yellow)),
        Span::raw("Apply to DBus"),
    ]);
    let guide_widget = Paragraph::new(guide_line).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Controls "),
    );
    f.render_widget(guide_widget, chunks[2]);
}
