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
            Style::default().fg(theme.dim)
        })
        .title(title.into())
        .padding(Padding::horizontal(1))
}

pub fn render_header<B: Backend>(f: &mut Frame, area: Rect, app: &AppState<B>, theme: &Theme) {
    let mut left_spans = vec![
        Span::raw(" "),
        Span::styled(
            " ROGTUI ",
            Style::default()
                .bg(theme.accent)
                .fg(theme.sel_fg)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(
            "v0.1.0 (asusd v6.5.0 + supergfxd v5.2.7)",
            Style::default().fg(theme.dim),
        ),
    ];

    let prof_name = app.platform_profile.as_str();
    let prof_col = theme.profile_color(app.platform_profile);
    let bat_str = if let Some(b) = app.sensors.battery_charge_percent {
        format!("[BAT: {b}%]")
    } else {
        "[AC Power]".to_string()
    };

    let right_spans = vec![
        Span::styled(format!("[Profile: {prof_name}]"), Style::default().fg(prof_col).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled(bat_str, Style::default().fg(theme.ok)),
        Span::raw(" "),
    ];

    let left_len: usize = left_spans.iter().map(|s| s.content.len()).sum();
    let right_len: usize = right_spans.iter().map(|s| s.content.len()).sum();
    let total_width = area.width as usize;

    if total_width > left_len + right_len {
        let padding = total_width - left_len - right_len;
        left_spans.push(Span::raw(" ".repeat(padding)));
        left_spans.extend(right_spans);
    }

    f.render_widget(Paragraph::new(Line::from(left_spans)), area);
}

pub fn render_tab_bar(f: &mut Frame, area: Rect, active_tab: ActiveTab, theme: &Theme) {
    let tabs = [
        (ActiveTab::Dashboard, "1 Dash"),
        (ActiveTab::Power, "2 Power"),
        (ActiveTab::FanCurves, "3 Fan"),
        (ActiveTab::Aura, "4 RGB"),
        (ActiveTab::Gpu, "5 GPU"),
        (ActiveTab::Armoury, "6 Armoury"),
        (ActiveTab::About, "7 About"),
    ];

    let mut spans = vec![Span::raw(" ")];
    for (tab, name) in tabs.iter() {
        let is_active = *tab == active_tab;
        if is_active {
            spans.push(Span::styled(
                format!(" [ {name} ] "),
                Style::default()
                    .bg(theme.accent)
                    .fg(theme.sel_fg)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(
                format!("   {name}   "),
                Style::default().fg(theme.dim),
            ));
        }
        spans.push(Span::raw(" "));
    }

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

pub fn render_footer<B: Backend>(f: &mut Frame, area: Rect, app: &AppState<B>, theme: &Theme) {
    let keybinds: Vec<(&str, &str)> = if app.show_color_picker_modal {
        vec![
            ("j/k", "Select Slider"),
            ("h/l", "Adjust Color"),
            ("Enter", "Apply"),
            ("Esc", "Cancel"),
        ]
    } else if app.show_advanced_power_modal {
        vec![
            ("Esc/Enter", "Close Modal"),
        ]
    } else {
        match app.current_tab {
            ActiveTab::Dashboard => vec![
                ("Tab", "Next Tab"),
                ("p", "Cycle Profile"),
                ("a", "Power Limits"),
                ("o", "Overdrive"),
                ("b", "Auto Bright"),
                ("?", "Help"),
                ("q", "Quit"),
            ],
            ActiveTab::Power => vec![
                ("j/k", "Select Field"),
                ("h/l", "Adjust / Limit"),
                ("Enter", "Toggle"),
                ("?", "Help"),
                ("q", "Quit"),
            ],
            ActiveTab::FanCurves => vec![
                ("h/l", "Select Pt"),
                ("j/k", "Speed (+/-1%)"),
                ("H/L", "Temp (+/-1°C)"),
                ("s", "Preset"),
                ("x", "Auto Repair"),
                ("w", "Apply to EC"),
                ("?", "Help"),
                ("q", "Quit"),
            ],
            ActiveTab::Aura => {
                if app.aura_brightness == 0 {
                    vec![
                        ("h/l", "Turn On Backlight"),
                        ("j/k", "Select Field"),
                        ("w", "Save to BIOS"),
                        ("?", "Help"),
                        ("q", "Quit"),
                    ]
                } else {
                    vec![
                        ("j/k", "Select Field"),
                        ("h/l", "Cycle Mode / Preset"),
                        ("c/Enter", "Color Picker"),
                        ("w", "Save to BIOS"),
                        ("?", "Help"),
                        ("q", "Quit"),
                    ]
                }
            }
            ActiveTab::Gpu => vec![
                ("j/k", "Select Mode"),
                ("Enter", "Switch GPU Mode"),
                ("?", "Help"),
                ("q", "Quit"),
            ],
            ActiveTab::Armoury => vec![
                ("j/k", "Select Attr"),
                ("h/l", "Adjust Value"),
                ("Enter", "Toggle"),
                ("u", "Toggle Unavail"),
                ("r", "Restore Default"),
                ("?", "Help"),
                ("q", "Quit"),
            ],
            ActiveTab::About => vec![
                ("Tab", "Next Tab"),
                ("y", "Export Report"),
                ("?", "Help"),
                ("q", "Quit"),
            ],
        }
    };

    let mut spans = vec![Span::raw(" ")];
    for (key, desc) in keybinds.iter() {
        spans.push(Span::styled(
            format!("[{key}]"),
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(" "));
        spans.push(Span::styled(format!("{desc}  "), Style::default().fg(theme.fg)));
    }

    // Status / Toast right-aligned
    let status_span = if let Some((msg, is_error, _)) = &app.status_message {
        if *is_error {
            Span::styled(format!(" [err] {msg} "), Style::default().bg(theme.err).fg(Color::White).add_modifier(Modifier::BOLD))
        } else {
            Span::styled(format!(" [ok] {msg} "), Style::default().bg(theme.ok).fg(Color::Black).add_modifier(Modifier::BOLD))
        }
    } else if let Some((hint, _)) = &app.fan_clamp_hint {
        Span::styled(format!(" [hint: {hint}] "), Style::default().bg(theme.warn).fg(Color::Black))
    } else {
        Span::raw("")
    };

    let left_len: usize = spans.iter().map(|s| s.content.len()).sum();
    let status_len = status_span.content.len();
    let total_width = area.width as usize;

    if total_width > left_len + status_len + 2 {
        let padding = total_width - left_len - status_len;
        spans.push(Span::raw(" ".repeat(padding)));
        spans.push(status_span);
    }

    f.render_widget(Paragraph::new(Line::from(spans)), area);
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

    // Btop-style empty track color
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
    let popup_area = centered_rect(60, 25, area);
    f.render_widget(Clear, popup_area);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(prompt.to_string(), Style::default().fg(theme.fg).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(theme.dim)),
            Span::styled("'y'", Style::default().fg(theme.ok).add_modifier(Modifier::BOLD)),
            Span::styled(" to Confirm   |   Press ", Style::default().fg(theme.dim)),
            Span::styled("'n' or Esc", Style::default().fg(theme.err).add_modifier(Modifier::BOLD)),
            Span::styled(" to Cancel", Style::default().fg(theme.dim)),
        ]),
    ];

    let widget = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.warn))
                .title(Span::styled(" Confirm Action ", Style::default().fg(theme.warn).add_modifier(Modifier::BOLD))),
        );
    f.render_widget(widget, popup_area);
}

