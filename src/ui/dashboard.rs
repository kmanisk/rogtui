use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::AppState;
use crate::backend::Backend;
use crate::ui::theme::Theme;
use crate::ui::widgets::{choice_widget, grid_row, section_rule};

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

fn plot_single_series(history: &[u64], plot_w: usize, plot_h: usize) -> Vec<Vec<u8>> {
    let px_w = plot_w * 2;
    let px_h = plot_h * 4;
    let mut grid = vec![vec![0u8; plot_w]; plot_h];

    if history.is_empty() || plot_w == 0 || plot_h == 0 {
        return grid;
    }

    let count = history.len();
    let mut prev: Option<(usize, usize)> = None;

    for (i, &temp) in history.iter().enumerate() {
        if temp == 0 {
            prev = None;
            continue;
        }
        let px = if count <= 1 {
            0
        } else {
            (i * (px_w.saturating_sub(1))) / (count - 1)
        };
        // Normalize between 30°C and 90°C (60°C range across 12 vertical dots = exactly 5°C/dot)
        let t_norm = ((temp as f32 - 30.0) / 60.0).clamp(0.0, 1.0);
        let py = (t_norm * (px_h - 1) as f32).round() as usize;
        let py_inv = (px_h - 1).saturating_sub(py);

        if let Some((x0, y0)) = prev {
            let steps = px.abs_diff(x0).max(py_inv.abs_diff(y0)).max(1);
            for s in 0..=steps {
                let cur_x = x0 + (px.saturating_sub(x0) * s) / steps;
                let cur_y = if py_inv >= y0 {
                    y0 + ((py_inv - y0) * s) / steps
                } else {
                    y0 - ((y0 - py_inv) * s) / steps
                };
                let cx = (cur_x / 2).min(plot_w.saturating_sub(1));
                let cy = (cur_y / 4).min(plot_h.saturating_sub(1));
                let dx = cur_x % 2;
                let dy = cur_y % 4;
                grid[cy][cx] |= dot_to_bit(dx, dy);
            }
        } else {
            let cx = (px / 2).min(plot_w.saturating_sub(1));
            let cy = (py_inv / 4).min(plot_h.saturating_sub(1));
            let dx = px % 2;
            let dy = py_inv % 4;
            grid[cy][cx] |= dot_to_bit(dx, dy);
        }
        prev = Some((px, py_inv));
    }

    grid
}

struct TempSeriesInfo<'a> {
    label: &'a str,
    dot_color: Color,
    history: &'a [u64],
    temp: u32,
    rpm: u32,
    is_idle: bool,
}

fn format_temp_legend(
    info: &TempSeriesInfo,
    col_w: usize,
    theme: &Theme,
) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    spans.push(Span::raw("  "));
    spans.push(Span::styled("● ", Style::default().fg(info.dot_color)));

    if info.is_idle && info.temp == 0 {
        spans.push(Span::styled(format!("{} Idle", info.label), Style::default().fg(theme.dim)));
        let cur_len = 2 + 2 + info.label.len() + 5;
        if col_w >= cur_len + 10 {
            spans.push(Span::raw("   "));
            spans.push(Span::styled("D3cold", Style::default().fg(theme.dim)));
        }
    } else {
        spans.push(Span::styled(
            format!("{} {}°C", info.label, info.temp),
            Style::default().fg(theme.temp_color(info.temp)).add_modifier(Modifier::BOLD),
        ));

        let peak = info.history.iter().copied().max().unwrap_or(info.temp as u64).max(info.temp as u64) as u32;
        if col_w >= 28 {
            spans.push(Span::raw("   "));
            spans.push(Span::styled(format!("Peak {peak}°C"), Style::default().fg(theme.dim)));
        }

        if col_w >= 39 && info.rpm > 0 {
            spans.push(Span::raw("   "));
            spans.push(Span::styled(format!("{} rpm", info.rpm), Style::default().fg(theme.dim)));
        }
    }

    let len: usize = spans.iter().map(|s| s.content.chars().count()).sum();
    if len < col_w {
        spans.push(Span::raw(" ".repeat(col_w - len)));
    }
    spans
}

