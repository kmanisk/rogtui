use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::app::AppState;
use crate::backend::Backend;

pub fn render_armoury<B: Backend>(f: &mut Frame, app: &AppState<B>, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(12), Constraint::Length(4)])
        .split(area);

    let mut lines = Vec::new();
    lines.push(Line::from(""));

    if app.armoury_attributes.is_empty() {
        lines.push(Line::from(Span::styled(
            "  No firmware attributes reported by asusd.",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for (idx, attr) in app.armoury_attributes.iter().enumerate() {
            let is_selected = app.armoury_selected_idx == idx;
            let prefix = if is_selected { "  > " } else { "    " };

            let label_style = if !attr.is_available {
                Style::default().fg(Color::DarkGray)
            } else if is_selected {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let val_str = if !attr.is_available {
                "[ UNAVAILABLE ON THIS HARDWARE ]".to_string()
            } else {
                match attr.kind {
                    crate::backend::types::AttributeKind::Toggle => {
                        if attr.current_value == 1 {
                            "[ ON / ENABLED ]".to_string()
                        } else {
                            "[ OFF / DISABLED ]".to_string()
                        }
                    }
                    crate::backend::types::AttributeKind::Enum => {
                        format!("< Value: {} >  (Allowed: {:?})", attr.current_value, attr.possible_values)
                    }
                    crate::backend::types::AttributeKind::Slider => {
                        format!("< {} >  (Range: {}..={})", attr.current_value, attr.min_value, attr.max_value)
                    }
                    crate::backend::types::AttributeKind::ReadOnly => {
                        format!("{} (Read-only)", attr.current_value)
                    }
                }
            };

            let val_style = if !attr.is_available {
                Style::default().fg(Color::DarkGray)
            } else if is_selected {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };

            lines.push(Line::from(vec![
                Span::styled(format!("{prefix}{: <38}: ", attr.display_name), label_style),
                Span::styled(val_str, val_style),
            ]));
            lines.push(Line::from(""));
        }
    }

    let list_widget = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(Span::styled(" Firmware & Armoury Hardware Attributes ", Style::default().add_modifier(Modifier::BOLD))),
    );
    f.render_widget(list_widget, chunks[0]);

    let guide_line = Line::from(vec![
        Span::styled("  j/k or Up/Down: ", Style::default().fg(Color::Yellow)),
        Span::raw("Select attribute   |   "),
        Span::styled("h/l or Left/Right: ", Style::default().fg(Color::Yellow)),
        Span::raw("Adjust / toggle value   |   "),
        Span::styled("r: ", Style::default().fg(Color::Yellow)),
        Span::raw("Restore hardware default"),
    ]);
    let guide_widget = Paragraph::new(guide_line).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Navigation "),
    );
    f.render_widget(guide_widget, chunks[1]);
}
