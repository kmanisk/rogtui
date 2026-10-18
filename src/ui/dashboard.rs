use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::app::AppState;
use crate::backend::Backend;

pub fn render_dashboard<B: Backend>(f: &mut Frame, app: &AppState<B>, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Length(7),
            Constraint::Min(4),
        ])
        .split(area);

    // Block 1: System & Power Summary
    let profile_color = match app.platform_profile {
        crate::backend::types::PlatformProfile::Quiet => Color::Blue,
        crate::backend::types::PlatformProfile::Balanced => Color::Green,
        crate::backend::types::PlatformProfile::Performance => Color::Red,
        crate::backend::types::PlatformProfile::Custom => Color::Magenta,
    };

    let summary_lines = vec![
        Line::from(vec![
            Span::styled("  Hardware Platform:  ", Style::default().fg(Color::DarkGray)),
            Span::styled("ASUS TUF Gaming F16 (FX608JH)", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  Platform Profile:   ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("[ {} ]", app.platform_profile.as_str()),
                Style::default().fg(profile_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  (Press 'p' or Enter to cycle)", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(vec![
            Span::styled("  Active GPU Mode:    ", Style::default().fg(Color::DarkGray)),
            Span::styled(app.gpu_mode.as_str(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  dGPU Power State:   ", Style::default().fg(Color::DarkGray)),
            Span::styled(app.gpu_power.as_str(), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("  Battery Limit:      ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}%", app.charge_limit),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
            Span::styled("  |  Current Charge: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}%", app.sensors.battery_charge_percent.unwrap_or(0)),
                Style::default().fg(Color::Green),
            ),
        ]),
    ];

    let summary_widget = Paragraph::new(summary_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(Span::styled(" Hardware & Power Status ", Style::default().add_modifier(Modifier::BOLD))),
    );
    f.render_widget(summary_widget, chunks[0]);

    // Block 2: Live Thermal & Fan Telemetry
    let cpu_temp_s = app.sensors.cpu_temp_c.map(|t| format!("{t} C")).unwrap_or_else(|| "-- C".to_string());
    let gpu_temp_s = app.sensors.gpu_temp_c.map(|t| format!("{t} C")).unwrap_or_else(|| "-- C".to_string());
    let cpu_fan_s = app.sensors.cpu_fan_rpm.map(|r| format!("{r} RPM")).unwrap_or_else(|| "-- RPM".to_string());
    let gpu_fan_s = app.sensors.gpu_fan_rpm.map(|r| format!("{r} RPM")).unwrap_or_else(|| "-- RPM".to_string());

    let thermal_lines = vec![
        Line::from(vec![
            Span::styled("  CPU Package Temp:   ", Style::default().fg(Color::DarkGray)),
            Span::styled(cpu_temp_s, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("       CPU Fan Speed:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(cpu_fan_s, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  GPU Core Temp:      ", Style::default().fg(Color::DarkGray)),
            Span::styled(gpu_temp_s, Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::styled("       GPU Fan Speed:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(gpu_fan_s, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  Telemetry Interval: ", Style::default().fg(Color::DarkGray)),
            Span::styled("1.0 s event tick (sysfs/hwmon-direct)", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let thermal_widget = Paragraph::new(thermal_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(Span::styled(" Live Telemetry ", Style::default().add_modifier(Modifier::BOLD))),
    );
    f.render_widget(thermal_widget, chunks[1]);

    // Block 3: Quick Controls Guide
    let guide_lines = vec![
        Line::from(vec![
            Span::styled("  Tab / 1-6   ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("Switch between tabs"),
        ]),
        Line::from(vec![
            Span::styled("  p           ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("Quick-cycle Platform Profile (Quiet -> Balanced -> Performance)"),
        ]),
        Line::from(vec![
            Span::styled("  ?           ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("Show complete keyboard shortcuts help overlay"),
        ]),
        Line::from(vec![
            Span::styled("  q / Esc     ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("Exit rogtui"),
        ]),
    ];

    let guide_widget = Paragraph::new(guide_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(Span::styled(" Quick Actions ", Style::default().add_modifier(Modifier::BOLD))),
    );
    f.render_widget(guide_widget, chunks[2]);
}
