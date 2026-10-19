use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph, Wrap},
    Frame,
};

use crate::app::AppState;
use crate::backend::Backend;
use crate::backend::types::{RgbColor, SoftwareEffect};
use crate::ui::widgets::panel_block;

pub fn get_color_presets() -> [(&'static str, RgbColor); 10] {
    let (_, theme_col) = crate::ui::theme::Theme::get_system_theme_color();
    [
        ("Dynamic Theme", theme_col),
        ("Gruvbox", RgbColor::new(235, 219, 178)),
        ("Catppuccin", RgbColor::new(245, 194, 231)),
        ("Cyberpunk", RgbColor::new(0, 255, 170)),
        ("Nord", RgbColor::new(136, 192, 208)),
        ("Sunset Amber", RgbColor::new(255, 122, 0)),
        ("Red", RgbColor::new(255, 0, 0)),
        ("Green", RgbColor::new(0, 255, 0)),
        ("Blue", RgbColor::new(0, 136, 255)),
        ("White", RgbColor::new(255, 255, 255)),
    ]
}

pub fn get_active_preset_name(color: RgbColor) -> &'static str {
    let (_, theme_col) = crate::ui::theme::Theme::get_system_theme_color();
    if color == theme_col {
        "Dynamic Theme"
    } else if color == RgbColor::new(235, 219, 178) {
        "Gruvbox"
    } else if color == RgbColor::new(245, 194, 231) {
        "Catppuccin"
    } else if color == RgbColor::new(0, 255, 170) {
        "Cyberpunk"
    } else if color == RgbColor::new(136, 192, 208) {
        "Nord"
    } else if color == RgbColor::new(255, 122, 0) {
        "Sunset Amber"
    } else if color == RgbColor::new(255, 0, 0) {
        "Red"
    } else if color == RgbColor::new(0, 255, 0) {
        "Green"
    } else if color == RgbColor::new(0, 136, 255) {
        "Blue"
    } else if color == RgbColor::new(255, 255, 255) {
        "White"
    } else {
        "Custom"
    }
}

pub fn get_current_mode_index<B: Backend>(app: &AppState<B>) -> usize {
    match app.active_software_effect {
        SoftwareEffect::RainbowWave => 4,
        SoftwareEffect::ColorCycle => 5,
        SoftwareEffect::FireFlicker => 6,
        SoftwareEffect::Strobe => 7,
        SoftwareEffect::VoicePulse => 8,
        SoftwareEffect::AudioPulse => 9,
        SoftwareEffect::AudioWave => 10,
        SoftwareEffect::AmbientLight => 11,
        SoftwareEffect::Heatmap => 12,
        SoftwareEffect::GpuMode => 13,
        SoftwareEffect::BatteryStatus => 14,
        SoftwareEffect::CpuLoad => 15,
        SoftwareEffect::None => match app.aura_mode {
            1 => 1,
            2 => 2,
            10 => 3,
            _ => 0,
        },
    }
}