pub fn render_help_overlay(f: &mut Frame, area: Rect, current_tab: ActiveTab, theme: &Theme) {
    let popup_area = centered_rect(70, 80, area);
    f.render_widget(Clear, popup_area);

    let mut help_lines = vec![
        Line::from(Span::styled(
            "  ROGTUI Hardware Controller (asusd v6.5.0 + supergfxd v5.2.7)",
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled("  -- Global Shortcuts --", Style::default().fg(theme.dim))),
        Line::from(vec![
            Span::styled("  Tab / Shift+Tab  : ", Style::default().fg(theme.warn)),
            Span::raw("Cycle to next / previous tab"),
        ]),
        Line::from(vec![
            Span::styled("  1 .. 6           : ", Style::default().fg(theme.warn)),
            Span::raw("Direct jump to tab"),
        ]),
        Line::from(vec![
            Span::styled("  p                : ", Style::default().fg(theme.warn)),
            Span::raw("Cycle Platform Profile (Quiet -> Balanced -> Performance)"),
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
            help_lines.push(Line::from(Span::styled("  -- Dashboard Keys --", Style::default().fg(theme.dim))));
            help_lines.push(Line::from(vec![
                Span::styled("  p                : ", Style::default().fg(theme.warn)),
                Span::raw("Cycle active platform profile immediately"),
            ]));
        }
        ActiveTab::Power => {
            help_lines.push(Line::from(Span::styled("  -- Power Tab Keys --", Style::default().fg(theme.dim))));
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
            help_lines.push(Line::from(Span::styled("  -- Fan Curves Tab Keys --", Style::default().fg(theme.dim))));
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
                Span::raw("Adjust point temperature +/- 1 C (Alt for 5 C)"),
            ]));
            help_lines.push(Line::from(vec![
                Span::styled("  w / Enter        : ", Style::default().fg(theme.warn)),
                Span::raw("Apply and write curves to hardware via DBus"),
            ]));
            help_lines.push(Line::from(vec![
                Span::styled("  x                : ", Style::default().fg(theme.warn)),
                Span::raw("Repair non-monotonic points (enforces running max)"),
            ]));
            help_lines.push(Line::from(vec![
                Span::styled("  u                : ", Style::default().fg(theme.warn)),
                Span::raw("Revert to last applied curve"),
            ]));
            help_lines.push(Line::from(vec![
                Span::styled("  r                : ", Style::default().fg(theme.warn)),
                Span::raw("Reset curves to factory BIOS defaults (confirm)"),
            ]));
            help_lines.push(Line::from(vec![
                Span::styled("  s / S            : ", Style::default().fg(theme.warn)),
                Span::raw("Cycle presets (Silent/Balanced/Aggressive) / Save Custom"),
            ]));
            help_lines.push(Line::from(vec![
                Span::styled("  f                : ", Style::default().fg(theme.warn)),
                Span::raw("Switch active fan (CPU <-> GPU)"),
            ]));
            help_lines.push(Line::from(vec![
                Span::styled("  t                : ", Style::default().fg(theme.warn)),
                Span::raw("Toggle 8-point numerical table"),
            ]));
            help_lines.push(Line::from(vec![
                Span::styled("  o                : ", Style::default().fg(theme.warn)),
                Span::raw("Toggle other fan comparison overlay"),
            ]));
        }
        ActiveTab::Aura => {
            help_lines.push(Line::from(Span::styled("  -- RGB Lighting Keys --", Style::default().fg(theme.dim))));
            help_lines.push(Line::from(vec![
                Span::styled("  j / k or Up/Down : ", Style::default().fg(theme.warn)),
                Span::raw("Select field (skips effect fields when backlight is Off)"),
            ]));
            help_lines.push(Line::from(vec![
                Span::styled("  h / l or L/R     : ", Style::default().fg(theme.warn)),
                Span::raw("Adjust brightness, mode, color channel, or preset"),
            ]));
            help_lines.push(Line::from(vec![
                Span::styled("  c / Enter        : ", Style::default().fg(theme.warn)),
                Span::raw("Open 24-bit RGB/HSV Color Picker modal (live keyboard preview)"),
            ]));
            help_lines.push(Line::from(vec![
                Span::styled("  w                : ", Style::default().fg(theme.warn)),
                Span::raw("Save current lighting parameters and power states"),
            ]));
        }
        ActiveTab::Gpu => {
            help_lines.push(Line::from(Span::styled("  -- GPU Tab Keys --", Style::default().fg(theme.dim))));
            help_lines.push(Line::from(vec![
                Span::styled("  j / k or Up/Down : ", Style::default().fg(theme.warn)),
                Span::raw("Select target GPU mode"),
            ]));
            help_lines.push(Line::from(vec![
                Span::styled("  Enter            : ", Style::default().fg(theme.warn)),
                Span::raw("Switch mode (opens confirmation stating required action)"),
            ]));
        }
        ActiveTab::Armoury => {
            help_lines.push(Line::from(Span::styled("  -- Armoury Tab Keys --", Style::default().fg(theme.dim))));
            help_lines.push(Line::from(vec![
                Span::styled("  j / k or Up/Down : ", Style::default().fg(theme.warn)),
                Span::raw("Select attribute"),
            ]));
            help_lines.push(Line::from(vec![
                Span::styled("  h / l or Enter   : ", Style::default().fg(theme.warn)),
                Span::raw("Adjust or toggle attribute (confirm modal on MUX write)"),
            ]));
            help_lines.push(Line::from(vec![
                Span::styled("  u                : ", Style::default().fg(theme.warn)),
                Span::raw("Toggle visibility of unavailable attributes"),
            ]));
            help_lines.push(Line::from(vec![
                Span::styled("  r                : ", Style::default().fg(theme.warn)),
                Span::raw("Restore hardware default for selected attribute"),
            ]));
        }
        ActiveTab::About => {
            help_lines.push(Line::from(Span::styled("  -- About Tab Keys --", Style::default().fg(theme.dim))));
            help_lines.push(Line::from(vec![
                Span::styled("  y                : ", Style::default().fg(theme.warn)),
                Span::raw("Export system diagnostics report to ~/rogtui-diagnostics.txt"),
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
