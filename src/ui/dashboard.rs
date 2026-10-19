use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::AppState;
use crate::backend::Backend;
use crate::ui::widgets::{choice_widget, render_sparkline};

fn clean_cpu_name(raw: &str) -> String {
    let s = raw
        .replace("13th Gen ", "")
        .replace("12th Gen ", "")
        .replace("14th Gen ", "")
        .replace("(R)", "")
        .replace("(TM)", "")
        .replace("CPU ", "");
    if let Some(pos) = s.find('@') {
        s[..pos].trim().to_string()
    } else {
        s.split_whitespace().collect::<Vec<_>>().join(" ")
    }
}

pub fn render_dashboard<B: Backend>(f: &mut Frame, app: &AppState<B>, area: Rect) {
    let theme = &app.theme;

    // Outer padding: 1 line top, 2 columns left
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Spacing
            Constraint::Min(16),   // Main dashboard content
        ])
        .split(area);

    let content_area = chunks[1];
    let width = content_area.width as usize;
    let bar_width = (width.saturating_sub(44) / 2).clamp(10, 20);
    let sparkline_width = 12.min(width.saturating_sub(60));

    let mut lines = Vec::new();

    // -------------------------------------------------------------------------
    // 1. Profile Row
    // -------------------------------------------------------------------------
    let mut profile_spans = vec![
        Span::raw(" "),
        Span::styled("Profile   ", Style::default().fg(theme.dim)),
    ];
    profile_spans.extend(choice_widget(app.platform_profile.as_str(), true, theme));
    profile_spans.push(Span::raw("                         "));
    profile_spans.push(Span::styled("p", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)));
    profile_spans.push(Span::styled(" cycle", Style::default().fg(theme.dim)));
    lines.push(Line::from(profile_spans));
    lines.push(Line::from(""));

    // -------------------------------------------------------------------------
    // 2. CPU Row
    // -------------------------------------------------------------------------
    let raw_cpu = if app.sensors.cpu_model.is_empty() {
        "Intel Core i5-13450HX"
    } else {
        app.sensors.cpu_model.as_str()
    };
    let cpu_name = clean_cpu_name(raw_cpu);
    lines.push(Line::from(vec![
        Span::raw(" "),
        Span::styled("CPU   ", Style::default().fg(theme.dim)),
        Span::styled(cpu_name, Style::default().fg(theme.fg).add_modifier(Modifier::BOLD)),
    ]));

    let cpu_usage = app.sensors.cpu_usage_percent.unwrap_or(0).clamp(0, 100);
    let cpu_temp = app.sensors.cpu_temp_c.unwrap_or(0);
    let cpu_rpm = app.sensors.cpu_fan_rpm.unwrap_or(0);

    let cpu_filled = (cpu_usage as usize * bar_width) / 100;
    let cpu_empty = bar_width.saturating_sub(cpu_filled);
    let cpu_bar_filled = "━".repeat(cpu_filled);
    let cpu_bar_empty = "─".repeat(cpu_empty);

    let mut cpu_metric_spans = vec![
        Span::raw("      "),
        Span::styled(format!("{cpu_usage:>3}%  "), Style::default().fg(theme.fg)),
        Span::styled(cpu_bar_filled, Style::default().fg(theme.accent)),
        Span::styled(cpu_bar_empty, Style::default().fg(Color::Rgb(70, 75, 85))),
        Span::raw("   "),
        Span::styled(format!("{cpu_temp:>2}°C"), Style::default().fg(theme.temp_color(cpu_temp)).add_modifier(Modifier::BOLD)),
        Span::raw("   "),
        Span::styled(format!("{cpu_rpm:>4} rpm"), Style::default().fg(theme.dim)),
        Span::raw("   "),
    ];

    let cpu_history: Vec<u64> = app.cpu_temp_history.iter().copied().collect();
    cpu_metric_spans.extend(render_sparkline(&cpu_history, sparkline_width, theme.temp_color(cpu_temp)));
    lines.push(Line::from(cpu_metric_spans));

    // -------------------------------------------------------------------------
    // 3. GPU Row
    // -------------------------------------------------------------------------
    let dgpu_state_lower = app.sensors.dgpu_status.to_lowercase();
    let is_suspended = dgpu_state_lower.contains("d3cold")
        || dgpu_state_lower.contains("suspended")
        || dgpu_state_lower.contains("idle")
        || app.sensors.gpu_temp_c.unwrap_or(0) == 0;

    lines.push(Line::from(vec![
        Span::raw(" "),
        Span::styled("GPU   ", Style::default().fg(theme.dim)),
        Span::styled("RTX 5050", Style::default().fg(theme.fg).add_modifier(Modifier::BOLD)),
    ]));

    if is_suspended {
        lines.push(Line::from(vec![
            Span::raw("      "),
            Span::styled("Idle", Style::default().fg(theme.dim)),
        ]));
    } else {
        let gpu_usage = app.sensors.gpu_usage_percent.unwrap_or(0).clamp(0, 100);
        let gpu_temp = app.sensors.gpu_temp_c.unwrap_or(0);
        let gpu_rpm = app.sensors.gpu_fan_rpm.unwrap_or(0);

        let gpu_filled = (gpu_usage as usize * bar_width) / 100;
        let gpu_empty = bar_width.saturating_sub(gpu_filled);
        let gpu_bar_filled = "━".repeat(gpu_filled);
        let gpu_bar_empty = "─".repeat(gpu_empty);

        let mut gpu_metric_spans = vec![
            Span::raw("      "),
            Span::styled(format!("{gpu_usage:>3}%  "), Style::default().fg(theme.fg)),
            Span::styled(gpu_bar_filled, Style::default().fg(theme.accent)),
            Span::styled(gpu_bar_empty, Style::default().fg(Color::Rgb(70, 75, 85))),
            Span::raw("   "),
            Span::styled(format!("{gpu_temp:>2}°C"), Style::default().fg(theme.temp_color(gpu_temp)).add_modifier(Modifier::BOLD)),
            Span::raw("   "),
            Span::styled(format!("{gpu_rpm:>4} rpm"), Style::default().fg(theme.dim)),
            Span::raw("   "),
        ];

        let gpu_history: Vec<u64> = app.gpu_temp_history.iter().copied().collect();
        gpu_metric_spans.extend(render_sparkline(&gpu_history, sparkline_width, theme.temp_color(gpu_temp)));
        lines.push(Line::from(gpu_metric_spans));
    }

    // -------------------------------------------------------------------------
    // 4. RAM Row
    // -------------------------------------------------------------------------
    let ram_used = app.sensors.ram_used_mb.unwrap_or(0) as f64 / 1024.0;
    let ram_total = app.sensors.ram_total_mb.unwrap_or(1024) as f64 / 1024.0;
    let ram_pct = (ram_used / ram_total.max(0.1) * 100.0).clamp(0.0, 100.0) as usize;

    let ram_filled = (ram_pct * bar_width) / 100;
    let ram_empty = bar_width.saturating_sub(ram_filled);
    let ram_bar_filled = "━".repeat(ram_filled);
    let ram_bar_empty = "─".repeat(ram_empty);

    lines.push(Line::from(vec![
        Span::raw(" "),
        Span::styled("RAM   ", Style::default().fg(theme.dim)),
        Span::styled(format!("{ram_used:.1} / {ram_total:.1} GiB   "), Style::default().fg(theme.fg)),
        Span::styled(ram_bar_filled, Style::default().fg(theme.accent)),
        Span::styled(ram_bar_empty, Style::default().fg(Color::Rgb(70, 75, 85))),
    ]));
    lines.push(Line::from(""));

    // -------------------------------------------------------------------------
    // 5. Battery Section
    // -------------------------------------------------------------------------
    let bat_pct = app.sensors.battery_charge_percent.unwrap_or(60).clamp(0, 100);
    let bat_filled = (bat_pct as usize * bar_width) / 100;
    let bat_empty = bar_width.saturating_sub(bat_filled);
    let bat_bar_filled = "━".repeat(bat_filled);
    let bat_bar_empty = "─".repeat(bat_empty);

    let bat_state_str = if app.sensors.battery_status.is_empty() {
        "Plugged in, not charging"
    } else {
        match app.sensors.battery_status.as_str() {
            "Not charging" => "Plugged in, not charging",
            "Charging" => "Charging",
            "Discharging" => "Discharging",
            "Full" => "Full",
            other => other,
        }
    };

    let bat_color = if bat_pct <= 20 {
        theme.err
    } else if bat_pct <= 60 {
        theme.warn
    } else {
        theme.ok
    };

    lines.push(Line::from(vec![
        Span::raw(" "),
        Span::styled("Battery   ", Style::default().fg(theme.dim)),
        Span::styled(format!("{bat_pct:>2}%  "), Style::default().fg(theme.fg)),
        Span::styled(bat_bar_filled, Style::default().fg(bat_color)),
        Span::styled(bat_bar_empty, Style::default().fg(Color::Rgb(70, 75, 85))),
        Span::raw("   "),
        Span::styled(bat_state_str, Style::default().fg(theme.fg)),
    ]));

    let health_pct = app.sensors.battery_health_percent.unwrap_or(98);
    lines.push(Line::from(vec![
        Span::raw("           "),
        Span::styled(format!("Limit {}%", app.charge_limit), Style::default().fg(theme.dim)),
        Span::styled("   ·   ", Style::default().fg(Color::Rgb(70, 75, 85))),
        Span::styled(format!("Health {}%", health_pct), Style::default().fg(theme.dim)),
    ]));

    f.render_widget(Paragraph::new(lines), content_area);
}
