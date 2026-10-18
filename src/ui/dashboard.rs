use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::AppState;
use crate::backend::Backend;

pub fn render_dashboard<B: Backend>(f: &mut Frame, app: &AppState<B>, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(52), Constraint::Percentage(48)])
        .split(area);

    let profile_color = match app.platform_profile {
        crate::backend::types::PlatformProfile::Quiet => Color::Blue,
        crate::backend::types::PlatformProfile::Balanced => Color::Green,
        crate::backend::types::PlatformProfile::Performance => Color::Red,
        crate::backend::types::PlatformProfile::Custom => Color::Magenta,
    };

    let gpu_summary = match app.gpu_mode {
        crate::backend::types::GpuMode::AsusMuxDgpu => "AsusMuxDgpu (Direct MUX / Active)".to_string(),
        crate::backend::types::GpuMode::Hybrid => format!("Hybrid ({})", app.gpu_power.as_str()),
        crate::backend::types::GpuMode::Integrated => "Integrated (dGPU powered off)".to_string(),
        other => format!("{} ({})", other.as_str(), app.gpu_power.as_str()),
    };

    // Column 1: System & Power
    let left_lines = vec![
        Line::from(Span::styled("-- System & Power --", Style::default().fg(Color::DarkGray))),
        Line::from(vec![
            Span::styled("Model       : ", Style::default().fg(Color::DarkGray)),
            Span::raw("ASUS TUF Gaming F16 (FX608JH)"),
        ]),
        Line::from(vec![
            Span::styled("Profile     : ", Style::default().fg(Color::DarkGray)),
            Span::styled(app.platform_profile.as_str(), Style::default().fg(profile_color).add_modifier(Modifier::BOLD)),
            Span::styled("  (press 'p' to cycle)", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled("GPU Mode    : ", Style::default().fg(Color::DarkGray)),
            Span::styled(gpu_summary, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("Battery     : ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}%", app.sensors.battery_charge_percent.unwrap_or(0)), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" (limit: {}%)", app.charge_limit), Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(""),
        Line::from(Span::styled("-- Quick Actions --", Style::default().fg(Color::DarkGray))),
        Line::from(vec![
            Span::styled("p", Style::default().fg(Color::Yellow)),
            Span::raw(" cycle profile   "),
            Span::styled("2", Style::default().fg(Color::Yellow)),
            Span::raw(" power/battery   "),
            Span::styled("3", Style::default().fg(Color::Yellow)),
            Span::raw(" fan curves"),
        ]),
    ];

    f.render_widget(Paragraph::new(left_lines), cols[0]);

    // Column 2: Telemetry
    let cpu_temp_s = app.sensors.cpu_temp_c.map(|t| format!("{t} C")).unwrap_or_else(|| "-- C".to_string());
    let gpu_temp_s = app.sensors.gpu_temp_c.map(|t| format!("{t} C")).unwrap_or_else(|| "-- C".to_string());
    let cpu_fan_s = app.sensors.cpu_fan_rpm.map(|r| format!("{r} RPM")).unwrap_or_else(|| "-- RPM".to_string());
    let gpu_fan_s = app.sensors.gpu_fan_rpm.map(|r| format!("{r} RPM")).unwrap_or_else(|| "-- RPM".to_string());

    let right_lines = vec![
        Line::from(Span::styled("-- Thermal & Fans --", Style::default().fg(Color::DarkGray))),
        Line::from(vec![
            Span::styled("CPU Package : ", Style::default().fg(Color::DarkGray)),
            Span::styled(cpu_temp_s, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("   Fan: ", Style::default().fg(Color::DarkGray)),
            Span::styled(cpu_fan_s, Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("GPU Core    : ", Style::default().fg(Color::DarkGray)),
            Span::styled(gpu_temp_s, Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::styled("   Fan: ", Style::default().fg(Color::DarkGray)),
            Span::styled(gpu_fan_s, Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Sensors     : ", Style::default().fg(Color::DarkGray)),
            Span::styled("sysfs/hwmon live direct", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    f.render_widget(Paragraph::new(right_lines), cols[1]);
}
