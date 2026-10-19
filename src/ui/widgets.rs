use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph},
    Frame,
};

use crate::app::{ActiveTab, AppState};
use crate::backend::Backend;
use crate::ui::theme::Theme;

pub fn panel_block<'a>(title: impl Into<Span<'a>>, focused: bool, theme: &Theme) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(if focused {
            Style::default().fg(theme.accent)
        } else {
            Style::default().fg(Color::Rgb(80, 90, 105))
        })
        .title(title.into())
        .padding(Padding::horizontal(1))
}

pub fn section_rule(title: &str, width: usize, theme: &Theme) -> Line<'static> {
    let prefix = format!("── {title} ");
    let prefix_len = prefix.chars().count();
    let fill = if width > prefix_len + 2 {
        "─".repeat(width.saturating_sub(prefix_len))
    } else {
        "──".to_string()
    };
    Line::from(vec![
        Span::styled(format!("── {title} "), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled(fill, Style::default().fg(Color::Rgb(70, 75, 85))),
    ])
}

pub fn label_value_row(
    label: &str,
    value_spans: Vec<Span<'static>>,
    is_selected: bool,
    label_width: usize,
    theme: &Theme,
) -> Line<'static> {
    let mut spans = Vec::new();
    if is_selected {
        spans.push(Span::styled("▌ ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)));
        spans.push(Span::styled(
            format!("{label: <width$} ", width = label_width),
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        ));
    } else {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            format!("{label: <width$} ", width = label_width),
            Style::default().fg(theme.dim),
        ));
    }
    spans.extend(value_spans);
    Line::from(spans)
}

pub fn choice_widget(current_val: &str, is_selected: bool, theme: &Theme) -> Vec<Span<'static>> {
    let arrow_col = if is_selected { theme.accent } else { theme.dim };
    let val_style = if is_selected {
        Style::default().fg(theme.fg).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.fg)
    };

    vec![
        Span::styled("‹ ", Style::default().fg(arrow_col)),
        Span::styled(current_val.to_string(), val_style),
        Span::styled(" ›", Style::default().fg(arrow_col)),
    ]
}

pub fn toggle_widget(enabled: bool, is_selected: bool, theme: &Theme) -> Vec<Span<'static>> {
    if enabled {
        let style = if is_selected {
            Style::default().fg(theme.ok).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.ok)
        };
        vec![Span::styled("● On", style)]
    } else {
        vec![Span::styled("○ Off", Style::default().fg(theme.dim))]
    }
}

pub fn slider_widget(
    value: u32,
    min: u32,
    max: u32,
    width: usize,
    unit: &str,
    is_selected: bool,
    theme: &Theme,
) -> Vec<Span<'static>> {
    let clamped_val = value.clamp(min, max);
    let span_range = max.saturating_sub(min).max(1);
    let ratio = (clamped_val.saturating_sub(min) as f32 / span_range as f32).clamp(0.0, 1.0);
    let filled_count = (ratio * width as f32).round() as usize;
    let empty_count = width.saturating_sub(filled_count);

    let filled_col = if is_selected { theme.accent } else { theme.dim };
    let track_col = Color::Rgb(70, 75, 85);

    vec![
        Span::styled("━".repeat(filled_count), Style::default().fg(filled_col)),
        Span::styled("─".repeat(empty_count), Style::default().fg(track_col)),
        Span::styled(
            format!("  {value}{unit}"),
            if is_selected {
                Style::default().fg(theme.fg).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.dim)
            },
        ),
    ]
}

pub fn render_sparkline(values: &[u64], width: usize, color: Color) -> Vec<Span<'static>> {
    if values.is_empty() {
        return vec![Span::raw(" ".repeat(width))];
    }
    let glyphs = [' ', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let min = *values.iter().min().unwrap_or(&0);
    let max = *values.iter().max().unwrap_or(&100);
    let range = (max - min).max(1);

    let mut spans = Vec::new();
    let start_idx = values.len().saturating_sub(width);
    let slice = &values[start_idx..];

    // Left padding if fewer items than width
    if slice.len() < width {
        spans.push(Span::raw(" ".repeat(width - slice.len())));
    }

    for &v in slice {
        let idx = (((v.saturating_sub(min)) * (glyphs.len() - 1) as u64) / range) as usize;
        let g = glyphs[idx.min(glyphs.len() - 1)];
        spans.push(Span::styled(g.to_string(), Style::default().fg(color)));
    }

    spans
}

