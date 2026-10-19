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
    let inner_w = width.saturating_sub(4);
    let prefix = format!("── {title} ");
    let prefix_len = prefix.chars().count();
    let fill = if inner_w > prefix_len {
        "─".repeat(inner_w.saturating_sub(prefix_len))
    } else {
        "──".to_string()
    };
    Line::from(vec![
        Span::raw("  "),
        Span::styled(prefix, Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
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

pub fn segmented_control(
    options: &[&str],
    selected_idx: usize,
    is_row_selected: bool,
    theme: &Theme,
) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    for (i, &opt) in options.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" · ", Style::default().fg(theme.dim)));
        }
        if i == selected_idx {
            if is_row_selected {
                spans.push(Span::styled(
                    format!("[{opt}]"),
                    Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                ));
            } else {
                spans.push(Span::styled(
                    opt.to_string(),
                    Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
                ));
            }
        } else {
            spans.push(Span::styled(
                opt.to_string(),
                Style::default().fg(theme.dim),
            ));
        }
    }
    spans
}

#[allow(clippy::too_many_arguments)]
pub fn grid_row(
    label: &str,
    value_spans: Vec<Span<'static>>,
    bar_spans: Vec<Span<'static>>,
    extras_spans: Vec<Span<'static>>,
    is_selected: bool,
    label_w: usize,
    val_w: usize,
    theme: &Theme,
) -> Line<'static> {
    let mut spans = Vec::new();
    if is_selected {
        spans.push(Span::styled("▌ ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)));
        spans.push(Span::styled(
            format!("{label: <width$}", width = label_w),
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD),
        ));
    } else {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            format!("{label: <width$}", width = label_w),
            Style::default().fg(theme.dim),
        ));
    }

    let val_len: usize = value_spans.iter().map(|s| s.content.len()).sum();
    spans.extend(value_spans);
    if val_w > val_len {
        spans.push(Span::raw(" ".repeat(val_w - val_len)));
    } else {
        spans.push(Span::raw(" "));
    }

    spans.extend(bar_spans);
    spans.push(Span::raw("   "));
    spans.extend(extras_spans);

    Line::from(spans)
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
    slider_widget_tinted(value, min, max, width, unit, is_selected, None, theme)
}

#[allow(clippy::too_many_arguments)]
pub fn slider_widget_tinted(
    value: u32,
    min: u32,
    max: u32,
    width: usize,
    unit: &str,
    is_selected: bool,
    custom_tint: Option<Color>,
    theme: &Theme,
) -> Vec<Span<'static>> {
    let clamped_val = value.clamp(min, max);
    let span_range = max.saturating_sub(min).max(1);
    let ratio = (clamped_val.saturating_sub(min) as f32 / span_range as f32).clamp(0.0, 1.0);
    let filled_count = (ratio * width as f32).round() as usize;
    let empty_count = width.saturating_sub(filled_count);

    let filled_col = custom_tint.unwrap_or(if is_selected { theme.accent } else { theme.dim });
    let track_col = Theme::track_color();

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

pub fn charge_limit_slider(
    value: u8,
    width: usize,
    is_selected: bool,
    theme: &Theme,
) -> Vec<Span<'static>> {
    let min = 40.0;
    let max = 100.0;
    let val_f = (value as f32).clamp(min, max);
    let ratio = (val_f - min) / (max - min);

    let filled_col = if is_selected { theme.accent } else { theme.dim };
    let track_col = Theme::track_color();
    let warn_100 = value == 100;

    let mut spans = Vec::new();
    spans.push(Span::styled("40 ", Style::default().fg(theme.dim)));

    let tick_60_idx = (((60.0 - min) / (max - min)) * (width.saturating_sub(1)) as f32).round() as usize;
    let tick_80_idx = (((80.0 - min) / (max - min)) * (width.saturating_sub(1)) as f32).round() as usize;
    let thumb_idx = (ratio * (width.saturating_sub(1)) as f32).round() as usize;

    for i in 0..width {
        if i == thumb_idx {
            spans.push(Span::styled(
                "◆",
                Style::default().fg(if is_selected { theme.accent } else { theme.fg }).add_modifier(Modifier::BOLD),
            ));
        } else if i < thumb_idx {
            let ch = if i == tick_60_idx || i == tick_80_idx { "╵" } else { "━" };
            spans.push(Span::styled(ch, Style::default().fg(filled_col)));
        } else {
            let ch = if i == tick_60_idx || i == tick_80_idx { "╷" } else { "─" };
            let col = if i == tick_60_idx || i == tick_80_idx { theme.dim } else { track_col };
            spans.push(Span::styled(ch, Style::default().fg(col)));
        }
    }

    spans.push(Span::styled(" 100   ", Style::default().fg(theme.dim)));

    let limit_desc = match value {
        60 => "60% (max lifespan)",
        80 => "80% (balanced)",
        100 => "100% (full charge)",
        _ => "",
    };

    let text_val = if limit_desc.is_empty() {
        format!("{value}% (stops at {value}%)")
    } else {
        limit_desc.to_string()
    };

    spans.push(Span::styled(
        text_val,
        if warn_100 {
            Style::default().fg(theme.warn).add_modifier(Modifier::BOLD)
        } else if is_selected {
            Style::default().fg(theme.fg).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.dim)
        },
    ));

    spans
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

    let mut spans = Vec::new();
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

