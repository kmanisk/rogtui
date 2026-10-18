pub mod about;
pub mod armoury;
pub mod aura;
pub mod dashboard;
pub mod fancurve;
pub mod gpu;
pub mod power;
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
    panel_block, render_confirm_modal, render_footer, render_header, render_help_overlay,
    render_tab_bar,
};

pub fn render_ui<B: Backend>(f: &mut Frame, app: &AppState<B>) {
    let size = f.area();

    // Enforce minimum terminal size: under 60x16, render small warning
    if size.width < 60 || size.height < 16 {
        render_terminal_too_small(f, size, &app.theme);
        return;
    }

    // 4 vertical chunks: Header (1), Tab bar (1), Tab content (min), Footer (1)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Top header badge + machine + daemons
            Constraint::Length(1), // Tab bar with '|' separators
            Constraint::Min(12),   // Tab content area
            Constraint::Length(1), // Footer keybinds + right toast
        ])
        .split(size);

    render_header(f, chunks[0], app, &app.theme);
    render_tab_bar(f, chunks[1], app.current_tab, &app.theme);

    match app.current_tab {
        ActiveTab::Dashboard => dashboard::render_dashboard(f, app, chunks[2]),
        ActiveTab::Power => power::render_power(f, app, chunks[2]),
        ActiveTab::FanCurves => fancurve::render_fancurve(f, app, chunks[2]),
        ActiveTab::Aura => aura::render_aura(f, app, chunks[2]),
        ActiveTab::Gpu => gpu::render_gpu(f, app, chunks[2]),
        ActiveTab::Armoury => armoury::render_armoury(f, app, chunks[2]),
        ActiveTab::About => about::render_about(f, app, chunks[2]),
    }

    render_footer(f, chunks[3], app, &app.theme);

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
            "Required: at least 60x16",
            Style::default().fg(theme.warn),
        )),
        Line::from(""),
        Line::from(Span::raw("Resize the terminal to continue.")),
    ];
    let widget = Paragraph::new(msg)
        .alignment(Alignment::Center)
        .block(panel_block(" Warning ", false, theme));
    f.render_widget(widget, area);
}