pub fn render_gauge(
    label: &str,
    val_str: &str,
    percent: u32,
    width: usize,
    _color: Color,
    theme: &Theme,
) -> Line<'static> {
    let p = percent.clamp(0, 100) as usize;
    let filled_blocks = (p * width + 50) / 100;

    let mut spans = Vec::new();
    spans.push(Span::styled(format!("  {label: <14}: "), Style::default().fg(theme.dim)));

    let empty_color = Color::Rgb(55, 60, 68);
    let is_battery = label.to_ascii_lowercase().contains("battery");

    for i in 0..width {
        if i < filled_blocks {
            let seg_pct = (i * 100) / width;
            let seg_color = if is_battery {
                if seg_pct < 20 {
                    theme.err
                } else if seg_pct < 60 {
                    theme.warn
                } else {
                    theme.ok
                }
            } else if seg_pct < 45 {
                theme.ok
            } else if seg_pct < 75 {
                theme.warn
            } else {
                theme.err
            };
            spans.push(Span::styled("■", Style::default().fg(seg_color)));
        } else {
            spans.push(Span::styled("■", Style::default().fg(empty_color)));
        }
    }

    spans.push(Span::styled(format!("  {val_str}"), Style::default().fg(theme.fg).add_modifier(Modifier::BOLD)));
    Line::from(spans)
}

pub fn render_tab_bar(f: &mut Frame, area: Rect, active_tab: ActiveTab, theme: &Theme) {
    let tabs = [
        (ActiveTab::Dashboard, "Dash"),
        (ActiveTab::Power, "Power"),
        (ActiveTab::FanCurves, "Fan"),
        (ActiveTab::Aura, "RGB"),
        (ActiveTab::System, "System"),
    ];

    let mut spans = vec![Span::raw(" ")];
    for (tab, name) in tabs.iter() {
        let is_active = *tab == active_tab;
        if is_active {
            spans.push(Span::styled(
                format!(" {name} "),
                Style::default()
                    .bg(theme.accent)
                    .fg(theme.sel_fg)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(
                format!(" {name} "),
                Style::default().fg(theme.dim),
            ));
        }
        spans.push(Span::raw("  "));
    }

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

pub fn render_hint_line(f: &mut Frame, area: Rect, hint_text: &str, theme: &Theme) {
    let line = Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(hint_text.to_string(), Style::default().fg(theme.dim)),
    ]);
    f.render_widget(Paragraph::new(line), area);
}

