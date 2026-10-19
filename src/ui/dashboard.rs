use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
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
            .constraints([Constraint::Percentage(52), Constraint::Percentage(48)])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area)
    };

    // -------------------------------------------------------------------------
    // 1. Hardware Monitor & Thermals (Left Pane)
    // -------------------------------------------------------------------------
    let hw_block = panel_block(" Hardware Monitor ", false, theme);
    let hw_inner = hw_block.inner(chunks[0]);
    f.render_widget(hw_block, chunks[0]);

    let bar_width = (hw_inner.width.saturating_sub(28) as usize).clamp(8, 20);

    // CPU Metrics
    let cpu_model = if app.sensors.cpu_model.is_empty() {
        "Intel Core i5-13450HX"
    } else {
        app.sensors.cpu_model.as_str()
    };
    let cpu_freq_str = if let Some(mhz) = app.sensors.cpu_freq_mhz {
        format!("{mhz} MHz")
    } else {
        "-- MHz".to_string()
    };

    let cpu_usage = app.sensors.cpu_usage_percent.unwrap_or(0);
    let cpu_usage_str = format!("{cpu_usage}%");

    let cpu_temp = app.sensors.cpu_temp_c.unwrap_or(0);
    let cpu_temp_val = if cpu_temp > 0 {
        format!("{cpu_temp} C")
    } else {
        "-- C".to_string()
    };
    let cpu_color = theme.temp_color(cpu_temp);

    // RAM Metrics
    let ram_used = app.sensors.ram_used_mb.unwrap_or(0);
    let ram_total = app.sensors.ram_total_mb.unwrap_or(1);
    let ram_pct = app.sensors.ram_usage_percent.unwrap_or(0);
    let ram_str = format!("{ram_used}/{ram_total} MB ({ram_pct}%)");

    // Fan Metrics
    let cpu_rpm = app.sensors.cpu_fan_rpm.unwrap_or(0);
    let cpu_rpm_pct = (cpu_rpm * 100 / 6000).clamp(0, 100);
    let cpu_rpm_val = if cpu_rpm > 0 { format!("{cpu_rpm} RPM") } else { "0 RPM (Off)".to_string() };

    let gpu_rpm = app.sensors.gpu_fan_rpm.unwrap_or(0);
    let gpu_rpm_pct = (gpu_rpm * 100 / 6000).clamp(0, 100);
    let gpu_rpm_val = if gpu_rpm > 0 { format!("{gpu_rpm} RPM") } else { "0 RPM (Off)".to_string() };

    // dGPU status & temp
    let gpu_temp = app.sensors.gpu_temp_c.unwrap_or(0);
    let gpu_temp_val = if gpu_temp > 0 {
        format!("{gpu_temp} C")
    } else {
        "-- C".to_string()
    };
    let gpu_color = theme.temp_color(gpu_temp);

    let dgpu_status_str = if app.sensors.dgpu_status.is_empty() {
        "Guarded (D3cold)".to_string()
    } else {
        app.sensors.dgpu_status.clone()
    };

    let hw_rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // CPU Model & Freq
            Constraint::Length(1), // CPU Usage
            Constraint::Length(1), // CPU Temp
            Constraint::Length(1), // CPU Fan RPM
            Constraint::Length(1), // RAM Usage
            Constraint::Length(1), // dGPU status / model
            Constraint::Length(1), // GPU Temp
            Constraint::Length(1), // GPU Fan RPM
            Constraint::Min(0),
        ])
        .split(hw_inner);

    // Row 0: CPU Model & Freq
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("  CPU           : ", Style::default().fg(theme.dim)),
            Span::styled(cpu_model, Style::default().fg(theme.fg).add_modifier(Modifier::BOLD)),
            Span::raw(" "),
            Span::styled(format!("({cpu_freq_str})"), Style::default().fg(theme.accent)),
        ])),
        hw_rows[0],
    );

    // Row 1: CPU Usage
    f.render_widget(
        Paragraph::new(render_gauge("CPU Usage", &cpu_usage_str, cpu_usage as u32, bar_width, theme.accent, theme)),
        hw_rows[1],
    );

    // Row 2: CPU Temp
    f.render_widget(
        Paragraph::new(render_gauge("CPU Temp", &cpu_temp_val, cpu_temp, bar_width, cpu_color, theme)),
        hw_rows[2],
    );

    // Row 3: CPU Fan
    f.render_widget(
        Paragraph::new(render_gauge("CPU Fan", &cpu_rpm_val, cpu_rpm_pct, bar_width, theme.fg, theme)),
        hw_rows[3],
    );

    // Row 4: RAM
    f.render_widget(
        Paragraph::new(render_gauge("Memory (RAM)", &ram_str, ram_pct as u32, bar_width, theme.ok, theme)),
        hw_rows[4],
    );

    // Row 5: dGPU status
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("  dGPU (RTX 5050): ", Style::default().fg(theme.dim)),
            Span::styled(
                dgpu_status_str,
                if app.sensors.dgpu_status.contains("Active") {
                    Style::default().fg(theme.ok).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.dim)
                },
            ),
        ])),
        hw_rows[5],
    );

    // Row 6: GPU Temp
    f.render_widget(
        Paragraph::new(render_gauge("GPU Temp", &gpu_temp_val, gpu_temp, bar_width, gpu_color, theme)),
        hw_rows[6],
    );

    // Row 7: GPU Fan
    f.render_widget(
        Paragraph::new(render_gauge("GPU Fan", &gpu_rpm_val, gpu_rpm_pct, bar_width, theme.fg, theme)),
        hw_rows[7],
    );

    // -------------------------------------------------------------------------
    // 2. System & Platform Control (Right Pane)
    // -------------------------------------------------------------------------
    let sys_block = panel_block(" System Control ", false, theme);
    let sys_inner = sys_block.inner(chunks[1]);
    f.render_widget(sys_block, chunks[1]);

    let prof_color = theme.profile_color(app.platform_profile);

    let bat_pct = app.sensors.battery_charge_percent.unwrap_or(0);
    let bat_val = format!("{bat_pct}% (limit: {}%)", app.charge_limit);

    let bat_status = if app.sensors.battery_status.is_empty() {
        "Not charging"
    } else {
        app.sensors.battery_status.as_str()
    };
    let bat_health = app.sensors.battery_health_percent.unwrap_or(98);
    let bat_power_str = if let Some(w) = app.sensors.battery_power_w {
        if w > 0.05 {
            format!("{w:.1} W")
        } else {
            "AC Power".to_string()
        }
    } else {
        "AC Power".to_string()
    };
    let bat_cycles_str = match app.sensors.battery_cycle_count {
        Some(c) if c > 0 => format!("{c}"),
        _ => "n/a".to_string(),
    };

    let overdrive_active = app.sensors.panel_overdrive.unwrap_or(true);
    let auto_bright_active = app.sensors.screen_auto_brightness.unwrap_or(false);

    let sys_lines = vec![
        Line::from(vec![
            Span::styled("  Platform Profile  : ", Style::default().fg(theme.dim)),
            Span::styled(
                format!(" < {} > ", app.platform_profile.as_str()),
                Style::default().bg(prof_color).fg(theme.sel_fg).add_modifier(Modifier::BOLD),
            ),
            Span::raw("   "),
            Span::styled("[p: cycle]", Style::default().fg(theme.dim)),
        ]),
        Line::from(vec![
            Span::styled("  Advanced Tuning   : ", Style::default().fg(theme.dim)),
            Span::styled("[Press 'a' for Power Limits Modal]", Style::default().fg(theme.accent)),
        ]),
        Line::from(vec![
            Span::styled("  AC Profile Bind : ", Style::default().fg(theme.dim)),
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
            Span::styled("  Auto-apply AC   : ", Style::default().fg(theme.dim)),
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
        render_gauge(
            "Battery Charge",
            &bat_val,
            bat_pct as u32,
            bar_width,
            theme.ok,
            theme,
        ),
        Line::from(vec![
            Span::styled("  Battery Telemetry: ", Style::default().fg(theme.dim)),
            Span::styled(format!("{bat_status} | Health: {bat_health}% | Cycles: {bat_cycles_str} | Draw: {bat_power_str}"), Style::default().fg(theme.fg)),
        ]),
        render_gauge(
            "Charge Limit",
            &format!("{}%", app.charge_limit),
            app.charge_limit as u32,
            bar_width,
            theme.accent,
            theme,
        ),
        Line::from(""),
        Line::from(Span::styled("  -- Armoury Quick Toggles --", Style::default().fg(theme.dim))),
        Line::from(vec![
            Span::styled("  Panel Overdrive  [o]: ", Style::default().fg(theme.dim)),
            Span::styled(
                if overdrive_active { "[x] Enabled (3ms)" } else { "[ ] Disabled" },
                if overdrive_active { Style::default().fg(theme.ok).add_modifier(Modifier::BOLD) } else { Style::default().fg(theme.dim) },
            ),
        ]),
        Line::from(vec![
            Span::styled("  Auto-Brightness  [b]: ", Style::default().fg(theme.dim)),
            Span::styled(
                if auto_bright_active { "[x] Enabled (ALS)" } else { "[ ] Disabled" },
                if auto_bright_active { Style::default().fg(theme.ok).add_modifier(Modifier::BOLD) } else { Style::default().fg(theme.dim) },
            ),
        ]),
    ];

    f.render_widget(Paragraph::new(sys_lines), sys_inner);

    // -------------------------------------------------------------------------
    // 3. Advanced Power Limits Modal (Rendered on 'a')
    // -------------------------------------------------------------------------
    if app.show_advanced_power_modal {
        render_advanced_power_modal(f, area, theme);
    }
}

fn render_advanced_power_modal(f: &mut Frame, area: Rect, theme: &crate::ui::theme::Theme) {
    let modal_w = 72.min(area.width.saturating_sub(4));
    let modal_h = 12.min(area.height.saturating_sub(2));
    let x = (area.width.saturating_sub(modal_w)) / 2;
    let y = (area.height.saturating_sub(modal_h)) / 2;
    let modal_area = Rect::new(x, y, modal_w, modal_h);

    f.render_widget(Clear, modal_area);
    let block = panel_block(" Advanced Power Limits (SPL / sPPT / TGP) ", false, theme);
    let inner = block.inner(modal_area);
    f.render_widget(block, modal_area);

    let pl1_w = std::fs::read_to_string("/sys/class/powercap/intel-rapl:0/constraint_0_power_limit_uw")
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .map(|uw| uw / 1_000_000)
        .unwrap_or(100);
    let pl2_w = std::fs::read_to_string("/sys/class/powercap/intel-rapl:0/constraint_1_power_limit_uw")
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .map(|uw| uw / 1_000_000)
        .unwrap_or(135);
    let peak_w = std::fs::read_to_string("/sys/class/powercap/intel-rapl:0/constraint_2_power_limit_uw")
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())
        .map(|uw| uw / 1_000_000)
        .unwrap_or(195);

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  PL1 (Long-Term SPL)  : ", Style::default().fg(theme.dim)),
            Span::styled(format!("{pl1_w} W "), Style::default().fg(theme.fg).add_modifier(Modifier::BOLD)),
            Span::styled("(read-only Intel RAPL)", Style::default().fg(theme.dim)),
        ]),
        Line::from(vec![
            Span::styled("  PL2 (Short-Term sPPT) : ", Style::default().fg(theme.dim)),
            Span::styled(format!("{pl2_w} W "), Style::default().fg(theme.fg).add_modifier(Modifier::BOLD)),
            Span::styled("(read-only Intel RAPL)", Style::default().fg(theme.dim)),
        ]),
        Line::from(vec![
            Span::styled("  Peak Power Limit     : ", Style::default().fg(theme.dim)),
            Span::styled(format!("{peak_w} W"), Style::default().fg(theme.fg).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(Span::styled("  Firmware Attributes   : Mutable sliders unavailable via ASUS WMI on FX608JH", Style::default().fg(theme.warn))),
        Line::from(""),
        Line::from(Span::styled("  [Press Esc or Enter to dismiss]", Style::default().fg(theme.accent))),
    ];
    f.render_widget(Paragraph::new(lines), inner);
}
