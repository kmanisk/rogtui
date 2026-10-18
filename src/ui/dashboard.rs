use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Sparkline},
    Frame,
};

use crate::app::AppState;
use crate::backend::Backend;
use crate::ui::widgets::{panel_block, render_gauge};

pub fn render_dashboard<B: Backend>(f: &mut Frame, app: &AppState<B>, area: Rect) {
    let theme = &app.theme;

    // Below 100 columns: stack panels vertically; otherwise side-by-side
    let is_narrow = area.width < 100;
    let chunks = if is_narrow {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(10), Constraint::Min(10)])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(area)
    };

    // -------------------------------------------------------------------------
    // 1. System Panel (Left or Top)
    // -------------------------------------------------------------------------
    let sys_block = panel_block(" System ", false, theme);
    let sys_inner = sys_block.inner(chunks[0]);
    f.render_widget(sys_block, chunks[0]);

    let bar_width = (sys_inner.width.saturating_sub(30) as usize).clamp(10, 24);

    let profile_pct = match app.platform_profile {
        crate::backend::types::PlatformProfile::Quiet => 33,
        crate::backend::types::PlatformProfile::Balanced => 66,
        crate::backend::types::PlatformProfile::Performance => 100,
        crate::backend::types::PlatformProfile::Custom => 50,
    };
    let prof_color = theme.profile_color(app.platform_profile);

    let gpu_text = match app.gpu_mode {
        crate::backend::types::GpuMode::AsusMuxDgpu => {
            format!("AsusMuxDgpu ({})", app.gpu_power.as_str())
        }
        crate::backend::types::GpuMode::Hybrid => {
            format!("Hybrid ({})", app.gpu_power.as_str())
        }
        crate::backend::types::GpuMode::Integrated => "Integrated (dGPU Off)".to_string(),
        other => format!("{} ({})", other.as_str(), app.gpu_power.as_str()),
    };

    let bat_pct = app.sensors.battery_charge_percent.unwrap_or(0);
    let bat_val = format!("{bat_pct}% (limit: {}%)", app.charge_limit);

    let sys_lines = vec![
        render_gauge(
            "Platform Profile",
            app.platform_profile.as_str(),
            profile_pct,
            bar_width,
            prof_color,
            theme,
        ),
        Line::from(vec![
            Span::styled("AC Profile Bind : ", Style::default().fg(theme.dim)),
            Span::styled(
                app.profile_ac.as_str(),
                Style::default().fg(theme.profile_color(app.profile_ac)),
            ),
            Span::raw("   "),
            Span::styled("BAT Profile: ", Style::default().fg(theme.dim)),
            Span::styled(
                app.profile_battery.as_str(),
                Style::default().fg(theme.profile_color(app.profile_battery)),
            ),
        ]),
        Line::from(vec![
            Span::styled("GPU State       : ", Style::default().fg(theme.dim)),
            Span::styled(
                gpu_text,
                Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
            ),
        ]),
        render_gauge(
            "Battery Charge",
            &bat_val,
            bat_pct as u32,
            bar_width,
            theme.ok,
            theme,
        ),
        render_gauge(
            "Charge Limit",
            &format!("{}%", app.charge_limit),
            app.charge_limit as u32,
            bar_width,
            theme.accent,
            theme,
        ),
        Line::from(vec![
            Span::styled("Auto-apply AC   : ", Style::default().fg(theme.dim)),
            Span::styled(
                if app.auto_ac { "[x] Yes" } else { "[ ] No" },
                Style::default().fg(theme.fg),
            ),
            Span::raw("   "),
            Span::styled("Auto BAT: ", Style::default().fg(theme.dim)),
            Span::styled(
                if app.auto_battery { "[x] Yes" } else { "[ ] No" },
                Style::default().fg(theme.fg),
            ),
        ]),
    ];

    f.render_widget(Paragraph::new(sys_lines), sys_inner);

    // -------------------------------------------------------------------------
    // 2. Thermals Panel (Right or Bottom)
    // -------------------------------------------------------------------------
    let therm_block = panel_block(" Thermals ", false, theme);
    let therm_inner = therm_block.inner(chunks[1]);
    f.render_widget(therm_block, chunks[1]);

    // Layout inside thermals: Gauges with Sparklines
    let t_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // CPU Temp Gauge
            Constraint::Length(1), // CPU Temp Sparkline
            Constraint::Length(1), // CPU Fan Gauge
            Constraint::Length(1), // Spacing
            Constraint::Length(1), // GPU Temp Gauge
            Constraint::Length(1), // GPU Temp Sparkline
            Constraint::Length(1), // GPU Fan Gauge
            Constraint::Min(0),
        ])
        .split(therm_inner);

    let t_bar_width = (therm_inner.width.saturating_sub(28) as usize).clamp(10, 26);

    // CPU Metrics
    let cpu_temp = app.sensors.cpu_temp_c.unwrap_or(0);
    let cpu_temp_val = if cpu_temp > 0 {
        format!("{cpu_temp} C")
    } else {
        "-- C".to_string()
    };
    let cpu_color = theme.temp_color(cpu_temp);
    f.render_widget(
        Paragraph::new(render_gauge(
            "CPU Temp",
            &cpu_temp_val,
            cpu_temp,
            t_bar_width,
            cpu_color,
            theme,
        )),
        t_rows[0],
    );

    // CPU Sparkline
    let cpu_history: Vec<u64> = app.cpu_temp_history.iter().copied().collect();
    let spark_cpu_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(18), // Label offset
            Constraint::Min(10),    // Sparkline width
        ])
        .split(t_rows[1]);
    f.render_widget(
        Paragraph::new(Span::styled("  History (60s) : ", Style::default().fg(theme.dim))),
        spark_cpu_chunks[0],
    );
    if !cpu_history.is_empty() {
        f.render_widget(
            Sparkline::default()
                .data(&cpu_history)
                .max(100)
                .style(Style::default().fg(cpu_color)),
            spark_cpu_chunks[1],
        );
    }

    // CPU Fan
    let cpu_rpm = app.sensors.cpu_fan_rpm.unwrap_or(0);
    let cpu_rpm_pct = (cpu_rpm * 100 / 6000).clamp(0, 100);
    let cpu_rpm_val = if cpu_rpm > 0 {
        format!("{cpu_rpm} RPM")
    } else {
        "0 RPM (Off)".to_string()
    };
    f.render_widget(
        Paragraph::new(render_gauge(
            "CPU Fan",
            &cpu_rpm_val,
            cpu_rpm_pct,
            t_bar_width,
            theme.fg,
            theme,
        )),
        t_rows[2],
    );

    // GPU Metrics
    let gpu_temp = app.sensors.gpu_temp_c.unwrap_or(0);
    let gpu_temp_val = if gpu_temp > 0 {
        format!("{gpu_temp} C")
    } else {
        "-- C".to_string()
    };
    let gpu_color = theme.temp_color(gpu_temp);
    f.render_widget(
        Paragraph::new(render_gauge(
            "GPU Temp",
            &gpu_temp_val,
            gpu_temp,
            t_bar_width,
            gpu_color,
            theme,
        )),
        t_rows[4],
    );

    // GPU Sparkline
    let gpu_history: Vec<u64> = app.gpu_temp_history.iter().copied().collect();
    let spark_gpu_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(18),
            Constraint::Min(10),
        ])
        .split(t_rows[5]);
    f.render_widget(
        Paragraph::new(Span::styled("  History (60s) : ", Style::default().fg(theme.dim))),
        spark_gpu_chunks[0],
    );
    if !gpu_history.is_empty() {
        f.render_widget(
            Sparkline::default()
                .data(&gpu_history)
                .max(100)
                .style(Style::default().fg(gpu_color)),
            spark_gpu_chunks[1],
        );
    }

    // GPU Fan
    let gpu_rpm = app.sensors.gpu_fan_rpm.unwrap_or(0);
    let gpu_rpm_pct = (gpu_rpm * 100 / 6000).clamp(0, 100);
    let gpu_rpm_val = if gpu_rpm > 0 {
        format!("{gpu_rpm} RPM")
    } else {
        "0 RPM (Off)".to_string()
    };
    f.render_widget(
        Paragraph::new(render_gauge(
            "GPU Fan",
            &gpu_rpm_val,
            gpu_rpm_pct,
            t_bar_width,
            theme.fg,
            theme,
        )),
        t_rows[6],
    );
}
