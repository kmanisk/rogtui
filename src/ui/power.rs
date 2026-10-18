use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::AppState;
use crate::backend::Backend;

pub fn render_power<B: Backend>(f: &mut Frame, app: &AppState<B>, area: Rect) {
    let fields = [
        ("Platform Profile", format!("< {} >", app.platform_profile.as_str())),
        ("AC Connected Profile", format!("< {} >", app.profile_ac.as_str())),
        ("Battery Mode Profile", format!("< {} >", app.profile_battery.as_str())),
        ("Auto-switch on AC", if app.auto_ac { "Enabled".to_string() } else { "Disabled".to_string() }),
        ("Auto-switch on Battery", if app.auto_battery { "Enabled".to_string() } else { "Disabled".to_string() }),
        ("Battery Charge Limit", format!("< {}% >  (step 5)", app.charge_limit)),
        ("One-Shot Full Charge", "[ Trigger 100% until unplug ]".to_string()),
        ("Link Profile to CPU EPP", if app.linked_epp { "Linked".to_string() } else { "Unlinked".to_string() }),
        ("Disable NV Powerd on Bat", if app.disable_powerd_battery { "Yes".to_string() } else { "No".to_string() }),
    ];

    let mut lines = Vec::new();
    lines.push(Line::from(Span::styled("-- Power & Battery Configuration --", Style::default().fg(Color::DarkGray))));

    for (idx, (label, val)) in fields.iter().enumerate() {
        let is_selected = app.power_selected_field == idx;
        let prefix = if is_selected { "> " } else { "  " };

        let label_style = if is_selected {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let val_style = if is_selected {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        lines.push(Line::from(vec![
            Span::styled(format!("{prefix}{label: <26}: "), label_style),
            Span::styled(val, val_style),
        ]));
    }

    f.render_widget(Paragraph::new(lines), area);
}
