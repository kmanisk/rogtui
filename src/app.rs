use std::collections::VecDeque;
use std::time::Instant;

use crate::backend::types::{
    ArmouryAttribute, AuraModeData, AuraPowerState, FanCurve, GpuMode, GpuPowerState,
    PendingUserAction, PlatformProfile, RgbColor, SensorData, SoftwareEffect,
};
use crate::backend::Backend;
use crate::ui::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Dashboard = 0,
    Power = 1,
    FanCurves = 2,
    Aura = 3,
    Gpu = 4,
    Armoury = 5,
    About = 6,
}

impl ActiveTab {
    pub fn all() -> &'static [ActiveTab] {
        &[
            ActiveTab::Dashboard,
            ActiveTab::Power,
            ActiveTab::FanCurves,
            ActiveTab::Aura,
            ActiveTab::Gpu,
            ActiveTab::Armoury,
            ActiveTab::About,
        ]
    }

    pub fn title(self) -> &'static str {
        match self {
            ActiveTab::Dashboard => "1.Dashboard",
            ActiveTab::Power => "2.Power",
            ActiveTab::FanCurves => "3.Fan Curves",
            ActiveTab::Aura => "4.RGB Aura",
            ActiveTab::Gpu => "5.GPU (supergfx)",
            ActiveTab::Armoury => "6.Armoury/Firmware",
            ActiveTab::About => "7.About",
        }
    }

    pub fn next(self) -> Self {
        match self {
            ActiveTab::Dashboard => ActiveTab::Power,
            ActiveTab::Power => ActiveTab::FanCurves,
            ActiveTab::FanCurves => ActiveTab::Aura,
            ActiveTab::Aura => ActiveTab::Gpu,
            ActiveTab::Gpu => ActiveTab::Armoury,
            ActiveTab::Armoury => ActiveTab::About,
            ActiveTab::About => ActiveTab::Dashboard,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            ActiveTab::Dashboard => ActiveTab::About,
            ActiveTab::Power => ActiveTab::Dashboard,
            ActiveTab::FanCurves => ActiveTab::Power,
            ActiveTab::Aura => ActiveTab::FanCurves,
            ActiveTab::Gpu => ActiveTab::Aura,
            ActiveTab::Armoury => ActiveTab::Gpu,
            ActiveTab::About => ActiveTab::Armoury,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConfirmAction {
    SwitchGpu(GpuMode),
    ApplyFanCurve(PlatformProfile, FanCurve),
    ApplyAllFanCurves(PlatformProfile, Vec<FanCurve>),
    ResetFanCurve(PlatformProfile),
    DiscardFanChangesSwitchFan(usize),
    DiscardFanChangesSwitchProfile(PlatformProfile),
    ApplyArmouryAttr { node_name: String, value: i32, display_name: String },
    RestoreArmouryDefault { node_name: String, display_name: String },
    SaveAuraFirmware,
}

pub struct AppState<B: Backend> {
    pub backend: B,
    pub theme: Theme,
    pub current_tab: ActiveTab,
    pub show_help: bool,
    pub should_quit: bool,
    pub status_message: Option<(String, bool, Instant)>,
    pub confirm_modal: Option<(String, ConfirmAction)>,
    pub show_advanced_power_modal: bool,

    // Hardware State
    pub platform_profile: PlatformProfile,
    pub profile_choices: Vec<PlatformProfile>,
    pub profile_ac: PlatformProfile,
    pub profile_battery: PlatformProfile,
    pub auto_ac: bool,
    pub auto_battery: bool,
    pub charge_limit: u8,
    pub linked_epp: bool,
    pub disable_powerd_battery: bool,

    // GPU State
    pub gpu_mode: GpuMode,
    pub gpu_supported: Vec<GpuMode>,
    pub gpu_power: GpuPowerState,
    pub gpu_pending_mode: GpuMode,
    pub gpu_pending_action: PendingUserAction,
    pub gpu_selected_idx: usize,

    // Fan Curve State
    pub fan_curves: Vec<FanCurve>,
    pub applied_fan_curves: Vec<FanCurve>,
    pub active_fan_idx: usize,
    pub active_point_idx: usize,
    pub last_known_good_curve: Option<FanCurve>,
    pub show_other_fan_overlay: bool,
    pub show_point_table: bool,
    pub apply_all_fans: bool,
    pub fan_clamp_hint: Option<(String, Instant)>,
    pub allow_unsafe_floor: bool,
    pub current_preset_idx: usize,

    // Aura RGB State
    pub aura_brightness: u32,
    pub aura_mode: u32,
    pub aura_data: AuraModeData,
    pub aura_power: AuraPowerState,
    pub aura_selected_field: usize,
    pub hex_input_buffer: Option<String>,

    // Color Picker Modal State
    pub show_color_picker_modal: bool,
    pub picker_target_slot: usize,
    pub picker_slider_idx: usize,
    pub picker_hue: f32,
    pub picker_sat: f32,
    pub picker_val: f32,
    pub picker_initial_color: RgbColor,

    // Software RGB Effects (1 Hz ticker-driven, volatile writes via asusd)
    pub active_software_effect: SoftwareEffect,
    /// Last color written to hardware; prevents redundant DBus writes when color unchanged.
    pub last_effect_color: Option<RgbColor>,
    pub rainbow_wave_hue: f32,
    pub color_cycle_idx: usize,
    pub strobe_state: bool,
    pub flicker_seed: u32,
    pub mic_level: u8,
    pub audio_level: u8,
    pub last_ambient_color: Option<RgbColor>,

    // Armoury Attributes State
    pub armoury_attributes: Vec<ArmouryAttribute>,
    pub armoury_selected_idx: usize,
    pub show_unavailable_armoury: bool,

    // Sensors and history
    pub sensors: SensorData,
    pub cpu_temp_history: VecDeque<u64>,
    pub gpu_temp_history: VecDeque<u64>,

    // Power Tab field index
    pub power_selected_field: usize,

    // Metrics Logging State (Page 1 toggleable telemetry recorder)
    pub is_logging_metrics: bool,
    pub metrics_log_path: std::path::PathBuf,
    pub metrics_logged_count: usize,
}

impl<B: Backend> AppState<B> {
    pub async fn new(backend: B) -> anyhow::Result<Self> {
        let platform_profile = backend.get_platform_profile().await.unwrap_or(PlatformProfile::Balanced);
        let profile_choices = backend.get_platform_profile_choices().await.unwrap_or_else(|_| vec![
            PlatformProfile::Quiet,
            PlatformProfile::Balanced,
            PlatformProfile::Performance,
        ]);
        let profile_ac = backend.get_profile_ac().await.unwrap_or(PlatformProfile::Performance);
        let profile_battery = backend.get_profile_battery().await.unwrap_or(PlatformProfile::Quiet);
        let auto_ac = backend.get_auto_ac().await.unwrap_or(true);
        let auto_battery = backend.get_auto_battery().await.unwrap_or(true);
        let charge_limit = backend.get_charge_limit().await.unwrap_or(60);
        let linked_epp = backend.get_linked_epp().await.unwrap_or(true);
        let disable_powerd_battery = backend.get_disable_nvidia_powerd_battery().await.unwrap_or(true);

        let gpu_mode = backend.get_gpu_mode().await.unwrap_or(GpuMode::Unknown);
        let gpu_supported = backend.get_gpu_supported().await.unwrap_or_else(|_| vec![gpu_mode]);
        let gpu_power = backend.get_gpu_power().await.unwrap_or(GpuPowerState::Unknown);
        let gpu_pending_mode = backend.get_gpu_pending_mode().await.unwrap_or(gpu_mode);
        let gpu_pending_action = backend.get_gpu_pending_action().await.unwrap_or(PendingUserAction::Nothing);

        let fan_curves = backend.get_fan_curves(platform_profile).await.unwrap_or_default();
        let last_known_good_curve = fan_curves.first().cloned();

        let aura_brightness = backend.get_aura_brightness().await.unwrap_or(0);
        let aura_mode = backend.get_aura_mode().await.unwrap_or(0);
        let aura_data = backend.get_aura_mode_data().await.unwrap_or(AuraModeData {
            mode: 0,
            zone: 0,
            color1: crate::backend::types::RgbColor::new(255, 255, 255),
            color2: crate::backend::types::RgbColor::new(0, 0, 0),
            speed: "Med".to_string(),
            direction: "Right".to_string(),
        });
        let aura_power = backend.get_aura_power().await.unwrap_or(AuraPowerState {
            boot: true,
            awake: true,
            sleep: false,
            shutdown: false,
        });

        let armoury_attributes = backend.get_armoury_attributes().await.unwrap_or_default();
        let sensors = backend.read_sensors().await.unwrap_or_default();

        let mut cpu_temp_history = VecDeque::with_capacity(60);
        let mut gpu_temp_history = VecDeque::with_capacity(60);
        if let Some(c) = sensors.cpu_temp_c {
            cpu_temp_history.push_back(c as u64);
        }
        if let Some(g) = sensors.gpu_temp_c {
            gpu_temp_history.push_back(g as u64);
        }

        let gpu_selected_idx = gpu_supported.iter().position(|m| *m == gpu_mode).unwrap_or(0);

        Ok(Self {
            backend,
            theme: Theme::load_or_default(),
            current_tab: ActiveTab::Dashboard,
            show_help: false,
            should_quit: false,
            status_message: None,
            confirm_modal: None,
            show_advanced_power_modal: false,
            platform_profile,
            profile_choices,
            profile_ac,
            profile_battery,
            auto_ac,
            auto_battery,
            charge_limit,
            linked_epp,
            disable_powerd_battery,
            gpu_mode,
            gpu_supported,
            gpu_power,
            gpu_pending_mode,
            gpu_pending_action,
            gpu_selected_idx,
            applied_fan_curves: fan_curves.clone(),
            fan_curves,
            active_fan_idx: 0,
            active_point_idx: 0,
            last_known_good_curve,
            show_other_fan_overlay: false,
            show_point_table: false,
            apply_all_fans: false,
            fan_clamp_hint: None,
            allow_unsafe_floor: false,
            current_preset_idx: 1, // Balanced
            aura_brightness,
            aura_mode,
            aura_data,
            aura_power,
            aura_selected_field: 0,
            hex_input_buffer: None,
            show_color_picker_modal: false,
            picker_target_slot: 1,
            picker_slider_idx: 0,
            picker_hue: 180.0,
            picker_sat: 100.0,
            picker_val: 100.0,
            picker_initial_color: RgbColor::new(0, 200, 255),
            active_software_effect: SoftwareEffect::None,
            last_effect_color: None,
            rainbow_wave_hue: 0.0,
            color_cycle_idx: 0,
            strobe_state: false,
            flicker_seed: 1234567,
            mic_level: 0,
            audio_level: 0,
            last_ambient_color: None,
            armoury_attributes,
            armoury_selected_idx: 0,
            show_unavailable_armoury: false,
            sensors,
            cpu_temp_history,
            gpu_temp_history,
            power_selected_field: 0,
            is_logging_metrics: false,
            metrics_log_path: if let Ok(home) = std::env::var("HOME") {
                std::path::PathBuf::from(home).join(".local/share/rogtui/metrics.csv")
            } else {
                std::path::PathBuf::from("/tmp/rogtui-metrics.csv")
            },
            metrics_logged_count: 0,
        })
    }

    pub fn open_color_picker(&mut self, slot: usize) {
        self.show_color_picker_modal = true;
        self.picker_target_slot = slot;
        self.picker_slider_idx = 0;
        let col = if slot == 2 {
            self.aura_data.color2
        } else {
            self.aura_data.color1
        };
        self.picker_initial_color = col;
        let (h, s, v) = col.to_hsv();
        self.picker_hue = h;
        self.picker_sat = s;
        self.picker_val = v;
    }

    pub fn picker_current_color(&self) -> RgbColor {
        RgbColor::from_hsv(self.picker_hue, self.picker_sat, self.picker_val)
    }

    pub fn set_status(&mut self, msg: impl Into<String>, is_error: bool) {
        self.status_message = Some((msg.into(), is_error, Instant::now()));
    }

    pub fn set_clamp_hint(&mut self, hint: impl Into<String>) {
        self.fan_clamp_hint = Some((hint.into(), Instant::now()));
    }

    pub fn is_fan_curve_modified(&self) -> bool {
        if let (Some(cur), Some(appl)) = (self.fan_curves.get(self.active_fan_idx), self.applied_fan_curves.get(self.active_fan_idx)) {
            cur != appl
        } else {
            false
        }
    }

    pub fn active_point_is_modified(&self, idx: usize) -> bool {
        if let (Some(cur), Some(appl)) = (self.fan_curves.get(self.active_fan_idx), self.applied_fan_curves.get(self.active_fan_idx)) {
            if let (Some(p_cur), Some(p_appl)) = (cur.points.get(idx), appl.points.get(idx)) {
                return p_cur != p_appl;
            }
        }
        false
    }

    pub fn repair_active_fan_curve(&mut self) -> bool {
        if let Some(curve) = self.fan_curves.get_mut(self.active_fan_idx) {
            let repaired = curve.repair();
            if repaired {
                self.set_status("Repaired curve to monotonic non-decreasing", false);
            } else {
                self.set_status("Curve is already monotonic", false);
            }
            repaired
        } else {
            false
        }
    }

    pub fn clear_expired_messages(&mut self) -> bool {
        let mut changed = false;
        if let Some((_, _, inst)) = &self.status_message {
            if inst.elapsed().as_secs() >= 4 {
                self.status_message = None;
                changed = true;
            }
        }
        if let Some((_, inst)) = &self.fan_clamp_hint {
            if inst.elapsed().as_secs() >= 4 {
                self.fan_clamp_hint = None;
                changed = true;
            }
        }
        changed
    }

    pub async fn refresh_sensors(&mut self) {
        if let Ok(s) = self.backend.read_sensors().await {
            if let Some(c) = s.cpu_temp_c {
                self.cpu_temp_history.push_back(c as u64);
                if self.cpu_temp_history.len() > 60 {
                    self.cpu_temp_history.pop_front();
                }
            }
            if let Some(g) = s.gpu_temp_c {
                self.gpu_temp_history.push_back(g as u64);
                if self.gpu_temp_history.len() > 60 {
                    self.gpu_temp_history.pop_front();
                }
            }
            self.sensors = s;
        }
    }

    pub async fn refresh_platform(&mut self) {
        if let Ok(p) = self.backend.get_platform_profile().await {
            self.platform_profile = p;
        }
        if let Ok(lim) = self.backend.get_charge_limit().await {
            self.charge_limit = lim;
        }
    }

    pub async fn refresh_gpu(&mut self) {
        if let Ok(m) = self.backend.get_gpu_mode().await {
            self.gpu_mode = m;
        }
        if let Ok(p) = self.backend.get_gpu_power().await {
            self.gpu_power = p;
        }
        if let Ok(a) = self.backend.get_gpu_pending_action().await {
            self.gpu_pending_action = a;
        }
        if let Ok(pm) = self.backend.get_gpu_pending_mode().await {
            self.gpu_pending_mode = pm;
        }
    }

    /// Compute the heatmap color from the higher of CPU/GPU temperatures.
    /// Gradient waypoints: 30C=blue, 60C=yellow, 80C=orange, 90C+=red.
    pub fn compute_heatmap_color(cpu_temp: Option<u32>, gpu_temp: Option<u32>) -> RgbColor {
        let temp = match (cpu_temp, gpu_temp) {
            (Some(c), Some(g)) => c.max(g),
            (Some(c), None) => c,
            (None, Some(g)) => g,
            (None, None) => return RgbColor::new(0, 0, 128), // no data: dim blue
        };

        struct HeatmapSegment {
            t_lo: u32,
            t_hi: u32,
            c_lo: (u8, u8, u8),
            c_hi: (u8, u8, u8),
        }

        let segments = [
            HeatmapSegment { t_lo: 0,  t_hi: 30,  c_lo: (0, 0, 128),   c_hi: (0, 0, 255) },   // cool: dark -> blue
            HeatmapSegment { t_lo: 30, t_hi: 60,  c_lo: (0, 0, 255),   c_hi: (255, 255, 0) }, // blue -> yellow
            HeatmapSegment { t_lo: 60, t_hi: 80,  c_lo: (255, 255, 0), c_hi: (255, 128, 0) }, // yellow -> orange
            HeatmapSegment { t_lo: 80, t_hi: 90,  c_lo: (255, 128, 0), c_hi: (255, 0, 0) },   // orange -> red
            HeatmapSegment { t_lo: 90, t_hi: 120, c_lo: (255, 0, 0),   c_hi: (255, 0, 0) },   // red (max)
        ];

        for s in &segments {
            if temp <= s.t_hi || s.t_hi == 120 {
                let t = temp.clamp(s.t_lo, s.t_hi);
                let span = (s.t_hi - s.t_lo) as f32;
                let frac = if span > 0.0 { (t - s.t_lo) as f32 / span } else { 0.0 };
                let lerp = |a: u8, b: u8| -> u8 {
                    (a as f32 + (b as f32 - a as f32) * frac).round() as u8
                };
                return RgbColor::new(
                    lerp(s.c_lo.0, s.c_hi.0),
                    lerp(s.c_lo.1, s.c_hi.1),
                    lerp(s.c_lo.2, s.c_hi.2),
                );
            }
        }
        RgbColor::new(255, 0, 0)
    }

    /// Map the supergfx GPU mode to a keyboard color.
    pub fn compute_gpu_mode_color(mode: GpuMode) -> RgbColor {
        match mode {
            GpuMode::Integrated => RgbColor::new(0, 200, 0),       // green: iGPU only, power-saver
            GpuMode::Hybrid     => RgbColor::new(200, 200, 0),     // yellow: PRIME offload ready
            GpuMode::AsusMuxDgpu => RgbColor::new(255, 80, 0),     // orange-red: direct MUX dGPU
            _                   => RgbColor::new(80, 80, 80),      // dim white: unknown
        }
    }

    /// Map battery charge percentage and status string to a keyboard color.
    pub fn compute_battery_color(pct: Option<u8>, status: &str) -> RgbColor {
        let charging = status.eq_ignore_ascii_case("charging")
            || status.eq_ignore_ascii_case("full");
        match pct {
            Some(_) if charging     => RgbColor::new(0, 220, 0),        // green: charging
            Some(p) if p < 20       => RgbColor::new(220, 0, 0),        // red: critically low
            Some(p) if p < 50       => RgbColor::new(220, 120, 0),      // orange: low
            Some(_)                 => RgbColor::new(0, 180, 120),      // teal-green: comfortable
            None                    => RgbColor::new(60, 60, 60),        // dim: no data
        }
    }

    /// Map CPU usage percent to color.
    pub fn compute_cpu_load_color(usage_pct: Option<u8>) -> RgbColor {
        match usage_pct {
            Some(u) if u < 30 => RgbColor::new(0, 220, 0),      // Green: cool idle/light (<30%)
            Some(u) if u < 65 => RgbColor::new(220, 220, 0),    // Yellow: moderate (30-65%)
            Some(u) if u < 85 => RgbColor::new(255, 120, 0),    // Orange: heavy (65-85%)
            Some(_)           => RgbColor::new(255, 0, 0),      // Red: peak load (>85%)
            None              => RgbColor::new(80, 80, 80),     // Dim: unknown
        }
    }

    /// Compute the next hue and color for Rainbow Wave.
    pub fn compute_rainbow_wave_color(&mut self) -> RgbColor {
        let speed_deg = match self.aura_data.speed.to_ascii_lowercase().as_str() {
            "low" => 15.0,
            "high" => 60.0,
            _ => 30.0,
        };
        let dir_mult = if self.aura_data.direction.eq_ignore_ascii_case("left") {
            -1.0
        } else {
            1.0
        };
        self.rainbow_wave_hue = (self.rainbow_wave_hue + dir_mult * speed_deg + 360.0) % 360.0;
        RgbColor::from_hsv(self.rainbow_wave_hue, 100.0, 100.0)
    }

    /// Compute the next step for Color Cycle (7 primary colors).
    pub fn compute_color_cycle_color(&mut self) -> RgbColor {
        const CYCLE_COLORS: [RgbColor; 7] = [
            RgbColor::new(255, 0, 0),     // Red
            RgbColor::new(255, 128, 0),   // Orange
            RgbColor::new(255, 255, 0),   // Yellow
            RgbColor::new(0, 255, 0),     // Green
            RgbColor::new(0, 255, 255),   // Cyan
            RgbColor::new(0, 0, 255),     // Blue
            RgbColor::new(255, 0, 255),   // Magenta
        ];
        let step = if self.aura_data.direction.eq_ignore_ascii_case("left") {
            6
        } else {
            1
        };
        self.color_cycle_idx = (self.color_cycle_idx + step) % CYCLE_COLORS.len();
        CYCLE_COLORS[self.color_cycle_idx]
    }

    /// Compute warm flickering fireplace ember color.
    pub fn compute_fire_flicker_color(&mut self) -> RgbColor {
        // LCG PRNG for organic flame variance
        self.flicker_seed = self.flicker_seed.wrapping_mul(1103515245).wrapping_add(12345);
        let rand1 = ((self.flicker_seed >> 16) & 0xFF) as u8;
        self.flicker_seed = self.flicker_seed.wrapping_mul(1103515245).wrapping_add(12345);
        let rand2 = ((self.flicker_seed >> 16) & 0xFF) as u8;

        let r = 220 + (rand1 % 36);
        let g = 40 + (rand2 % 71);
        let b = rand1 % 16;
        RgbColor::new(r, g, b)
    }

    /// Compute high-contrast strobe pulse.
    pub fn compute_strobe_color(&mut self) -> RgbColor {
        self.strobe_state = !self.strobe_state;
        if self.strobe_state {
            if self.aura_data.color1.r == 0 && self.aura_data.color1.g == 0 && self.aura_data.color1.b == 0 {
                RgbColor::new(255, 255, 255)
            } else {
                self.aura_data.color1
            }
        } else {
            RgbColor::new(0, 0, 0)
        }
    }

    /// Sample a 100ms slice of microphone input audio via parec from @DEFAULT_SOURCE@
    /// (corresponds to clean ghelper-audio / built-in microphone).
    pub fn sample_mic_level() -> u8 {
        use std::io::Read;
        let mut child = match std::process::Command::new("parec")
            .args(["--channels=1", "--rate=8000", "--format=u8", "-d", "@DEFAULT_SOURCE@"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
        {
            Ok(c) => c,
            Err(_) => return 0,
        };

        let mut buf = [128u8; 800];
        let mut bytes_read = 0;
        if let Some(mut stdout) = child.stdout.take() {
            while bytes_read < 800 {
                match stdout.read(&mut buf[bytes_read..]) {
                    Ok(0) => break,
                    Ok(n) => bytes_read += n,
                    Err(_) => break,
                }
            }
        }
        let _ = child.kill();
        let _ = child.wait();

        if bytes_read == 0 {
            return 0;
        }
        let peak = buf[..bytes_read]
            .iter()
            .map(|&b| (b as i16 - 128).abs())
            .max()
            .unwrap_or(0);
        ((peak as u32 * 100) / 127).min(100) as u8
    }

    /// Sample a 100ms slice of audio playback output via parec from @DEFAULT_MONITOR@.
    pub fn sample_audio_out_level() -> u8 {
        use std::io::Read;
        let mut child = match std::process::Command::new("parec")
            .args(["--channels=1", "--rate=8000", "--format=u8", "-d", "@DEFAULT_MONITOR@"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
        {
            Ok(c) => c,
            Err(_) => return 0,
        };

        let mut buf = [128u8; 800];
        let mut bytes_read = 0;
        if let Some(mut stdout) = child.stdout.take() {
            while bytes_read < 800 {
                match stdout.read(&mut buf[bytes_read..]) {
                    Ok(0) => break,
                    Ok(n) => bytes_read += n,
                    Err(_) => break,
                }
            }
        }
        let _ = child.kill();
        let _ = child.wait();

        if bytes_read == 0 {
            return 0;
        }
        let peak = buf[..bytes_read]
            .iter()
            .map(|&b| (b as i16 - 128).abs())
            .max()
            .unwrap_or(0);
        ((peak as u32 * 100) / 127).min(100) as u8
    }

    /// Sample real-time center pixel of the active Wayland screen via grim.
    pub fn sample_wayland_screen_color() -> Option<RgbColor> {
        let output = std::process::Command::new("grim")
            .args(["-g", "960,600 1x1", "-t", "ppm", "-"])
            .output()
            .ok()?;
        if output.status.success() && output.stdout.len() >= 3 {
            let len = output.stdout.len();
            let r = output.stdout[len - 3];
            let g = output.stdout[len - 2];
            let b = output.stdout[len - 1];
            Some(RgbColor::new(r, g, b))
        } else {
            None
        }
    }

    /// Voice Pulse: Speech reactive using clean microphone (ghelper-audio).
    /// G-Helper style mic sync: lights up brightly when speaking, rests when silent.
    pub fn compute_voice_pulse_color(&mut self) -> RgbColor {
        let level = Self::sample_mic_level();
        self.mic_level = level;
        let c = self.aura_data.color1;
        // Ambient room noise gate: <= 8% is resting silence
        if level <= 8 {
            RgbColor::new(
                (c.r as f32 * 0.15) as u8,
                (c.g as f32 * 0.15) as u8,
                (c.b as f32 * 0.15) as u8,
            )
        } else {
            let mult = 0.20 + 0.80 * ((level - 8) as f32 / 92.0);
            RgbColor::new(
                (c.r as f32 * mult).min(255.0) as u8,
                (c.g as f32 * mult).min(255.0) as u8,
                (c.b as f32 * mult).min(255.0) as u8,
            )
        }
    }

    /// Audio Pulse: Sound output reactive visualizer.
    /// Pulses keyboard illumination with music beat drops and audio volume.
    pub fn compute_audio_pulse_color(&mut self) -> RgbColor {
        let level = Self::sample_audio_out_level();
        self.audio_level = level;
        let c = self.aura_data.color1;
        if level <= 5 {
            RgbColor::new(
                (c.r as f32 * 0.15) as u8,
                (c.g as f32 * 0.15) as u8,
                (c.b as f32 * 0.15) as u8,
            )
        } else {
            let mult = 0.20 + 0.80 * (level as f32 / 100.0);
            RgbColor::new(
                (c.r as f32 * mult).min(255.0) as u8,
                (c.g as f32 * mult).min(255.0) as u8,
                (c.b as f32 * mult).min(255.0) as u8,
            )
        }
    }

    /// Audio Wave: Dynamic audio spectrum EQ.
    /// Shifts hue across the color spectrum based on playback intensity.
    pub fn compute_audio_wave_color(&mut self) -> RgbColor {
        let level = Self::sample_audio_out_level();
        self.audio_level = level;
        let hue = if level < 15 {
            240.0 // Calm deep blue
        } else if level < 40 {
            180.0 // Cyan / teal
        } else if level < 70 {
            100.0 // Lime green / yellow
        } else if level < 85 {
            40.0  // Warm orange
        } else {
            0.0   // Peak red
        };
        let val = 30.0 + 70.0 * (level as f32 / 100.0);
        RgbColor::from_hsv(hue, 100.0, val)
    }

    /// Ambient Light: Real-time screen ambient sync on Wayland.
    pub fn compute_ambient_color(&mut self) -> RgbColor {
        if let Some(col) = Self::sample_wayland_screen_color() {
            self.last_ambient_color = Some(col);
            col
        } else if let Some(last) = self.last_ambient_color {
            last
        } else {
            self.aura_data.color1
        }
    }

    /// Apply the active software effect to the keyboard via asusd DBus.
    /// No-op if brightness is Off (0) or effect is None.
    /// Only writes to hardware when the computed color changes (for telemetry effects)
    /// or on every tick for animated/audio effects.
    pub async fn apply_software_effect(&mut self) {
        if self.aura_brightness == 0 || self.active_software_effect == SoftwareEffect::None {
            return;
        }

        let is_animated = self.active_software_effect.is_animated();
        let color = match self.active_software_effect {
            SoftwareEffect::RainbowWave => self.compute_rainbow_wave_color(),
            SoftwareEffect::ColorCycle => self.compute_color_cycle_color(),
            SoftwareEffect::FireFlicker => self.compute_fire_flicker_color(),
            SoftwareEffect::Strobe => self.compute_strobe_color(),
            SoftwareEffect::VoicePulse => self.compute_voice_pulse_color(),
            SoftwareEffect::AudioPulse => self.compute_audio_pulse_color(),
            SoftwareEffect::AudioWave => self.compute_audio_wave_color(),
            SoftwareEffect::AmbientLight => self.compute_ambient_color(),
            SoftwareEffect::Heatmap => {
                Self::compute_heatmap_color(self.sensors.cpu_temp_c, self.sensors.gpu_temp_c)
            }
            SoftwareEffect::GpuMode => {
                Self::compute_gpu_mode_color(self.gpu_mode)
            }
            SoftwareEffect::BatteryStatus => {
                Self::compute_battery_color(
                    self.sensors.battery_charge_percent,
                    &self.sensors.battery_status.clone(),
                )
            }
            SoftwareEffect::CpuLoad => {
                Self::compute_cpu_load_color(self.sensors.cpu_usage_percent)
            }
            SoftwareEffect::None => return,
        };

        // Deduplicate telemetry effects: skip DBus write if color has not changed
        if !is_animated && self.last_effect_color == Some(color) {
            return;
        }
        self.last_effect_color = Some(color);

        // Ensure Static mode (0) is the active firmware mode so the color shows
        if self.aura_mode != 0 {
            let _ = self.backend.set_aura_mode(0).await;
            self.aura_mode = 0;
            self.aura_data.mode = 0;
        }

        self.aura_data.color1 = color;
        let _ = self.backend.set_aura_mode_data(&self.aura_data).await;
    }

    /// Toggle metrics logging to CSV file on/off.
    pub fn toggle_metrics_logging(&mut self) {
        if self.is_logging_metrics {
            self.is_logging_metrics = false;
            let count = self.metrics_logged_count;
            self.set_status(format!("Metrics logging stopped ({count} records written to {})", self.metrics_log_path.display()), false);
        } else {
            self.is_logging_metrics = true;
            self.metrics_logged_count = 0;
            if let Some(parent) = self.metrics_log_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let needs_header = !self.metrics_log_path.exists()
                || std::fs::metadata(&self.metrics_log_path).map(|m| m.len() == 0).unwrap_or(true);
            if needs_header {
                let header = "timestamp,cpu_model,cpu_temp_c,gpu_temp_c,cpu_usage_pct,cpu_freq_mhz,cpu_fan_rpm,gpu_fan_rpm,ram_used_mb,ram_total_mb,dgpu_status,dgpu_power_state,battery_pct,battery_w,battery_status\n";
                let _ = std::fs::write(&self.metrics_log_path, header);
            }
            let path_str = self.metrics_log_path.display().to_string();
            self.set_status(format!("Metrics logging started -> {path_str}"), false);
        }
    }

    /// Record one row of telemetry metrics to the CSV file.
    pub fn log_metrics_sample(&mut self) {
        if !self.is_logging_metrics {
            return;
        }
        use std::io::Write;
        let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.metrics_log_path)
        else {
            return;
        };

        // Format ISO-style timestamp using libc
        let mut tm: libc::tm = unsafe { std::mem::zeroed() };
        let now_t = unsafe { libc::time(std::ptr::null_mut()) };
        unsafe { libc::localtime_r(&now_t, &mut tm) };
        let timestamp = format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            tm.tm_year + 1900,
            tm.tm_mon + 1,
            tm.tm_mday,
            tm.tm_hour,
            tm.tm_min,
            tm.tm_sec
        );

        let cpu_model_clean = self.sensors.cpu_model.replace(',', " ");
        let cpu_temp = self.sensors.cpu_temp_c.map(|t| t.to_string()).unwrap_or_default();
        let gpu_temp = self.sensors.gpu_temp_c.map(|t| t.to_string()).unwrap_or_default();
        let cpu_usage = self.sensors.cpu_usage_percent.map(|u| u.to_string()).unwrap_or_default();
        let cpu_freq = self.sensors.cpu_freq_mhz.map(|f| f.to_string()).unwrap_or_default();
        let cpu_fan = self.sensors.cpu_fan_rpm.map(|r| r.to_string()).unwrap_or_default();
        let gpu_fan = self.sensors.gpu_fan_rpm.map(|r| r.to_string()).unwrap_or_default();
        let ram_used = self.sensors.ram_used_mb.map(|m| m.to_string()).unwrap_or_default();
        let ram_total = self.sensors.ram_total_mb.map(|m| m.to_string()).unwrap_or_default();
        let dgpu_status = self.sensors.dgpu_status.replace(',', " ");
        let dgpu_power = self.sensors.dgpu_power_state.replace(',', " ");
        let bat_pct = self.sensors.battery_charge_percent.map(|b| b.to_string()).unwrap_or_default();
        let bat_w = self.sensors.battery_power_w.map(|w| format!("{w:.2}")).unwrap_or_default();
        let bat_status = self.sensors.battery_status.replace(',', " ");

        let line = format!(
            "{timestamp},{cpu_model_clean},{cpu_temp},{gpu_temp},{cpu_usage},{cpu_freq},{cpu_fan},{gpu_fan},{ram_used},{ram_total},{dgpu_status},{dgpu_power},{bat_pct},{bat_w},{bat_status}\n"
        );

        if file.write_all(line.as_bytes()).is_ok() {
            self.metrics_logged_count += 1;
        }
    }
}