pub fn render_horizontal_rule(f: &mut Frame, area: Rect, _theme: &Theme) {
    let width = area.width as usize;
    let rule = Paragraph::new(Line::from(Span::styled(
        "─".repeat(width),
        Style::default().fg(Color::Rgb(65, 75, 85)),
    )));
    f.render_widget(rule, area);
}

pub fn render_hint_line(f: &mut Frame, area: Rect, hint_text: &str, theme: &Theme) {
    let line = Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(hint_text.to_string(), Style::default().fg(theme.dim)),
    ]);
    f.render_widget(Paragraph::new(line), area);
}

pub fn render_footer<B: Backend>(f: &mut Frame, area: Rect, app: &AppState<B>, theme: &Theme) {
    let hint = app.get_active_hint();
    if area.height >= 2 {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Length(1)])
            .split(area);
        render_footer_two_rows(f, chunks[0], chunks[1], &hint, app, theme);
    } else {
        render_footer_two_rows(f, area, area, &hint, app, theme);
    }
}

pub fn render_footer_two_rows<B: Backend>(
    f: &mut Frame,
    hint_area: Rect,
    keys_area: Rect,
    hint_text: &str,
    app: &AppState<B>,
    theme: &Theme,
) {
    let width = hint_area.width as usize;

    // Row 1: Hint / Toast
    let hint_line = if let Some((msg, is_error, _)) = &app.status_message {
        if *is_error {
            Line::from(vec![
                Span::raw("  "),
                Span::styled(format!("[!] {msg}"), Style::default().fg(theme.err).add_modifier(Modifier::BOLD)),
            ])
        } else {
            Line::from(vec![
                Span::raw("  "),
                Span::styled(format!("✓ {msg}"), Style::default().fg(theme.ok).add_modifier(Modifier::BOLD)),
            ])
        }
    } else if let Some((clamp_hint, _)) = &app.fan_clamp_hint {
        Line::from(vec![
            Span::raw("  "),
            Span::styled(clamp_hint.clone(), Style::default().fg(theme.warn)),
        ])
    } else {
        let max_len = width.saturating_sub(6);
        let clean_hint = if hint_text.chars().count() > max_len {
            let truncated: String = hint_text.chars().take(max_len.saturating_sub(1)).collect();
            format!("{truncated}…")
        } else {
            hint_text.to_string()
        };
        Line::from(vec![
            Span::raw("  "),
            Span::styled(clean_hint, Style::default().fg(theme.dim)),
        ])
    };
    f.render_widget(Paragraph::new(hint_line), hint_area);

    // Row 2: Keys
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
            ],
            ActiveTab::Power => {
                match app.power_selected_field {
                    0..=2 => vec![("h/l", "cycle"), ("Space", "toggle")],
                    3 => vec![("h/l", "adjust limit")],
                    4 => vec![("Enter", "full charge")],
                    _ => vec![],
                }
            }
            ActiveTab::FanCurves => vec![
                ("j/k", "point"),
                ("h/l", "adjust"),
                ("Space", "temp/speed"),
                ("s", "preset"),
                ("f", "fan"),
                ("c", "custom"),
                ("w", "apply"),
            ],
            ActiveTab::Aura => {
                match app.aura_selected_field {
                    0 => vec![("h/l", "brightness")],
                    1 => vec![("h/l", "mode")],
                    2 => vec![("h/l", "preset"), ("c", "picker"), ("#", "hex")],
                    3..=5 => vec![("h/l", "adjust channel")],
                    6 => vec![("h/l", "secondary preset")],
                    7 => vec![("h/l", "speed")],
                    8 => vec![("h/l", "direction")],
                    9..=12 => vec![("h/l", "toggle"), ("Space", "toggle")],
                    13 => vec![("h/l", "slash mode")],
                    14 => vec![("h/l", "slash brightness")],
                    _ => vec![("h/l", "adjust")],
                }
            }
            ActiveTab::System => {
                let has_auto_bright = app.sensors.screen_auto_brightness.is_some();
                let export_row_idx = if has_auto_bright { 2 } else { 1 };
                if app.system_selected_row == export_row_idx {
                    vec![("Enter", "export report")]
                } else {
                    vec![("Space", "toggle")]
                }
            }
        }
    };

    let mut left_spans = vec![Span::raw("  ")];
    let mut left_len = 2;
    for (k, a) in keybinds {
        let entry_len = k.len() + a.len() + 3;
        if left_len + entry_len < width.saturating_sub(18) {
            left_spans.push(Span::styled(k, Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)));
            left_spans.push(Span::raw(" "));
            left_spans.push(Span::styled(format!("{a}  "), Style::default().fg(theme.dim)));
            left_len += entry_len;
        }
    }

    let right_spans = vec![
        Span::styled("?", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled("help  ", Style::default().fg(theme.dim)),
        Span::styled("q", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled("quit  ", Style::default().fg(theme.dim)),
    ];
    let right_len = 16;

    let mut total_spans = left_spans;
    if width > left_len + right_len {
        let pad = width - left_len - right_len;
        total_spans.push(Span::raw(" ".repeat(pad)));
    } else {
        total_spans.push(Span::raw(" "));
    }
    total_spans.extend(right_spans);

    f.render_widget(Paragraph::new(Line::from(total_spans)), keys_area);
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