fn render_two_temp_charts(
    cpu: &TempSeriesInfo,
    gpu: &TempSeriesInfo,
    width: usize,
    theme: &Theme,
) -> Vec<Line<'static>> {
    let gap = if width >= 90 { 4 } else { 2 };
    let left_col_w = (width.saturating_sub(gap)) / 2;
    let right_col_w = width.saturating_sub(left_col_w + gap);

    let plot_h = 3;
    let label_w = 8;
    let plot_w_left = left_col_w.saturating_sub(label_w);
    let plot_w_right = right_col_w.saturating_sub(label_w);

    let cpu_grid = plot_single_series(cpu.history, plot_w_left, plot_h);
    let gpu_grid = plot_single_series(gpu.history, plot_w_right, plot_h);

    let mut lines = Vec::new();

    // 1. Legend Row
    let mut legend_spans = format_temp_legend(cpu, left_col_w, theme);
    legend_spans.push(Span::raw(" ".repeat(gap)));
    let right_legend_spans = format_temp_legend(gpu, right_col_w, theme);
    legend_spans.extend(right_legend_spans);
    lines.push(Line::from(legend_spans));

    // 2. Braille Rows (8 chars: 2 spaces indent + 5 chars temp + 1 char pipe)
    let y_labels = ["  90°C │", "  60°C │", "  30°C │"];
    let gpu_is_idle = gpu.is_idle && (gpu.temp == 0 || gpu.history.iter().all(|&t| t == 0));

    for row in 0..plot_h {
        let mut row_spans = Vec::new();

        // Left chart (CPU)
        row_spans.push(Span::styled(y_labels[row], Style::default().fg(theme.dim)));
        for &bits in cpu_grid[row].iter().take(plot_w_left) {
            if bits > 0 {
                let ch = char::from_u32(0x2800 + bits as u32).unwrap_or(' ');
                row_spans.push(Span::styled(ch.to_string(), Style::default().fg(theme.accent)));
            } else {
                row_spans.push(Span::raw(" "));
            }
        }

        // Gap
        row_spans.push(Span::raw(" ".repeat(gap)));

        // Right chart (GPU)
        row_spans.push(Span::styled(y_labels[row], Style::default().fg(theme.dim)));
        if gpu_is_idle {
            if row == 1 {
                let msg = "[ dGPU Suspended in D3cold ]";
                let msg_len = msg.chars().count();
                if plot_w_right >= msg_len {
                    let left_pad = (plot_w_right - msg_len) / 2;
                    let right_pad = plot_w_right - msg_len - left_pad;
                    row_spans.push(Span::raw(" ".repeat(left_pad)));
                    row_spans.push(Span::styled(msg, Style::default().fg(theme.dim)));
                    row_spans.push(Span::raw(" ".repeat(right_pad)));
                } else {
                    row_spans.push(Span::raw(" ".repeat(plot_w_right)));
                }
            } else {
                row_spans.push(Span::raw(" ".repeat(plot_w_right)));
            }
        } else {
            for &bits in gpu_grid[row].iter().take(plot_w_right) {
                if bits > 0 {
                    let ch = char::from_u32(0x2800 + bits as u32).unwrap_or(' ');
                    row_spans.push(Span::styled(ch.to_string(), Style::default().fg(Color::Magenta)));
                } else {
                    row_spans.push(Span::raw(" "));
                }
            }
        }

        lines.push(Line::from(row_spans));
    }

    // 3. Bottom Axis Row (7 spaces + 1 corner = 8 chars, exactly aligning with │ above)
    let bottom_spans = vec![
        Span::styled("       └", Style::default().fg(Color::Rgb(65, 75, 85))),
        Span::styled("─".repeat(plot_w_left), Style::default().fg(Color::Rgb(65, 75, 85))),
        Span::raw(" ".repeat(gap)),
        Span::styled("       └", Style::default().fg(Color::Rgb(65, 75, 85))),
        Span::styled("─".repeat(plot_w_right), Style::default().fg(Color::Rgb(65, 75, 85))),
    ];
    lines.push(Line::from(bottom_spans));

    lines
}

