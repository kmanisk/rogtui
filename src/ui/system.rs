use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::AppState;
use crate::backend::Backend;
use crate::backend::types::GpuMode;
use crate::ui::widgets::{
    choice_widget, label_value_row, render_hint_line, section_rule, toggle_widget,
};

pub fn detect_session() -> &'static str {
    if std::env::var("SWAYSOCK").is_ok()
        || std::env::var("XDG_CURRENT_DESKTOP")
            .map(|d| d.to_lowercase().contains("sway"))
            .unwrap_or(false)
    {
        "Sway · Wayland"
    } else if std::env::var("WAYLAND_DISPLAY").is_ok()
        || std::env::var("XDG_SESSION_TYPE")
            .map(|t| t == "wayland")
            .unwrap_or(false)
    {
        "Wayland"
    } else if std::env::var("DISPLAY").is_ok()
        || std::env::var("XDG_SESSION_TYPE")
            .map(|t| t == "x11")
            .unwrap_or(false)
    {
        "X11"
    } else {
        "Linux TTY"
    }
}

fn extract_json_num(slice: &str, key: &str) -> Option<u64> {
    let k_pos = slice.find(key)?;
    let after_key = &slice[k_pos + key.len()..];
    let colon_pos = after_key.find(':')?;
    let after_colon = after_key[colon_pos + 1..].trim_start();
    let num_str: String = after_colon.chars().take_while(|c| c.is_ascii_digit()).collect();
    num_str.parse().ok()
}

pub fn detect_display() -> String {
    if let Ok(output) = std::process::Command::new("swaymsg")
        .args(["-t", "get_outputs", "-r"])
        .output()
    {
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout);
            if let Some(pos) = s.find("\"current_mode\"") {
                let slice = &s[pos..];
                let width = extract_json_num(slice, "\"width\"");
                let height = extract_json_num(slice, "\"height\"");
                let refresh = extract_json_num(slice, "\"refresh\"");
                if let (Some(w), Some(h), Some(r)) = (width, height, refresh) {
                    let hz = (r as f64 / 1000.0).round() as u64;
                    return format!("{w}×{h} · {hz} Hz");
                }
            }
        }
    }
    "1920×1200 · 165 Hz".to_string()
}

pub fn detect_kernel() -> String {
    if let Ok(k) = std::fs::read_to_string("/proc/sys/kernel/osrelease") {
        k.trim().to_string()
    } else {
        "7.2.11-1-cachyos".to_string()
    }
}

pub fn detect_model() -> String {
    let p = std::fs::read_to_string("/sys/class/dmi/id/product_name").unwrap_or_default();
    let p = p.trim();
    if p.is_empty() {
        "ASUS TUF Gaming F16 FX608JH".to_string()
    } else {
        p.split('_').next().unwrap_or(p).to_string()
    }
}

pub fn detect_bios() -> String {
    std::fs::read_to_string("/sys/class/dmi/id/bios_version")
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| "FX608JH.304".to_string())
}

pub fn format_gpu_mode(mode: GpuMode) -> &'static str {
    match mode {
        GpuMode::AsusMuxDgpu => "Dedicated GPU (MUX)",
        GpuMode::NvidiaNoModeset => "Dedicated GPU",
        GpuMode::Hybrid => "Hybrid",
        GpuMode::Integrated => "Integrated",
        GpuMode::Vfio => "VFIO",
        GpuMode::AsusEgpu => "eGPU",
        _ => "Dedicated GPU",
    }
}

