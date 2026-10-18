use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::AppState;
use crate::backend::Backend;
use crate::ui::widgets::panel_block;

pub fn render_power<B: Backend>(f: &mut Frame, app: &AppState<B>, area: Rect) {
    let theme = &app.theme;
    let is_narrow = area.width < 100;

    let chunks = if is_narrow {
        vec![area]
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area)
            .to_vec()
    };

    // -------------------------------------------------------------------------
    // Left Pane: Settings List
    // -------------------------------------------------------------------------
    let settings_block = panel_block(" Power & Battery Settings ", true, theme);
    let settings_inner = settings_block.inner(chunks[0]);
    f.render_widget(settings_block, chunks[0]);

    let bat_gauge_width = (settings_inner.width.saturating_sub(42) as usize).clamp(8, 16);
    let bat_filled = (app.charge_limit as usize * bat_gauge_width) / 100;
    let bat_empty = bat_gauge_width.saturating_sub(bat_filled);
    let bat_gauge_str = format!("{}% [{}{}]", app.charge_limit, "█".repeat(bat_filled), "░".repeat(bat_empty));

    let fields = [
        // 0: Profiles group
        (
            0,
            "Profiles",
            "Active Platform Profile",
            format!("< {} >", app.platform_profile.as_str()),
        ),
        (
            1,
            "Profiles",
            "AC Profile Binding",
            format!("< {} >", app.profile_ac.as_str()),
        ),
        (
            2,
            "Profiles",
            "Battery Profile Binding",
            format!("< {} >", app.profile_battery.as_str()),
        ),
        (
            3,
            "Profiles",
            "Auto-apply AC Profile",
            if app.auto_ac { "[x] Enabled".to_string() } else { "[ ] Disabled".to_string() },
        ),
        (
            4,
            "Profiles",
            "Auto-apply Battery Profile",
            if app.auto_battery { "[x] Enabled".to_string() } else { "[ ] Disabled".to_string() },
        ),
        // 5: Battery group
        (
            5,
            "Battery",
            "Battery Charge Limit",
            format!("< {bat_gauge_str} >"),
        ),
        (
            6,
            "Battery",
            "One-Shot Full Charge",
            "[ Trigger 100% until unplug ]".to_string(),
        ),
        // 7: Power Tuning group
        (
            7,
            "Power Tuning",
            "Link Profile to CPU EPP",
            if app.linked_epp { "[x] Enabled".to_string() } else { "[ ] Disabled".to_string() },
        ),
        (
            8,
            "Power Tuning",
            "Disable NV Powerd on Battery",
            if app.disable_powerd_battery { "[x] Enabled".to_string() } else { "[ ] Disabled".to_string() },
        ),
    ];

    let mut lines = Vec::new();
    let mut current_group = "";

    for (idx, group, label, val) in &fields {
        if *group != current_group {
            if !current_group.is_empty() {
                lines.push(Line::from(""));
            }
            lines.push(Line::from(Span::styled(
                format!("  -- {group} --"),
                Style::default().fg(theme.dim),
            )));
            current_group = group;
        }

        let is_selected = app.power_selected_field == *idx;
        let line_content = format!("  {label: <30}: {val}");
        let total_w = settings_inner.width as usize;
        let padded_line = if line_content.len() < total_w {
            format!("{}{}", line_content, " ".repeat(total_w - line_content.len()))
        } else {
            line_content
        };

        if is_selected {
            lines.push(Line::from(Span::styled(padded_line, theme.sel_style())));
        } else {
            lines.push(Line::from(vec![
                Span::styled(format!("  {label: <30}: "), Style::default().fg(theme.dim)),
                Span::styled(val.clone(), Style::default().fg(theme.fg)),
            ]));
        }

        if *idx == 6 {
            let bat_status = if app.sensors.battery_status.is_empty() {
                "Not charging"
            } else {
                app.sensors.battery_status.as_str()
            };
            let bat_health = app.sensors.battery_health_percent.unwrap_or(98);
            let bat_power_str = if let Some(w) = app.sensors.battery_power_w {
                if w > 0.05 { format!("{w:.1} W") } else { "0.0 W (AC Wall Power)".to_string() }
            } else {
                "0.0 W (AC Wall Power)".to_string()
            };
            let bat_cycles_str = match app.sensors.battery_cycle_count {
                Some(c) if c > 0 => format!("{c} cycles"),
                _ => "n/a".to_string(),
            };

            lines.push(Line::from(vec![
                Span::styled("    Battery Health   : ", Style::default().fg(theme.dim)),
                Span::styled(format!("{bat_health}% (90 Wh Li-ion, cycles: {bat_cycles_str})"), Style::default().fg(theme.ok)),
            ]));
            lines.push(Line::from(vec![
                Span::styled("    State & Draw     : ", Style::default().fg(theme.dim)),
                Span::styled(format!("{bat_status} ({bat_power_str})"), Style::default().fg(theme.fg)),
            ]));
        }
    }

    f.render_widget(Paragraph::new(lines), settings_inner);

    // -------------------------------------------------------------------------
    // Right Pane: Details / Explanation
    // -------------------------------------------------------------------------
    if !is_narrow {
        let details_block = panel_block(" Setting Details ", false, theme);
        let details_inner = details_block.inner(chunks[1]);
        f.render_widget(details_block, chunks[1]);

        let (sel_title, sel_desc, sel_effect) = match app.power_selected_field {
            0 => (
                "Active Platform Profile",
                "Controls CPU TDP package limits, fan curve aggression, and system energy policy via asusd and the ASUS ACPI platform_profile interface.",
                "Quiet: 25W TDP / silent fan ceiling\nBalanced: 45W TDP / dynamic fans\nPerformance: 75W+ TDP / full boost",
            ),
            1 => (
                "AC Profile Binding",
                "Specifies which platform profile asusd automatically restores when the laptop connects to AC power (barrel jack or USB-PD).",
                "Hardware auto-switches to this profile upon plug-in event.",
            ),
            2 => (
                "Battery Profile Binding",
                "Specifies which platform profile asusd automatically engages when operating on battery.",
                "Enables power saving and quieter acoustic floor on battery.",
            ),
            3 => (
                "Auto-apply AC Profile",
                "Enables udev event listener in asusd to automatically engage the AC Profile Binding when plugged in.",
                "Instant, hands-free power state transition on AC connection.",
            ),
            4 => (
                "Auto-apply Battery Profile",
                "Enables udev event listener in asusd to automatically switch to the Battery Profile Binding upon AC disconnect.",
                "Protects battery runtime when unplugging on the go.",
            ),
            5 => (
                "Battery Charge Limit",
                "Writes a hardware threshold to the ASUS embedded controller via sysfs charge_control_limit.",
                "Stops charging at the specified percentage (e.g. 80%) to minimize lithium chemical degradation during continuous AC use.",
            ),
            6 => (
                "One-Shot Full Charge",
                "Temporarily lifts the battery charge limit to 100% until the next unplug event.",
                "Allows charging to 100% before travel without having to permanently change your battery threshold.",
            ),
            7 => (
                "Link Profile to CPU EPP",
                "Binds the Intel CPU Energy Performance Preference (EPP) directly to the active platform profile.",
                "Quiet -> power\nBalanced -> balance_performance\nPerformance -> performance",
            ),
            8 => (
                "Disable NVIDIA Powerd on Battery",
                "Stops nvidia-powerd from waking up the discrete GPU while on battery.",
                "Ensures the RTX 5050 Mobile stays suspended in D3cold power state.",
            ),
            _ => ("Setting", "Select a setting to view details.", ""),
        };

        let mut detail_lines = vec![
            Line::from(Span::styled(
                format!("  {sel_title}"),
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled("  Description:", Style::default().fg(theme.dim))),
        ];
        for l in sel_desc.lines() {
            detail_lines.push(Line::from(Span::styled(format!("  {l}"), Style::default().fg(theme.fg))));
        }
        detail_lines.push(Line::from(""));
        detail_lines.push(Line::from(Span::styled("  Hardware Effect:", Style::default().fg(theme.dim))));
        for l in sel_effect.lines() {
            detail_lines.push(Line::from(Span::styled(format!("  {l}"), Style::default().fg(theme.ok))));
        }
        detail_lines.push(Line::from(""));
        detail_lines.push(Line::from(Span::styled("  Navigation:", Style::default().fg(theme.dim))));
        detail_lines.push(Line::from(Span::styled("  Use 'h' / 'l' or Enter / Space to adjust values.", Style::default().fg(theme.dim))));

        f.render_widget(Paragraph::new(detail_lines), details_inner);
    }
}
