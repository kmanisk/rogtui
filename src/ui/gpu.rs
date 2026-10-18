use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::app::AppState;
use crate::backend::Backend;

pub fn render_gpu<B: Backend>(f: &mut Frame, app: &AppState<B>, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Min(8),
            Constraint::Length(4),
        ])
        .split(area);

    // Current Status Block
    let status_lines = vec![
        Line::from(vec![
            Span::styled("  Current GPU Mode:     ", Style::default().fg(Color::DarkGray)),
            Span::styled(app.gpu_mode.as_str(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  dGPU Power State:     ", Style::default().fg(Color::DarkGray)),
            Span::styled(app.gpu_power.as_str(), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("  Pending Mode:         ", Style::default().fg(Color::DarkGray)),
            Span::styled(app.gpu_pending_mode.as_str(), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("  Required Action:      ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                app.gpu_pending_action.as_str(),
                if app.gpu_pending_action != crate::backend::types::PendingUserAction::Nothing {
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Green)
                },
            ),
        ]),
    ];

    let status_widget = Paragraph::new(status_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(Span::styled(" supergfxctl Daemon Status ", Style::default().add_modifier(Modifier::BOLD))),
    );
    f.render_widget(status_widget, chunks[0]);

    // Mode Selector Block
    let mut mode_lines = Vec::new();
    mode_lines.push(Line::from(""));
    mode_lines.push(Line::from(Span::styled(
        "  Select target graphics mode (press Enter to request mode switch):",
        Style::default().fg(Color::DarkGray),
    )));
    mode_lines.push(Line::from(""));

    for (idx, mode) in app.gpu_supported.iter().enumerate() {
        let is_selected = app.gpu_selected_idx == idx;
        let is_current = *mode == app.gpu_mode;
        let prefix = if is_selected { "   > [x] " } else { "     [ ] " };

        let mut spans = vec![
            Span::styled(
                prefix,
                if is_selected {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                },
            ),
            Span::styled(
                mode.as_str(),
                if is_selected {
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Gray)
                },
            ),
        ];

        if is_current {
            spans.push(Span::styled("  (ACTIVE)", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)));
        }

        mode_lines.push(Line::from(spans));
        mode_lines.push(Line::from(""));
    }

    let mode_widget = Paragraph::new(mode_lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(Span::styled(" Supported Hardware Modes ", Style::default().add_modifier(Modifier::BOLD))),
    );
    f.render_widget(mode_widget, chunks[1]);

    // Footer guide
    let guide_line = Line::from(vec![
        Span::styled("  j/k or Up/Down: ", Style::default().fg(Color::Yellow)),
        Span::raw("Select mode   |   "),
        Span::styled("Enter: ", Style::default().fg(Color::Yellow)),
        Span::raw("Switch mode with confirmation prompt"),
    ]);
    let guide_widget = Paragraph::new(guide_line).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Navigation "),
    );
    f.render_widget(guide_widget, chunks[2]);
}
