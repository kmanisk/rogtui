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
use crate::ui::widgets::{choice_widget, panel_block, toggle_widget};

pub fn render_fancurve<B: Backend>(f: &mut Frame, app: &AppState<B>, area: Rect) {
    let theme = &app.theme;

    let active_curve = app.fan_curves.get(app.active_fan_idx);
    let fan_name = active_curve.map(|c| c.fan_name.as_str()).unwrap_or("CPU");
    let is_custom = active_curve.map(|c| c.enabled).unwrap_or(false);

    let preset_name = match app.current_preset_idx {
        0 => "Silent",
        1 => "Balanced",
        _ => "Aggressive",
    };

    let is_monotonic = active_curve.map(|c| c.check_monotonicity().is_ok()).unwrap_or(true);

    // Layout: Clean Header (1), Warning if any (0 or 1), Chart + Table (Min), Readout (1)
    let warn_height = if !is_monotonic { 1 } else { 0 };
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),           // Header row
            Constraint::Length(warn_height), // Non-monotonic warning
            Constraint::Min(12),             // Chart & Table
            Constraint::Length(1),           // Bottom Readout
        ])
        .split(area);

    // 1. Clean Header: Fan ‹ CPU ›   Preset ‹ Balanced ›   Custom ● Off
    let mut header_spans = vec![
        Span::raw(" "),
        Span::styled("Fan ", Style::default().fg(theme.dim)),
    ];
    header_spans.extend(choice_widget(fan_name, false, theme));
    header_spans.push(Span::raw("     "));
    header_spans.push(Span::styled("Preset ", Style::default().fg(theme.dim)));
    header_spans.extend(choice_widget(preset_name, false, theme));
    header_spans.push(Span::raw("     "));
    header_spans.push(Span::styled("Custom ", Style::default().fg(theme.dim)));
    header_spans.extend(toggle_widget(is_custom, false, theme));

    if app.is_fan_curve_modified() {
        header_spans.push(Span::raw("     "));
        header_spans.push(Span::styled("Unsaved changes", Style::default().fg(theme.warn)));
    }

    f.render_widget(Paragraph::new(Line::from(header_spans)), main_chunks[0]);

    if !is_monotonic {
        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::raw(" "),
                Span::styled("Non-monotonic curve detected. Temperatures must increase with speed.", Style::default().fg(theme.err)),
            ])),
            main_chunks[1],
        );
    }

    // 2. Chart & Table
    let show_table = area.width >= 100;
    let body_chunks = if show_table {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(50), Constraint::Length(30)])
            .split(main_chunks[2])
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(100)])
            .split(main_chunks[2])
    };

    if let Some(curve) = active_curve {
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

        // Generate smooth curve coordinates passing exactly through the 8 control points
        let mut curve_data: Vec<(f64, f64)> = Vec::new();
        if let Some(first) = curve.points.first() {
            if (first.temp as f64) > 30.0 {
                curve_data.push((30.0, first.pwm_percent as f64));
            }
        }
        for i in 0..curve.points.len() - 1 {
            let p1 = &curve.points[i];
            let p2 = &curve.points[i + 1];
            let steps = ((p2.temp - p1.temp) * 2).max(1);
            for step in 0..steps {
                let t = p1.temp as f64 + (step as f64 / steps as f64) * (p2.temp - p1.temp) as f64;
                let pwm = p1.pwm_percent as f64 + (step as f64 / steps as f64) * (p2.pwm_percent as f64 - p1.pwm_percent as f64);
                curve_data.push((t, pwm));
            }
        }
        if let Some(last) = curve.points.last() {
            curve_data.push((last.temp as f64, last.pwm_percent as f64));
            if (last.temp as f64) < 100.0 {
                curve_data.push((100.0, last.pwm_percent as f64));
            }
        }

        // Unselected control points
        let other_points: Vec<(f64, f64)> = curve
            .points
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != app.active_point_idx)
            .map(|(_, p)| (p.temp as f64, p.pwm_percent as f64))
            .collect();

        // Selected control point
        let selected_point: Vec<(f64, f64)> = curve
            .points
            .get(app.active_point_idx)
            .map(|p| vec![(p.temp as f64, p.pwm_percent as f64)])
            .unwrap_or_default();

        // Live point & crosshairs
        let live_v_line: Vec<(f64, f64)> = vec![(live_temp, 0.0), (live_temp, 100.0)];
        let live_h_line: Vec<(f64, f64)> = vec![(30.0, live_speed), (live_temp, live_speed)];
        let live_pt: Vec<(f64, f64)> = vec![(live_temp, live_speed)];

        let mut datasets = Vec::new();

        // Active curve line
        datasets.push(
            Dataset::default()
                .marker(Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
                .data(&curve_data),
        );

        // Dim control points
        datasets.push(
            Dataset::default()
                .marker(Marker::Braille)
                .graph_type(GraphType::Scatter)
                .style(Style::default().fg(theme.dim))
                .data(&other_points),
        );

        // Selected point (highlighted in accent)
        datasets.push(
            Dataset::default()
                .marker(Marker::Block)
                .graph_type(GraphType::Scatter)
                .style(Style::default().fg(theme.accent))
                .data(&selected_point),
        );

        // Live crosshair lines and operating point
        if (30.0..=100.0).contains(&live_temp) {
            datasets.push(
                Dataset::default()
                    .marker(Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(Color::Rgb(70, 75, 85)))
                    .data(&live_v_line),
            );
            datasets.push(
                Dataset::default()
                    .marker(Marker::Braille)
                    .graph_type(GraphType::Line)
                    .style(Style::default().fg(Color::Rgb(70, 75, 85)))
                    .data(&live_h_line),
            );
            datasets.push(
                Dataset::default()
                    .marker(Marker::Block)
                    .graph_type(GraphType::Scatter)
                    .style(Style::default().fg(theme.ok))
                    .data(&live_pt),
            );
        }

        let chart = Chart::new(datasets)
            .block(panel_block(format!(" {fan_name} Fan Curve "), false, theme))
            .x_axis(
                Axis::default()
                    .title("°C")
                    .style(Style::default().fg(theme.dim))
                    .bounds([30.0, 100.0])
                    .labels(["30°C", "40°C", "50°C", "60°C", "70°C", "80°C", "90°C", "100°C"]),
            )
            .y_axis(
                Axis::default()
                    .title("%")
                    .style(Style::default().fg(theme.dim))
                    .bounds([0.0, 100.0])
                    .labels(["0%", "20%", "40%", "60%", "80%", "100%"]),
            );

        f.render_widget(chart, body_chunks[0]);

        // Point Table (right pane if space permits)
        if show_table {
            let tbl_block = panel_block(" Calibration ", false, theme);
            let tbl_inner = tbl_block.inner(body_chunks[1]);
            f.render_widget(tbl_block, body_chunks[1]);

            let mut tbl_lines = Vec::new();
            tbl_lines.push(Line::from(vec![
                Span::styled("   Pt     Temp    Speed", Style::default().fg(theme.dim)),
            ]));

            for (idx, pt) in curve.points.iter().enumerate() {
                let is_sel = idx == app.active_point_idx;
                let marker = if is_sel { "▌ " } else { "  " };

                // Validate safety: at or above 80C, speed must be >= 30%
                let is_invalid = pt.temp >= 80 && pt.pwm_percent < 30;

                let mut row_spans = vec![
                    Span::styled(marker, if is_sel { Style::default().fg(theme.accent) } else { Style::default() }),
                    Span::styled(format!("{:<3}", idx + 1), if is_sel { Style::default().fg(theme.fg).add_modifier(Modifier::BOLD) } else { Style::default().fg(theme.dim) }),
                    Span::styled(format!("{:>6}°C", pt.temp), if is_sel { Style::default().fg(theme.fg).add_modifier(Modifier::BOLD) } else { Style::default().fg(theme.fg) }),
                    Span::styled(format!("{:>7}%", pt.pwm_percent), if is_sel { Style::default().fg(theme.fg).add_modifier(Modifier::BOLD) } else { Style::default().fg(theme.fg) }),
                ];

                if is_invalid {
                    row_spans.push(Span::styled("  <!>", Style::default().fg(theme.err)));
                }

                tbl_lines.push(Line::from(row_spans));
            }

            f.render_widget(Paragraph::new(tbl_lines), tbl_inner);
        }

        // 3. Readout row at bottom
        let (p_temp, p_speed) = if let Some(p) = curve.points.get(app.active_point_idx) {
            (p.temp, p.pwm_percent)
        } else {
            (0, 0)
        };

        let readout_spans = vec![
            Span::raw(" "),
            Span::styled(
                format!("Point {}/8: ", app.active_point_idx + 1),
                Style::default().fg(theme.dim),
            ),
            Span::styled(
                format!("{}°C → {}%", p_temp, p_speed),
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
            ),
            Span::styled("   ·   ", Style::default().fg(Color::Rgb(70, 75, 85))),
            Span::styled("Operating: ", Style::default().fg(theme.dim)),
            Span::styled(
                format!("{:.0}°C → {:.0}% · {} rpm", live_temp, live_speed, live_rpm),
                Style::default().fg(theme.ok),
            ),
        ];

        f.render_widget(Paragraph::new(Line::from(readout_spans)), main_chunks[3]);
    }
}
