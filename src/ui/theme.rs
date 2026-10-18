use ratatui::style::{Color, Modifier, Style};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

use crate::backend::types::PlatformProfile;

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
            dim: Color::DarkGray,
            fg: Color::White,
            sel_bg: Color::Cyan,
            sel_fg: Color::Black,
        }
    }
}

impl Theme {
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
