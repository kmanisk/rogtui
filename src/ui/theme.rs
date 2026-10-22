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
    /// ~/.local/state/theme/colors.sh, sway borders, Hyprland borders, or pywal.
    /// Prioritizes designated theme accents (e.g. THEME_ACCENT or focused border),
    /// resolves light theme neutral fallbacks, and boosts saturation/luminance so
    /// physical keyboard LED backlighting pops vividly without washing out.
    pub fn read_system_theme() -> (String, RgbColor) {
        Self::read_system_theme_from_home(std::env::var_os("HOME").map(PathBuf::from).as_deref())
    }

    /// Read a theme from an explicit home without mutating process-wide environment.
    pub fn read_system_theme_from_home(home: Option<&std::path::Path>) -> (String, RgbColor) {
        let mut theme_name = "Theme".to_string();

        if let Some(home_path) = home {

            // 1. Primary: Parse ~/.local/state/theme/colors.sh
            let colors_sh = home_path.join(".local/state/theme/colors.sh");
            if let Ok(content) = fs::read_to_string(&colors_sh) {
                let mut theme_accent: Option<RgbColor> = None;
                let mut fallback_colors: Vec<RgbColor> = Vec::new();

                for line in content.lines() {
                    let trimmed = line.trim();
                    if let Some(n) = trimmed.strip_prefix("export THEME_NAME=") {
                        theme_name = n.trim_matches('"').trim_matches('\'').to_string();
                    } else if let Some(n) = trimmed.strip_prefix("export THEME_ACCENT=") {
                        let hex = n.trim_matches('"').trim_matches('\'').trim();
                        if let Ok(col) = RgbColor::from_hex(hex) {
                            theme_accent = Some(col);
                        }
                    } else if trimmed.starts_with("export THEME_BLUE=")
                        || trimmed.starts_with("export THEME_CYAN=")
                        || trimmed.starts_with("export THEME_PURPLE=")
                        || trimmed.starts_with("export THEME_TEAL=")
                        || trimmed.starts_with("export THEME_GREEN=")
                        || trimmed.starts_with("export THEME_RED=")
                    {
                        if let Some(val) = trimmed.split('=').nth(1) {
                            let hex = val.trim_matches('"').trim_matches('\'').trim();
                            if let Ok(col) = RgbColor::from_hex(hex) {
                                fallback_colors.push(col);
                            }
                        }
                    }
                }

                // If THEME_ACCENT is valid and not washed out (sat >= 18%), use it directly!
                if let Some(accent) = theme_accent {
                    let (_, s, v) = accent.to_hsv();
                    if s >= 18.0 && v >= 20.0 {
                        return (theme_name, Self::boost_for_backlight(accent));
                    }
                }

                // For light/neutral themes where THEME_ACCENT is near-gray (sat < 18%),
                // select the best vibrant primary color from the theme's palette.
                for col in fallback_colors {
                    let (_, s, v) = col.to_hsv();
                    if s >= 20.0 && v >= 25.0 {
                        return (theme_name, Self::boost_for_backlight(col));
                    }
                }

                // If THEME_ACCENT existed even if low saturation, use it rather than hardcoded default
                if let Some(accent) = theme_accent {
                    return (theme_name, Self::boost_for_backlight(accent));
                }
            }

            // 2. Check ~/.local/state/theme/folder_accent or sway.conf
            let folder_accent = home_path.join(".local/state/theme/folder_accent");
            if let Ok(content) = fs::read_to_string(&folder_accent) {
                if let Ok(col) = RgbColor::from_hex(content.trim()) {
                    let (_, s, v) = col.to_hsv();
                    if s >= 15.0 && v >= 20.0 {
                        return (theme_name, Self::boost_for_backlight(col));
                    }
                }
            }

            // 3. Check Sway borders
            let sway_conf = home_path.join(".local/state/theme/sway.conf");
            let sway_theme = if sway_conf.exists() {
                sway_conf
            } else {
                home_path.join(".config/sway/config.d/80-theme-colors.conf")
            };
            if let Ok(content) = fs::read_to_string(&sway_theme) {
                for line in content.lines() {
                    if line.starts_with("client.focused ") {
                        if let Some(first_tok) = line.split_whitespace().nth(1) {
                            if let Ok(col) = RgbColor::from_hex(first_tok) {
                                let (_, s, v) = col.to_hsv();
                                if s >= 15.0 && v >= 20.0 {
                                    return (theme_name, Self::boost_for_backlight(col));
                                }
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
                                let (_, s, v) = col.to_hsv();
                                if s >= 18.0 && v >= 20.0 {
                                    return (theme_name, Self::boost_for_backlight(col));
                                }
                            }
                        }
                    }
                }
            }

            // 5. Check pywal cache (common for Hyprland / BSPWM / Sway)
            let wal_cache = home_path.join(".cache/wal/colors.sh");
            if let Ok(content) = fs::read_to_string(&wal_cache) {
                theme_name = "Pywal".to_string();
                let mut wal_accent: Option<RgbColor> = None;
                for line in content.lines() {
                    let trimmed = line.trim();
                    // In pywal, color4 is typically the primary accent blue
                    if trimmed.starts_with("color4=") || trimmed.starts_with("accent=") {
                        if let Some(val) = trimmed.split('=').nth(1) {
                            let hex = val.trim_matches('\'').trim_matches('"').trim();
                            if let Ok(col) = RgbColor::from_hex(hex) {
                                wal_accent = Some(col);
                                break;
                            }
                        }
                    }
                }
                if let Some(col) = wal_accent {
                    return (theme_name, Self::boost_for_backlight(col));
                }
            }
        }

        ("Theme".to_string(), RgbColor::new(62, 193, 149))
    }

    /// Boost color saturation and value specifically for keyboard LED illumination
    /// to avoid dull, muddy, or washed-out lighting.
    fn boost_for_backlight(col: RgbColor) -> RgbColor {
        let (h, s, v) = col.to_hsv();
        let boosted_s = (s * 1.35 + 18.0).clamp(20.0, 100.0);
        let boosted_v = v.max(85.0).clamp(0.0, 100.0);
        RgbColor::from_hsv(h, boosted_s, boosted_v)
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
