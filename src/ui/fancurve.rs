use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::AppState;
use crate::backend::Backend;
use crate::backend::types::FanCurve;
use crate::ui::theme::Theme;
use crate::ui::widgets::{choice_widget, section_rule, toggle_widget};

fn dot_to_bit(dx: usize, dy: usize) -> u8 {
    match (dx, dy) {
        (0, 0) => 0x01,
        (0, 1) => 0x02,
        (0, 2) => 0x04,
        (1, 0) => 0x08,
        (1, 1) => 0x10,
        (1, 2) => 0x20,
        (0, 3) => 0x40,
        (1, 3) => 0x80,
        _ => 0,
    }
}

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

    // Vertical layout:
    // Header (1)
    // Blank (1)
    // Warning if non-monotonic (0 or 1)
    // Body: Chart & Table (Min 14)
    // Blank or spacing (0 or 1)
    // Readout (1)
    let warn_height = if !is_monotonic { 1 } else { 0 };
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),           // Header row
            Constraint::Length(1),           // Blank space
            Constraint::Length(warn_height), // Non-monotonic warning
            Constraint::Min(14),             // Chart & Table
            Constraint::Length(1),           // Readout
        ])
        .split(area);

    // 1. Header: Fan ‹ CPU ›  f    Preset ‹ Balanced ›  s    Custom curve ● On  c
    let mut header_spans = vec![
        Span::raw("  "),
        Span::styled("Fan ", Style::default().fg(theme.dim)),
    ];
    header_spans.extend(choice_widget(fan_name, false, theme));
    header_spans.push(Span::styled("  f    ", Style::default().fg(theme.dim)));

    header_spans.push(Span::styled("Preset ", Style::default().fg(theme.dim)));
    header_spans.extend(choice_widget(preset_name, false, theme));
    header_spans.push(Span::styled("  s    ", Style::default().fg(theme.dim)));

    header_spans.push(Span::styled("Custom curve ", Style::default().fg(theme.dim)));
    header_spans.extend(toggle_widget(is_custom, false, theme));
    header_spans.push(Span::styled("  c", Style::default().fg(theme.dim)));

    if app.is_fan_curve_modified() {
        header_spans.push(Span::raw("    "));
        header_spans.push(Span::styled("Unsaved changes", Style::default().fg(theme.warn)));
    }

    f.render_widget(Paragraph::new(Line::from(header_spans)), main_chunks[0]);

    if !is_monotonic {
        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::raw("  "),
                Span::styled("Non-monotonic curve detected. Temperatures must increase with speed.", Style::default().fg(theme.err)),
            ])),
            main_chunks[2],
        );
    }

    // 2. Chart & Calibration Table
    let body_area = main_chunks[3];
    let show_table = body_area.width >= 80;
    let (chart_area, table_area) = if show_table {
        let table_w = 28.min(body_area.width.saturating_sub(50));
        let h_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(45), Constraint::Length(table_w)])
            .split(body_area);
        (h_chunks[0], Some(h_chunks[1]))
    } else {
        (body_area, None)
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

        // Render Chart
        render_fan_chart(f, chart_area, curve, app.active_point_idx, live_temp, live_speed, theme);

        // Render Table (if visible)
        if let Some(tbl_rect) = table_area {
            render_fan_table(f, tbl_rect, curve, app.active_point_idx, app.fan_selected_col, theme);
        }

        // 3. Readout row at bottom
        let (p_temp, p_speed) = if let Some(p) = curve.points.get(app.active_point_idx) {
            (p.temp, p.pwm_percent)
        } else {
            (0, 0)
        };

        let col_name = if app.fan_selected_col == 0 { "Speed" } else { "Temp" };

        let readout_spans = vec![
            Span::raw("  "),
            Span::styled(
                format!("Point {}/8: ", app.active_point_idx + 1),
                Style::default().fg(theme.dim),
            ),
            Span::styled(
                format!("{}°C → {}%", p_temp, p_speed),
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" (editing {})", col_name),
                Style::default().fg(theme.dim),
            ),
            Span::styled("   ·   ", Style::default().fg(Color::Rgb(70, 75, 85))),
            Span::styled("Operating: ", Style::default().fg(theme.dim)),
            Span::styled(
                format!("{:.0}°C → {:.0}% · {} rpm", live_temp, live_speed, live_rpm),
                Style::default().fg(theme.ok),
            ),
        ];

        f.render_widget(Paragraph::new(Line::from(readout_spans)), main_chunks[4]);
    }
}

