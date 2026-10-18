use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::AppState;
use crate::backend::Backend;

pub fn render_aura<B: Backend>(f: &mut Frame, app: &AppState<B>, area: Rect) {
    let c1 = app.aura_data.color1;
    let c2 = app.aura_data.color2;
    let c1_style = Style::default().fg(Color::Rgb(c1.r, c1.g, c1.b));
    let c2_style = Style::default().fg(Color::Rgb(c2.r, c2.g, c2.b));

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
        ("Primary Color Preset", c1.to_hex()),
        ("Color 1 Red Channel", format!("< {} >", c1.r)),
        ("Color 1 Green Channel", format!("< {} >", c1.g)),
        ("Color 1 Blue Channel", format!("< {} >", c1.b)),
        ("Secondary Color Preset", c2.to_hex()),
        ("Effect Animation Speed", format!("< {} >", app.aura_data.speed)),
        ("Animation Direction", format!("< {} >", app.aura_data.direction)),
        ("Power State: Boot", if app.aura_power.boot { "[ ON ]".to_string() } else { "[ OFF ]".to_string() }),
        ("Power State: Awake", if app.aura_power.awake { "[ ON ]".to_string() } else { "[ OFF ]".to_string() }),
        ("Power State: Sleep", if app.aura_power.sleep { "[ ON ]".to_string() } else { "[ OFF ]".to_string() }),
        ("Power State: Shutdown", if app.aura_power.shutdown { "[ ON ]".to_string() } else { "[ OFF ]".to_string() }),
    ];

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled("-- Live Color Swatch --", Style::default().fg(Color::DarkGray))));
    lines.push(Line::from(vec![
        Span::styled("  Color 1 : ", Style::default().fg(Color::DarkGray)),
        Span::styled("████████████", c1_style),
        Span::styled(format!("  {} (R:{} G:{} B:{})", c1.to_hex(), c1.r, c1.g, c1.b), Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Color 2 : ", Style::default().fg(Color::DarkGray)),
        Span::styled("████████████", c2_style),
        Span::styled(format!("  {} (R:{} G:{} B:{})", c2.to_hex(), c2.r, c2.g, c2.b), Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("-- Lighting Parameters & Power States --", Style::default().fg(Color::DarkGray))));

    for (idx, (label, val)) in fields.iter().enumerate() {
        let is_selected = app.aura_selected_field == idx;
        let prefix = if is_selected { "> " } else { "  " };
        let label_style = if is_selected {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let val_style = if is_selected {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        lines.push(Line::from(vec![
            Span::styled(format!("{prefix}{label: <26}: "), label_style),
            Span::styled(val, val_style),
        ]));
    }

    f.render_widget(Paragraph::new(lines), area);
}
