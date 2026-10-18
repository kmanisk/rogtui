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
        Span::styled(
            " ROGTUI ",
            Style::default()
                .bg(theme.accent)
                .fg(theme.sel_fg)
                .add_modifier(Modifier::BOLD),
        ),
    ];

    if app.is_logging_metrics {
        left_spans.push(Span::raw(" "));
        left_spans.push(Span::styled(
            format!("[● REC: {} samples]", app.metrics_logged_count),
            Style::default().fg(theme.err).add_modifier(Modifier::BOLD),
        ));
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

    let mut spans = Vec::new();
    for (i, (tab, name)) in tabs.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" | ", Style::default().fg(theme.dim)));
        }
        let is_active = *tab == active_tab;
        let style = if is_active {
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            Style::default().fg(theme.dim)
        };
        spans.push(Span::styled(*name, style));
    }

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

pub fn render_footer<B: Backend>(f: &mut Frame, area: Rect, app: &AppState<B>, theme: &Theme) {
    let keybinds: Vec<(&str, &str)> = if app.show_color_picker_modal {
        vec![
            ("j/k", "slider"),
            ("h/l", "adjust"),
            ("enter", "apply"),
            ("esc", "cancel"),
        ]
    } else if app.show_advanced_power_modal {
        vec![
            ("esc/enter", "close"),
        ]
    } else {
        match app.current_tab {
            ActiveTab::Dashboard => vec![
                ("tab", "switch"),
                ("p", "profile"),
                ("l", if app.is_logging_metrics { "stop log" } else { "log metrics" }),
                ("a", "limits"),
                ("o", "overdrive"),
                ("b", "auto-bright"),
                ("?", "help"),
                ("q", "quit"),
            ],
            ActiveTab::Power => vec![
                ("j/k", "select"),
                ("h/l", "adjust"),
                ("enter", "toggle"),
                ("?", "help"),
                ("q", "quit"),
            ],
            ActiveTab::FanCurves => vec![
                ("h/l", "pt"),
                ("j/k", "%"),
                ("H/L", "C"),
                ("w", "apply"),
                ("x", "repair"),
                ("s", "preset"),
                ("f", "fan"),
                ("?", "help"),
                ("q", "quit"),
            ],
            ActiveTab::Aura => {
                if app.aura_brightness == 0 {
                    vec![
                        ("h/l", "brightness (Off)"),
                        ("j/k", "select"),
                        ("w", "save to hw"),
                        ("?", "help"),
                        ("q", "quit"),
                    ]
                } else {
                    vec![
                        ("j/k", "select"),
                        ("h/l", "adjust"),
                        ("c/enter", "picker"),
                        ("w", "save to hw"),
                        ("?", "help"),
                        ("q", "quit"),
                    ]
                }
            }
        ActiveTab::Gpu => vec![
            ("j/k", "select mode"),
            ("enter", "switch"),
            ("?", "help"),
            ("q", "quit"),
        ],
        ActiveTab::Armoury => vec![
            ("j/k", "select"),
            ("h/l", "adjust"),
            ("enter", "toggle"),
            ("u", "unavail"),
            ("r", "reset"),
            ("?", "help"),
            ("q", "quit"),
        ],
        ActiveTab::About => vec![
            ("tab", "switch"),
            ("y", "export report"),
            ("?", "help"),
            ("q", "quit"),
        ],
        }
    };

    let mut spans = Vec::new();
    for (i, (key, desc)) in keybinds.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" | ", Style::default().fg(theme.dim)));
        }
        spans.push(Span::styled(*key, Style::default().fg(theme.dim)));
        spans.push(Span::raw(" "));
        spans.push(Span::styled(*desc, Style::default().fg(theme.fg)));
    }

    // Status / Toast right-aligned
    let status_span = if let Some((msg, is_error, _)) = &app.status_message {
        if *is_error {
            Span::styled(format!("err  {msg}"), Style::default().fg(theme.err).add_modifier(Modifier::BOLD))
        } else {
            Span::styled(format!("ok  {msg}"), Style::default().fg(theme.ok))
        }
    } else if let Some((hint, _)) = &app.fan_clamp_hint {
        Span::styled(format!("hint: {hint}"), Style::default().fg(theme.warn))
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
    color: Color,
    theme: &Theme,
) -> Line<'static> {
    let p = percent.clamp(0, 100) as usize;
    let total_substeps = width * 8;
    let filled_substeps = (p * total_substeps) / 100;
    let full_blocks = filled_substeps / 8;
    let rem = filled_substeps % 8;

    let mut spans = Vec::new();
    spans.push(Span::styled(format!("{label: <16}: "), Style::default().fg(theme.dim)));
    spans.push(Span::styled("[", Style::default().fg(theme.dim)));

    if full_blocks > 0 {
        spans.push(Span::styled("█".repeat(full_blocks), Style::default().fg(color)));
    }
    if rem > 0 && full_blocks < width {
        const FRACTIONAL: [char; 8] = [' ', '▏', '▎', '▍', '▌', '▋', '▊', '▉'];
        spans.push(Span::styled(FRACTIONAL[rem].to_string(), Style::default().fg(color)));
    }

    let filled_cells = if rem > 0 { full_blocks + 1 } else { full_blocks };
    let empty_cells = width.saturating_sub(filled_cells);
    if empty_cells > 0 {
        spans.push(Span::styled("░".repeat(empty_cells), Style::default().fg(theme.dim)));
    }

    spans.push(Span::styled("]", Style::default().fg(theme.dim)));
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
