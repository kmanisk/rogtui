use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::AppState;
use crate::backend::Backend;

pub fn render_gpu<B: Backend>(f: &mut Frame, app: &AppState<B>, area: Rect) {
    let mut lines = Vec::new();

    // Section 1: Daemon Status
    lines.push(Line::from(Span::styled("-- supergfxd Daemon Status --", Style::default().fg(Color::DarkGray))));
    lines.push(Line::from(vec![
        Span::styled("  Current GPU Mode: ", Style::default().fg(Color::DarkGray)),
        Span::styled(app.gpu_mode.as_str(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  dGPU Power State: ", Style::default().fg(Color::DarkGray)),
        Span::styled(app.gpu_power.as_str(), Style::default().fg(Color::Yellow)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Pending Mode    : ", Style::default().fg(Color::DarkGray)),
        Span::styled(app.gpu_pending_mode.as_str(), Style::default().fg(Color::White)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Required Action : ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            app.gpu_pending_action.as_str(),
            if app.gpu_pending_action != crate::backend::types::PendingUserAction::Nothing {
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Green)
            },
        ),
    ]));

    lines.push(Line::from(""));

    // Section 2: Mode Selector
    lines.push(Line::from(Span::styled("-- Supported Hardware Modes (Enter to switch) --", Style::default().fg(Color::DarkGray))));

    for (idx, mode) in app.gpu_supported.iter().enumerate() {
        let is_selected = app.gpu_selected_idx == idx;
        let is_current = *mode == app.gpu_mode;
        let prefix = if is_selected { "> " } else { "  " };
        let box_marker = if is_current { "[x] " } else { "[ ] " };

        let mut spans = vec![
            Span::styled(prefix, if is_selected { Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::DarkGray) }),
            Span::styled(box_marker, if is_current { Style::default().fg(Color::Green).add_modifier(Modifier::BOLD) } else { Style::default().fg(Color::DarkGray) }),
            Span::styled(
                mode.as_str(),
                if is_selected {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else if is_current {
                    Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Gray)
                },
            ),
        ];

        if is_current {
            spans.push(Span::styled("  (ACTIVE)", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)));
        }

        lines.push(Line::from(spans));
    }

    f.render_widget(Paragraph::new(lines), area);
}
