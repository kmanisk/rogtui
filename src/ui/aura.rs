use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
    Frame,
};

use crate::app::AppState;
use crate::backend::Backend;
use crate::backend::types::{RgbColor, SoftwareEffect};
use crate::ui::widgets::{
    choice_widget, label_value_row, panel_block, section_rule, segmented_control,
    slider_widget_tinted, toggle_widget,
};

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
        SoftwareEffect::Clouds => 7,
        SoftwareEffect::ThemeSync => 8,
        SoftwareEffect::Strobe => 9,
        SoftwareEffect::VoicePulse => 10,
        SoftwareEffect::AudioPulse => 11,
        SoftwareEffect::AudioWave => 12,
        SoftwareEffect::AmbientLight => 13,
        SoftwareEffect::Heatmap => 14,
        SoftwareEffect::GpuMode => 15,
        SoftwareEffect::BatteryStatus => 16,
        SoftwareEffect::CpuLoad => 17,
        SoftwareEffect::None => match app.aura_mode {
            1 => 1,
            2 => 2,
            10 => 3,
            _ => 0,
        },
    }
}

pub fn is_aura_field_supported<B: Backend>(app: &AppState<B>, field: usize) -> bool {
    let mode_idx = get_current_mode_index(app);
    let is_firmware_breathe = app.active_software_effect == SoftwareEffect::None && app.aura_mode == 1;
    let is_firmware_rainbow = app.active_software_effect == SoftwareEffect::None && app.aura_mode == 2;
    let is_animated = mode_idx == 4 || mode_idx == 5 || mode_idx == 6 || mode_idx == 7 || mode_idx == 9 || is_firmware_breathe || is_firmware_rainbow;
    let has_direction = mode_idx == 4 || is_firmware_rainbow;

    match field {
        0 | 1 | 9..=12 => true,
        2..=5 => !is_firmware_rainbow,
        6 => is_firmware_breathe,
        7 => is_animated,
        8 => has_direction,
        13 | 14 => app.has_slash_led,
        _ => false,
    }
}