pub fn render_dashboard<B: Backend>(f: &mut Frame, app: &AppState<B>, area: Rect) {
    let theme = &app.theme;
    let width = area.width as usize;

    let label_w = 9;
    let val_w = 16;
    let bar_w = 20;

    // Distribute extra vertical space evenly across section gaps so the Dashboard
    // takes the full window height cleanly without an empty void at the bottom.
    let spacers = crate::ui::widgets::distribute_spacers(area.height as usize, 18, 4, 2);

    let mut lines = Vec::new();

    // -------------------------------------------------------------------------
    // Profile Header (no inline 'p cycle', footer has it)
    // -------------------------------------------------------------------------
    let profile_spans = choice_widget(app.platform_profile.as_str(), true, theme);
    lines.push(grid_row(
        "Profile",
        profile_spans,
        vec![],
        vec![],
        true,
        label_w,
        val_w,
        theme,
    ));

    // -------------------------------------------------------------------------
    // 1. Processor
    // -------------------------------------------------------------------------
    lines.push(section_rule("Processor", width, theme));

    let raw_cpu = if app.sensors.cpu_model.is_empty() {
        "Intel Core i5-13450HX"
    } else {
        app.sensors.cpu_model.as_str()
    };
    let cpu_name = clean_cpu_name(raw_cpu);

    let cpu_usage = app.sensors.cpu_usage_percent.unwrap_or(0).clamp(0, 100);
    let cpu_temp = app.sensors.cpu_temp_c.unwrap_or(0);
    let cpu_rpm = app.sensors.cpu_fan_rpm.unwrap_or(0);

    let cpu_filled = (cpu_usage as usize * bar_w) / 100;
    let cpu_empty = bar_w.saturating_sub(cpu_filled);
    let cpu_bar_spans = vec![
        Span::styled("━".repeat(cpu_filled), Style::default().fg(theme.accent)),
        Span::styled("─".repeat(cpu_empty), Style::default().fg(Theme::track_color())),
    ];
    let cpu_extras = vec![
        Span::styled(format!("{cpu_temp:>2}°C"), Style::default().fg(theme.temp_color(cpu_temp)).add_modifier(Modifier::BOLD)),
        Span::raw("   "),
        Span::styled(format!("{cpu_rpm:>4} rpm"), Style::default().fg(theme.dim)),
    ];

    lines.push(grid_row(
        "CPU",
        vec![Span::styled(format!("{cpu_usage}%"), Style::default().fg(theme.fg))],
        cpu_bar_spans,
        cpu_extras,
        false,
        label_w,
        val_w,
        theme,
    ));
    let cpu_detail = if let Some(w) = app.sensors.cpu_power_w {
        format!("{cpu_name}   ·   {w:.0} W")
    } else {
        cpu_name.to_string()
    };
    lines.push(Line::from(vec![
        Span::raw("            "),
        Span::styled(cpu_detail, Style::default().fg(theme.dim)),
    ]));
    for _ in 0..spacers[0] {
        lines.push(Line::from(""));
    }

    // -------------------------------------------------------------------------
    // 2. Graphics
    // -------------------------------------------------------------------------
    lines.push(section_rule("Graphics", width, theme));

    let is_suspended = app.sensors.dgpu_power_state.eq_ignore_ascii_case("suspended")
        || app.sensors.dgpu_power_state.eq_ignore_ascii_case("d3cold");
    let gpu_usage = if is_suspended { 0 } else { app.sensors.gpu_usage_percent.unwrap_or(0).clamp(0, 100) };
    let gpu_temp = app.sensors.gpu_temp_c.unwrap_or(0);
    let gpu_rpm = app.sensors.gpu_fan_rpm.unwrap_or(0);

    let gpu_val_spans = if is_suspended || gpu_usage == 0 {
        vec![Span::styled("Idle", Style::default().fg(theme.dim))]
    } else {
        vec![Span::styled(format!("{gpu_usage}%"), Style::default().fg(theme.fg))]
    };

    let gpu_filled = (gpu_usage as usize * bar_w) / 100;
    let gpu_empty = bar_w.saturating_sub(gpu_filled);
    let gpu_bar_spans = vec![
        Span::styled("━".repeat(gpu_filled), Style::default().fg(theme.accent)),
        Span::styled("─".repeat(gpu_empty), Style::default().fg(Theme::track_color())),
    ];

    let gpu_extras = if is_suspended || gpu_temp == 0 {
        vec![Span::styled("D3cold", Style::default().fg(theme.dim))]
    } else {
        vec![
            Span::styled(format!("{gpu_temp:>2}°C"), Style::default().fg(theme.temp_color(gpu_temp)).add_modifier(Modifier::BOLD)),
            Span::raw("   "),
            Span::styled(format!("{gpu_rpm:>4} rpm"), Style::default().fg(theme.dim)),
        ]
    };

    lines.push(grid_row(
        "GPU",
        gpu_val_spans,
        gpu_bar_spans,
        gpu_extras,
        false,
        label_w,
        val_w,
        theme,
    ));
    let gpu_detail = if is_suspended {
        "RTX 5050 Mobile   ·   D3cold (0 W)".to_string()
    } else if let Some(w) = app.sensors.gpu_power_w {
        format!("RTX 5050 Mobile   ·   {w:.0} W")
    } else {
        "RTX 5050 Mobile".to_string()
    };
    lines.push(Line::from(vec![
        Span::raw("            "),
        Span::styled(gpu_detail, Style::default().fg(theme.dim)),
    ]));
    for _ in 0..spacers[1] {
        lines.push(Line::from(""));
    }

    // -------------------------------------------------------------------------
    // 3. Memory
    // -------------------------------------------------------------------------
    lines.push(section_rule("Memory", width, theme));

    let ram_used = app.sensors.ram_used_mb.unwrap_or(0) as f64 / 1024.0;
    let ram_total = app.sensors.ram_total_mb.unwrap_or(1024) as f64 / 1024.0;
    let ram_pct = (ram_used / ram_total.max(0.1) * 100.0).clamp(0.0, 100.0) as usize;

    let ram_filled = (ram_pct * bar_w) / 100;
    let ram_empty = bar_w.saturating_sub(ram_filled);
    let ram_bar_spans = vec![
        Span::styled("━".repeat(ram_filled), Style::default().fg(theme.accent)),
        Span::styled("─".repeat(ram_empty), Style::default().fg(Theme::track_color())),
    ];

    lines.push(grid_row(
        "RAM",
        vec![Span::styled(format!("{ram_used:.1} / {ram_total:.1} GiB"), Style::default().fg(theme.fg))],
        ram_bar_spans,
        vec![Span::styled(format!("{ram_pct}%"), Style::default().fg(theme.dim))],
        false,
        label_w,
        val_w,
        theme,
    ));
    for _ in 0..spacers[2] {
        lines.push(Line::from(""));
    }

    // -------------------------------------------------------------------------
    // 4. Battery
    // -------------------------------------------------------------------------
    lines.push(section_rule("Battery", width, theme));

    let bat_pct = app.sensors.battery_charge_percent.unwrap_or(60).clamp(0, 100);
    let is_discharging = app.sensors.battery_status.eq_ignore_ascii_case("discharging");
    let bat_color = theme.battery_color(bat_pct, is_discharging);

    let bat_filled = (bat_pct as usize * bar_w) / 100;
    let limit_pos = ((app.charge_limit as usize) * bar_w) / 100;

    let mut bat_bar_chars = Vec::new();
    for i in 0..bar_w {
        let is_limit = i == limit_pos;
        if i < bat_filled {
            if is_limit {
                bat_bar_chars.push('╵');
            } else {
                bat_bar_chars.push('━');
            }
        } else if is_limit {
            bat_bar_chars.push('╷');
        } else {
            bat_bar_chars.push('─');
        }
    }
    let bat_bar_str: String = bat_bar_chars.into_iter().collect();

    let bat_status_text = if app.sensors.battery_status.is_empty() {
        "Plugged in, not charging".to_string()
    } else {
        match app.sensors.battery_status.as_str() {
            "Not charging" => "Plugged in, not charging".to_string(),
            "Charging" => {
                if let Some(w) = app.sensors.battery_power_w {
                    format!("Charging · {w:.0} W")
                } else {
                    "Charging".to_string()
                }
            }
            "Discharging" => {
                if let Some(w) = app.sensors.battery_power_w {
                    format!("Discharging · {w:.0} W")
                } else {
                    "Discharging".to_string()
                }
            }
            "Full" => "Full".to_string(),
            other => other.to_string(),
        }
    };

    lines.push(grid_row(
        "Battery",
        vec![Span::styled(format!("{bat_pct}%"), Style::default().fg(theme.fg))],
        vec![
            Span::styled(bat_bar_str, Style::default().fg(bat_color)),
        ],
        vec![Span::styled(bat_status_text, Style::default().fg(theme.fg))],
        false,
        label_w,
        val_w,
        theme,
    ));

    let health_pct = app.sensors.battery_health_percent.unwrap_or(98);
    let mut bat_details = vec![
        Span::raw("            "),
        Span::styled(format!("Limit {}%", app.charge_limit), Style::default().fg(theme.dim)),
        Span::styled("   ·   ", Style::default().fg(Color::Rgb(70, 75, 85))),
        Span::styled(format!("Health {}%", health_pct), Style::default().fg(theme.dim)),
    ];
    if is_discharging {
        if let Some(w) = app.sensors.battery_power_w {
            let pwr_col = if w > 20.0 {
                theme.err
            } else if w > 10.0 {
                theme.warn
            } else {
                theme.ok
            };
            bat_details.push(Span::styled("   ·   ", Style::default().fg(Color::Rgb(70, 75, 85))));
            bat_details.push(Span::styled(format!("{w:.1} W discharge"), Style::default().fg(pwr_col)));
        }
    } else if app.sensors.battery_status.eq_ignore_ascii_case("charging") {
        if let Some(w) = app.sensors.battery_power_w {
            bat_details.push(Span::styled("   ·   ", Style::default().fg(Color::Rgb(70, 75, 85))));
            bat_details.push(Span::styled(format!("+{w:.1} W charge"), Style::default().fg(theme.ok)));
        }
    } else {
        bat_details.push(Span::styled("   ·   ", Style::default().fg(Color::Rgb(70, 75, 85))));
        bat_details.push(Span::styled("AC Wall Power", Style::default().fg(theme.dim)));
    }
    lines.push(Line::from(bat_details));
    for _ in 0..spacers[3] {
        lines.push(Line::from(""));
    }

    // -------------------------------------------------------------------------
    // 5. Temperatures, last 2 min (Two side-by-side Braille Line Charts)
    // -------------------------------------------------------------------------
    lines.push(section_rule("Temperatures, last 2 min", width, theme));

    let cpu_history: Vec<u64> = app.cpu_temp_history.iter().copied().collect();
    let gpu_history: Vec<u64> = app.gpu_temp_history.iter().copied().collect();

    let cpu_info = TempSeriesInfo {
        label: "CPU",
        dot_color: theme.accent,
        history: &cpu_history,
        temp: cpu_temp,
        rpm: cpu_rpm,
        is_idle: false,
    };
    let gpu_dot_color = if is_suspended && gpu_temp == 0 {
        theme.dim
    } else {
        Color::Magenta
    };
    let gpu_info = TempSeriesInfo {
        label: "GPU",
        dot_color: gpu_dot_color,
        history: &gpu_history,
        temp: gpu_temp,
        rpm: gpu_rpm,
        is_idle: is_suspended,
    };

    let chart_lines = render_two_temp_charts(&cpu_info, &gpu_info, width, theme);
    lines.extend(chart_lines);

    f.render_widget(Paragraph::new(lines), area);
}
