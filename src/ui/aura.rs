use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::AppState;
use crate::backend::Backend;
use crate::backend::types::RgbColor;
use crate::ui::widgets::panel_block;

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
    // Left Pane: Structured Controls (Effect, Colours, Power States)
    // -------------------------------------------------------------------------
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7), // Effect & Brightness
            Constraint::Length(9), // Colours
            Constraint::Length(4), // Power States
            Constraint::Min(0),
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
        0 => "━ ━ ━",
        1 => "█ ━ ━",
        2 => "█ █ ━",
        _ => "█ █ █",
    };

    let mode_str = match app.aura_mode {
        0 => "Static",
        1 => "Breathe",
        2 => "Rainbow Cycle",
        10 => "Pulse",
        _ => "Custom",
    };

    let eff_fields = [
        (0, "Backlight Brightness", format!("< {brightness_str} >  [{b_bars}]")),
        (1, "Aura Effect Mode", if is_off { "< Disabled (Backlight Off) >".to_string() } else { format!("< {mode_str} >") }),
        (7, "Animation Speed", if is_off { "< Disabled >".to_string() } else { format!("< {} >", app.aura_data.speed) }),
        (8, "Animation Direction", if is_off { "< Disabled >".to_string() } else { format!("< {} >", app.aura_data.direction) }),
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
        format!("[{}{}] {:>3}", "█".repeat(filled), "━".repeat(empty), val)
    }

    let mut col_lines = Vec::new();

    // Primary Color Row
    let c1_sel = app.aura_selected_field == 2;
    let c1_hex = c1.to_hex();
    let c1_header = format!("  Primary Color (h/l preset): {}  ", c1_hex);
    col_lines.push(Line::from(vec![
        Span::styled(c1_header, if c1_sel { theme.sel_style() } else { Style::default().fg(theme.dim) }),
        Span::styled("██████", c1_swatch),
    ]));

    // RGB Mini-gauges Row
    let r_sel = app.aura_selected_field == 3;
    let g_sel = app.aura_selected_field == 4;
    let b_sel = app.aura_selected_field == 5;
    col_lines.push(Line::from(vec![
        Span::styled("    R: ", Style::default().fg(theme.dim)),
        Span::styled(mini_bar(c1.r), if r_sel { theme.sel_style() } else { Style::default().fg(if is_off { theme.dim } else { theme.err }) }),
        Span::styled("   G: ", Style::default().fg(theme.dim)),
        Span::styled(mini_bar(c1.g), if g_sel { theme.sel_style() } else { Style::default().fg(if is_off { theme.dim } else { theme.ok }) }),
        Span::styled("   B: ", Style::default().fg(theme.dim)),
        Span::styled(mini_bar(c1.b), if b_sel { theme.sel_style() } else { Style::default().fg(if is_off { theme.dim } else { Color::Blue }) }),
    ]));

    // Secondary Color Row (only for effects supporting 2 colors: breathe, pulse, custom)
    let c2_sel = app.aura_selected_field == 6;
    let c2_hex = c2.to_hex();
    let c2_header = format!("  Secondary Color           : {}  ", c2_hex);
    col_lines.push(Line::from(vec![
        Span::styled(c2_header, if c2_sel { theme.sel_style() } else { Style::default().fg(theme.dim) }),
        Span::styled("██████", c2_swatch),
    ]));

    // Palette Swatches Row
    let presets = [
        ("Red", RgbColor::new(255, 0, 0)),
        ("Green", RgbColor::new(0, 255, 0)),
        ("Blue", RgbColor::new(0, 0, 255)),
        ("Cyan", RgbColor::new(0, 255, 255)),
        ("Magenta", RgbColor::new(255, 0, 255)),
        ("Yellow", RgbColor::new(255, 255, 0)),
        ("White", RgbColor::new(255, 255, 255)),
    ];
    let mut palette_spans = vec![Span::styled("  Presets: ", Style::default().fg(theme.dim))];
    for (name, col) in &presets {
        let is_cur = c1.r == col.r && c1.g == col.g && c1.b == col.b;
        let p_style = if is_cur {
            Style::default().fg(Color::Rgb(col.r, col.g, col.b)).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
        } else {
            Style::default().fg(Color::Rgb(col.r, col.g, col.b))
        };
        palette_spans.push(Span::styled(format!("[{name}] "), p_style));
    }
    col_lines.push(Line::from(palette_spans));

    f.render_widget(Paragraph::new(col_lines), col_inner);

    // 3. Power States Panel
    let pwr_block = panel_block(" Hardware Power States ", (9..=12).contains(&app.aura_selected_field), theme);
    let pwr_inner = pwr_block.inner(left_chunks[2]);
    f.render_widget(pwr_block, left_chunks[2]);

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
        let text = format!("{name}: {tag}   ");
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

        let effect_desc = match app.aura_mode {
            0 => "Static: Displays a continuous solid color across the entire keyboard matrix.",
            1 => "Breathe: Smooth sinusoidal pulsing transition between Color 1 and Color 2.",
            2 => "Rainbow Cycle: Continuous spectral progression through the full RGB color wheel.",
            10 => "Pulse: Rhythmic strobe pulses at the specified animation speed.",
            _ => "Custom effect.",
        };

        let right_lines = vec![
            Line::from(Span::styled(
                "  Hardware Architecture",
                Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
            )),
            Line::from(vec![
                Span::styled("  Device     : ", Style::default().fg(theme.dim)),
                Span::styled("ASUS TUF Aura Single-Zone Keyboard", Style::default().fg(theme.fg)),
            ]),
            Line::from(vec![
                Span::styled("  Controller : ", Style::default().fg(theme.dim)),
                Span::styled("xyz.ljones.Asusd / Aura DBus Interface", Style::default().fg(theme.fg)),
            ]),
            Line::from(vec![
                Span::styled("  Backlight  : ", Style::default().fg(theme.dim)),
                Span::styled(
                    if is_off { "Off (Hardware LED controller gated)" } else { "Active" },
                    if is_off { Style::default().fg(theme.warn) } else { Style::default().fg(theme.ok) },
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled("  Active Effect", Style::default().fg(theme.dim))),
            Line::from(Span::styled(format!("  {mode_str}"), Style::default().fg(theme.warn).add_modifier(Modifier::BOLD))),
            Line::from(Span::styled(format!("  {effect_desc}"), Style::default().fg(theme.fg))),
            Line::from(""),
            Line::from(Span::styled("  Power States Policy", Style::default().fg(theme.dim))),
            Line::from(Span::styled(
                "  Controls LED behavior during system power transitions. Disabling Sleep\n  and Shutdown LEDs prevents unwanted glow in dark rooms.",
                Style::default().fg(theme.fg),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "  Press 'w' to save lighting and power configuration to firmware.",
                Style::default().fg(theme.dim),
            )),
        ];

        f.render_widget(Paragraph::new(right_lines), details_inner);
    }
}
