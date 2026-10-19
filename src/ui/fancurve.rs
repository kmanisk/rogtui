use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::Marker,
    text::{Line, Span},
    widgets::{Axis, Chart, Dataset, GraphType, Paragraph},
    Frame,
};

use crate::app::AppState;
use crate::backend::Backend;
use crate::ui::widgets::panel_block;

pub fn render_fancurve<B: Backend>(f: &mut Frame, app: &AppState<B>, area: Rect) {
    let theme = &app.theme;

    let active_curve = app.fan_curves.get(app.active_fan_idx);
    let fan_name = active_curve.map(|c| c.fan_name.as_str()).unwrap_or("Fan");
    let is_custom = active_curve.map(|c| c.enabled).unwrap_or(false);
    let is_modified = app.is_fan_curve_modified();

    let preset_name = match app.current_preset_idx {
        0 => "Silent",
        1 => "Balanced",
        _ => "Aggressive",
    };

    let is_monotonic = active_curve.map(|c| c.check_monotonicity().is_ok()).unwrap_or(true);

    // Layout: Top Selector (1), Optional Warning Line (0 or 1), Chart Area (Min), Readout (1)
    let warn_height = if !is_monotonic { 1 } else { 0 };
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),           // Profile / Fan / Preset row
            Constraint::Length(warn_height), // Non-monotonic warning
            Constraint::Min(12),             // Chart & optional Table
            Constraint::Length(1),           // Readout & live operating point
        ])
        .split(area);

    // 1. Profile selector row
    let mut top_spans = vec![
        Span::styled("Profile: ", Style::default().fg(theme.dim)),
        Span::styled(
            format!("[ {} ]", app.platform_profile.as_str()),
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled("Fan: ", Style::default().fg(theme.dim)),
        Span::styled(
            format!("[ {} ({}/{}) ]", fan_name, app.active_fan_idx + 1, app.fan_curves.len()),
            Style::default().fg(theme.warn).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled("Preset: ", Style::default().fg(theme.dim)),
        Span::styled(
            format!("[ {} ]", preset_name),
            Style::default().fg(theme.fg),
        ),
        Span::raw("  "),
        Span::styled("Custom: ", Style::default().fg(theme.dim)),
        Span::styled(
            if is_custom { "[ On ]" } else { "[ Off ]" },
            if is_custom { Style::default().fg(theme.ok) } else { Style::default().fg(theme.dim) },
        ),
    ];

    if is_modified {
        top_spans.push(Span::raw("  "));
        top_spans.push(Span::styled(
            "[*] Unsaved changes",
            Style::default().fg(theme.warn).add_modifier(Modifier::BOLD),
        ));
    }
    if app.apply_all_fans {
        top_spans.push(Span::raw("  "));
        top_spans.push(Span::styled(
            "[All fans linked]",
            Style::default().fg(theme.accent),
        ));
    }

    f.render_widget(Paragraph::new(Line::from(top_spans)), main_chunks[0]);

    // Warning row if non-monotonic
    if !is_monotonic {
        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(
                    "! Non-monotonic curve detected. Press 'x' to repair.",
                    Style::default().fg(theme.warn).add_modifier(Modifier::BOLD),
                ),
            ])),
            main_chunks[1],
        );
    }

    // 2. Body: Chart and side Table (if width >= 100 or table toggled)
    let show_table = app.show_point_table || area.width >= 100;
    let body_chunks = if show_table {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(48), Constraint::Length(28)])
            .split(main_chunks[2])
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(100)])
            .split(main_chunks[2])
    };

    if let Some(curve) = active_curve {
        let applied_curve = app.applied_fan_curves.get(app.active_fan_idx);
        let other_fan_idx = if app.active_fan_idx == 0 { 1 } else { 0 };
        let other_curve = if app.show_other_fan_overlay {
            app.fan_curves.get(other_fan_idx)
        } else {
            None
        };

        // Live telemetry
        let live_temp = if app.active_fan_idx == 0 {
            app.sensors.cpu_temp_c.unwrap_or(50) as f64
        } else {
            app.sensors.gpu_temp_c.unwrap_or(45) as f64
        };
        let live_rpm = if app.active_fan_idx == 0 {
            app.sensors.cpu_fan_rpm.unwrap_or(0)
        } else {
            app.sensors.gpu_fan_rpm.unwrap_or(0)
        };
        let live_speed = curve.interpolate(live_temp);

        // High-resolution interpolated line coordinates for active curve (smooth Braille line)
        let mut curve_data: Vec<(f64, f64)> = Vec::with_capacity(141);
        for t in (300..=1000).step_by(5) {
            let temp = t as f64 / 10.0;
            curve_data.push((temp, curve.interpolate(temp)));
        }

        // Applied curve line if modified
        let mut applied_data: Vec<(f64, f64)> = Vec::new();
        if is_modified {
            if let Some(appl) = applied_curve {
                for t in (300..=1000).step_by(5) {
                    let temp = t as f64 / 10.0;
                    applied_data.push((temp, appl.interpolate(temp)));
                }
            }
        }

        // Other fan overlay
        let mut other_data: Vec<(f64, f64)> = Vec::new();
        if let Some(other) = other_curve {
            for t in (300..=1000).step_by(5) {
                let temp = t as f64 / 10.0;
                other_data.push((temp, other.interpolate(temp)));
            }
        }

        // Control points
        let point_coords: Vec<(f64, f64)> = curve
            .points
            .iter()
            .map(|p| (p.temp as f64, p.pwm_percent as f64))
            .collect();

        // Selected point
        let selected_coord: Vec<(f64, f64)> = curve
            .points
            .get(app.active_point_idx)
            .map(|p| vec![(p.temp as f64, p.pwm_percent as f64)])
            .unwrap_or_default();

        // Live vertical indicator line & crosshair
        let live_v_line: Vec<(f64, f64)> = vec![(live_temp, 0.0), (live_temp, 100.0)];
        let live_h_line: Vec<(f64, f64)> = vec![(30.0, live_speed), (live_temp, live_speed)];
        let live_pt: Vec<(f64, f64)> = vec![(live_temp, live_speed)];

        let mut datasets = Vec::new();

        // Other fan overlay dataset
        if !other_data.is_empty() {
            datasets.push(
                Dataset::default()
                    .marker(Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(ratatui::style::Color::Magenta))
                    .data(&other_data),
            );
        }

        // Applied curve dataset
        if !applied_data.is_empty() {
            datasets.push(
                Dataset::default()
                    .marker(Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(Color::Rgb(110, 120, 135)))
                    .data(&applied_data),
            );
        }

        // Active curve dataset
        datasets.push(
            Dataset::default()
                .marker(Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
                .data(&curve_data),
        );

        // Control points
        datasets.push(
            Dataset::default()
                .marker(Marker::Braille)
                .graph_type(GraphType::Scatter)
                .style(Style::default().fg(Color::White))
                .data(&point_coords),
        );

        // Live crosshair lines
        if (30.0..=100.0).contains(&live_temp) {
            datasets.push(
                Dataset::default()
                    .marker(Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(Color::Rgb(85, 95, 110)))
                    .data(&live_v_line),
            );
            datasets.push(
                Dataset::default()
                    .marker(Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(Color::Rgb(85, 95, 110)))
                    .data(&live_h_line),
            );
            // Live operating point
            datasets.push(
                Dataset::default()
                    .marker(Marker::Block)
                    .graph_type(GraphType::Scatter)
                    .style(Style::default().fg(theme.ok))
                    .data(&live_pt),
            );
        }

        // Selected point
        datasets.push(
            Dataset::default()
                .marker(Marker::Block)
                .graph_type(GraphType::Scatter)
                .style(Style::default().fg(theme.warn))
                .data(&selected_coord),
        );

        let chart_title = format!(
            " {} Fan Curve [Braille Hi-Res · Live: {:.0}°C / {:.0}% ({} RPM)] ",
            fan_name, live_temp, live_speed, live_rpm
        );
        let chart_block = panel_block(chart_title, true, theme);

        let chart = Chart::new(datasets)
            .block(chart_block)
            .x_axis(
                Axis::default()
                    .title("Temp (°C)")
                    .style(Style::default().fg(theme.dim))
                    .bounds([30.0, 100.0])
                    .labels(["30°C", "40°C", "50°C", "60°C", "70°C", "80°C", "90°C", "100°C"]),
            )
            .y_axis(
                Axis::default()
                    .title("Speed (%)")
                    .style(Style::default().fg(theme.dim))
                    .bounds([0.0, 100.0])
                    .labels(["0%", "20%", "40%", "60%", "80%", "100%"]),
            );

        f.render_widget(chart, body_chunks[0]);

        // Point Table
        if show_table {
            let tbl_block = panel_block(" 8-Point Calibration ", false, theme);
            let tbl_inner = tbl_block.inner(body_chunks[1]);
            f.render_widget(tbl_block, body_chunks[1]);

            let mut tbl_lines = Vec::new();
            tbl_lines.push(Line::from(vec![
                Span::styled("Pt ", Style::default().fg(theme.dim)),
                Span::styled("Temp   ", Style::default().fg(theme.dim)),
                Span::styled("Speed  ", Style::default().fg(theme.dim)),
                Span::styled("Status", Style::default().fg(theme.dim)),
            ]));

            for (idx, pt) in curve.points.iter().enumerate() {
                let is_sel = idx == app.active_point_idx;
                let is_pt_mod = app.active_point_is_modified(idx);

                let marker = if is_pt_mod { "*" } else { " " };
                let status = if is_pt_mod { "mod" } else { "ok" };

                let row_str = format!("{}{:<2} {:>3}°C  {:>3}%   {:<3}", if is_sel { ">" } else { " " }, format!("{}{}", idx + 1, marker), pt.temp, pt.pwm_percent, status);
                let w = tbl_inner.width as usize;
                let padded = if row_str.len() < w {
                    format!("{}{}", row_str, " ".repeat(w - row_str.len()))
                } else {
                    row_str
                };

                if is_sel {
                    tbl_lines.push(Line::from(Span::styled(padded, theme.sel_style())));
                } else if is_pt_mod {
                    tbl_lines.push(Line::from(Span::styled(padded, Style::default().fg(theme.warn))));
                } else {
                    tbl_lines.push(Line::from(Span::styled(padded, Style::default().fg(theme.fg))));
                }
            }

            f.render_widget(Paragraph::new(tbl_lines), tbl_inner);
        }

        // 3. Readout row at bottom
        let (p_temp, p_speed) = if let Some(p) = curve.points.get(app.active_point_idx) {
            (p.temp, p.pwm_percent)
        } else {
            (0, 0)
        };
        let was_speed = applied_curve
            .and_then(|c| c.points.get(app.active_point_idx))
            .map(|p| p.pwm_percent);
        let delta_str = match was_speed {
            Some(w) if w != p_speed => format!(" (was {w}%)"),
            _ => String::new(),
        };

        let readout_spans = vec![
            Span::styled(
                format!("Point {}/8: ", app.active_point_idx + 1),
                Style::default().fg(theme.dim),
            ),
            Span::styled(
                format!("{}°C -> {}% PWM{}", p_temp, p_speed, delta_str),
                Style::default().fg(theme.warn).add_modifier(Modifier::BOLD),
            ),
            Span::raw("   |   "),
            Span::styled("Live Operating Point: ", Style::default().fg(theme.dim)),
            Span::styled(
                format!("{:.0}°C -> {:.0}% ({} RPM)", live_temp, live_speed, live_rpm),
                Style::default().fg(theme.ok).add_modifier(Modifier::BOLD),
            ),
        ];

        f.render_widget(Paragraph::new(Line::from(readout_spans)), main_chunks[3]);
    }
}