fn render_fan_chart(
    f: &mut Frame,
    area: Rect,
    curve: &FanCurve,
    active_pt_idx: usize,
    live_temp: f64,
    live_speed: f64,
    theme: &Theme,
) {
    let label_w = 7;
    let plot_w = (area.width as usize).saturating_sub(label_w + 2).max(20);
    // 4k+1 height rule: use 17 if space permits, else 13
    let plot_h = if area.height >= 20 { 17 } else { 13 };

    // 1. Compute Braille curve
    let px_w = plot_w * 2;
    let px_h = plot_h * 4;
    let mut braille_grid = vec![vec![0u8; plot_w]; plot_h];

    let mut prev: Option<(usize, usize)> = None;
    for x_dot in 0..px_w {
        let temp = 30.0 + (x_dot as f64 / (px_w - 1) as f64) * 70.0;
        let pwm = curve.interpolate(temp).clamp(0.0, 100.0);
        let py = ((pwm / 100.0) * (px_h - 1) as f64).round() as usize;
        let py_inv = (px_h - 1).saturating_sub(py);

        if let Some((x0, y0)) = prev {
            let steps = x_dot.abs_diff(x0).max(py_inv.abs_diff(y0)).max(1);
            for s in 0..=steps {
                let cur_x = x0 + (x_dot.saturating_sub(x0) * s) / steps;
                let cur_y = if py_inv >= y0 {
                    y0 + ((py_inv - y0) * s) / steps
                } else {
                    y0 - ((y0 - py_inv) * s) / steps
                };
                let cx = (cur_x / 2).min(plot_w - 1);
                let cy = (cur_y / 4).min(plot_h - 1);
                braille_grid[cy][cx] |= dot_to_bit(cur_x % 2, cur_y % 4);
            }
        }
        prev = Some((x_dot, py_inv));
    }

    // 2. Character & Style overlay grid
    let mut char_grid: Vec<Vec<char>> = vec![vec![' '; plot_w]; plot_h];
    let mut style_grid: Vec<Vec<Style>> = vec![vec![Style::default(); plot_w]; plot_h];

    // Populate Braille into grids
    for y in 0..plot_h {
        for x in 0..plot_w {
            let mask = braille_grid[y][x];
            if mask > 0 {
                char_grid[y][x] = char::from_u32(0x2800 + mask as u32).unwrap_or(' ');
                style_grid[y][x] = Style::default().fg(theme.accent);
            }
        }
    }

    // 3. Place 8 curve points with '●'
    for (idx, pt) in curve.points.iter().enumerate() {
        let col = (((pt.temp as f64 - 30.0) / 70.0) * (plot_w - 1) as f64).round() as usize;
        let col = col.min(plot_w - 1);
        let row_f = (pt.pwm_percent as f64 / 100.0) * (plot_h - 1) as f64;
        let row = (plot_h - 1).saturating_sub(row_f.round() as usize);

        char_grid[row][col] = '●';
        if idx == active_pt_idx {
            style_grid[row][col] = Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD);
        } else {
            style_grid[row][col] = Style::default().fg(theme.dim);
        }
    }

    // 4. Place live operating point with '◆'
    if (30.0..=100.0).contains(&live_temp) {
        let live_col = (((live_temp - 30.0) / 70.0) * (plot_w - 1) as f64).round() as usize;
        let live_col = live_col.min(plot_w - 1);
        let row_f = (live_speed / 100.0) * (plot_h - 1) as f64;
        let live_row = (plot_h - 1).saturating_sub(row_f.round() as usize);

        char_grid[live_row][live_col] = '◆';
        style_grid[live_row][live_col] = Style::default().fg(theme.ok).add_modifier(Modifier::BOLD);
    }

    // 5. Build lines with Y-Axis labels (0%, 25%, 50%, 75%, 100% on exact rows)
    let step = (plot_h - 1) / 4;
    let mut lines = Vec::new();

    for r in 0..plot_h {
        let y_label = if r == 0 {
            " 100% ┤"
        } else if r == step {
            "  75% ┤"
        } else if r == step * 2 {
            "  50% ┤"
        } else if r == step * 3 {
            "  25% ┤"
        } else if r == plot_h - 1 {
            "   0% ┴"
        } else {
            "      │"
        };

        let mut row_spans = vec![
            Span::styled(y_label, Style::default().fg(Color::Rgb(70, 75, 85))),
        ];

        for c in 0..plot_w {
            let ch = char_grid[r][c];
            let st = style_grid[r][c];
            row_spans.push(Span::styled(ch.to_string(), st));
        }

        lines.push(Line::from(row_spans));
    }

    // 6. X-Axis horizontal line with exact tick marks
    let ticks = [30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0];
    let tick_cols: Vec<usize> = ticks
        .iter()
        .map(|&t| (((t - 30.0) / 70.0) * (plot_w - 1) as f64).round() as usize)
        .collect();

    let mut axis_chars = vec!['─'; plot_w];
    for &tc in &tick_cols {
        if tc < plot_w {
            axis_chars[tc] = '┬';
        }
    }
    let axis_str: String = axis_chars.into_iter().collect();
    lines.push(Line::from(vec![
        Span::styled("      └", Style::default().fg(Color::Rgb(70, 75, 85))),
        Span::styled(axis_str, Style::default().fg(Color::Rgb(70, 75, 85))),
    ]));

    // 7. X-Axis Tick Labels placed using the EXACT same formula
    let mut label_chars = vec![' '; plot_w + 4];
    for (idx, &t) in ticks.iter().enumerate() {
        let tc = tick_cols[idx];
        let text = if t == 100.0 {
            "100°C"
        } else {
            match t as u32 {
                30 => "30",
                40 => "40",
                50 => "50",
                60 => "60",
                70 => "70",
                80 => "80",
                90 => "90",
                _ => "",
            }
        };

        let len = text.len();
        let start_pos = if t == 30.0 {
            tc
        } else if t == 100.0 {
            (tc + 1).saturating_sub(len)
        } else {
            tc.saturating_sub(len / 2)
        };

        for (ch_i, c) in text.chars().enumerate() {
            if start_pos + ch_i < label_chars.len() {
                label_chars[start_pos + ch_i] = c;
            }
        }
    }

    let labels_str: String = label_chars.into_iter().collect();
    lines.push(Line::from(vec![
        Span::raw("       "),
        Span::styled(labels_str, Style::default().fg(theme.dim)),
    ]));

    f.render_widget(Paragraph::new(lines), area);
}

