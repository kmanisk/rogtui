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

    // Base content lines: Profile (5) + Battery (5) + Status (5) = 15 lines.
    // 4 gaps: top, between Profile & Battery, between Battery & Status, bottom.
    let spacers = crate::ui::widgets::distribute_spacers(area.height as usize, 15, 4, 3);

    let mut lines = Vec::new();
    for _ in 0..spacers[0] {
        lines.push(Line::from(""));
    }

    // -------------------------------------------------------------------------
    // 1. Profile Section
    // -------------------------------------------------------------------------
    lines.push(section_rule("Profile", width, theme));

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

    // Profile characteristics details
    let prof_desc = match app.platform_profile {
        crate::backend::types::PlatformProfile::Balanced => "Balanced CPU/GPU wattage · Dynamic fan curve · Everyday computing",
        crate::backend::types::PlatformProfile::Performance => "Max sustained wattage · Higher thermal headroom · Gaming & compute",
        crate::backend::types::PlatformProfile::Quiet => "Conservative power limits · Acoustic ceiling · Extended battery life",
        crate::backend::types::PlatformProfile::Custom => "User-defined platform profile and fan curve parameters",
    };
    lines.push(label_value_row("Tuning profile", vec![Span::styled(prof_desc, Style::default().fg(theme.dim))], false, label_width, theme));

    for _ in 0..spacers[1] {
        lines.push(Line::from(""));
    }

    // -------------------------------------------------------------------------
    // 2. Battery Section
    // -------------------------------------------------------------------------
    lines.push(section_rule("Battery", width, theme));

    // Current Battery Level (Read-only row to eliminate confusion between level and limit)
    let current_level = app.sensors.battery_charge_percent.unwrap_or(60);
    let level_status = if app.sensors.battery_status.eq_ignore_ascii_case("Charging") {
        format!("{current_level}%   ·   Charging")
    } else if app.sensors.battery_status.eq_ignore_ascii_case("Full") {
        format!("{current_level}%   ·   Full")
    } else {
        format!("{current_level}%")
    };
    lines.push(label_value_row("Current level", vec![Span::styled(level_status, Style::default().fg(theme.fg))], false, label_width, theme));

    // Row 3: Charge Limit Slider
    let slider_width = 24.min(width.saturating_sub(label_width + 24));
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
    let mut one_shot_spans = vec![
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
    if app.is_full_charge_active {
        one_shot_spans.push(Span::styled("   Charges to 100% until unplugged", Style::default().fg(theme.dim)));
    } else {
        one_shot_spans.push(Span::styled("   Charge to 100% once for travel", Style::default().fg(theme.dim)));
    }
    lines.push(label_value_row("Full charge once", one_shot_spans, app.power_selected_field == 4, label_width, theme));

    // Battery care guidance
    let advice = match app.charge_limit {
        40..=65 => "Preserves lithium cells by reducing high-voltage anode stress (recommended for AC power).",
        66..=85 => "Balanced threshold extending battery lifespan by 2-3x while retaining portable runtime.",
        _ => "100% full charge maximizes portable runtime; consider 80% on AC to prevent degradation.",
    };
    lines.push(label_value_row("Battery care", vec![Span::styled(advice, Style::default().fg(theme.dim))], false, label_width, theme));

    for _ in 0..spacers[2] {
        lines.push(Line::from(""));
    }

    // -------------------------------------------------------------------------
    // 3. Status Section (Read-only, skipped in navigation)
    // -------------------------------------------------------------------------
    lines.push(section_rule("Status", width, theme));

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
            format!("+{w:.1} W (Charging)")
        } else if is_full {
            "Plugged in (Full)".to_string()
        } else if w > 0.0 {
            format!("{w:.1} W")
        } else {
            "Plugged in, not charging".to_string()
        }
    } else if !app.sensors.battery_status.is_empty() {
        match app.sensors.battery_status.as_str() {
            "Not charging" => "Plugged in, not charging".to_string(),
            other => other.to_string(),
        }
    } else {
        "Plugged in, not charging".to_string()
    };
    lines.push(label_value_row(
        discharge_label,
        vec![Span::styled(discharge_val, Style::default().fg(theme.fg))],
        false,
        label_width,
        theme,
    ));

    let supply_text = if is_charging {
        "AC Adapter (Charging battery)"
    } else if is_full {
        "AC Wall Power (Battery fully charged)"
    } else if let Some(w) = power_w {
        if w > 0.0 {
            "Internal Battery (Discharging)"
        } else {
            "AC Adapter (Wall Power)"
        }
    } else {
        "AC Wall Power"
    };
    lines.push(label_value_row(
        "Power source",
        vec![Span::styled(supply_text, Style::default().fg(theme.fg))],
        false,
        label_width,
        theme,
    ));

    for _ in 0..spacers[3] {
        lines.push(Line::from(""));
    }

    f.render_widget(Paragraph::new(lines), area);
}
