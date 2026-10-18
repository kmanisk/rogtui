use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use crate::app::AppState;
use crate::backend::Backend;

pub fn render_power<B: Backend>(f: &mut Frame, app: &AppState<B>, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(12), Constraint::Length(4)])
        .split(area);

    let fields = [
        ("Platform Profile", format!("< {} >", app.platform_profile.as_str())),
        ("AC Connected Profile", format!("< {} >", app.profile_ac.as_str())),
        ("Battery Mode Profile", format!("< {} >", app.profile_battery.as_str())),
        ("Auto-switch on AC", if app.auto_ac { "[ Enabled ]".to_string() } else { "[ Disabled ]".to_string() }),
        ("Auto-switch on Battery", if app.auto_battery { "[ Enabled ]".to_string() } else { "[ Disabled ]".to_string() }),
        ("Battery Charge Limit", format!("< {}% >  [20-100% step 5]", app.charge_limit)),
        ("One-Shot Full Charge", "[ Press Enter to Trigger 100% Charge ]".to_string()),
        ("Link Profile to CPU EPP", if app.linked_epp { "[ Linked ]".to_string() } else { "[ Unlinked ]".to_string() }),
        ("Disable NV Powerd on Bat", if app.disable_powerd_battery { "[ Yes ]".to_string() } else { "[ No ]".to_string() }),
    ];

    let mut lines = Vec::new();
    lines.push(Line::from(""));

    for (idx, (label, val)) in fields.iter().enumerate() {
        let is_selected = app.power_selected_field == idx;
        let prefix = if is_selected { "  > " } else { "    " };
        let label_style = if is_selected {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let val_style = if is_selected {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        lines.push(Line::from(vec![
            Span::styled(format!("{prefix}{label: <28}: "), label_style),
            Span::styled(val, val_style),
        ]));
        lines.push(Line::from(""));
    }

    let widget = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(Span::styled(" Power Profiles & Charging Policy ", Style::default().add_modifier(Modifier::BOLD))),
    );
    f.render_widget(widget, chunks[0]);

    let help_line = Line::from(vec![
        Span::styled("  j/k or Up/Down: ", Style::default().fg(Color::Yellow)),
        Span::raw("Select field   |   "),
        Span::styled("h/l or Left/Right: ", Style::default().fg(Color::Yellow)),
        Span::raw("Adjust value / toggle   |   "),
        Span::styled("Enter: ", Style::default().fg(Color::Yellow)),
        Span::raw("Execute action"),
    ]);
    let help_widget = Paragraph::new(help_line).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Navigation "),
    );
    f.render_widget(help_widget, chunks[1]);
}