pub fn render_aura<B: Backend>(f: &mut Frame, app: &AppState<B>, area: Rect) {
    let theme = &app.theme;
    let is_off = app.aura_brightness == 0;
    let width = area.width as usize;
    let label_width = 18;

    let mut lines = Vec::new();
    if area.height >= 22 {
        lines.push(Line::from(""));
    }

    // -------------------------------------------------------------------------
    // 1. Lighting Controls
    // -------------------------------------------------------------------------
    lines.push(section_rule("Keyboard Lighting", width, theme));
    if area.height >= 22 {
        lines.push(Line::from(""));
    }

    // Row 0: Brightness (Segmented control: Off · Low · Med · High)
    let brightness_opts = ["Off", "Low", "Med", "High"];
    let b_idx = app.aura_brightness.clamp(0, 3) as usize;
    let b_spans = segmented_control(&brightness_opts, b_idx, app.aura_selected_field == 0, theme);
    lines.push(label_value_row("Brightness", b_spans, app.aura_selected_field == 0, label_width, theme));

    // Row 1: Mode
    let mode_names = [
        "Static", "Breathe", "Rainbow", "Pulse",
        "Rainbow Wave", "Color Cycle", "Fire Flicker", "Clouds",
        "Theme Sync", "Strobe", "Voice Pulse", "Audio Pulse",
        "Audio Wave", "Ambient Screen Sync", "Heatmap", "GPU Mode",
        "Battery Status", "CPU Load",
    ];
    let mode_idx = get_current_mode_index(app);
    let mode_name = mode_names.get(mode_idx).unwrap_or(&"Static");
    let mode_spans = choice_widget(mode_name, app.aura_selected_field == 1 && !is_off, theme);
    lines.push(label_value_row("Mode", mode_spans, app.aura_selected_field == 1 && !is_off, label_width, theme));

    // Optional Speed & Direction if supported by active mode
    if is_aura_field_supported(app, 7) && !is_off {
        let speed_spans = choice_widget(&app.aura_data.speed, app.aura_selected_field == 7, theme);
        lines.push(label_value_row("Animation speed", speed_spans, app.aura_selected_field == 7, label_width, theme));
    }
    if is_aura_field_supported(app, 8) && !is_off {
        let dir_spans = choice_widget(&app.aura_data.direction, app.aura_selected_field == 8, theme);
        lines.push(label_value_row("Direction", dir_spans, app.aura_selected_field == 8, label_width, theme));
    }

    // Color controls if supported by active mode
    if is_aura_field_supported(app, 2) && !is_off {
        lines.push(Line::from(""));
        let col = app.aura_data.color1;
        let preset_name = get_active_preset_name(col);
        let swatch_color = Color::Rgb(col.r, col.g, col.b);

        // Row: Preset
        let preset_spans = choice_widget(preset_name, app.aura_selected_field == 2, theme);
        lines.push(label_value_row("Preset", preset_spans, app.aura_selected_field == 2, label_width, theme));

        // Row: Custom hex with 2-char block preview
        let hex_str = if let Some(buf) = &app.hex_input_buffer {
            format!("#{buf}_")
        } else {
            col.to_hex()
        };
        let hex_spans = vec![
            Span::styled(format!("{hex_str: <9}"), Style::default().fg(theme.fg).add_modifier(Modifier::BOLD)),
            Span::styled("  ██  ", Style::default().fg(swatch_color)),
            Span::styled("c", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled(" picker  ", Style::default().fg(theme.dim)),
            Span::styled("#", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled(" hex", Style::default().fg(theme.dim)),
        ];
        lines.push(label_value_row("Custom hex", hex_spans, false, label_width, theme));

        // R / G / B sliders tinted with respective colors
        let r_slider = slider_widget_tinted(
            col.r as u32,
            0,
            255,
            16,
            "",
            app.aura_selected_field == 3,
            Some(Color::Rgb(220, 80, 80)),
            theme,
        );
        lines.push(label_value_row("R", r_slider, app.aura_selected_field == 3, label_width, theme));

        let g_slider = slider_widget_tinted(
            col.g as u32,
            0,
            255,
            16,
            "",
            app.aura_selected_field == 4,
            Some(Color::Rgb(80, 200, 80)),
            theme,
        );
        lines.push(label_value_row("G", g_slider, app.aura_selected_field == 4, label_width, theme));

        let b_slider = slider_widget_tinted(
            col.b as u32,
            0,
            255,
            16,
            "",
            app.aura_selected_field == 5,
            Some(Color::Rgb(80, 130, 240)),
            theme,
        );
        lines.push(label_value_row("B", b_slider, app.aura_selected_field == 5, label_width, theme));
    }

    // Secondary Color if supported (e.g. Breathe)
    if is_aura_field_supported(app, 6) && !is_off {
        let col2 = app.aura_data.color2;
        let swatch_color2 = Color::Rgb(col2.r, col2.g, col2.b);
        let mut col2_spans = vec![
            Span::styled("██  ", Style::default().fg(swatch_color2)),
            Span::styled(col2.to_hex(), Style::default().fg(theme.fg).add_modifier(Modifier::BOLD)),
            Span::raw("   "),
        ];
        col2_spans.extend(choice_widget(get_active_preset_name(col2), app.aura_selected_field == 6, theme));
        lines.push(label_value_row("Secondary color", col2_spans, app.aura_selected_field == 6, label_width, theme));
    }

    // -------------------------------------------------------------------------
    // 2. Power States
    // -------------------------------------------------------------------------
    if area.height >= 22 {
        lines.push(Line::from(""));
    }
    lines.push(section_rule("Hardware Power States", width, theme));
    if area.height >= 22 {
        lines.push(Line::from(""));
    }

    let is_boot_sel = app.aura_selected_field == 9;
    let is_awake_sel = app.aura_selected_field == 10;
    let is_sleep_sel = app.aura_selected_field == 11;
    let is_shut_sel = app.aura_selected_field == 12;

    lines.push(label_value_row(
        "Boot",
        toggle_widget(app.aura_power.boot, is_boot_sel, theme),
        is_boot_sel,
        label_width,
        theme,
    ));

    lines.push(label_value_row(
        "Awake",
        toggle_widget(app.aura_power.awake, is_awake_sel, theme),
        is_awake_sel,
        label_width,
        theme,
    ));

    lines.push(label_value_row(
        "Sleep",
        toggle_widget(app.aura_power.sleep, is_sleep_sel, theme),
        is_sleep_sel,
        label_width,
        theme,
    ));

    lines.push(label_value_row(
        "Shutdown",
        toggle_widget(app.aura_power.shutdown, is_shut_sel, theme),
        is_shut_sel,
        label_width,
        theme,
    ));

    if app.has_slash_led {
        if area.height >= 22 {
            lines.push(Line::from(""));
        }
        lines.push(section_rule("Slash Lighting", width, theme));
        if area.height >= 22 {
            lines.push(Line::from(""));
        }

        let is_slash_mode_sel = app.aura_selected_field == 13;
        let is_slash_b_sel = app.aura_selected_field == 14;

        let mode_spans = choice_widget(&app.slash_mode, is_slash_mode_sel, theme);
        lines.push(label_value_row("Slash mode", mode_spans, is_slash_mode_sel, label_width, theme));

        let b_opts = ["Off", "Low", "Med", "High"];
        let b_idx = app.slash_brightness.clamp(0, 3) as usize;
        let b_spans = segmented_control(&b_opts, b_idx, is_slash_b_sel, theme);
        lines.push(label_value_row("Slash brightness", b_spans, is_slash_b_sel, label_width, theme));
    }

    f.render_widget(Paragraph::new(lines), area);

    if app.show_color_picker_modal {
        render_color_picker_modal(f, app, area);
    }
}

pub fn render_color_picker_modal<B: Backend>(f: &mut Frame, app: &AppState<B>, area: Rect) {
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

    // 2. 24-bit half-block gradient strip
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
    let slider_width = 20;
    let make_slider_bar = |val: f32, max: f32| -> String {
        let frac = (val / max).clamp(0.0, 1.0);
        let filled = (frac * slider_width as f32).round() as usize;
        let filled = filled.min(slider_width);
        let empty = slider_width.saturating_sub(filled);
        format!("{}{}", "━".repeat(filled), "─".repeat(empty))
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
        let prefix = if is_sel { "▌ " } else { "  " };
        let col_tint = if is_sel {
            theme.accent
        } else {
            match s_idx {
                3 => theme.err,
                4 => theme.ok,
                5 => Color::Blue,
                _ => theme.dim,
            }
        };
        lines.push(Line::from(vec![
            Span::styled(format!("{prefix}{label} "), if is_sel { Style::default().fg(theme.fg).add_modifier(Modifier::BOLD) } else { Style::default().fg(theme.dim) }),
            Span::styled(bar, Style::default().fg(col_tint)),
            Span::styled(format!("  {val_str}"), Style::default().fg(theme.fg)),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  j/k slider  h/l adjust  ", Style::default().fg(theme.dim)),
        Span::styled("Enter apply  ", Style::default().fg(theme.ok).add_modifier(Modifier::BOLD)),
        Span::styled("Esc cancel", Style::default().fg(theme.dim)),
    ]));

    f.render_widget(Paragraph::new(lines), inner);
}
