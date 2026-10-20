use ratatui::style::{Color, Modifier, Style};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

use crate::backend::types::{PlatformProfile, RgbColor};

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub accent: Color,
    pub ok: Color,
    pub warn: Color,
    pub err: Color,
    pub dim: Color,
    pub fg: Color,
    pub sel_bg: Color,
    pub sel_fg: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            accent: Color::Cyan,
            ok: Color::Green,
            warn: Color::Yellow,
            err: Color::Red,
            dim: Color::Rgb(150, 160, 175),
            fg: Color::White,
            sel_bg: Color::Cyan,
            sel_fg: Color::Black,
        }
    }
}

impl Theme {
    pub fn monochrome() -> Self {
        Self {
            accent: Color::White,
            ok: Color::White,
            warn: Color::White,
            err: Color::White,
            dim: Color::Gray,
            fg: Color::White,
            sel_bg: Color::White,
            sel_fg: Color::Black,
        }
    }

    pub fn load_or_default() -> Self {
        let config_dir = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|_| std::env::var("HOME").map(|h| PathBuf::from(h).join(".config")))
            .ok();

        if let Some(config_dir) = config_dir {
            let path = config_dir.join("rogtui").join("theme.toml");
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(theme) = toml::from_str::<ThemeFile>(&content) {
                        return theme.into_theme();
                    }
                }
            }
        }
        Self::default()
    }

    /// Convert the active UI accent color into an RgbColor.
    pub fn accent_rgb(&self) -> RgbColor {
        match self.accent {
            Color::Cyan => RgbColor::new(0, 220, 255),
            Color::Green => RgbColor::new(0, 220, 100),
            Color::Yellow => RgbColor::new(240, 200, 0),
            Color::Red => RgbColor::new(255, 60, 60),
            Color::Blue => RgbColor::new(60, 140, 255),
            Color::Magenta => RgbColor::new(255, 80, 220),
            Color::Rgb(r, g, b) => RgbColor::new(r, g, b),
            _ => RgbColor::new(0, 220, 255),
        }
    }

    /// Read the active desktop theme dynamically from system configs:
    /// ~/.local/state/theme/colors.sh, ~/.cache/wal/colors.json, Hyprland borders, Sway borders, or terminal palettes.
    /// Evaluates color vibrancy (chroma, saturation, luminosity) to choose the most vivid,
    /// striking accent color for physical keyboard backlighting rather than dull or washed-out tones.
    pub fn read_system_theme() -> (String, RgbColor) {
        let mut candidates: Vec<RgbColor> = Vec::new();
        let mut theme_name = "Theme".to_string();

        if let Ok(home) = std::env::var("HOME") {
            let home_path = PathBuf::from(home);

            // 1. Check colors.sh
            let colors_sh = home_path.join(".local/state/theme/colors.sh");
            if let Ok(content) = fs::read_to_string(&colors_sh) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if let Some(n) = trimmed.strip_prefix("export THEME_NAME=") {
                        theme_name = n.trim_matches('"').to_string();
                    } else if let Some(val) = trimmed.split('=').nth(1) {
                        let hex = val.trim_matches('"').trim_matches('\'').trim();
                        if let Ok(col) = RgbColor::from_hex(hex) {
                            candidates.push(col);
                        }
                    }
                }
            }

            // 2. Check pywal cache (common for Hyprland / BSPWM / Sway)
            let wal_cache = home_path.join(".cache/wal/colors.sh");
            if candidates.is_empty() {
                if let Ok(content) = fs::read_to_string(&wal_cache) {
                    theme_name = "Pywal".to_string();
                    for line in content.lines() {
                        if let Some(val) = line.trim().split('=').nth(1) {
                            let hex = val.trim_matches('\'').trim_matches('"').trim();
                            if let Ok(col) = RgbColor::from_hex(hex) {
                                candidates.push(col);
                            }
                        }
                    }
                }
            }

            // 3. Check Sway borders
            let sway_theme = home_path.join(".config/sway/config.d/80-theme-colors.conf");
            if let Ok(content) = fs::read_to_string(&sway_theme) {
                for line in content.lines() {
                    if line.starts_with("client.focused ") {
                        for token in line.split_whitespace().skip(1) {
                            if let Ok(col) = RgbColor::from_hex(token) {
                                candidates.push(col);
                            }
                        }
                    }
                }
            }

            // 4. Check Hyprland borders
            let hypr_conf = home_path.join(".config/hypr/hyprland.conf");
            if let Ok(content) = fs::read_to_string(&hypr_conf) {
                for line in content.lines() {
                    if line.contains("active_border") {
                        for part in line.split(['(', ')', ' ', ',']) {
                            let clean = part.trim().trim_start_matches("rgba").trim_start_matches("rgb");
                            if let Ok(col) = RgbColor::from_hex(clean) {
                                candidates.push(col);
                            }
                        }
                    }
                }
            }
        }

        // Score all candidates by perceived chroma and saturation
        let mut best_col: Option<RgbColor> = None;
        let mut best_score: f32 = 0.0;

        for c in candidates {
            let max = c.r.max(c.g).max(c.b) as f32;
            let min = c.r.min(c.g).min(c.b) as f32;
            if max == 0.0 {
                continue;
            }
            let sat = (max - min) / max;
            let val = max / 255.0;
            // Reject near-black, near-white, or neutral grays
            if val < 0.25 || (val > 0.94 && sat < 0.15) || sat < 0.18 {
                continue;
            }
            let score = sat * sat * val * 100.0;
            if score > best_score {
                best_score = score;
                best_col = Some(c);
            }
        }

        if let Some(col) = best_col {
            // Boost saturation by +25% so the backlight pops vividly
            let (h, s, v) = col.to_hsv();
            let boosted = RgbColor::from_hsv(h, (s + 25.0).clamp(0.0, 100.0), (v + 5.0).clamp(0.0, 100.0));
            (theme_name, boosted)
        } else {
            ("Theme".to_string(), RgbColor::new(62, 193, 149))
        }
    }

    pub fn get_system_theme_color() -> (String, RgbColor) {
        Self::read_system_theme()
    }

    pub fn sel_style(&self) -> Style {
        Style::default()
            .bg(self.sel_bg)
            .fg(self.sel_fg)
            .add_modifier(Modifier::BOLD)
    }

    pub fn profile_color(&self, profile: PlatformProfile) -> Color {
        match profile {
            PlatformProfile::Quiet => Color::Blue,
            PlatformProfile::Balanced => self.ok,
            PlatformProfile::Performance => self.err,
            PlatformProfile::Custom => Color::Magenta,
        }
    }

    pub fn temp_color(&self, temp_c: u32) -> Color {
        if temp_c < 70 {
            self.ok
        } else if temp_c <= 85 {
            self.warn
        } else {
            self.err
        }
    }

    pub fn usage_color(&self, percent: u32) -> Color {
        if percent < 60 {
            self.ok
        } else if percent <= 85 {
            self.warn
        } else {
            self.err
        }
    }

    pub fn battery_color(&self, percent: u8, is_discharging: bool) -> Color {
        if is_discharging {
            if percent < 15 {
                self.err
            } else if percent < 30 {
                self.warn
            } else {
                self.accent
            }
        } else {
            self.accent
        }
    }

    pub fn track_color() -> Color {
        Color::Rgb(85, 95, 110)
    }
}

