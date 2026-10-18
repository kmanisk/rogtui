use serde::{Deserialize, Serialize};

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
        let points = temps
            .iter()
            .zip(pwms.iter())
            .map(|(&t, &p)| {
                // Convert raw pwm (0..=255) to percent (0..=100) if > 100
                let pwm_percent = if p > 100 {
                    ((p as u32 * 100 + 127) / 255) as u8
                } else {
                    p
                };
                FanCurvePoint {
                    temp: t,
                    pwm_percent,
                }
            })
            .collect();
        Self {
            fan_name: fan_name.into(),
            points,
            enabled,
        }
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
    Dedicated = 0,
    Integrated = 1,
    Hybrid = 2,
    Vfio = 3,
    AsusEgpu = 4,
    AsusMuxDgpu = 5,
    Unknown = 99,
}

impl GpuMode {
    pub fn from_u32(val: u32) -> Self {
        match val {
            0 => GpuMode::Dedicated,
            1 => GpuMode::Integrated,
            2 => GpuMode::Hybrid,
            3 => GpuMode::Vfio,
            4 => GpuMode::AsusEgpu,
            5 => GpuMode::AsusMuxDgpu,
            _ => GpuMode::Unknown,
        }
    }

    pub fn to_u32(self) -> u32 {
        self as u32
    }

    pub fn as_str(self) -> &'static str {
        match self {
            GpuMode::Dedicated => "Dedicated",
            GpuMode::Integrated => "Integrated",
            GpuMode::Hybrid => "Hybrid",
            GpuMode::Vfio => "VFIO",
            GpuMode::AsusEgpu => "ASUS eGPU",
            GpuMode::AsusMuxDgpu => "AsusMuxDgpu (Direct dGPU MUX)",
            GpuMode::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GpuPowerState {
    Unknown = 0,
    Off = 1,
    Active = 2,
    Suspended = 3,
    AsusMuxDiscreet = 4,
    AsusDisabled = 5,
}

impl GpuPowerState {
    pub fn from_u32(val: u32) -> Self {
        match val {
            1 => GpuPowerState::Off,
            2 => GpuPowerState::Active,
            3 => GpuPowerState::Suspended,
            4 => GpuPowerState::AsusMuxDiscreet,
            5 => GpuPowerState::AsusDisabled,
            _ => GpuPowerState::Unknown,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            GpuPowerState::Unknown => "Unknown",
            GpuPowerState::Off => "Powered Off",
            GpuPowerState::Active => "Active",
            GpuPowerState::Suspended => "Suspended (D3cold)",
            GpuPowerState::AsusMuxDiscreet => "Direct MUX (Hardware Bypass)",
            GpuPowerState::AsusDisabled => "Disabled via ACPI",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PendingUserAction {
    Nothing = 0,
    Logout = 1,
    Reboot = 2,
    SwitchToIntegrated = 3,
    Unknown = 99,
}

impl PendingUserAction {
    pub fn from_u32(val: u32) -> Self {
        match val {
            0 => PendingUserAction::Nothing,
            1 => PendingUserAction::Logout,
            2 => PendingUserAction::Reboot,
            3 => PendingUserAction::SwitchToIntegrated,
            _ => PendingUserAction::Unknown,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            PendingUserAction::Nothing => "None required",
            PendingUserAction::Logout => "Session Logout Required",
            PendingUserAction::Reboot => "System Reboot Required",
            PendingUserAction::SwitchToIntegrated => "Must switch to Integrated first",
            PendingUserAction::Unknown => "Unknown action",
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
    pub fn new(r: u8, g: u8, b: u8) -> Self {
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

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SensorData {
    pub cpu_temp_c: Option<u32>,
    pub gpu_temp_c: Option<u32>,
    pub cpu_fan_rpm: Option<u32>,
    pub gpu_fan_rpm: Option<u32>,
    pub battery_charge_percent: Option<u8>,
    pub battery_charge_limit: u8,
    pub panel_overdrive: Option<bool>,
    pub gpu_mux_mode: Option<bool>,
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

