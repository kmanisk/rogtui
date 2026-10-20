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
    panel_block, render_confirm_modal, render_footer, render_help_overlay,
    render_horizontal_rule, render_tab_bar,
};

pub fn render_ui<B: Backend>(f: &mut Frame, app: &AppState<B>) {
    let size = f.area();

    // Enforce minimum terminal size: under 80x24, render warning
    if size.width < 80 || size.height < 24 {
        render_terminal_too_small(f, size, &app.theme);
        return;
    }

    // Determine centered content column
    // Fixed max width 104 columns across all tabs so margins and layout geometry remain
    // perfectly consistent and never jump or expand when switching to the Fan tab.
    let max_w = 104;
    let content_w = if size.width >= max_w {
        max_w
    } else {
        size.width.saturating_sub(4)
    };
    let content_x = (size.width.saturating_sub(content_w)) / 2;

    let content_rect = Rect::new(content_x, 0, content_w, size.height);

    // Vertical rhythm:
    // Row 0: empty (1)
    // Row 1: tab bar (1)
    // Row 2: dim horizontal rule (1)
    // Row 3..N: tab content (Min 10)
    // Row N+1: dim horizontal rule (1)
    // Row N+2: single-line footer (1)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Row 0: empty
            Constraint::Length(1), // Row 1: tab bar
            Constraint::Length(1), // Row 2: dim rule
            Constraint::Min(10),   // Row 3..N: content
            Constraint::Length(1), // Row N+1: dim rule
            Constraint::Length(1), // Row N+2: single-line footer
        ])
        .split(content_rect);

    render_tab_bar(f, chunks[1], app.current_tab, &app.theme);
    render_horizontal_rule(f, chunks[2], &app.theme);

    match app.current_tab {
        ActiveTab::Dashboard => dashboard::render_dashboard(f, app, chunks[3]),
        ActiveTab::Power => power::render_power(f, app, chunks[3]),
        ActiveTab::FanCurves => fancurve::render_fancurve(f, app, chunks[3]),
        ActiveTab::Aura => aura::render_aura(f, app, chunks[3]),
        ActiveTab::System => system::render_system(f, app, chunks[3]),
    }

    render_horizontal_rule(f, chunks[4], &app.theme);
    render_footer(f, chunks[5], app, &app.theme);

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
