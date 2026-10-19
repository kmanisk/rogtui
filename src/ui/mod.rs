pub mod aura;
pub mod dashboard;
pub mod fancurve;
pub mod power;
pub mod system;
pub mod theme;
pub mod widgets;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::{ActiveTab, AppState};
use crate::backend::Backend;
use crate::ui::widgets::{
    panel_block, render_confirm_modal, render_footer, render_help_overlay, render_tab_bar,
};

pub fn render_ui<B: Backend>(f: &mut Frame, app: &AppState<B>) {
    let size = f.area();

    // Enforce minimum terminal size: under 80x24, render warning
    if size.width < 80 || size.height < 24 {
        render_terminal_too_small(f, size, &app.theme);
        return;
    }

    // 3 vertical chunks: Tab bar (1), Tab content (min), Footer (1)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // First row is ONLY the tab bar
            Constraint::Min(10),   // Content area
            Constraint::Length(1), // Single-line footer
        ])
        .split(size);

    render_tab_bar(f, chunks[0], app.current_tab, &app.theme);

    match app.current_tab {
        ActiveTab::Dashboard => dashboard::render_dashboard(f, app, chunks[1]),
        ActiveTab::Power => power::render_power(f, app, chunks[1]),
        ActiveTab::FanCurves => fancurve::render_fancurve(f, app, chunks[1]),
        ActiveTab::Aura => aura::render_aura(f, app, chunks[1]),
        ActiveTab::System => system::render_system(f, app, chunks[1]),
    }

    render_footer(f, chunks[2], app, &app.theme);

    if app.show_help {
        render_help_overlay(f, size, app.current_tab, &app.theme);
    }
    if let Some((prompt, _)) = &app.confirm_modal {
        render_confirm_modal(f, prompt, size, &app.theme);
    }
}

fn render_terminal_too_small(f: &mut Frame, area: Rect, theme: &crate::ui::theme::Theme) {
    let msg = vec![
        Line::from(Span::styled(
            "Terminal Too Small",
            Style::default().fg(theme.err).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::raw(format!("Current: {}x{}", area.width, area.height))),
        Line::from(Span::styled(
            "Required: at least 80x24",
            Style::default().fg(theme.warn),
        )),
        Line::from(""),
        Line::from(Span::raw("Please resize your terminal window to continue.")),
    ];
    let widget = Paragraph::new(msg)
        .alignment(Alignment::Center)
        .block(panel_block(" Notice ", false, theme));
    f.render_widget(widget, area);
}
