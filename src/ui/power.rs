use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::AppState;
use crate::backend::Backend;
use crate::ui::widgets::{
    choice_widget, label_value_row, render_hint_line, section_rule, slider_widget,
};

pub fn render_power<B: Backend>(f: &mut Frame, app: &AppState<B>, area: Rect) {
    let theme = &app.theme;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Top spacing
            Constraint::Min(12),   // Power settings list
            Constraint::Length(1), // Bottom hint line
        ])
        .split(area);

    let content_area = chunks[1];
    let width = content_area.width as usize;
    let label_width = 20;

    let mut lines = Vec::new();

    // -------------------------------------------------------------------------
    // 1. Profile Section
    // -------------------------------------------------------------------------
    lines.push(section_rule("Profile", width.saturating_sub(4), theme));
    lines.push(Line::from(""));

    // Row 0: Active Profile
    let active_spans = choice_widget(app.platform_profile.as_str(), app.power_selected_field == 0, theme);
    lines.push(label_value_row("Active profile", active_spans, app.power_selected_field == 0, label_width, theme));

    // Row 1: When plugged in
    let ac_label = if app.auto_ac {
        app.profile_ac.as_str()
    } else {
        "Off"
    };
    let ac_spans = choice_widget(ac_label, app.power_selected_field == 1, theme);
    lines.push(label_value_row("When plugged in", ac_spans, app.power_selected_field == 1, label_width, theme));

    // Row 2: On battery
    let bat_label = if app.auto_battery {
        app.profile_battery.as_str()
    } else {
        "Off"
    };
    let bat_spans = choice_widget(bat_label, app.power_selected_field == 2, theme);
    lines.push(label_value_row("On battery", bat_spans, app.power_selected_field == 2, label_width, theme));
    lines.push(Line::from(""));

    // -------------------------------------------------------------------------
    // 2. Battery Section
    // -------------------------------------------------------------------------
    lines.push(section_rule("Battery", width.saturating_sub(4), theme));
    lines.push(Line::from(""));

    // Row 3: Charge Limit Slider
    let slider_width = 16.min(width.saturating_sub(label_width + 16));
    let limit_spans = slider_widget(
        app.charge_limit as u32,
        20,
        100,
        slider_width,
        "%",
        app.power_selected_field == 3,
        theme,
    );
    lines.push(label_value_row("Charge limit", limit_spans, app.power_selected_field == 3, label_width, theme));

    // Row 4: One-Shot Full Charge
    let one_shot_spans = vec![
        Span::styled(
            "[ Charge to 100% once ]",
            if app.power_selected_field == 4 {
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.dim)
            },
        ),
    ];
    lines.push(label_value_row("Full charge override", one_shot_spans, app.power_selected_field == 4, label_width, theme));

    // Read-only battery metadata rows
    let bat_state_str = if app.sensors.battery_status.is_empty() {
        "Plugged in, not charging".to_string()
    } else {
        match app.sensors.battery_status.as_str() {
            "Not charging" => "Plugged in, not charging".to_string(),
            "Charging" => "Charging".to_string(),
            "Discharging" => "Discharging".to_string(),
            "Full" => "Full".to_string(),
            other => other.to_string(),
        }
    };
    lines.push(label_value_row(
        "State",
        vec![Span::styled(bat_state_str, Style::default().fg(theme.fg))],
        false,
        label_width,
        theme,
    ));

    let health_pct = app.sensors.battery_health_percent.unwrap_or(98);
    lines.push(label_value_row(
        "Health",
        vec![Span::styled(format!("{health_pct}%"), Style::default().fg(theme.ok))],
        false,
        label_width,
        theme,
    ));

    // Battery cycles: only display if cycle_count > 0
    if let Some(cycles) = app.sensors.battery_cycle_count {
        if cycles > 0 {
            lines.push(label_value_row(
                "Cycles",
                vec![Span::styled(format!("{cycles}"), Style::default().fg(theme.fg))],
                false,
                label_width,
                theme,
            ));
        }
    }

    f.render_widget(Paragraph::new(lines), content_area);

    // Bottom Hint Line
    let hint_text = match app.power_selected_field {
        0 => "h/l: cycle active profile   Enter: apply",
        1 => "h/l: set AC plugged-in profile (or Off)",
        2 => "h/l: set on-battery profile (or Off)",
        3 => "h/l: adjust charge threshold (+/- 5%)",
        4 => "Enter: temporarily charge to 100% until power unplugged",
        _ => "",
    };
    render_hint_line(f, chunks[2], hint_text, theme);
}
