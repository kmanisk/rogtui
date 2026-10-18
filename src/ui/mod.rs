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
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Tabs},
    Frame,
};

use crate::app::{ActiveTab, AppState};
use crate::backend::Backend;

pub fn render_ui<B: Backend>(f: &mut Frame, app: &AppState<B>) {
    let size = f.area();

    // Top Header + Tabs + Body + Status Footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(size);

    // 1. Header Bar
    let header_text = vec![Line::from(vec![
        Span::styled(" ROGTUI ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD | Modifier::REVERSED)),
        Span::raw(" "),
        Span::styled("ASUS Linux ROG & TUF Control Center", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled("v0.1.0 (asusd & supergfxd)", Style::default().fg(Color::DarkGray)),
    ])];
    let header_widget = Paragraph::new(header_text).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    );
    f.render_widget(header_widget, chunks[0]);

    // 2. Tabs Bar
    let tab_titles: Vec<Line> = ActiveTab::all()
        .iter()
        .map(|t| Line::from(Span::raw(t.title())))
        .collect();

    let tabs_widget = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .select(app.current_tab as usize)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )
        .divider(Span::styled(" | ", Style::default().fg(Color::DarkGray)));
    f.render_widget(tabs_widget, chunks[1]);

    // 3. Tab Body
    match app.current_tab {
        ActiveTab::Dashboard => dashboard::render_dashboard(f, app, chunks[2]),
        ActiveTab::Power => power::render_power(f, app, chunks[2]),
        ActiveTab::FanCurves => fancurve::render_fancurve(f, app, chunks[2]),
        ActiveTab::Aura => aura::render_aura(f, app, chunks[2]),
        ActiveTab::Gpu => gpu::render_gpu(f, app, chunks[2]),
        ActiveTab::Armoury => armoury::render_armoury(f, app, chunks[2]),
    }

    // 4. Status Bar / Footer
    let footer_line = if let Some((msg, is_error, _)) = &app.status_message {
        Line::from(vec![
            Span::styled(
                if *is_error { " [ERROR] " } else { " [OK] " },
                Style::default()
                    .fg(if *is_error { Color::Red } else { Color::Green })
                    .add_modifier(Modifier::BOLD | Modifier::REVERSED),
            ),
            Span::raw(" "),
            Span::styled(msg, Style::default().add_modifier(Modifier::BOLD)),
        ])
    } else {
        Line::from(vec![
            Span::styled(" [Tab/1-6] ", Style::default().fg(Color::Yellow)),
            Span::raw("Switch Tab  "),
            Span::styled(" [?] ", Style::default().fg(Color::Yellow)),
            Span::raw("Help  "),
            Span::styled(" [p] ", Style::default().fg(Color::Yellow)),
            Span::raw("Cycle Profile  "),
            Span::styled(" [q] ", Style::default().fg(Color::Yellow)),
            Span::raw("Quit"),
        ])
    };

    let footer_widget = Paragraph::new(footer_line).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Status "),
    );
    f.render_widget(footer_widget, chunks[3]);

    // 5. Help Overlay Modal
    if app.show_help {
        render_help_overlay(f, size);
    }

    // 6. Confirmation Modal
    if let Some((prompt, _)) = &app.confirm_modal {
        render_confirm_modal(f, prompt, size);
    }
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
    let popup_area = centered_rect(65, 75, area);
    f.render_widget(Clear, popup_area);

    let help_lines = vec![
        Line::from(Span::styled("  ROGTUI KEYBOARD COMMANDS", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Tab / Shift+Tab  : ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("Cycle to next / previous tab"),
        ]),
        Line::from(vec![
            Span::styled("  1 .. 6           : ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("Directly jump to tab number"),
        ]),
        Line::from(vec![
            Span::styled("  j / k or Up/Down : ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("Move selection up / down"),
        ]),
        Line::from(vec![
            Span::styled("  h / l or L/R     : ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("Adjust value or toggle field"),
        ]),
        Line::from(vec![
            Span::styled("  Enter / Space    : ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("Execute action or confirm"),
        ]),
        Line::from(vec![
            Span::styled("  p                : ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("Quick-cycle Platform Profile (Quiet -> Balanced -> Perf)"),
        ]),
        Line::from(vec![
            Span::styled("  w                : ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("Apply and write fan curve or lighting data to hardware"),
        ]),
        Line::from(vec![
            Span::styled("  r                : ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("Reset fan curve or armoury attribute to default"),
        ]),
        Line::from(vec![
            Span::styled("  u                : ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("Undo / revert to last-known-good fan curve"),
        ]),
        Line::from(vec![
            Span::styled("  ?                : ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("Toggle this help screen"),
        ]),
        Line::from(vec![
            Span::styled("  q / Esc          : ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("Close modal or exit rogtui"),
        ]),
        Line::from(""),
        Line::from(Span::styled("  Press Esc or '?' to close this help window.", Style::default().fg(Color::DarkGray))),
    ];

    let widget = Paragraph::new(help_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(Span::styled(" Keyboard Shortcuts Help ", Style::default().add_modifier(Modifier::BOLD))),
    );
    f.render_widget(widget, popup_area);
}

fn render_confirm_modal(f: &mut Frame, prompt: &str, area: Rect) {
    let popup_area = centered_rect(55, 30, area);
    f.render_widget(Clear, popup_area);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled(format!("  {prompt}"), Style::default().add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Press ", Style::default().fg(Color::DarkGray)),
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
                .border_type(BorderType::Double)
                .title(Span::styled(" CONFIRM ACTION ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        );
    f.render_widget(widget, popup_area);
}