pub fn render_aura<B: Backend>(f: &mut Frame, app: &AppState<B>, area: Rect) {
    let theme = &app.theme;
    let is_off = app.aura_brightness == 0;
    let is_narrow = area.width < 100;

    let chunks = if is_narrow {
        vec![area]
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area)
            .to_vec()
    };

    // -------------------------------------------------------------------------
    // Left Pane: Structured Controls (Effect & Brightness, Colours, Bottom Power)
    // -------------------------------------------------------------------------
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // Effect & Brightness
            Constraint::Length(6), // Colours & Calibration
            Constraint::Min(0),    // Flexible spacer pushing power states to bottom
            Constraint::Length(3), // Hardware Power States (bottom bar)
        ])
        .split(chunks[0]);

    // 1. Effect & Brightness Panel
    let eff_block = panel_block(" Effect & Brightness ", app.aura_selected_field <= 1 || (7..=8).contains(&app.aura_selected_field), theme);
    let eff_inner = eff_block.inner(left_chunks[0]);
    f.render_widget(eff_block, left_chunks[0]);

    let brightness_str = match app.aura_brightness {
        0 => "Off",
        1 => "Low",
        2 => "Medium",
        3 => "High",
        _ => "Custom",
    };
    let b_bars = match app.aura_brightness {
        0 => "░░░",
        1 => "■░░",
        2 => "■■░",
        _ => "■■■",
    };

    let mode_idx = get_current_mode_index(app);
    let mode_names = [
        "Static",
        "Breathe",
        "Rainbow Cycle",
        "Pulse",
        "Rainbow Wave (Custom)",
        "Color Cycle (Custom)",
        "Fire Flicker (Custom)",
        "Strobe / Flash (Custom)",
        "Voice Pulse (Clean Mic)",
        "Audio Pulse (Speaker Out)",
        "Audio Wave (Spectrum EQ)",
        "Ambient Light (Wayland)",
        "Heatmap (Telemetry)",
        "GPU Mode (Telemetry)",
        "Battery Status (Telemetry)",
        "CPU Load (Telemetry)",
    ];
    let mode_str = mode_names.get(mode_idx).unwrap_or(&"Custom");

    let eff_fields = [
        (0, "Backlight Brightness", format!("< {brightness_str} >  [{b_bars}]")),
        (1, "Aura Mode", if is_off { "< Disabled (Backlight Off) >".to_string() } else { format!("< {mode_str} >") }),
        (7, "Animation Speed", if is_off || mode_idx == 0 || mode_idx == 3 { "< Disabled >".to_string() } else { format!("< {} >", app.aura_data.speed) }),
        (8, "Animation Direction", if is_off || mode_idx == 0 || mode_idx == 1 || mode_idx == 3 { "< Disabled >".to_string() } else { format!("< {} >", app.aura_data.direction) }),
    ];

    let mut eff_lines = Vec::new();
    for (f_idx, label, val) in &eff_fields {
        let is_sel = app.aura_selected_field == *f_idx;
        let is_disabled = is_off && *f_idx != 0;

        let row_str = format!("  {label: <24}: {val}");
        let w = eff_inner.width as usize;
        let padded = if row_str.len() < w { format!("{}{}", row_str, " ".repeat(w - row_str.len())) } else { row_str };

        if is_sel && !is_disabled {
            eff_lines.push(Line::from(Span::styled(padded, theme.sel_style())));
        } else if is_disabled {
            eff_lines.push(Line::from(Span::styled(padded, Style::default().fg(theme.dim))));
        } else {
            eff_lines.push(Line::from(vec![
                Span::styled(format!("  {label: <24}: "), Style::default().fg(theme.dim)),
                Span::styled(val.clone(), Style::default().fg(theme.fg)),
            ]));
        }
    }
    f.render_widget(Paragraph::new(eff_lines), eff_inner);

    // 2. Colours Panel
    let col_block = panel_block(" Colours & Calibration ", (2..=6).contains(&app.aura_selected_field), theme);
    let col_inner = col_block.inner(left_chunks[1]);
    f.render_widget(col_block, left_chunks[1]);

    let c1 = app.aura_data.color1;
    let c2 = app.aura_data.color2;
    let c1_swatch = if is_off { Style::default().fg(theme.dim) } else { Style::default().fg(Color::Rgb(c1.r, c1.g, c1.b)) };
    let c2_swatch = if is_off { Style::default().fg(theme.dim) } else { Style::default().fg(Color::Rgb(c2.r, c2.g, c2.b)) };

    fn mini_bar(val: u8) -> String {
        let filled = (val as usize * 8) / 255;
        let empty = 8 - filled;
        format!("{}{}", "■".repeat(filled), "░".repeat(empty))
    }

    let preset_name = get_active_preset_name(c1);
    let mut col_lines = Vec::new();

    // Row 0: Color Preset Selector (Field 2)
    let p_sel = app.aura_selected_field == 2;
    let p_header = format!("  Color Preset [h/l]      : < {preset_name} >");
    col_lines.push(Line::from(vec![
        Span::styled(p_header, if p_sel { theme.sel_style() } else { Style::default().fg(theme.dim) }),
    ]));

    // Row 1: Primary Color Hex + Swatch
    let c1_hex = c1.to_hex();
    col_lines.push(Line::from(vec![
        Span::styled(format!("  Primary Color [c: picker]: {c1_hex}  "), Style::default().fg(theme.dim)),
        Span::styled("██████", c1_swatch),
    ]));

    // Row 2: RGB Sliders (Fields 3, 4, 5)
    let r_sel = app.aura_selected_field == 3;
    let g_sel = app.aura_selected_field == 4;
    let b_sel = app.aura_selected_field == 5;
    col_lines.push(Line::from(vec![
        Span::styled("    R: ", Style::default().fg(theme.dim)),
        Span::styled(format!("{} {:>3}", mini_bar(c1.r), c1.r), if r_sel { theme.sel_style() } else { Style::default().fg(if is_off { theme.dim } else { theme.err }) }),
        Span::styled("   G: ", Style::default().fg(theme.dim)),
        Span::styled(format!("{} {:>3}", mini_bar(c1.g), c1.g), if g_sel { theme.sel_style() } else { Style::default().fg(if is_off { theme.dim } else { theme.ok }) }),
        Span::styled("   B: ", Style::default().fg(theme.dim)),
        Span::styled(format!("{} {:>3}", mini_bar(c1.b), c1.b), if b_sel { theme.sel_style() } else { Style::default().fg(if is_off { theme.dim } else { Color::Blue }) }),
    ]));

    // Row 3: Secondary Color (Field 6)
    let c2_sel = app.aura_selected_field == 6;
    let c2_hex = c2.to_hex();
    col_lines.push(Line::from(vec![
        Span::styled(format!("  Secondary Color [c: picker]: {c2_hex}  "), if c2_sel { theme.sel_style() } else { Style::default().fg(theme.dim) }),
        Span::styled("██████", c2_swatch),
    ]));

    f.render_widget(Paragraph::new(col_lines), col_inner);

    // 3. Hardware Power States Panel (Pushed to bottom bar)
    let pwr_block = panel_block(" Hardware Power States (ACPI) ", (9..=12).contains(&app.aura_selected_field), theme);
    let pwr_inner = pwr_block.inner(left_chunks[3]);
    f.render_widget(pwr_block, left_chunks[3]);

    let states = [
        (9, "Boot", app.aura_power.boot),
        (10, "Awake", app.aura_power.awake),
        (11, "Sleep", app.aura_power.sleep),
        (12, "Shutdown", app.aura_power.shutdown),
    ];

    let mut state_spans = vec![Span::raw("  ")];
    for (idx, name, is_on) in &states {
        let is_sel = app.aura_selected_field == *idx;
        let tag = if *is_on { "[x]" } else { "[ ]" };
        let text = format!("{tag} {name}     ");
        if is_sel {
            state_spans.push(Span::styled(text, theme.sel_style()));
        } else {
            state_spans.push(Span::styled(
                text,
                if *is_on { Style::default().fg(theme.ok) } else { Style::default().fg(theme.dim) },
            ));
        }
    }

    f.render_widget(Paragraph::new(Line::from(state_spans)), pwr_inner);

    // -------------------------------------------------------------------------
    // Right Pane: Aura Details & Zone Info
    // -------------------------------------------------------------------------
    if !is_narrow {
        let details_block = panel_block(" Aura Lighting Details ", false, theme);
        let details_inner = details_block.inner(chunks[1]);
        f.render_widget(details_block, chunks[1]);

        let (active_title, active_desc, effect_category) = match mode_idx {
            0 => ("Static", "Displays a continuous solid single-zone color. Applies Primary Color.", "Hardware Firmware"),
            1 => ("Breathe", "Smooth sinusoidal pulsing transition between Primary Color and Secondary Color.", "Hardware Firmware"),
            2 => ("Rainbow Cycle", "Hardware spectral progression through the full 360° RGB color wheel.", "Hardware Firmware"),
            3 => ("Pulse", "Rhythmic strobe pulses driven directly by the keyboard controller.", "Hardware Firmware"),
            4 => ("Rainbow Wave", "Smooth continuous spectrum sweep across the full 360° color wheel. Rotates hue continuously based on Animation Speed and Direction.", "Custom Animation"),
            5 => ("Color Cycle", "Sequentially transitions through 7 distinct primary colors. Steps at interval defined by Animation Speed and Direction.", "Custom Animation"),
            6 => ("Fire Flicker", "Simulates warm fireplace embers with shifting amber, orange, and red tones. Generates organic color variance on each tick.", "Custom Animation"),
            7 => ("Strobe / Flash", "High-contrast blinking pulse alternating between Primary Color and black. Creates a rhythmic flash effect without saving to BIOS flash.", "Custom Animation"),
            8 => ("Voice Pulse (Mic)", "Microphone voice-reactive illumination. G-Helper style speech sync using the clean microphone (ghelper-audio) - lights up when speaking, rests when silent.", "Audio & Visual Sync"),
            9 => ("Audio Pulse", "Sound output reactive visualizer. Pulses keyboard illumination to music beat drops and audio playback volume across speakers and headphones.", "Audio & Visual Sync"),
            10 => ("Audio Wave (EQ)", "Dynamic audio spectrum equalizer. Shifts keyboard color hue and intensity across the frequency and amplitude spectrum from deep blue to peak red.", "Audio & Visual Sync"),
            11 => ("Ambient Light (Wayland)", "Real-time screen ambient sync on Wayland. Samples the display compositor via grim to continuously mirror movie, video, or game visuals on the keyboard.", "Audio & Visual Sync"),
            12 => ("Heatmap (Temp)", "Maps highest component temperature to smooth RGB gradient: Cool (<30°C) -> Blue | Warm (60°C) -> Yellow | Hot (>=90°C) -> Red.", "System Telemetry"),
            13 => ("GPU Mode Indicator", "Color reflects active supergfx GPU mode: Integrated -> Green | Hybrid -> Yellow | Direct MUX -> Orange-Red.", "System Telemetry"),
            14 => ("Battery Status", "Color reflects battery charge level and state: Charging -> Bright Green | Low (<20%) -> Red | Balanced -> Teal.", "System Telemetry"),
            15 => ("CPU Load Indicator", "Keyboard color dynamically reflects real-time CPU utilization: Idle (<30%) -> Green | Moderate (30-65%) -> Yellow | Peak (>85%) -> Red.", "System Telemetry"),
            _ => ("Custom", "Aura lighting effect.", "Hardware Firmware"),
        };

        let mut right_lines = vec![
            Line::from(vec![
                Span::styled("  Effect: ", Style::default().fg(theme.dim)),
                Span::styled(active_title, Style::default().fg(theme.warn).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled("  Class : ", Style::default().fg(theme.dim)),
                Span::styled(effect_category, Style::default().fg(theme.accent)),
                Span::styled("   State: ", Style::default().fg(theme.dim)),
                Span::styled(
                    if is_off { "Backlight Off" } else { "Active" },
                    if is_off { Style::default().fg(theme.warn) } else { Style::default().fg(theme.ok).add_modifier(Modifier::BOLD) },
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(format!("  {active_desc}"), Style::default().fg(theme.fg))),
        ];

        // Context-aware effect telemetry readouts
        if app.active_software_effect == SoftwareEffect::VoicePulse {
            let level = app.mic_level.min(100);
            let filled = (level as usize * 12) / 100;
            let empty = 12 - filled;
            right_lines.push(Line::from(""));
            right_lines.push(Line::from(vec![
                Span::styled("  Clean Mic (ghelper-audio): ", Style::default().fg(theme.dim)),
                Span::styled("■".repeat(filled), Style::default().fg(if level > 25 { theme.ok } else { theme.warn })),
                Span::styled("■".repeat(empty), Style::default().fg(Color::Rgb(55, 60, 68))),
                Span::styled(format!(" {:>3}%", level), Style::default().fg(theme.fg)),
                Span::styled(if level > 25 { "  [SPEECH]" } else { "  [IDLE]" }, Style::default().fg(if level > 25 { theme.warn } else { theme.dim })),
            ]));
        } else if matches!(app.active_software_effect, SoftwareEffect::AudioPulse | SoftwareEffect::AudioWave) {
            let level = app.audio_level.min(100);
            let filled = (level as usize * 12) / 100;
            let empty = 12 - filled;
            right_lines.push(Line::from(""));
            right_lines.push(Line::from(vec![
                Span::styled("  Audio Output VU: ", Style::default().fg(theme.dim)),
                Span::styled("■".repeat(filled), Style::default().fg(if level > 10 { theme.accent } else { theme.warn })),
                Span::styled("■".repeat(empty), Style::default().fg(Color::Rgb(55, 60, 68))),
                Span::styled(format!(" {:>3}%", level), Style::default().fg(theme.fg)),
            ]));
        } else if app.active_software_effect == SoftwareEffect::AmbientLight {
            right_lines.push(Line::from(""));
            let sample_desc = if let Some(c) = app.last_ambient_color {
                format!("  Sampled Wayland Center: {} (via grim)", c.to_hex())
            } else {
                "  Sampling Wayland screen via grim...".to_string()
            };
            right_lines.push(Line::from(Span::styled(sample_desc, Style::default().fg(theme.dim))));
        } else if app.active_software_effect == SoftwareEffect::Heatmap {
            let cpu = app.sensors.cpu_temp_c.unwrap_or(0);
            let gpu = app.sensors.gpu_temp_c.unwrap_or(0);
            let peak = cpu.max(gpu);
            right_lines.push(Line::from(""));
            right_lines.push(Line::from(vec![
                Span::styled(format!("  Sensors: CPU {}°C | GPU {}°C -> Peak {}°C", cpu, gpu, peak), Style::default().fg(theme.accent)),
            ]));
        } else if app.active_software_effect == SoftwareEffect::GpuMode {
            right_lines.push(Line::from(""));
            right_lines.push(Line::from(vec![
                Span::styled(format!("  supergfxctl: {}", app.gpu_mode.as_str()), Style::default().fg(theme.accent)),
            ]));
        } else if app.active_software_effect == SoftwareEffect::BatteryStatus {
            let pct = app.sensors.battery_charge_percent.unwrap_or(0);
            let status = &app.sensors.battery_status;
            right_lines.push(Line::from(""));
            right_lines.push(Line::from(vec![
                Span::styled(format!("  Battery: {}% ({})", pct, status), Style::default().fg(theme.accent)),
            ]));
        } else if app.active_software_effect == SoftwareEffect::CpuLoad {
            let load = app.sensors.cpu_usage_percent.unwrap_or(0);
            right_lines.push(Line::from(""));
            right_lines.push(Line::from(vec![
                Span::styled(format!("  CPU Utilization: {}%", load), Style::default().fg(theme.accent)),
            ]));
        }

        // Live Color Swatch
        if let Some(col) = app.last_effect_color {
            right_lines.push(Line::from(""));
            right_lines.push(Line::from(vec![
                Span::styled("  Live Color: ", Style::default().fg(theme.dim)),
                Span::styled(format!("{} ", col.to_hex()), Style::default().fg(theme.fg).add_modifier(Modifier::BOLD)),
                Span::styled("████████", Style::default().fg(Color::Rgb(col.r, col.g, col.b))),
            ]));
        } else if app.active_software_effect == SoftwareEffect::None {
            right_lines.push(Line::from(""));
            right_lines.push(Line::from(vec![
                Span::styled("  Primary  : ", Style::default().fg(theme.dim)),
                Span::styled(format!("{} ", app.aura_data.color1.to_hex()), Style::default().fg(theme.fg).add_modifier(Modifier::BOLD)),
                Span::styled("████████", Style::default().fg(Color::Rgb(app.aura_data.color1.r, app.aura_data.color1.g, app.aura_data.color1.b))),
            ]));
            if app.aura_mode == 1 {
                right_lines.push(Line::from(vec![
                    Span::styled("  Secondary: ", Style::default().fg(theme.dim)),
                    Span::styled(format!("{} ", app.aura_data.color2.to_hex()), Style::default().fg(theme.fg).add_modifier(Modifier::BOLD)),
                    Span::styled("████████", Style::default().fg(Color::Rgb(app.aura_data.color2.r, app.aura_data.color2.g, app.aura_data.color2.b))),
                ]));
            }
        }

        f.render_widget(Paragraph::new(right_lines).wrap(Wrap { trim: true }), details_inner);
    }

    if app.show_color_picker_modal {
        render_color_picker_modal(f, app, area);
    }
}

fn render_color_picker_modal<B: Backend>(f: &mut Frame, app: &AppState<B>, area: Rect) {
    let theme = &app.theme;
    let modal_w = 70.min(area.width.saturating_sub(4));
    let modal_h = 13.min(area.height.saturating_sub(2));
    let x = (area.width.saturating_sub(modal_w)) / 2;
    let y = (area.height.saturating_sub(modal_h)) / 2;
    let modal_area = Rect::new(x, y, modal_w, modal_h);

    f.render_widget(Clear, modal_area);
    let title = format!(" 24-bit RGB / HSV Color Picker (Slot {}) ", app.picker_target_slot);
    let block = panel_block(&title, true, theme);
    let inner = block.inner(modal_area);
    f.render_widget(block, modal_area);

    let cur_col = app.picker_current_color();
    let cur_hex = cur_col.to_hex();
    let swatch_style = Style::default().fg(Color::Rgb(cur_col.r, cur_col.g, cur_col.b));

    let mut lines = Vec::new();

    // 1. Target Slot & Hex readout & Live swatch
    lines.push(Line::from(vec![
        Span::styled(format!("  Target: Slot {}   ", app.picker_target_slot), Style::default().fg(theme.dim)),
        Span::styled(format!("Hex: {cur_hex}   "), Style::default().fg(theme.fg).add_modifier(Modifier::BOLD)),
        Span::styled("Swatch: ", Style::default().fg(theme.dim)),
        Span::styled("████████████", swatch_style),
    ]));

    // 2. 24-bit half-block gradient strip (top half = pure hue, bottom half = shaded by S & V)
    let strip_len = (inner.width as usize).saturating_sub(6).clamp(20, 56);
    let mut strip_spans = vec![Span::styled("  [", Style::default().fg(theme.dim))];
    for i in 0..strip_len {
        let h = (i as f32 / strip_len as f32) * 360.0;
        let c_top = RgbColor::from_hsv(h, 100.0, 100.0);
        let c_bot = RgbColor::from_hsv(h, app.picker_sat, app.picker_val);
        strip_spans.push(Span::styled(
            "▀",
            Style::default()
                .fg(Color::Rgb(c_top.r, c_top.g, c_top.b))
                .bg(Color::Rgb(c_bot.r, c_bot.g, c_bot.b)),
        ));
    }
    strip_spans.push(Span::styled("]", Style::default().fg(theme.dim)));
    lines.push(Line::from(strip_spans));

    // 3. Indicator marker
    let marker_pos = ((app.picker_hue / 360.0) * (strip_len as f32)).round() as usize;
    let marker_pos = marker_pos.min(strip_len.saturating_sub(1));
    let mut marker_line = String::from("   ");
    marker_line.push_str(&" ".repeat(marker_pos));
    marker_line.push('▲');
    lines.push(Line::from(Span::styled(
        marker_line,
        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
    )));

    // 4. Sliders (HSV and RGB)
    let slider_width = 24;
    let make_slider_bar = |val: f32, max: f32| -> String {
        let frac = (val / max).clamp(0.0, 1.0);
        let filled = (frac * slider_width as f32).round() as usize;
        let filled = filled.min(slider_width);
        let empty = slider_width.saturating_sub(filled);
        format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
    };

    let sliders: [(usize, &str, String, String); 6] = [
        (0, "Hue (H)        ", make_slider_bar(app.picker_hue, 360.0), format!("{:>3}°", app.picker_hue.round() as u32)),
        (1, "Saturation (S) ", make_slider_bar(app.picker_sat, 100.0), format!("{:>3}%", app.picker_sat.round() as u32)),
        (2, "Value / Luma(V)", make_slider_bar(app.picker_val, 100.0), format!("{:>3}%", app.picker_val.round() as u32)),
        (3, "Red (R)        ", make_slider_bar(cur_col.r as f32, 255.0), format!("{:>3}", cur_col.r)),
        (4, "Green (G)      ", make_slider_bar(cur_col.g as f32, 255.0), format!("{:>3}", cur_col.g)),
        (5, "Blue (B)       ", make_slider_bar(cur_col.b as f32, 255.0), format!("{:>3}", cur_col.b)),
    ];

    for (s_idx, label, bar, val_str) in sliders {
        let is_sel = app.picker_slider_idx == s_idx;
        let row_content = format!("  {label}: {bar} {val_str}");
        if is_sel {
            lines.push(Line::from(Span::styled(row_content, theme.sel_style())));
        } else {
            let col_tint = match s_idx {
                3 => theme.err,
                4 => theme.ok,
                5 => Color::Blue,
                _ => theme.fg,
            };
            lines.push(Line::from(vec![
                Span::styled(format!("  {label}: "), Style::default().fg(theme.dim)),
                Span::styled(bar, Style::default().fg(col_tint)),
                Span::styled(format!(" {val_str}"), Style::default().fg(theme.fg)),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  j/k: select slider  h/l: adjust  ", Style::default().fg(theme.dim)),
        Span::styled("Enter: apply  ", Style::default().fg(theme.ok).add_modifier(Modifier::BOLD)),
        Span::styled("Esc: cancel", Style::default().fg(theme.dim)),
    ]));

    f.render_widget(Paragraph::new(lines), inner);
}
