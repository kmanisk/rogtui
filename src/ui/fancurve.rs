use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        canvas::{Canvas, Line as CanvasLine, Points},
        Block, BorderType, Borders, Paragraph,
    },
    Frame,
};

use crate::app::AppState;
use crate::backend::Backend;

pub fn render_fancurve<B: Backend>(f: &mut Frame, app: &AppState<B>, area: Rect) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Top status line
            Constraint::Min(10),   // Graph + Table
        ])
        .split(area);

    let active_curve = app.fan_curves.get(app.active_fan_idx);
    let fan_name = active_curve.map(|c| c.fan_name.as_str()).unwrap_or("Fan");
    let is_custom = active_curve.map(|c| c.enabled).unwrap_or(false);
    let is_modified = app.is_fan_curve_modified();

    // 1. Top status line
    let mut top_spans = vec![
        Span::styled("Profile: ", Style::default().fg(Color::DarkGray)),
        Span::styled(app.platform_profile.as_str(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("   "),
        Span::styled("Fan: ", Style::default().fg(Color::DarkGray)),
        Span::styled(fan_name, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw("   "),
        Span::styled("Custom: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            if is_custom { "on" } else { "off" },
            if is_custom { Style::default().fg(Color::Green) } else { Style::default().fg(Color::DarkGray) },
        ),
    ];

    if is_modified {
        top_spans.push(Span::raw("   "));
        top_spans.push(Span::styled("[modified]", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
    }
    if app.apply_all_fans {
        top_spans.push(Span::raw("   "));
        top_spans.push(Span::styled("[apply all fans: ON]", Style::default().fg(Color::Magenta)));
    }
    if app.show_other_fan_overlay {
        top_spans.push(Span::raw("   "));
        top_spans.push(Span::styled("[overlay: ON]", Style::default().fg(Color::DarkGray)));
    }

    f.render_widget(Paragraph::new(Line::from(top_spans)), main_chunks[0]);

    // 2. Body: Split into Graph (left) and optional Point Table (right)
    let show_table = app.show_point_table || area.width >= 100;
    let body_chunks = if show_table {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(50), Constraint::Length(26)])
            .split(main_chunks[1])
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(100)])
            .split(main_chunks[1])
    };

    // Render Canvas
    if let Some(curve) = active_curve {
        let applied_curve = app.applied_fan_curves.get(app.active_fan_idx);
        let other_fan_idx = if app.active_fan_idx == 0 { 1 } else { 0 };
        let other_curve = if app.show_other_fan_overlay {
            app.fan_curves.get(other_fan_idx)
        } else {
            None
        };

        // Live telemetry for operating point
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

        let live_readout = format!("now {:.0}C -> {:.0}% ({} RPM)", live_temp, live_speed, live_rpm);

        let canvas = Canvas::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(format!(" Fan Curve Graph [{live_readout}] "), Style::default().fg(Color::White))),
            )
            .x_bounds([30.0, 100.0])
            .y_bounds([0.0, 100.0])
            .paint(|ctx| {
                // 1. Faint grid lines at 10C increments (X) and 20% increments (Y)
                for t in (40..=90).step_by(10) {
                    ctx.draw(&CanvasLine {
                        x1: t as f64,
                        y1: 0.0,
                        x2: t as f64,
                        y2: 100.0,
                        color: Color::DarkGray,
                    });
                }
                for s in (20..=80).step_by(20) {
                    ctx.draw(&CanvasLine {
                        x1: 30.0,
                        y1: s as f64,
                        x2: 100.0,
                        y2: s as f64,
                        color: Color::DarkGray,
                    });
                }

                // 2. Draw other fan overlay in dim magenta if toggled (key 'o')
                if let Some(other) = other_curve {
                    if !other.points.is_empty() {
                        let first = &other.points[0];
                        ctx.draw(&CanvasLine {
                            x1: 30.0,
                            y1: first.pwm_percent as f64,
                            x2: first.temp as f64,
                            y2: first.pwm_percent as f64,
                            color: Color::Magenta,
                        });
                        for i in 0..other.points.len().saturating_sub(1) {
                            let p1 = &other.points[i];
                            let p2 = &other.points[i + 1];
                            ctx.draw(&CanvasLine {
                                x1: p1.temp as f64,
                                y1: p1.pwm_percent as f64,
                                x2: p2.temp as f64,
                                y2: p2.pwm_percent as f64,
                                color: Color::Magenta,
                            });
                        }
                        let last = &other.points[other.points.len() - 1];
                        ctx.draw(&CanvasLine {
                            x1: last.temp as f64,
                            y1: last.pwm_percent as f64,
                            x2: 100.0,
                            y2: last.pwm_percent as f64,
                            color: Color::Magenta,
                        });
                    }
                }

                // 3. Draw applied curve in dim dark gray if [modified]
                if is_modified {
                    if let Some(appl) = applied_curve {
                        if !appl.points.is_empty() {
                            let first = &appl.points[0];
                            ctx.draw(&CanvasLine {
                                x1: 30.0,
                                y1: first.pwm_percent as f64,
                                x2: first.temp as f64,
                                y2: first.pwm_percent as f64,
                                color: Color::DarkGray,
                            });
                            for i in 0..appl.points.len().saturating_sub(1) {
                                let p1 = &appl.points[i];
                                let p2 = &appl.points[i + 1];
                                ctx.draw(&CanvasLine {
                                    x1: p1.temp as f64,
                                    y1: p1.pwm_percent as f64,
                                    x2: p2.temp as f64,
                                    y2: p2.pwm_percent as f64,
                                    color: Color::DarkGray,
                                });
                            }
                            let last = &appl.points[appl.points.len() - 1];
                            ctx.draw(&CanvasLine {
                                x1: last.temp as f64,
                                y1: last.pwm_percent as f64,
                                x2: 100.0,
                                y2: last.pwm_percent as f64,
                                color: Color::DarkGray,
                            });
                        }
                    }
                }

                // 4. Draw current curve (flat to edge, segments, flat to edge)
                if !curve.points.is_empty() {
                    let first = &curve.points[0];
                    ctx.draw(&CanvasLine {
                        x1: 30.0,
                        y1: first.pwm_percent as f64,
                        x2: first.temp as f64,
                        y2: first.pwm_percent as f64,
                        color: Color::Cyan,
                    });

                    for i in 0..curve.points.len().saturating_sub(1) {
                        let p1 = &curve.points[i];
                        let p2 = &curve.points[i + 1];
                        ctx.draw(&CanvasLine {
                            x1: p1.temp as f64,
                            y1: p1.pwm_percent as f64,
                            x2: p2.temp as f64,
                            y2: p2.pwm_percent as f64,
                            color: Color::Cyan,
                        });
                    }

                    let last = &curve.points[curve.points.len() - 1];
                    ctx.draw(&CanvasLine {
                        x1: last.temp as f64,
                        y1: last.pwm_percent as f64,
                        x2: 100.0,
                        y2: last.pwm_percent as f64,
                        color: Color::Cyan,
                    });

                    // 5. Draw all 8 points
                    let coords: Vec<(f64, f64)> = curve.points.iter().map(|p| (p.temp as f64, p.pwm_percent as f64)).collect();
                    ctx.draw(&Points {
                        coords: &coords,
                        color: Color::White,
                    });

                    // 6. Selected point in bright contrasting Yellow
                    if let Some(sel) = curve.points.get(app.active_point_idx) {
                        ctx.draw(&Points {
                            coords: &[(sel.temp as f64, sel.pwm_percent as f64)],
                            color: Color::Yellow,
                        });
                        // Label next to selected point
                        let lbl = format!("{}C {}%", sel.temp, sel.pwm_percent);
                        let offset_y = if sel.pwm_percent > 85 { -6.0 } else { 4.0 };
                        ctx.print(sel.temp as f64 + 1.0, (sel.pwm_percent as f64 + offset_y).clamp(2.0, 96.0), lbl);
                    }

                    // 7. Live Operating Point (vertical line + cross marker)
                    if (30.0..=100.0).contains(&live_temp) {
                        ctx.draw(&CanvasLine {
                            x1: live_temp,
                            y1: 0.0,
                            x2: live_temp,
                            y2: 100.0,
                            color: Color::DarkGray,
                        });
                        ctx.draw(&Points {
                            coords: &[(live_temp, live_speed)],
                            color: Color::Green,
                        });
                    }
                }
            });

        f.render_widget(canvas, body_chunks[0]);

        // Render Point Table (if visible)
        if show_table {
            let mut tbl_lines = Vec::new();
            tbl_lines.push(Line::from(Span::styled("-- Points Table --", Style::default().fg(Color::DarkGray))));
            tbl_lines.push(Line::from(vec![
                Span::styled("Pt  ", Style::default().fg(Color::DarkGray)),
                Span::styled("Temp   ", Style::default().fg(Color::DarkGray)),
                Span::styled("Speed", Style::default().fg(Color::DarkGray)),
            ]));

            for (idx, pt) in curve.points.iter().enumerate() {
                let is_sel = idx == app.active_point_idx;
                let is_pt_mod = app.active_point_is_modified(idx);

                let marker = if is_pt_mod { "*" } else { " " };
                let style = if is_sel {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else if is_pt_mod {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default().fg(Color::White)
                };

                let prefix = if is_sel { ">" } else { " " };
                tbl_lines.push(Line::from(vec![
                    Span::styled(format!("{prefix}{}{}  ", idx + 1, marker), style),
                    Span::styled(format!("{: >3} C   ", pt.temp), style),
                    Span::styled(format!("{: >3} %", pt.pwm_percent), style),
                ]));
            }

            f.render_widget(Paragraph::new(tbl_lines), body_chunks[1]);
        }
    }
}