fn render_fan_table(
    f: &mut Frame,
    area: Rect,
    curve: &FanCurve,
    active_pt_idx: usize,
    selected_col: usize,
    theme: &Theme,
) {
    let width = area.width as usize;
    let mut lines = Vec::new();

    // Section rule: ── Points ──
    lines.push(section_rule("Points", width, theme));

    // Header row: Point   Temp   Speed
    let temp_hdr_style = if selected_col == 1 {
        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.dim)
    };
    let speed_hdr_style = if selected_col == 0 {
        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.dim)
    };

    lines.push(Line::from(vec![
        Span::styled("  Point    ", Style::default().fg(theme.dim)),
        Span::styled("Temp    ", temp_hdr_style),
        Span::styled("Speed", speed_hdr_style),
    ]));

    for (idx, pt) in curve.points.iter().enumerate() {
        let is_sel = idx == active_pt_idx;
        let gutter = if is_sel { "▌ " } else { "  " };
        let is_invalid = pt.temp >= 80 && pt.pwm_percent < 30;

        let num_str = format!("{:>5}    ", idx + 1);

        let (temp_style, speed_style) = if is_sel {
            if selected_col == 1 {
                (
                    Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                    Style::default().fg(theme.fg),
                )
            } else {
                (
                    Style::default().fg(theme.fg),
                    Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                )
            }
        } else {
            (
                Style::default().fg(theme.dim),
                Style::default().fg(theme.dim),
            )
        };

        let temp_str = if is_sel && selected_col == 1 {
            format!("‹{:>3}°C›  ", pt.temp)
        } else {
            format!("{:>4}°C    ", pt.temp)
        };

        let speed_str = if is_sel && selected_col == 0 {
            format!("‹{:>3}%›", pt.pwm_percent)
        } else {
            format!("{:>4}% ", pt.pwm_percent)
        };

        let mut row_spans = vec![
            Span::styled(
                gutter,
                if is_sel {
                    Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                },
            ),
            Span::styled(num_str, if is_sel { Style::default().fg(theme.fg).add_modifier(Modifier::BOLD) } else { Style::default().fg(theme.dim) }),
            Span::styled(temp_str, temp_style),
            Span::styled(speed_str, speed_style),
        ];

        if is_invalid {
            row_spans.push(Span::styled(" <!>", Style::default().fg(theme.err).add_modifier(Modifier::BOLD)));
        }

        lines.push(Line::from(row_spans));
    }

    f.render_widget(Paragraph::new(lines), area);
}
