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
    let is_off = app.aura_brightness == 0;
    let c1 = app.aura_data.color1;
    let c2 = app.aura_data.color2;
    let c1_style = if is_off {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::Rgb(c1.r, c1.g, c1.b))
    };
    let c2_style = if is_off {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default().fg(Color::Rgb(c2.r, c2.g, c2.b))
    };

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

    let mode_display = if is_off {
        "< Disabled (Backlight Off) >".to_string()
    } else {
        format!("< {mode_str} >")
    };

    let fields = [
        ("Backlight Brightness", format!("< {brightness_str} >")),
        ("Aura Effect Mode", mode_display),
        ("Primary Color Preset", if is_off { format!("{} (Off)", c1.to_hex()) } else { c1.to_hex() }),
        ("Color 1 Red Channel", if is_off { format!("< {} > (Off)", c1.r) } else { format!("< {} >", c1.r) }),
        ("Color 1 Green Channel", if is_off { format!("< {} > (Off)", c1.g) } else { format!("< {} >", c1.g) }),
        ("Color 1 Blue Channel", if is_off { format!("< {} > (Off)", c1.b) } else { format!("< {} >", c1.b) }),
        ("Secondary Color Preset", if is_off { format!("{} (Off)", c2.to_hex()) } else { c2.to_hex() }),
        ("Effect Animation Speed", if is_off { format!("< {} > (Off)", app.aura_data.speed) } else { format!("< {} >", app.aura_data.speed) }),
        ("Animation Direction", if is_off { format!("< {} > (Off)", app.aura_data.direction) } else { format!("< {} >", app.aura_data.direction) }),
        ("Power State: Boot", if app.aura_power.boot { "[ ON ]".to_string() } else { "[ OFF ]".to_string() }),
        ("Power State: Awake", if app.aura_power.awake { "[ ON ]".to_string() } else { "[ OFF ]".to_string() }),
        ("Power State: Sleep", if app.aura_power.sleep { "[ ON ]".to_string() } else { "[ OFF ]".to_string() }),
        ("Power State: Shutdown", if app.aura_power.shutdown { "[ ON ]".to_string() } else { "[ OFF ]".to_string() }),
    ];

    let mut lines = Vec::new();
    let swatch_header = if is_off {
        "-- Live Color Swatch (Backlight Off) --"
    } else {
        "-- Live Color Swatch --"
    };
    lines.push(Line::from(Span::styled(swatch_header, Style::default().fg(Color::DarkGray))));
    lines.push(Line::from(vec![
        Span::styled("  Color 1 : ", Style::default().fg(Color::DarkGray)),
        Span::styled("████████████", c1_style),
        Span::styled(
            format!("  {} (R:{} G:{} B:{}){}", c1.to_hex(), c1.r, c1.g, c1.b, if is_off { " [Off]" } else { "" }),
            if is_off { Style::default().fg(Color::DarkGray) } else { Style::default().fg(Color::White) },
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Color 2 : ", Style::default().fg(Color::DarkGray)),
        Span::styled("████████████", c2_style),
        Span::styled(
            format!("  {} (R:{} G:{} B:{}){}", c2.to_hex(), c2.r, c2.g, c2.b, if is_off { " [Off]" } else { "" }),
            if is_off { Style::default().fg(Color::DarkGray) } else { Style::default().fg(Color::White) },
        ),
    ]));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled("-- Lighting Parameters & Power States --", Style::default().fg(Color::DarkGray))));

    for (idx, (label, val)) in fields.iter().enumerate() {
        let is_effect_field = (1..=8).contains(&idx);
        let is_disabled = is_off && is_effect_field;
        let is_selected = !is_disabled && app.aura_selected_field == idx;

        let prefix = if is_selected { "> " } else { "  " };
        let label_style = if is_disabled {
            Style::default().fg(Color::DarkGray)
        } else if is_selected {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let val_style = if is_disabled {
            Style::default().fg(Color::DarkGray)
        } else if is_selected {
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