pub fn render_footer<B: Backend>(f: &mut Frame, area: Rect, app: &AppState<B>, theme: &Theme) {
    let keybinds: Vec<(&str, &str)> = if app.show_color_picker_modal {
        vec![
            ("j/k", "slider"),
            ("h/l", "adjust"),
            ("Enter", "apply"),
            ("Esc", "cancel"),
        ]
    } else {
        match app.current_tab {
            ActiveTab::Dashboard => vec![
                ("p", "cycle profile"),
                ("Tab", "next tab"),
                ("?", "help"),
                ("q", "quit"),
            ],
            ActiveTab::Power => vec![
                ("j/k", "select"),
                ("h/l", "adjust"),
                ("Enter", "toggle"),
                ("?", "help"),
                ("q", "quit"),
            ],
            ActiveTab::FanCurves => vec![
                ("h/l", "point"),
                ("j/k", "speed"),
                ("H/L", "temp"),
                ("s", "preset"),
                ("w", "apply"),
                ("?", "help"),
            ],
            ActiveTab::Aura => vec![
                ("j/k", "select"),
                ("h/l", "mode / color"),
                ("c", "color picker"),
                ("w", "save"),
                ("?", "help"),
            ],
            ActiveTab::System => vec![
                ("j/k", "select"),
                ("Enter", "toggle / switch"),
                ("y", "export report"),
                ("?", "help"),
                ("q", "quit"),
            ],
        }
    };

    let toast_span = if let Some((msg, is_error, _)) = &app.status_message {
        if *is_error {
            Span::styled(format!(" [!] {msg} "), Style::default().bg(theme.err).fg(Color::White).add_modifier(Modifier::BOLD))
        } else {
            Span::styled(format!(" {msg} "), Style::default().bg(theme.ok).fg(Color::Black).add_modifier(Modifier::BOLD))
        }
    } else if let Some((hint, _)) = &app.fan_clamp_hint {
        Span::styled(format!(" {hint} "), Style::default().bg(theme.warn).fg(Color::Black))
    } else {
        Span::raw("")
    };

    let toast_len = toast_span.content.len();
    let total_width = area.width as usize;

    let mut spans = vec![Span::raw(" ")];
    let mut current_len = 1 + toast_len + 2;

    for (key, desc) in keybinds.iter().take(5) {
        let entry_len = key.len() + desc.len() + 3;
        if current_len + entry_len < total_width {
            spans.push(Span::styled(
                (*key).to_string(),
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::raw(" "));
            spans.push(Span::styled(
                format!("{desc}  "),
                Style::default().fg(theme.dim),
            ));
            current_len += entry_len;
        } else {
            break;
        }
    }

    let left_len: usize = spans.iter().map(|s| s.content.len()).sum();
    if total_width > left_len + toast_len {
        let padding = total_width - left_len - toast_len;
        spans.push(Span::raw(" ".repeat(padding)));
        spans.push(toast_span);
    }

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn render_confirm_modal(f: &mut Frame, prompt: &str, area: Rect, theme: &Theme) {
    let popup_area = centered_rect(50, 22, area);
    f.render_widget(Clear, popup_area);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(prompt.to_string(), Style::default().fg(theme.fg).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![
            Span::styled("y", Style::default().fg(theme.ok).add_modifier(Modifier::BOLD)),
            Span::styled(" Confirm   ·   ", Style::default().fg(theme.dim)),
            Span::styled("n / Esc", Style::default().fg(theme.err).add_modifier(Modifier::BOLD)),
            Span::styled(" Cancel", Style::default().fg(theme.dim)),
        ]),
    ];

    let widget = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.warn))
                .title(Span::styled(" Confirm ", Style::default().fg(theme.warn).add_modifier(Modifier::BOLD))),
        );
    f.render_widget(widget, popup_area);
}

