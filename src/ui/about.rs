use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::AppState;
use crate::backend::Backend;
use crate::ui::widgets::panel_block;

pub fn render_about<B: Backend>(f: &mut Frame, app: &AppState<B>, area: Rect) {
    let theme = &app.theme;
    let is_narrow = area.width < 100;

    let chunks = if is_narrow {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(12), Constraint::Min(10)])
            .split(area)
            .to_vec()
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area)
            .to_vec()
    };

    // -------------------------------------------------------------------------
    // Left Panel: System & Build Architecture
    // -------------------------------------------------------------------------
    let arch_block = panel_block(" System & Software Architecture ", false, theme);
    let arch_inner = arch_block.inner(chunks[0]);
    f.render_widget(arch_block, chunks[0]);

    let arch_lines = vec![
        Line::from(vec![
            Span::styled("  rogtui Version      : ", Style::default().fg(theme.dim)),
            Span::styled("v0.1.0", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled(" (opt-level=3, lto=fat, stripped)", Style::default().fg(theme.dim)),
        ]),
        Line::from(vec![
            Span::styled("  asusd Daemon        : ", Style::default().fg(theme.dim)),
            Span::styled("v6.5.0", Style::default().fg(theme.fg)),
            Span::styled(" (xyz.ljones.Asusd DBus)", Style::default().fg(theme.dim)),
        ]),
        Line::from(vec![
            Span::styled("  supergfxd Daemon    : ", Style::default().fg(theme.dim)),
            Span::styled("v5.2.7", Style::default().fg(theme.fg)),
            Span::styled(" (org.supergfxctl.Daemon DBus)", Style::default().fg(theme.dim)),
        ]),
        Line::from(vec![
            Span::styled("  Linux Kernel        : ", Style::default().fg(theme.dim)),
            Span::styled("7.2.9-2-cachyos", Style::default().fg(theme.fg)),
            Span::styled(" (BORE/EEVDF, x86-64-v3)", Style::default().fg(theme.dim)),
        ]),
        Line::from(vec![
            Span::styled("  Hardware Platform   : ", Style::default().fg(theme.dim)),
            Span::styled("ASUS TUF Gaming F16 (FX608JH)", Style::default().fg(theme.ok).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  BIOS Firmware       : ", Style::default().fg(theme.dim)),
            Span::styled("FX608JH.304", Style::default().fg(theme.fg)),
            Span::styled(" (ASUSTeK COMPUTER INC.)", Style::default().fg(theme.dim)),
        ]),
        Line::from(vec![
            Span::styled("  Display Compositor  : ", Style::default().fg(theme.dim)),
            Span::styled("i3wm / X11", Style::default().fg(theme.fg)),
            Span::styled(" (1920x1200 @ 165Hz)", Style::default().fg(theme.dim)),
        ]),
        Line::from(vec![
            Span::styled("  Idle CPU Target     : ", Style::default().fg(theme.dim)),
            Span::styled("~0.0%", Style::default().fg(theme.ok).add_modifier(Modifier::BOLD)),
            Span::styled(" (event-driven, 1 Hz tick on Dash/Fan)", Style::default().fg(theme.dim)),
        ]),
    ];

    f.render_widget(Paragraph::new(arch_lines), arch_inner);

    // -------------------------------------------------------------------------
    // Right Panel: Hardware Feature Parity & Diagnostics
    // -------------------------------------------------------------------------
    let feat_block = panel_block(" Hardware Capabilities & Diagnostics ", false, theme);
    let feat_inner = feat_block.inner(chunks[1]);
    f.render_widget(feat_block, chunks[1]);

    let feat_lines = vec![
        Line::from(vec![
            Span::styled("  ROGCC Parity Status : ", Style::default().fg(theme.dim)),
            Span::styled("Full Parity", Style::default().fg(theme.ok).add_modifier(Modifier::BOLD)),
            Span::styled(" (Replaces ROG Control Center)", Style::default().fg(theme.dim)),
        ]),
        Line::from(vec![
            Span::styled("  Aura RGB Architecture: ", Style::default().fg(theme.dim)),
            Span::styled("SingleZone Matrix", Style::default().fg(theme.fg)),
            Span::styled(" (Static, Breathe, Cycle, Pulse)", Style::default().fg(theme.dim)),
        ]),
        Line::from(vec![
            Span::styled("  Display MUX Switch  : ", Style::default().fg(theme.dim)),
            Span::styled("Hardware MUX", Style::default().fg(theme.accent)),
            Span::styled(" (AsusMuxDgpu / Optimus Hybrid)", Style::default().fg(theme.dim)),
        ]),
        Line::from(vec![
            Span::styled("  Discrete GPU Guard  : ", Style::default().fg(theme.dim)),
            Span::styled("Active", Style::default().fg(theme.ok)),
            Span::styled(" (Never wakes D3cold dGPU on telemetry)", Style::default().fg(theme.dim)),
        ]),
        Line::from(vec![
            Span::styled("  Fan Curve Calibration: ", Style::default().fg(theme.dim)),
            Span::styled("8-Point Monotonic", Style::default().fg(theme.fg)),
            Span::styled(" (CPU & GPU dual channels)", Style::default().fg(theme.dim)),
        ]),
        Line::from(vec![
            Span::styled("  Battery Management  : ", Style::default().fg(theme.dim)),
            Span::styled("EC Stepper (20-100%)", Style::default().fg(theme.fg)),
            Span::styled(" + One-Shot 100% Charge", Style::default().fg(theme.dim)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Diagnostic Report   : ", Style::default().fg(theme.dim)),
            Span::styled("Press 'y' to export report to ~/rogtui-diagnostics.txt", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  Hardware Probe Mode : ", Style::default().fg(theme.dim)),
            Span::styled("Run 'rogtui probe-modes' in shell to audit physical presets", Style::default().fg(theme.dim)),
        ]),
    ];

    f.render_widget(Paragraph::new(feat_lines), feat_inner);
}
