use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::AppState;
use crate::backend::Backend;
use crate::backend::types::{GpuMode, PendingUserAction};
use crate::ui::widgets::panel_block;

pub fn render_gpu<B: Backend>(f: &mut Frame, app: &AppState<B>, area: Rect) {
    let theme = &app.theme;
    let is_narrow = area.width < 100;

    let chunks = if is_narrow {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(10), Constraint::Min(10)])
            .split(area)
            .to_vec()
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(area)
            .to_vec()
    };

    // -------------------------------------------------------------------------
    // 1. GPU Status Panel (Left or Top)
    // -------------------------------------------------------------------------
    let status_block = panel_block(" GPU Architecture & Daemon Status ", false, theme);
    let status_inner = status_block.inner(chunks[0]);
    f.render_widget(status_block, chunks[0]);

    let pending_mode_str = if app.gpu_pending_mode == GpuMode::None || app.gpu_pending_mode == app.gpu_mode {
        "None".to_string()
    } else {
        app.gpu_pending_mode.as_str().to_string()
    };

    let pending_action_str = app.gpu_pending_action.as_str();
    let has_pending_action = app.gpu_pending_action != PendingUserAction::Nothing;

    let mut status_lines = vec![
        Line::from(vec![
            Span::styled("  Active GPU Mode  : ", Style::default().fg(theme.dim)),
            Span::styled(
                app.gpu_mode.as_str(),
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  dGPU Power State : ", Style::default().fg(theme.dim)),
            Span::styled(
                app.gpu_power.as_str(),
                Style::default().fg(theme.ok).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Pending Mode     : ", Style::default().fg(theme.dim)),
            Span::styled(
                pending_mode_str,
                if has_pending_action {
                    Style::default().fg(theme.warn).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.dim)
                },
            ),
        ]),
        Line::from(vec![
            Span::styled("  Required Action  : ", Style::default().fg(theme.dim)),
            Span::styled(
                pending_action_str,
                if has_pending_action {
                    Style::default().fg(theme.err).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.dim)
                },
            ),
        ]),
        Line::from(vec![
            Span::styled("  NV Powerd on BAT : ", Style::default().fg(theme.dim)),
            Span::styled(
                if app.disable_powerd_battery { "[x] Disabled (Powersave)" } else { "[ ] Enabled (Full Boost)" },
                if app.disable_powerd_battery { Style::default().fg(theme.ok) } else { Style::default().fg(theme.dim) },
            ),
        ]),
    ];

    if app.gpu_supported.len() <= 1 {
        status_lines.push(Line::from(""));
        status_lines.push(Line::from(Span::styled(
            "  * supergfxd reports 1 supported mode in direct MUX (Ultimate);",
            Style::default().fg(theme.dim),
        )));
        status_lines.push(Line::from(Span::styled(
            "    switch MUX to Optimus/Hybrid via Armoury (tab 6) or BIOS to unlock all runtime modes.",
            Style::default().fg(theme.dim),
        )));
    }

    f.render_widget(Paragraph::new(status_lines), status_inner);

    // -------------------------------------------------------------------------
    // 2. Mode Selection Panel (Right or Bottom)
    // -------------------------------------------------------------------------
    let mode_block = panel_block(" Hardware GPU Mode Selection ", true, theme);
    let mode_inner = mode_block.inner(chunks[1]);
    f.render_widget(mode_block, chunks[1]);

    let mut mode_lines = vec![
        Line::from(Span::styled(
            "  Select target mode and press Enter to switch:",
            Style::default().fg(theme.dim),
        )),
        Line::from(""),
    ];

    for (idx, mode) in app.gpu_supported.iter().enumerate() {
        let is_selected = app.gpu_selected_idx == idx;
        let is_current = *mode == app.gpu_mode;

        let marker = if is_current { "[x]" } else { "[ ]" };
        let active_tag = if is_current { "  (ACTIVE)" } else { "" };
        let row_text = format!("  {marker} {}{active_tag}", mode.as_str());

        let w = mode_inner.width as usize;
        let padded = if row_text.len() < w {
            format!("{}{}", row_text, " ".repeat(w - row_text.len()))
        } else {
            row_text
        };

        if is_selected {
            mode_lines.push(Line::from(Span::styled(padded, theme.sel_style())));
        } else if is_current {
            mode_lines.push(Line::from(vec![
                Span::styled(format!("  {marker} "), Style::default().fg(theme.ok)),
                Span::styled(mode.as_str(), Style::default().fg(theme.fg).add_modifier(Modifier::BOLD)),
                Span::styled(active_tag, Style::default().fg(theme.ok)),
            ]));
        } else {
            mode_lines.push(Line::from(vec![
                Span::styled(format!("  {marker} "), Style::default().fg(theme.dim)),
                Span::styled(mode.as_str(), Style::default().fg(theme.fg)),
            ]));
        }

        // Mode description
        let desc = match mode {
            GpuMode::AsusMuxDgpu => {
                "      Direct dGPU MUX: RTX 5050 Mobile wired directly to internal panel.\n      Bypasses Intel iGPU for minimal latency and maximum frame throughput."
            }
            GpuMode::Hybrid => {
                "      Optimus Hybrid: Intel UHD Graphics powers desktop composition.\n      RTX 5050 powers on dynamically via prime-run, idle in D3cold."
            }
            GpuMode::Integrated => {
                "      Integrated: Discrete GPU powered off via ACPI for maximum battery life."
            }
            GpuMode::Vfio => {
                "      VFIO: Binds dGPU to vfio-pci for direct PCI passthrough to virtual machines."
            }
            GpuMode::AsusEgpu => {
                "      ASUS eGPU: Directs graphics pipeline to external Thunderbolt GPU enclosure."
            }
            _ => "      Custom GPU routing profile.",
        };
        for line in desc.lines() {
            mode_lines.push(Line::from(Span::styled(line, Style::default().fg(theme.dim))));
        }
        mode_lines.push(Line::from(""));
    }

    f.render_widget(Paragraph::new(mode_lines), mode_inner);
}