pub fn render_help_overlay(f: &mut Frame, area: Rect, current_tab: ActiveTab, theme: &Theme) {
    let popup_area = centered_rect(65, 75, area);
    f.render_widget(Clear, popup_area);

    let mut help_lines = vec![
        Line::from(Span::styled(
            "  Keyboard Controls",
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled("  -- Global --", Style::default().fg(theme.dim))),
        Line::from(vec![
            Span::styled("  1 .. 5           : ", Style::default().fg(theme.warn)),
            Span::raw("Direct jump to tab (Dash, Power, Fan, RGB, System)"),
        ]),
        Line::from(vec![
            Span::styled("  Tab / Shift+Tab  : ", Style::default().fg(theme.warn)),
            Span::raw("Cycle to next / previous tab"),
        ]),
        Line::from(vec![
            Span::styled("  p                : ", Style::default().fg(theme.warn)),
            Span::raw("Quick cycle Platform Profile"),
        ]),
        Line::from(vec![
            Span::styled("  ?                : ", Style::default().fg(theme.warn)),
            Span::raw("Toggle this help overlay"),
        ]),
        Line::from(vec![
            Span::styled("  q / Esc          : ", Style::default().fg(theme.warn)),
            Span::raw("Close modal / quit application"),
        ]),
        Line::from(""),
    ];

    match current_tab {
        ActiveTab::Dashboard => {
            help_lines.push(Line::from(Span::styled("  -- Dash --", Style::default().fg(theme.dim))));
            help_lines.push(Line::from(vec![
                Span::styled("  p                : ", Style::default().fg(theme.warn)),
                Span::raw("Cycle active platform profile (Quiet -> Balanced -> Performance)"),
            ]));
        }
        ActiveTab::Power => {
            help_lines.push(Line::from(Span::styled("  -- Power --", Style::default().fg(theme.dim))));
            help_lines.push(Line::from(vec![
                Span::styled("  j / k or Up/Down : ", Style::default().fg(theme.warn)),
                Span::raw("Select configuration row"),
            ]));
            help_lines.push(Line::from(vec![
                Span::styled("  h / l or L/R     : ", Style::default().fg(theme.warn)),
                Span::raw("Adjust profile or charge limit percentage (+/- 5%)"),
            ]));
            help_lines.push(Line::from(vec![
                Span::styled("  Enter / Space    : ", Style::default().fg(theme.warn)),
                Span::raw("Toggle option / trigger one-shot 100% full charge"),
            ]));
        }
        ActiveTab::FanCurves => {
            help_lines.push(Line::from(Span::styled("  -- Fan --", Style::default().fg(theme.dim))));
            help_lines.push(Line::from(vec![
                Span::styled("  h / l or L/R     : ", Style::default().fg(theme.warn)),
                Span::raw("Select previous / next curve point"),
            ]));
            help_lines.push(Line::from(vec![
                Span::styled("  j / k or Up/Down : ", Style::default().fg(theme.warn)),
                Span::raw("Adjust fan speed +/- 1% (Shift+j/k for 5%)"),
            ]));
            help_lines.push(Line::from(vec![
                Span::styled("  H / L (Shift+h/l): ", Style::default().fg(theme.warn)),
                Span::raw("Adjust point temperature +/- 1 C"),
            ]));
            help_lines.push(Line::from(vec![
                Span::styled("  s                : ", Style::default().fg(theme.warn)),
                Span::raw("Cycle presets (Silent / Balanced / Aggressive)"),
            ]));
            help_lines.push(Line::from(vec![
                Span::styled("  f                : ", Style::default().fg(theme.warn)),
                Span::raw("Switch active fan (CPU <-> GPU)"),
            ]));
            help_lines.push(Line::from(vec![
                Span::styled("  w                : ", Style::default().fg(theme.warn)),
                Span::raw("Apply and write curves to hardware"),
            ]));
        }
        ActiveTab::Aura => {
            help_lines.push(Line::from(Span::styled("  -- RGB --", Style::default().fg(theme.dim))));
            help_lines.push(Line::from(vec![
                Span::styled("  j / k or Up/Down : ", Style::default().fg(theme.warn)),
                Span::raw("Select row"),
            ]));
            help_lines.push(Line::from(vec![
                Span::styled("  h / l or L/R     : ", Style::default().fg(theme.warn)),
                Span::raw("Cycle lighting mode / adjust sliders"),
            ]));
            help_lines.push(Line::from(vec![
                Span::styled("  c                : ", Style::default().fg(theme.warn)),
                Span::raw("Open 24-bit RGB Color Picker modal"),
            ]));
            help_lines.push(Line::from(vec![
                Span::styled("  w                : ", Style::default().fg(theme.warn)),
                Span::raw("Save current lighting parameters to firmware"),
            ]));
        }
        ActiveTab::System => {
            help_lines.push(Line::from(Span::styled("  -- System --", Style::default().fg(theme.dim))));
            help_lines.push(Line::from(vec![
                Span::styled("  j / k or Up/Down : ", Style::default().fg(theme.warn)),
                Span::raw("Select row"),
            ]));
            help_lines.push(Line::from(vec![
                Span::styled("  Enter / Space    : ", Style::default().fg(theme.warn)),
                Span::raw("Toggle display settings / switch GPU mode"),
            ]));
            help_lines.push(Line::from(vec![
                Span::styled("  y                : ", Style::default().fg(theme.warn)),
                Span::raw("Export system diagnostics report"),
            ]));
        }
    }

    help_lines.push(Line::from(""));
    help_lines.push(Line::from(Span::styled("  Press Esc or '?' to close.", Style::default().fg(theme.dim))));

    let widget = Paragraph::new(help_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.accent))
            .title(Span::styled(" Help ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))),
    );
    f.render_widget(widget, popup_area);
}