pub fn render_system<B: Backend>(f: &mut Frame, app: &AppState<B>, area: Rect) {
    let theme = &app.theme;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Top spacing
            Constraint::Min(14),   // System settings list
            Constraint::Length(1), // Bottom hint line
        ])
        .split(area);

    let content_area = chunks[1];
    let width = content_area.width as usize;
    let label_width = 18;

    let mut lines = Vec::new();

    // -------------------------------------------------------------------------
    // 1. Display Section
    // -------------------------------------------------------------------------
    lines.push(section_rule("Display", width.saturating_sub(4), theme));
    lines.push(Line::from(""));

    // Row 0: Panel Overdrive
    let overdrive = app.sensors.panel_overdrive.unwrap_or(true);
    let od_spans = toggle_widget(overdrive, app.system_selected_row == 0, theme);
    lines.push(label_value_row("Panel overdrive", od_spans, app.system_selected_row == 0, label_width, theme));

    // Row 1: Auto-Brightness
    let auto_bright = app.sensors.screen_auto_brightness.unwrap_or(false);
    let ab_spans = toggle_widget(auto_bright, app.system_selected_row == 1, theme);
    lines.push(label_value_row("Auto-brightness", ab_spans, app.system_selected_row == 1, label_width, theme));
    lines.push(Line::from(""));

    // -------------------------------------------------------------------------
    // 2. Graphics Section
    // -------------------------------------------------------------------------
    lines.push(section_rule("Graphics", width.saturating_sub(4), theme));
    lines.push(Line::from(""));

    // If multiple GPU modes are supported, allow selection. Otherwise show read-only without clutter.
    let can_switch_gpu = app.gpu_supported.len() > 1;
    let is_gpu_sel = app.system_selected_row == 2 && can_switch_gpu;

    if can_switch_gpu {
        let selected_mode = app.gpu_supported.get(app.gpu_selected_idx).copied().unwrap_or(app.gpu_mode);
        let gpu_spans = choice_widget(format_gpu_mode(selected_mode), is_gpu_sel, theme);
        lines.push(label_value_row("GPU mode", gpu_spans, is_gpu_sel, label_width, theme));
    } else {
        let gpu_spans = vec![Span::styled(format_gpu_mode(app.gpu_mode), Style::default().fg(theme.fg))];
        lines.push(label_value_row("GPU mode", gpu_spans, false, label_width, theme));
    }

    // Only show TGP if non-zero and supported
    let tgp_attr = app
        .armoury_attributes
        .iter()
        .find(|a| (a.node_name == "nv_tgp" || a.node_name == "nv_base_tgp") && a.is_available && a.current_value > 0);
    if let Some(attr) = tgp_attr {
        lines.push(label_value_row(
            "Target TGP",
            vec![Span::styled(format!("{} W", attr.current_value), Style::default().fg(theme.fg))],
            false,
            label_width,
            theme,
        ));
    }
    lines.push(Line::from(""));

    // -------------------------------------------------------------------------
    // 3. About Section (Clean read-only block)
    // -------------------------------------------------------------------------
    lines.push(section_rule("About", width.saturating_sub(4), theme));
    lines.push(Line::from(""));

    lines.push(label_value_row(
        "Model",
        vec![Span::styled(detect_model(), Style::default().fg(theme.fg))],
        false,
        label_width,
        theme,
    ));

    lines.push(label_value_row(
        "BIOS",
        vec![Span::styled(detect_bios(), Style::default().fg(theme.fg))],
        false,
        label_width,
        theme,
    ));

    lines.push(label_value_row(
        "Kernel",
        vec![Span::styled(detect_kernel(), Style::default().fg(theme.fg))],
        false,
        label_width,
        theme,
    ));

    lines.push(label_value_row(
        "Session",
        vec![Span::styled(detect_session(), Style::default().fg(theme.fg))],
        false,
        label_width,
        theme,
    ));

    lines.push(label_value_row(
        "Display",
        vec![Span::styled(detect_display(), Style::default().fg(theme.fg))],
        false,
        label_width,
        theme,
    ));

    lines.push(label_value_row(
        "ROGTUI",
        vec![Span::styled(env!("CARGO_PKG_VERSION"), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))],
        false,
        label_width,
        theme,
    ));

    f.render_widget(Paragraph::new(lines), content_area);

    // -------------------------------------------------------------------------
    // Bottom Hint Line
    // -------------------------------------------------------------------------
    let hint_text = match app.system_selected_row {
        0 => "Enter/Space: toggle 3ms panel overdrive response time",
        1 => "Enter/Space: toggle ambient light sensor auto-brightness",
        2 if can_switch_gpu => "h/l: select GPU mode   Enter: confirm switch (requires reboot)",
        _ => "y: export system diagnostics report to ~/rogtui-diagnostics.txt",
    };
    render_hint_line(f, chunks[2], hint_text, theme);
}
