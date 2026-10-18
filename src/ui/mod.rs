pub mod armoury;
pub mod aura;
pub mod dashboard;
pub mod fancurve;
pub mod gpu;
pub mod power;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

use crate::app::{ActiveTab, AppState};
use crate::backend::Backend;

pub fn render_ui<B: Backend>(f: &mut Frame, app: &AppState<B>) {
    let size = f.area();

    // Enforce minimum terminal size: under 60x16, render small warning
    if size.width < 60 || size.height < 16 {
        render_terminal_too_small(f, size);
        return;
    }

    // Chrome is exactly 2 rows: 1 row top header, 1 row bottom footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Top header line
            Constraint::Min(14),   // Tab content area
            Constraint::Length(1), // Bottom footer line
        ])
        .split(size);

    // 1. One-Line Header: "rogtui   1 Dash   2 Power   3 Fan   4 RGB   5 GPU   6 Armoury"
    let mut header_spans = vec![
        Span::styled("rogtui", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("   "),
    ];

    let tabs = [
        (ActiveTab::Dashboard, "1 Dash"),
        (ActiveTab::Power, "2 Power"),
        (ActiveTab::FanCurves, "3 Fan"),
        (ActiveTab::Aura, "4 RGB"),
        (ActiveTab::Gpu, "5 GPU"),
        (ActiveTab::Armoury, "6 Armoury"),
    ];

    for (tab, name) in tabs {
        let is_active = app.current_tab == tab;
        let style = if is_active {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        header_spans.push(Span::styled(name, style));
        header_spans.push(Span::raw("   "));
    }

    f.render_widget(Paragraph::new(Line::from(header_spans)), chunks[0]);

    // 2. Tab Content (No outer box!)
    match app.current_tab {
        ActiveTab::Dashboard => dashboard::render_dashboard(f, app, chunks[1]),
        ActiveTab::Power => power::render_power(f, app, chunks[1]),
        ActiveTab::FanCurves => fancurve::render_fancurve(f, app, chunks[1]),
        ActiveTab::Aura => aura::render_aura(f, app, chunks[1]),
        ActiveTab::Gpu => gpu::render_gpu(f, app, chunks[1]),
        ActiveTab::Armoury => armoury::render_armoury(f, app, chunks[1]),
    }

    // 3. One-Line Footer: Context keybinds (left) + Status/Hint (right)
    render_footer(f, app, chunks[2]);

    // 4. Overlays (Centered modals)
    if app.show_help {
        render_help_overlay(f, size);
    }
    if let Some((prompt, _)) = &app.confirm_modal {
        render_confirm_modal(f, prompt, size);
    }
}

fn render_footer<B: Backend>(f: &mut Frame, app: &AppState<B>, area: Rect) {
    let keybinds: Vec<Span> = match app.current_tab {
        ActiveTab::Dashboard => vec![
            Span::styled("tab", Style::default().fg(Color::DarkGray)),
            Span::raw(" switch  "),
            Span::styled("p", Style::default().fg(Color::DarkGray)),
            Span::raw(" cycle profile  "),
            Span::styled("?", Style::default().fg(Color::DarkGray)),
            Span::raw(" help  "),
            Span::styled("q", Style::default().fg(Color::DarkGray)),
            Span::raw(" quit"),
        ],
        ActiveTab::Power => vec![
            Span::styled("j/k", Style::default().fg(Color::DarkGray)),
            Span::raw(" select  "),
            Span::styled("h/l", Style::default().fg(Color::DarkGray)),
            Span::raw(" adjust  "),
            Span::styled("enter", Style::default().fg(Color::DarkGray)),
            Span::raw(" toggle  "),
            Span::styled("?", Style::default().fg(Color::DarkGray)),
            Span::raw(" help  "),
            Span::styled("q", Style::default().fg(Color::DarkGray)),
            Span::raw(" quit"),
        ],
        ActiveTab::FanCurves => vec![
            Span::styled("h/l", Style::default().fg(Color::DarkGray)),
            Span::raw(" pt  "),
            Span::styled("j/k", Style::default().fg(Color::DarkGray)),
            Span::raw(" %  "),
            Span::styled("H/L", Style::default().fg(Color::DarkGray)),
            Span::raw(" C  "),
            Span::styled("w", Style::default().fg(Color::DarkGray)),
            Span::raw(" apply  "),
            Span::styled("u", Style::default().fg(Color::DarkGray)),
            Span::raw(" undo  "),
            Span::styled("f", Style::default().fg(Color::DarkGray)),
            Span::raw(" fan  "),
            Span::styled("p", Style::default().fg(Color::DarkGray)),
            Span::raw(" prof  "),
            Span::styled("s", Style::default().fg(Color::DarkGray)),
            Span::raw(" preset  "),
            Span::styled("o", Style::default().fg(Color::DarkGray)),
            Span::raw(" over  "),
            Span::styled("t", Style::default().fg(Color::DarkGray)),
            Span::raw(" tbl"),
        ],
        ActiveTab::Aura => vec![
            Span::styled("j/k", Style::default().fg(Color::DarkGray)),
            Span::raw(" select  "),
            Span::styled("h/l", Style::default().fg(Color::DarkGray)),
            Span::raw(" adjust  "),
            Span::styled("w", Style::default().fg(Color::DarkGray)),
            Span::raw(" apply  "),
            Span::styled("?", Style::default().fg(Color::DarkGray)),
            Span::raw(" help  "),
            Span::styled("q", Style::default().fg(Color::DarkGray)),
            Span::raw(" quit"),
        ],
        ActiveTab::Gpu => vec![
            Span::styled("j/k", Style::default().fg(Color::DarkGray)),
            Span::raw(" select mode  "),
            Span::styled("enter", Style::default().fg(Color::DarkGray)),
            Span::raw(" switch  "),
            Span::styled("?", Style::default().fg(Color::DarkGray)),
            Span::raw(" help  "),
            Span::styled("q", Style::default().fg(Color::DarkGray)),
            Span::raw(" quit"),
        ],
        ActiveTab::Armoury => vec![
            Span::styled("j/k", Style::default().fg(Color::DarkGray)),
            Span::raw(" select  "),
            Span::styled("h/l", Style::default().fg(Color::DarkGray)),
            Span::raw(" adjust  "),
            Span::styled("r", Style::default().fg(Color::DarkGray)),
            Span::raw(" default  "),
            Span::styled("?", Style::default().fg(Color::DarkGray)),
            Span::raw(" help  "),
            Span::styled("q", Style::default().fg(Color::DarkGray)),
            Span::raw(" quit"),
        ],
    };

    // Calculate status or clamp hint
    let status_span = if let Some((msg, is_error, _)) = &app.status_message {
        if *is_error {
            Span::styled(format!("err  {msg}"), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        } else {
            Span::styled(format!("ok  {msg}"), Style::default().fg(Color::Green))
        }
    } else if let Some((hint, _)) = &app.fan_clamp_hint {
        Span::styled(format!("hint: {hint}"), Style::default().fg(Color::Yellow))
    } else {
        Span::raw("")
    };

    let total_width = area.width as usize;
    let mut left_len = 0;
    for s in &keybinds {
        left_len += s.content.len();
    }
    let status_len = status_span.content.len();

    let mut line_spans = keybinds;
    if total_width > left_len + status_len + 2 {
        let padding = total_width - left_len - status_len;
        line_spans.push(Span::raw(" ".repeat(padding)));
        line_spans.push(status_span);
    }

    f.render_widget(Paragraph::new(Line::from(line_spans)), area);
}

fn render_terminal_too_small(f: &mut Frame, area: Rect) {
    let msg = vec![
        Line::from(Span::styled("Terminal Too Small", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::raw(format!("Current: {}x{}", area.width, area.height))),
        Line::from(Span::styled("Required: at least 60x16", Style::default().fg(Color::Yellow))),
        Line::from(""),
        Line::from(Span::raw("Resize the terminal to continue.")),
    ];
    let widget = Paragraph::new(msg).alignment(Alignment::Center).block(
        Block::default().borders(Borders::ALL).border_type(BorderType::Rounded),
    );
    f.render_widget(widget, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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

fn render_help_overlay(f: &mut Frame, area: Rect) {
    let popup_area = centered_rect(70, 80, area);
    f.render_widget(Clear, popup_area);

    let help_lines = vec![
        Line::from(Span::styled("  ROGTUI v0.1.0 (asusd v6.5.0 + supergfxd v5.2.7)", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("  -- Global Shortcuts --", Style::default().fg(Color::DarkGray))),
        Line::from(vec![
            Span::styled("  Tab / Shift+Tab  : ", Style::default().fg(Color::Yellow)),
            Span::raw("Cycle to next / previous tab"),
        ]),
        Line::from(vec![
            Span::styled("  1 .. 6           : ", Style::default().fg(Color::Yellow)),
            Span::raw("Direct jump to tab"),
        ]),
        Line::from(vec![
            Span::styled("  p                : ", Style::default().fg(Color::Yellow)),
            Span::raw("Quick cycle Platform Profile (Quiet -> Balanced -> Perf)"),
        ]),
        Line::from(vec![
            Span::styled("  ?                : ", Style::default().fg(Color::Yellow)),
            Span::raw("Toggle this help overlay"),
        ]),
        Line::from(vec![
            Span::styled("  q / Esc          : ", Style::default().fg(Color::Yellow)),
            Span::raw("Close modal / quit application"),
        ]),
        Line::from(""),
        Line::from(Span::styled("  -- Fan Curves Tab Controls --", Style::default().fg(Color::DarkGray))),
        Line::from(vec![
            Span::styled("  h / l or L / R   : ", Style::default().fg(Color::Yellow)),
            Span::raw("Select previous / next point"),
        ]),
        Line::from(vec![
            Span::styled("  j / k            : ", Style::default().fg(Color::Yellow)),
            Span::raw("Adjust fan speed +/- 1% (Shift+j/k = 5%)"),
        ]),
        Line::from(vec![
            Span::styled("  H / L (Shift+h/l): ", Style::default().fg(Color::Yellow)),
            Span::raw("Adjust point temperature +/- 1 C (Alt = 5 C)"),
        ]),
        Line::from(vec![
            Span::styled("  w / Enter        : ", Style::default().fg(Color::Yellow)),
            Span::raw("Apply and write curve to DBus (re-reads from daemon)"),
        ]),
        Line::from(vec![
            Span::styled("  u                : ", Style::default().fg(Color::Yellow)),
            Span::raw("Revert to last applied curve"),
        ]),
        Line::from(vec![
            Span::styled("  r                : ", Style::default().fg(Color::Yellow)),
            Span::raw("Reset curves to factory BIOS defaults (confirm)"),
        ]),
        Line::from(vec![
            Span::styled("  f                : ", Style::default().fg(Color::Yellow)),
            Span::raw("Switch active fan (CPU <-> GPU)"),
        ]),
        Line::from(vec![
            Span::styled("  s / S            : ", Style::default().fg(Color::Yellow)),
            Span::raw("Cycle presets (Silent/Balanced/Aggressive) / Save Custom"),
        ]),
        Line::from(vec![
            Span::styled("  o                : ", Style::default().fg(Color::Yellow)),
            Span::raw("Toggle other fan comparison overlay"),
        ]),
        Line::from(vec![
            Span::styled("  t                : ", Style::default().fg(Color::Yellow)),
            Span::raw("Toggle 8-point numerical table"),
        ]),
        Line::from(vec![
            Span::styled("  a                : ", Style::default().fg(Color::Yellow)),
            Span::raw("Toggle 'Apply to all fans' lock"),
        ]),
        Line::from(""),
        Line::from(Span::styled("  Press Esc or '?' to close.", Style::default().fg(Color::DarkGray))),
    ];

    let widget = Paragraph::new(help_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Help "),
    );
    f.render_widget(widget, popup_area);
}

fn render_confirm_modal(f: &mut Frame, prompt: &str, area: Rect) {
    let popup_area = centered_rect(60, 25, area);
    f.render_widget(Clear, popup_area);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(prompt, Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::DarkGray)),
            Span::styled("'y'", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(" to Confirm   |   Press ", Style::default().fg(Color::DarkGray)),
            Span::styled("'n' or Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(" to Cancel", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let widget = Paragraph::new(lines)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(Span::styled(" Confirm Action ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        );
    f.render_widget(widget, popup_area);
}
