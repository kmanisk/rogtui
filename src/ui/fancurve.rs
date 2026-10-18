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
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Min(12),
            Constraint::Length(6),
            Constraint::Length(3),
        ])
        .split(area);

    let active_curve = app.fan_curves.get(app.active_fan_idx);
    let fan_name = active_curve.map(|c| c.fan_name.as_str()).unwrap_or("Fan");
    let is_enabled = active_curve.map(|c| c.enabled).unwrap_or(false);

    // Header info line
    let header_line = Line::from(vec![
        Span::styled("  Profile: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("[ {} ]", app.platform_profile.as_str()),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::styled("   Fan: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("[ {} Fan ]", fan_name),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  (press 'f' to switch)   Status: ", Style::default().fg(Color::DarkGray)),
        if is_enabled {
            Span::styled("CUSTOM CURVE ACTIVE", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
        } else {
            Span::styled("BIOS FACTORY DEFAULT", Style::default().fg(Color::Gray))
        },
        Span::styled("  (press 'e' to toggle)", Style::default().fg(Color::DarkGray)),
    ]);

    let header_widget = Paragraph::new(header_line).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(Span::styled(" Fan Curve Profile Control ", Style::default().add_modifier(Modifier::BOLD))),
    );
    f.render_widget(header_widget, chunks[0]);

    // Canvas Block (Braille-based high-res curve)
    if let Some(curve) = active_curve {
        let validation_err = curve.validate().err();

        let canvas = Canvas::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(if let Some(err) = validation_err {
                        Span::styled(
                            format!(" Curve Graph - VALIDATION ERROR: {err} "),
                            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                        )
                    } else {
                        Span::styled(" Curve Graph (Temp C vs PWM %) ", Style::default().add_modifier(Modifier::BOLD))
                    }),
            )
            .x_bounds([30.0, 105.0])
            .y_bounds([0.0, 105.0])
            .paint(|ctx| {
                // Draw 80C safety threshold line
                ctx.draw(&CanvasLine {
                    x1: 80.0,
                    y1: 0.0,
                    x2: 80.0,
                    y2: 100.0,
                    color: Color::DarkGray,
                });

                // Connect curve points with lines
                for i in 0..curve.points.len().saturating_sub(1) {
                    let p1 = &curve.points[i];
                    let p2 = &curve.points[i + 1];
                    ctx.draw(&CanvasLine {
                        x1: p1.temp as f64,
                        y1: p1.pwm_percent as f64,
                        x2: p2.temp as f64,
                        y2: p2.pwm_percent as f64,
                        color: if validation_err.is_some() { Color::Red } else { Color::Cyan },
                    });
                }

                // Draw points
                let all_pts: Vec<(f64, f64)> = curve.points.iter().map(|p| (p.temp as f64, p.pwm_percent as f64)).collect();
                ctx.draw(&Points {
                    coords: &all_pts,
                    color: Color::White,
                });

                // Highlight active point
                if let Some(selected_pt) = curve.points.get(app.active_point_idx) {
                    ctx.draw(&Points {
                        coords: &[(selected_pt.temp as f64, selected_pt.pwm_percent as f64)],
                        color: Color::Yellow,
                    });
                }
            });

        f.render_widget(canvas, chunks[1]);

        // Points Table
        let mut pt_spans = Vec::new();
        pt_spans.push(Span::raw("  "));
        for (idx, pt) in curve.points.iter().enumerate() {
            let is_sel = idx == app.active_point_idx;
            let style = if is_sel {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else {
                Style::default().fg(Color::White)
            };
            pt_spans.push(Span::styled(format!(" [P{}: {}C -> {}%] ", idx + 1, pt.temp, pt.pwm_percent), style));
            pt_spans.push(Span::raw(" "));
        }

        let pt_lines = vec![
            Line::from(""),
            Line::from(pt_spans),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Selected Point: ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("Point #{}", app.active_point_idx + 1), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled("   Use Left/Right to select point, Up/Down to adjust speed, [/] to adjust temp", Style::default().fg(Color::DarkGray)),
            ]),
        ];

        let pt_widget = Paragraph::new(pt_lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(Span::styled(" 8 Hardware Control Points ", Style::default().add_modifier(Modifier::BOLD))),
        );
        f.render_widget(pt_widget, chunks[2]);
    }

    // Guide
    let guide_line = Line::from(vec![
        Span::styled("  w: ", Style::default().fg(Color::Yellow)),
        Span::raw("Apply & Write curve   |   "),
        Span::styled("r: ", Style::default().fg(Color::Yellow)),
        Span::raw("Reset default   |   "),
        Span::styled("u: ", Style::default().fg(Color::Yellow)),
        Span::raw("Revert to last good   |   "),
        Span::styled("e: ", Style::default().fg(Color::Yellow)),
        Span::raw("Enable/Disable"),
    ]);
    let guide_widget = Paragraph::new(guide_line).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Actions "),
    );
    f.render_widget(guide_widget, chunks[3]);
}
