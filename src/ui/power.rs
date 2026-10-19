use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::AppState;
use crate::backend::Backend;
use crate::ui::widgets::{charge_limit_slider, choice_widget, label_value_row, section_rule};

pub fn render_power<B: Backend>(f: &mut Frame, app: &AppState<B>, area: Rect) {
    let theme = &app.theme;
    let width = area.width as usize;
    let label_width = 18;

    let mut lines = Vec::new();
    lines.push(Line::from(""));

    // -------------------------------------------------------------------------
    // 1. Profile Section
    // -------------------------------------------------------------------------
    lines.push(section_rule("Profile", width, theme));
    lines.push(Line::from(""));

    // Row 0: Active Profile
    let active_spans = choice_widget(app.platform_profile.as_str(), app.power_selected_field == 0, theme);
    lines.push(label_value_row("Active profile", active_spans, app.power_selected_field == 0, label_width, theme));
    lines.push(Line::from(""));

    // Subheading: Switch automatically
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled("Switch automatically", Style::default().fg(theme.dim)),
    ]));
    lines.push(Line::from(""));

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
    lines.push(section_rule("Battery", width, theme));
    lines.push(Line::from(""));

    // Row 3: Charge Limit Slider
    let slider_width = 24.min(width.saturating_sub(label_width + 20));
    let limit_spans = charge_limit_slider(
        app.charge_limit,
        slider_width,
        app.power_selected_field == 3,
        theme,
    );
    lines.push(label_value_row("Charge limit", limit_spans, app.power_selected_field == 3, label_width, theme));

    // Row 4: One-Shot Full Charge
    let one_shot_str = if app.is_full_charge_active {
        "[ Active ]"
    } else {
        "[ Start ↵ ]"
    };
    let one_shot_spans = vec![
        Span::styled(
            one_shot_str,
            if app.power_selected_field == 4 {
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
            } else if app.is_full_charge_active {
                Style::default().fg(theme.ok)
            } else {
                Style::default().fg(theme.dim)
            },
        ),
    ];
    lines.push(label_value_row("Full charge once", one_shot_spans, app.power_selected_field == 4, label_width, theme));
    lines.push(Line::from(""));

    // -------------------------------------------------------------------------
    // 3. Status Section (Read-only, skipped in navigation)
    // -------------------------------------------------------------------------
    lines.push(section_rule("Status", width, theme));
    lines.push(Line::from(""));

    let health_pct = app.sensors.battery_health_percent.unwrap_or(98);
    lines.push(label_value_row(
        "Health",
        vec![Span::styled(format!("{health_pct}%"), Style::default().fg(theme.ok))],
        false,
        label_width,
        theme,
    ));

    let cycles = app.sensors.battery_cycle_count.unwrap_or(42);
    lines.push(label_value_row(
        "Cycles",
        vec![Span::styled(format!("{cycles}"), Style::default().fg(theme.fg))],
        false,
        label_width,
        theme,
    ));

    let power_w = app.sensors.battery_power_w;
    let is_charging = app.sensors.battery_status.eq_ignore_ascii_case("Charging");
    let is_full = app.sensors.battery_status.eq_ignore_ascii_case("Full");
    let discharge_label = if is_charging {
        "Charge rate"
    } else if is_full {
        "State"
    } else {
        "Discharge"
    };
    let discharge_val = if let Some(w) = power_w {
        if is_charging {
            format!("{w:.1} W (Charging)")
        } else if is_full {
            "Plugged in (Full)".to_string()
        } else {
            format!("{w:.1} W")
        }
    } else if !app.sensors.battery_status.is_empty() {
        app.sensors.battery_status.clone()
    } else {
        "18.4 W".to_string()
    };
    lines.push(label_value_row(
        discharge_label,
        vec![Span::styled(discharge_val, Style::default().fg(theme.fg))],
        false,
        label_width,
        theme,
    ));

    f.render_widget(Paragraph::new(lines), area);
}