#[derive(Debug, Deserialize)]
struct ThemeFile {
    pub accent: Option<String>,
    pub ok: Option<String>,
    pub warn: Option<String>,
    pub err: Option<String>,
    pub dim: Option<String>,
    pub fg: Option<String>,
    pub sel_bg: Option<String>,
    pub sel_fg: Option<String>,
}

impl ThemeFile {
    fn parse_color(s: Option<&str>, default: Color) -> Color {
        match s {
            Some(v) => match v.to_lowercase().as_str() {
                "black" => Color::Black,
                "red" => Color::Red,
                "green" => Color::Green,
                "yellow" => Color::Yellow,
                "blue" => Color::Blue,
                "magenta" => Color::Magenta,
                "cyan" => Color::Cyan,
                "gray" | "grey" => Color::Gray,
                "darkgray" | "darkgrey" => Color::DarkGray,
                "white" => Color::White,
                _ => default,
            },
            None => default,
        }
    }

    fn into_theme(self) -> Theme {
        let def = Theme::default();
        Theme {
            accent: Self::parse_color(self.accent.as_deref(), def.accent),
            ok: Self::parse_color(self.ok.as_deref(), def.ok),
            warn: Self::parse_color(self.warn.as_deref(), def.warn),
            err: Self::parse_color(self.err.as_deref(), def.err),
            dim: Self::parse_color(self.dim.as_deref(), def.dim),
            fg: Self::parse_color(self.fg.as_deref(), def.fg),
            sel_bg: Self::parse_color(self.sel_bg.as_deref(), def.sel_bg),
            sel_fg: Self::parse_color(self.sel_fg.as_deref(), def.sel_fg),
        }
    }
}
