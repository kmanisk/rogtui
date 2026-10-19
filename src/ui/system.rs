use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::AppState;
use crate::backend::Backend;
use crate::backend::types::GpuMode;
use crate::ui::widgets::{label_value_row, section_rule, toggle_widget};

pub fn detect_session() -> &'static str {
    if std::env::var("SWAYSOCK").is_ok()
        || std::env::var("XDG_CURRENT_DESKTOP")
            .map(|d| d.to_lowercase().contains("sway"))
            .unwrap_or(false)
    {
        "Sway (Wayland)"
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

pub fn detect_cpu() -> String {
    if let Ok(cpuinfo) = std::fs::read_to_string("/proc/cpuinfo") {
        for line in cpuinfo.lines() {
            if line.starts_with("model name") {
                if let Some((_, val)) = line.split_once(':') {
                    let cleaned = val
                        .trim()
                        .replace("(R)", "")
                        .replace("(TM)", "")
                        .replace("  ", " ");
                    return cleaned;
                }
            }
        }
    }
    "13th Gen Intel Core i5-13450HX".to_string()
}

pub fn detect_gpu() -> String {
    let entries = std::fs::read_dir("/proc/driver/nvidia/gpus/").ok();
    if let Some(entries) = entries {
        for entry in entries.flatten() {
            let info_file = entry.path().join("information");
            if let Ok(content) = std::fs::read_to_string(info_file) {
                for line in content.lines() {
                    if line.starts_with("Model:") {
                        if let Some((_, model)) = line.split_once(':') {
                            return model.trim().replace("Laptop GPU", "Mobile");
                        }
                    }
                }
            }
        }
    }
    "NVIDIA GeForce RTX 5050 Mobile".to_string()
}

pub fn detect_ram() -> String {
    if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<u64>() {
                        let gb = (kb as f64) / (1024.0 * 1024.0);
                        return format!("{gb:.1} GiB (16 GB)");
                    }
                }
            }
        }
    }
    "16 GB".to_string()
}

pub fn detect_nvidia_driver() -> String {
    if let Ok(content) = std::fs::read_to_string("/proc/driver/nvidia/version") {
        for word in content.split_whitespace() {
            if word.contains('.') && word.chars().all(|c| c.is_ascii_digit() || c == '.') {
                return word.to_string();
            }
        }
    }
    "615.78.08".to_string()
}

pub fn detect_os() -> String {
    if let Ok(os_rel) = std::fs::read_to_string("/etc/os-release") {
        for line in os_rel.lines() {
            if line.starts_with("PRETTY_NAME=") {
                let name = line.trim_start_matches("PRETTY_NAME=").trim_matches('"');
                return format!("{name} Linux (x86_64)");
            }
        }
    }
    "CachyOS Linux (x86_64)".to_string()
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
    let width = area.width as usize;
    let label_width = 18;

    let mut lines = Vec::new();
    lines.push(Line::from(""));

    // -------------------------------------------------------------------------
    // 1. Display Section
    // -------------------------------------------------------------------------
    lines.push(section_rule("Display", width, theme));
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
    // 2. Graphics Section (Read-only display of current mode)
    // -------------------------------------------------------------------------
    lines.push(section_rule("Graphics", width, theme));
    lines.push(Line::from(""));

    let mode_str = format_gpu_mode(app.gpu_mode);
    let mut gpu_spans = vec![
        Span::styled(format!("‹ {mode_str} ›"), Style::default().fg(theme.fg)),
    ];
    if width >= 86 {
        gpu_spans.push(Span::styled("   Mode changes take effect on next login.", Style::default().fg(theme.dim)));
    }
    lines.push(label_value_row("GPU mode", gpu_spans, false, label_width, theme));
    lines.push(Line::from(""));

    // -------------------------------------------------------------------------
    // 3. Diagnostics Action
    // -------------------------------------------------------------------------
    lines.push(section_rule("Diagnostics", width, theme));
    lines.push(Line::from(""));

    let export_spans = vec![
        Span::styled(
            "[ Run ↵ ]",
            if app.system_selected_row == 2 {
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.dim)
            },
        ),
    ];
    lines.push(label_value_row("Export report", export_spans, app.system_selected_row == 2, label_width, theme));
    lines.push(Line::from(""));

    // -------------------------------------------------------------------------
    // 4. About Section (Clean read-only block)
    // -------------------------------------------------------------------------
    lines.push(section_rule("About", width, theme));
    lines.push(Line::from(""));

    lines.push(label_value_row(
        "Model",
        vec![Span::styled(detect_model(), Style::default().fg(theme.fg))],
        false,
        label_width,
        theme,
    ));

    lines.push(label_value_row(
        "Processor",
        vec![Span::styled(detect_cpu(), Style::default().fg(theme.fg))],
        false,
        label_width,
        theme,
    ));

    lines.push(label_value_row(
        "Graphics",
        vec![Span::styled(detect_gpu(), Style::default().fg(theme.fg))],
        false,
        label_width,
        theme,
    ));

    lines.push(label_value_row(
        "Memory",
        vec![Span::styled(detect_ram(), Style::default().fg(theme.fg))],
        false,
        label_width,
        theme,
    ));

    lines.push(label_value_row(
        "NVIDIA Driver",
        vec![Span::styled(detect_nvidia_driver(), Style::default().fg(theme.fg))],
        false,
        label_width,
        theme,
    ));

    lines.push(label_value_row(
        "OS",
        vec![Span::styled(detect_os(), Style::default().fg(theme.fg))],
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
        "BIOS",
        vec![Span::styled(detect_bios(), Style::default().fg(theme.fg))],
        false,
        label_width,
        theme,
    ));

    lines.push(label_value_row(
        "ROGTUI",
        vec![Span::styled(format!("v{}", env!("CARGO_PKG_VERSION")), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))],
        false,
        label_width,
        theme,
    ));

    f.render_widget(Paragraph::new(lines), area);
}
