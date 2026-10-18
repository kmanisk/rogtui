use serde::{Deserialize, Serialize};

/// Software-driven RGB lighting effects that run on the 1 Hz ticker.
/// These write static colors to the keyboard via the asusd DBus interface.
/// They are distinct from hardware/firmware effects (Static, Breathe, Rainbow Cycle, Pulse)
/// and only active while brightness > 0 and the effect is selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoftwareEffect {
    None,
    // Custom Animations
    RainbowWave,
    ColorCycle,
    FireFlicker,
    Strobe,
    // Audio & Visual Effects
    VoicePulse,
    AudioPulse,
    AudioWave,
    AmbientLight,
    // System Telemetry
    Heatmap,
    GpuMode,
    BatteryStatus,
    CpuLoad,
}

impl SoftwareEffect {
    pub fn all_custom() -> &'static [SoftwareEffect] {
        &[
            SoftwareEffect::RainbowWave,
            SoftwareEffect::ColorCycle,
            SoftwareEffect::FireFlicker,
            SoftwareEffect::Strobe,
            SoftwareEffect::VoicePulse,
            SoftwareEffect::AudioPulse,
            SoftwareEffect::AudioWave,
            SoftwareEffect::AmbientLight,
            SoftwareEffect::Heatmap,
            SoftwareEffect::GpuMode,
            SoftwareEffect::BatteryStatus,
            SoftwareEffect::CpuLoad,
        ]
    }

    pub fn is_animated(self) -> bool {
        matches!(
            self,
            SoftwareEffect::RainbowWave
                | SoftwareEffect::ColorCycle
                | SoftwareEffect::FireFlicker
                | SoftwareEffect::Strobe
                | SoftwareEffect::VoicePulse
                | SoftwareEffect::AudioPulse
                | SoftwareEffect::AudioWave
                | SoftwareEffect::AmbientLight
        )
    }

    pub fn category(self) -> &'static str {
        match self {
            SoftwareEffect::None => "None",
            SoftwareEffect::RainbowWave
            | SoftwareEffect::ColorCycle
            | SoftwareEffect::FireFlicker
            | SoftwareEffect::Strobe => "Custom Animation",
            SoftwareEffect::VoicePulse
            | SoftwareEffect::AudioPulse
            | SoftwareEffect::AudioWave
            | SoftwareEffect::AmbientLight => "Audio & Visual Sync",
            SoftwareEffect::Heatmap
            | SoftwareEffect::GpuMode
            | SoftwareEffect::BatteryStatus
            | SoftwareEffect::CpuLoad => "System Telemetry",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SoftwareEffect::None => "None",
            SoftwareEffect::RainbowWave => "Rainbow Wave",
            SoftwareEffect::ColorCycle => "Color Cycle",
            SoftwareEffect::FireFlicker => "Fire Flicker",
            SoftwareEffect::Strobe => "Strobe / Flash",
            SoftwareEffect::VoicePulse => "Voice Pulse (Mic)",
            SoftwareEffect::AudioPulse => "Audio Pulse",
            SoftwareEffect::AudioWave => "Audio Wave (Spectrum)",
            SoftwareEffect::AmbientLight => "Ambient (Wayland)",
            SoftwareEffect::Heatmap => "Heatmap",
            SoftwareEffect::GpuMode => "GPU Mode Indicator",
            SoftwareEffect::BatteryStatus => "Battery Status",
            SoftwareEffect::CpuLoad => "CPU Load Indicator",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            SoftwareEffect::None => "No software effect active.",
            SoftwareEffect::RainbowWave => {
                "Smooth continuous spectrum sweep across the full 360° color wheel.\n\
                 Rotates hue continuously based on Animation Speed and Direction."
            }
            SoftwareEffect::ColorCycle => {
                "Sequentially transitions through 7 distinct primary colors.\n\
                 Steps at interval defined by Animation Speed and Direction."
            }
            SoftwareEffect::FireFlicker => {
                "Simulates warm fireplace embers with shifting amber, orange, and red tones."
            }
            SoftwareEffect::Strobe => {
                "High-contrast blinking pulse between Primary Color and black."
            }
            SoftwareEffect::VoicePulse => {
                "Microphone voice-reactive illumination. G-Helper style speech sync using\n\
                 the clean microphone (ghelper-audio) - lights up when speaking, rests when silent."
            }
            SoftwareEffect::AudioPulse => {
                "Sound output reactive visualizer. Pulses keyboard illumination to music\n\
                 beat drops and audio playback volume across speakers and headphones."
            }
            SoftwareEffect::AudioWave => {
                "Dynamic audio spectrum equalizer. Shifts keyboard color hue and intensity\n\
                 across the frequency and amplitude spectrum from deep blue to peak red."
            }
            SoftwareEffect::AmbientLight => {
                "Real-time screen ambient sync on Wayland. Samples the display compositor via\n\
                 grim to continuously mirror movie, video, or game visuals on the keyboard."
            }
            SoftwareEffect::Heatmap => {
                "Maps CPU/GPU temperature to keyboard color.\n\
                 Blue (cool) -> Yellow (warm) -> Red (hot).\n\
                 Uses the higher of CPU/GPU readings."
            }
            SoftwareEffect::GpuMode => {
                "Reflects the active supergfx GPU operating mode:\n\
                 Green = Integrated, Yellow = Hybrid, Orange = AsusMuxDgpu."
            }
            SoftwareEffect::BatteryStatus => {
                "Reflects battery charge state:\n\
                 Green = charging, Yellow = discharging, Red = low (<20%)."
            }
            SoftwareEffect::CpuLoad => {
                "Reflects real-time processor workload:\n\
                 Green (<30%) -> Yellow (30-65%) -> Orange (65-85%) -> Red (>85%)."
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlatformProfile {
    Quiet = 2,
    Balanced = 0,
    Performance = 1,
    Custom = 3,
}

impl PlatformProfile {
    pub fn from_u32(val: u32) -> Self {
        match val {
            1 => PlatformProfile::Performance,
            2 => PlatformProfile::Quiet,
            3 => PlatformProfile::Custom,
            _ => PlatformProfile::Balanced,
        }
    }

    pub fn to_u32(self) -> u32 {
        self as u32
    }

    pub fn as_str(self) -> &'static str {
        match self {
            PlatformProfile::Quiet => "Quiet",
            PlatformProfile::Balanced => "Balanced",
            PlatformProfile::Performance => "Performance",
            PlatformProfile::Custom => "Custom",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FanCurvePoint {
    pub temp: u8,
    pub pwm_percent: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FanCurve {
    pub fan_name: String,
    pub points: Vec<FanCurvePoint>,
    pub enabled: bool,
}

pub type EightBytes = (u8, u8, u8, u8, u8, u8, u8, u8);
pub type FanCurveTuple = (String, EightBytes, EightBytes, bool);

impl FanCurve {
    pub fn new(fan_name: impl Into<String>, temps: [u8; 8], pwms: [u8; 8], enabled: bool) -> Self {
        // In asusd, DBus PWM bytes are always 0..=255.
        // If any byte > 100 or top byte == 255, scale from 0..=255 to 0..=100.
        let is_raw_255 = pwms.iter().any(|&p| p > 100) || pwms[7] == 255;
        let points = temps
            .iter()
            .zip(pwms.iter())
            .map(|(&t, &p)| {
                let pwm_percent = if is_raw_255 {
                    ((p as u32 * 100 + 127) / 255) as u8
                } else {
                    p
                };
                FanCurvePoint {
                    temp: t,
                    pwm_percent: pwm_percent.min(100),
                }
            })
            .collect();
        Self {
            fan_name: fan_name.into(),
            points,
            enabled,
        }
    }

    pub fn check_monotonicity(&self) -> Result<(), String> {
        for i in 0..self.points.len().saturating_sub(1) {
            if self.points[i].pwm_percent > self.points[i + 1].pwm_percent {
                return Err(format!(
                    "curve decreases between pt{} ({}%) and pt{} ({}%), press 'x' to repair",
                    i + 1,
                    self.points[i].pwm_percent,
                    i + 2,
                    self.points[i + 1].pwm_percent
                ));
            }
            if self.points[i].temp > self.points[i + 1].temp {
                return Err(format!(
                    "temperature decreases between pt{} ({} C) and pt{} ({} C), press 'x' to repair",
                    i + 1,
                    self.points[i].temp,
                    i + 2,
                    self.points[i + 1].temp
                ));
            }
        }
        Ok(())
    }

    pub fn repair(&mut self) -> bool {
        let mut modified = false;
        // 1. Enforce non-decreasing speed (running max)
        let mut cur_max_speed = 0u8;
        for pt in &mut self.points {
            if pt.pwm_percent < cur_max_speed {
                pt.pwm_percent = cur_max_speed;
                modified = true;
            } else {
                cur_max_speed = pt.pwm_percent;
            }
        }
        // 2. Enforce strictly increasing temperatures
        let mut cur_min_temp = 30u8;
        for pt in &mut self.points {
            if pt.temp < cur_min_temp {
                pt.temp = cur_min_temp;
                modified = true;
            }
            cur_min_temp = (pt.temp + 1).min(100);
        }
        modified
    }

    pub fn to_dbus_raw(&self) -> FanCurveTuple {
        let mut t = [0u8; 8];
        let mut p = [0u8; 8];
        for (i, pt) in self.points.iter().take(8).enumerate() {
            t[i] = pt.temp;
            p[i] = ((pt.pwm_percent as u32 * 255) / 100) as u8;
        }
        (
            self.fan_name.clone(),
            (p[0], p[1], p[2], p[3], p[4], p[5], p[6], p[7]),
            (t[0], t[1], t[2], t[3], t[4], t[5], t[6], t[7]),
            self.enabled,
        )
    }

    pub fn interpolate(&self, current_temp: f64) -> f64 {
        if self.points.is_empty() {
            return 0.0;
        }
        let first = &self.points[0];
        if current_temp <= first.temp as f64 {
            return first.pwm_percent as f64;
        }
        let last = &self.points[self.points.len() - 1];
        if current_temp >= last.temp as f64 {
            return last.pwm_percent as f64;
        }

        for i in 0..self.points.len() - 1 {
            let p1 = &self.points[i];
            let p2 = &self.points[i + 1];
            if current_temp >= p1.temp as f64 && current_temp <= p2.temp as f64 {
                let range = (p2.temp - p1.temp) as f64;
                if range <= 0.0 {
                    return p1.pwm_percent as f64;
                }
                let ratio = (current_temp - p1.temp as f64) / range;
                return p1.pwm_percent as f64 + ratio * ((p2.pwm_percent as i16 - p1.pwm_percent as i16) as f64);
            }
        }
        last.pwm_percent as f64
    }

    pub fn edit_speed(&mut self, idx: usize, delta: i16, allow_unsafe_floor: bool) -> Result<(), String> {
        if idx >= self.points.len() {
            return Err("Index out of bounds".to_string());
        }
        let cur = self.points[idx].pwm_percent as i16;
        let target = cur + delta;

        let lower_limit = if idx > 0 { self.points[idx - 1].pwm_percent as i16 } else { 0 };
        let upper_limit = if idx + 1 < self.points.len() { self.points[idx + 1].pwm_percent as i16 } else { 100 };

        let mut clamped = target.clamp(lower_limit, upper_limit).clamp(0, 100);

        // Safety floor rule: at or above 80 C, minimum speed is 30% unless override confirmed
        if self.points[idx].temp >= 80 && clamped < 30 && !allow_unsafe_floor {
            clamped = 30;
            self.points[idx].pwm_percent = clamped as u8;
            return Err("Safety policy: minimum 30% at >= 80 C (use Shift+O to override)".to_string());
        }

        self.points[idx].pwm_percent = clamped as u8;

        if target < lower_limit {
            return Err(format!("Limited by previous point (cannot be below {lower_limit}%)"));
        }
        if target > upper_limit {
            return Err(format!("Limited by next point (cannot exceed {upper_limit}%)"));
        }
        Ok(())
    }

    pub fn edit_temp(&mut self, idx: usize, delta: i16) -> Result<(), String> {
        if idx >= self.points.len() {
            return Err("Index out of bounds".to_string());
        }
        let cur = self.points[idx].temp as i16;
        let target = cur + delta;

        let lower_limit = if idx > 0 { self.points[idx - 1].temp as i16 + 1 } else { 30 };
        let upper_limit = if idx + 1 < self.points.len() { self.points[idx + 1].temp as i16 - 1 } else { 100 };

        let clamped = target.clamp(lower_limit, upper_limit).clamp(30, 100);
        self.points[idx].temp = clamped as u8;

        if target < lower_limit {
            return Err(format!("Limited by previous point (must be > {} C)", lower_limit - 1));
        }
        if target > upper_limit {
            return Err(format!("Limited by next point (must be < {} C)", upper_limit + 1));
        }
        Ok(())
    }

    pub fn apply_preset(&mut self, preset_name: &str) {
        match preset_name {
            "Silent" => {
                let temps = [45, 55, 65, 75, 82, 88, 92, 98];
                let speeds = [0, 10, 20, 35, 50, 65, 80, 100];
                for (i, p) in self.points.iter_mut().enumerate() {
                    p.temp = temps[i];
                    p.pwm_percent = speeds[i];
                }
            }
            "Aggressive" => {
                let temps = [40, 50, 58, 65, 72, 78, 85, 92];
                let speeds = [15, 30, 45, 60, 75, 85, 95, 100];
                for (i, p) in self.points.iter_mut().enumerate() {
                    p.temp = temps[i];
                    p.pwm_percent = speeds[i];
                }
            }
            _ => {
                // Balanced
                let temps = [45, 55, 65, 72, 78, 84, 90, 96];
                let speeds = [5, 15, 30, 50, 65, 78, 88, 100];
                for (i, p) in self.points.iter_mut().enumerate() {
                    p.temp = temps[i];
                    p.pwm_percent = speeds[i];
                }
            }
        }
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.points.len() != 8 {
            return Err("Fan curve must contain exactly 8 control points");
        }
        for i in 0..7 {
            if self.points[i].temp > self.points[i + 1].temp {
                return Err("Temperatures must be non-decreasing across points");
            }
            if self.points[i].pwm_percent > self.points[i + 1].pwm_percent {
                return Err("Fan speeds must be non-decreasing across points");
            }
        }
        for pt in &self.points {
            if pt.temp > 115 {
                return Err("Temperature exceeds maximum 115 C");
            }
            if pt.pwm_percent > 100 {
                return Err("Fan speed exceeds 100%");
            }
            if pt.temp >= 80 && pt.pwm_percent < 30 {
                return Err("Safety policy: fan speed cannot be below 30% at or above 80 C");
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GpuMode {
    Hybrid = 0,
    Integrated = 1,
    NvidiaNoModeset = 2,
    Vfio = 3,
    AsusEgpu = 4,
    AsusMuxDgpu = 5,
    None = 6,
    Unknown = 99,
}

impl GpuMode {
    pub fn from_u32(val: u32) -> Self {
        match val {
            0 => GpuMode::Hybrid,
            1 => GpuMode::Integrated,
            2 => GpuMode::NvidiaNoModeset,
            3 => GpuMode::Vfio,
            4 => GpuMode::AsusEgpu,
            5 => GpuMode::AsusMuxDgpu,
            6 => GpuMode::None,
            _ => GpuMode::Unknown,
        }
    }

    pub fn to_u32(self) -> u32 {
        self as u32
    }

    pub fn as_str(self) -> &'static str {
        match self {
            GpuMode::Hybrid => "Hybrid",
            GpuMode::Integrated => "Integrated",
            GpuMode::NvidiaNoModeset => "NvidiaNoModeset",
            GpuMode::Vfio => "VFIO",
            GpuMode::AsusEgpu => "ASUS eGPU",
            GpuMode::AsusMuxDgpu => "Ultimate (Direct dGPU MUX)",
            GpuMode::None => "None",
            GpuMode::Unknown => "-",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GpuPowerState {
    Active = 0,
    Suspended = 1,
    Off = 2,
    AsusDisabled = 3,
    AsusMuxDiscreet = 4,
    Unknown = 5,
}

impl GpuPowerState {
    pub fn from_u32(val: u32) -> Self {
        match val {
            0 => GpuPowerState::Active,
            1 => GpuPowerState::Suspended,
            2 => GpuPowerState::Off,
            3 => GpuPowerState::AsusDisabled,
            4 => GpuPowerState::AsusMuxDiscreet,
            _ => GpuPowerState::Unknown,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            GpuPowerState::Active => "Active",
            GpuPowerState::Suspended => "Suspended (D3cold)",
            GpuPowerState::Off => "Powered Off",
            GpuPowerState::AsusDisabled => "Disabled via ACPI",
            GpuPowerState::AsusMuxDiscreet => "Active (Direct MUX)",
            GpuPowerState::Unknown => "-",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PendingUserAction {
    Logout = 0,
    Reboot = 1,
    SwitchToIntegrated = 2,
    AsusEgpuDisable = 3,
    Nothing = 4,
    Unknown = 99,
}

impl PendingUserAction {
    pub fn from_u32(val: u32) -> Self {
        match val {
            0 => PendingUserAction::Logout,
            1 => PendingUserAction::Reboot,
            2 => PendingUserAction::SwitchToIntegrated,
            3 => PendingUserAction::AsusEgpuDisable,
            4 => PendingUserAction::Nothing,
            _ => PendingUserAction::Unknown,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            PendingUserAction::Nothing => "None",
            PendingUserAction::Logout => "Session Logout Required",
            PendingUserAction::Reboot => "System Reboot Required",
            PendingUserAction::SwitchToIntegrated => "Must switch to Integrated first",
            PendingUserAction::AsusEgpuDisable => "Disable ASUS eGPU first",
            PendingUserAction::Unknown => "-",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn to_hex(self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    pub fn from_hex(s: &str) -> Result<Self, &'static str> {
        let cleaned = s.trim().trim_start_matches('#');
        if cleaned.len() != 6 {
            return Err("Hex colour must be 6 hex characters (e.g. #FFAA00 or FFAA00)");
        }
        let r = u8::from_str_radix(&cleaned[0..2], 16).map_err(|_| "Invalid hex characters for Red")?;
        let g = u8::from_str_radix(&cleaned[2..4], 16).map_err(|_| "Invalid hex characters for Green")?;
        let b = u8::from_str_radix(&cleaned[4..6], 16).map_err(|_| "Invalid hex characters for Blue")?;
        Ok(Self { r, g, b })
    }

    pub fn to_hsv(&self) -> (f32, f32, f32) {
        let rf = self.r as f32 / 255.0;
        let gf = self.g as f32 / 255.0;
        let bf = self.b as f32 / 255.0;
        let max = rf.max(gf).max(bf);
        let min = rf.min(gf).min(bf);
        let delta = max - min;

        let v = max * 100.0;
        let s = if max == 0.0 { 0.0 } else { (delta / max) * 100.0 };
        let h = if delta == 0.0 {
            0.0
        } else if (max - rf).abs() < f32::EPSILON {
            60.0 * (((gf - bf) / delta) % 6.0)
        } else if (max - gf).abs() < f32::EPSILON {
            60.0 * (((bf - rf) / delta) + 2.0)
        } else {
            60.0 * (((rf - gf) / delta) + 4.0)
        };
        let h = (h + 360.0) % 360.0;
        (h, s, v)
    }

    pub fn from_hsv(h: f32, s: f32, v: f32) -> Self {
        let s_norm = (s / 100.0).clamp(0.0, 1.0);
        let v_norm = (v / 100.0).clamp(0.0, 1.0);
        let h_norm = (h % 360.0 + 360.0) % 360.0;
        let c = v_norm * s_norm;
        let x = c * (1.0 - ((h_norm / 60.0) % 2.0 - 1.0).abs());
        let m = v_norm - c;
        let (r_prime, g_prime, b_prime) = match (h_norm / 60.0) as i32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };
        Self {
            r: ((r_prime + m) * 255.0).round().clamp(0.0, 255.0) as u8,
            g: ((g_prime + m) * 255.0).round().clamp(0.0, 255.0) as u8,
            b: ((b_prime + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuraModeData {
    pub mode: u32,
    pub zone: u32,
    pub color1: RgbColor,
    pub color2: RgbColor,
    pub speed: String,
    pub direction: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuraPowerState {
    pub boot: bool,
    pub awake: bool,
    pub sleep: bool,
    pub shutdown: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttributeKind {
    Toggle,
    Enum,
    Slider,
    ReadOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArmouryAttribute {
    pub node_name: String,
    pub display_name: String,
    pub current_value: i32,
    pub default_value: i32,
    pub min_value: i32,
    pub max_value: i32,
    pub possible_values: Vec<i32>,
    pub scalar_increment: i32,
    pub kind: AttributeKind,
    pub is_available: bool,
}

impl ArmouryAttribute {
    pub fn category(&self) -> &'static str {
        match self.node_name.as_str() {
            "gpu_mux_mode" | "dgpu_disable" | "nv_base_tgp" | "nv_tgp" | "nv_dynamic_boost" | "nv_temp_target" => "GPU",
            "panel_overdrive" | "screen_auto_brightness" => "Display",
            "ppt_pl1_spl" | "ppt_pl2_sppt" => "Power limits",
            "charge_mode" => "Battery",
            _ => "General",
        }
    }

    pub fn display_value(&self) -> String {
        if !self.is_available {
            return "Unavailable".to_string();
        }
        match self.node_name.as_str() {
            "gpu_mux_mode" => match self.current_value {
                0 => "dGPU Direct (MUX)".to_string(),
                1 => "Optimus / Hybrid".to_string(),
                v => format!("Value {v}"),
            },
            "dgpu_disable" => match self.current_value {
                0 => "[ ] Enabled (dGPU on)".to_string(),
                1 => "[x] Disabled (dGPU off)".to_string(),
                v => format!("Value {v}"),
            },
            "panel_overdrive" => match self.current_value {
                1 => "[x] Enabled".to_string(),
                0 => "[ ] Disabled".to_string(),
                v => format!("Value {v}"),
            },
            "charge_mode" => match self.current_value {
                0 => "100% (Standard)".to_string(),
                1 => "80% (Balanced)".to_string(),
                2 => "60% (Max Lifespan)".to_string(),
                v => format!("Mode {v}"),
            },
            "screen_auto_brightness" => match self.current_value {
                1 => "[x] Enabled".to_string(),
                0 => "[ ] Disabled".to_string(),
                v => format!("Value {v}"),
            },
            _ => match self.kind {
                AttributeKind::Toggle => {
                    if self.current_value == 1 {
                        "[x] Enabled".to_string()
                    } else {
                        "[ ] Disabled".to_string()
                    }
                }
                AttributeKind::Enum => format!("< Value: {} >", self.current_value),
                AttributeKind::Slider => format!("< {} >", self.current_value),
                AttributeKind::ReadOnly => format!("{}", self.current_value),
            },
        }
    }

    pub fn details(&self) -> (&'static str, Option<&'static str>) {
        match self.node_name.as_str() {
            "gpu_mux_mode" => (
                "Hardware display panel multiplexer switch.\nRoutes internal eDP panel directly to dGPU or iGPU.",
                Some("Switching MUX mode requires a system reboot."),
            ),
            "dgpu_disable" => (
                "ACPI-level power gate for the discrete GPU.\nCompletely powers down the dGPU hardware.",
                Some("Requires session reload or reboot to safely apply."),
            ),
            "panel_overdrive" => (
                "LCD panel response time overdrive.\nReduces ghosting during high-refresh gaming.",
                None,
            ),
            "charge_mode" => (
                "Hardware battery charging thresholds.\nCaps charge limit to prolong lithium battery lifespan.",
                None,
            ),
            "screen_auto_brightness" => (
                "Ambient light sensor display auto-brightness control.",
                None,
            ),
            "nv_base_tgp" => (
                "Base total graphics power (TGP) limit in watts.",
                None,
            ),
            "nv_tgp" => (
                "Configurable total graphics power boost wattage.",
                None,
            ),
            "nv_dynamic_boost" => (
                "NVIDIA Dynamic Boost power allocation between CPU and GPU.",
                None,
            ),
            "nv_temp_target" => (
                "Thermal target ceiling for NVIDIA GPU before throttling.",
                None,
            ),
            "ppt_pl1_spl" => (
                "Sustained power limit (SPL) for CPU package in watts.",
                None,
            ),
            "ppt_pl2_sppt" => (
                "Slow package power tracking (sPPT) burst limit in watts.",
                None,
            ),
            _ => ("ASUS firmware hardware attribute.", None),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SensorData {
    pub cpu_temp_c: Option<u32>,
    pub gpu_temp_c: Option<u32>,
    pub cpu_fan_rpm: Option<u32>,
    pub gpu_fan_rpm: Option<u32>,
    pub battery_charge_percent: Option<u8>,
    pub battery_charge_limit: u8,
    pub panel_overdrive: Option<bool>,
    pub gpu_mux_mode: Option<bool>,

    // Phase 2 Hardware Telemetry
    pub cpu_model: String,
    pub cpu_freq_mhz: Option<u32>,
    pub cpu_usage_percent: Option<u8>,
    pub ram_used_mb: Option<u64>,
    pub ram_total_mb: Option<u64>,
    pub ram_usage_percent: Option<u8>,
    pub igpu_name: String,
    pub igpu_temp_c: Option<u32>,
    pub dgpu_name: String,
    pub dgpu_status: String,
    pub dgpu_power_state: String,

    // Battery Extended Telemetry
    pub battery_status: String,
    pub battery_power_w: Option<f32>,
    pub battery_health_percent: Option<u8>,
    pub battery_cycle_count: Option<u32>,
    pub screen_auto_brightness: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_color_valid() {
        let col = RgbColor::from_hex("#FF8800").expect("valid hex");
        assert_eq!(col.r, 255);
        assert_eq!(col.g, 136);
        assert_eq!(col.b, 0);
        assert_eq!(col.to_hex(), "#FF8800");

        let col2 = RgbColor::from_hex("00ffcc").expect("valid hex without hash");
        assert_eq!(col2.r, 0);
        assert_eq!(col2.g, 255);
        assert_eq!(col2.b, 204);
    }

    #[test]
    fn test_hex_color_invalid() {
        assert!(RgbColor::from_hex("#FFF").is_err());
        assert!(RgbColor::from_hex("ZZZZZZ").is_err());
        assert!(RgbColor::from_hex("").is_err());
    }

    #[test]
    fn test_fan_curve_validation_monotonic() {
        let valid_curve = FanCurve::new(
            "CPU",
            [40, 50, 60, 70, 78, 85, 90, 95],
            [10, 20, 30, 45, 60, 75, 85, 100],
            true,
        );
        assert!(valid_curve.validate().is_ok());

        let non_monotonic_temp = FanCurve::new(
            "CPU",
            [40, 50, 45, 70, 78, 85, 90, 95],
            [10, 20, 30, 45, 60, 75, 85, 100],
            true,
        );
        assert!(non_monotonic_temp.validate().is_err());

        let non_monotonic_speed = FanCurve::new(
            "CPU",
            [40, 50, 60, 70, 78, 85, 90, 95],
            [10, 30, 25, 45, 60, 75, 85, 100],
            true,
        );
        assert!(non_monotonic_speed.validate().is_err());
    }

    #[test]
    fn test_fan_curve_safety_clamp() {
        // Temp >= 80C with speed < 30% must fail safety validation
        let unsafe_curve = FanCurve::new(
            "CPU",
            [40, 50, 60, 70, 75, 80, 90, 95],
            [10, 15, 20, 22, 24, 25, 85, 100],
            true,
        );
        assert!(unsafe_curve.validate().is_err());
    }
}

