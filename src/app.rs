use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
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
    System = 4,
}

impl ActiveTab {
    pub fn all() -> &'static [ActiveTab] {
        &[
            ActiveTab::Dashboard,
            ActiveTab::Power,
            ActiveTab::FanCurves,
            ActiveTab::Aura,
            ActiveTab::System,
        ]
    }

    pub fn title(self) -> &'static str {
        match self {
            ActiveTab::Dashboard => "Dash",
            ActiveTab::Power => "Power",
            ActiveTab::FanCurves => "Fan",
            ActiveTab::Aura => "RGB",
            ActiveTab::System => "System",
        }
    }

    pub fn next(self) -> Self {
        match self {
            ActiveTab::Dashboard => ActiveTab::Power,
            ActiveTab::Power => ActiveTab::FanCurves,
            ActiveTab::FanCurves => ActiveTab::Aura,
            ActiveTab::Aura => ActiveTab::System,
            ActiveTab::System => ActiveTab::Dashboard,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            ActiveTab::Dashboard => ActiveTab::System,
            ActiveTab::Power => ActiveTab::Dashboard,
            ActiveTab::FanCurves => ActiveTab::Power,
            ActiveTab::Aura => ActiveTab::FanCurves,
            ActiveTab::System => ActiveTab::Aura,
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

pub struct AudioCaptureService {
    running: Arc<AtomicBool>,
    mic_level: Arc<AtomicU8>,
    audio_level: Arc<AtomicU8>,
    thread_handle: Option<JoinHandle<()>>,
    current_source: Option<String>,
}

impl Default for AudioCaptureService {
    fn default() -> Self {
        Self::new()
    }
}

impl AudioCaptureService {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            mic_level: Arc::new(AtomicU8::new(0)),
            audio_level: Arc::new(AtomicU8::new(0)),
            thread_handle: None,
            current_source: None,
        }
    }

    pub fn start(&mut self, source: &str) {
        if self.current_source.as_deref() == Some(source) && self.running.load(Ordering::Relaxed) {
            return;
        }
        self.stop();

        self.running.store(true, Ordering::Relaxed);
        let running_clone = self.running.clone();
        let is_mic = source.contains("SOURCE")
            || source.contains("input")
            || source.contains("mic")
            || source.contains("ghelper");
        let output_atomic = if is_mic {
            self.mic_level.clone()
        } else {
            self.audio_level.clone()
        };
        let src_str = source.to_string();

        let handle = std::thread::spawn(move || {
            let mut child = match std::process::Command::new("parec")
                .args([
                    "--raw",
                    "--latency-msec=25",
                    "--channels=1",
                    "--rate=8000",
                    "--format=s16le",
                    "-d",
                    &src_str,
                ])
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::null())
                .spawn()
            {
                Ok(c) => c,
                Err(_) => return,
            };

            let mut stdout = match child.stdout.take() {
                Some(s) => s,
                None => {
                    let _ = child.kill();
                    return;
                }
            };

            use std::io::Read;
            let mut buf = [0u8; 400]; // 200 samples (25ms)
            let mut rolling_peak: f32 = if is_mic { 4000.0 } else { 800.0 };
            let mut noise_baseline: f32 = if is_mic { 900.0 } else { 150.0 };
            let mut decay_brightness: f32 = 0.0;

            while running_clone.load(Ordering::Relaxed) {
                let mut read_bytes = 0;
                while read_bytes < buf.len() && running_clone.load(Ordering::Relaxed) {
                    match stdout.read(&mut buf[read_bytes..]) {
                        Ok(0) => break,
                        Ok(n) => read_bytes += n,
                        Err(_) => break,
                    }
                }
                if read_bytes < 2 {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    continue;
                }

                let num_samples = read_bytes / 2;
                let mut max_amp: u32 = 0;
                for i in 0..num_samples {
                    let sample = i16::from_le_bytes([buf[i * 2], buf[i * 2 + 1]]);
                    let amp = sample.unsigned_abs() as u32;
                    if amp > max_amp {
                        max_amp = amp;
                    }
                }

                let current_f = max_amp as f32;
                let raw_ratio = if is_mic {
                    // Adaptively track ambient noise floor in quiet room so fan/room hiss never triggers pulses
                    if current_f < noise_baseline * 1.5 {
                        noise_baseline = noise_baseline * 0.96 + current_f * 0.04;
                        noise_baseline = noise_baseline.clamp(400.0, 2200.0);
                    }
                    let speech_gate = (noise_baseline * 1.7).max(1200.0);
                    if current_f <= speech_gate {
                        0.0
                    } else {
                        rolling_peak = (rolling_peak * 0.95).max(current_f).max(4500.0);
                        ((current_f - speech_gate) / (rolling_peak - speech_gate)).clamp(0.0, 1.0)
                    }
                } else {
                    rolling_peak = (rolling_peak * 0.96).max(current_f).max(800.0);
                    let noise_floor = 150.0;
                    if current_f <= noise_floor {
                        0.0
                    } else {
                        ((current_f - noise_floor) / (rolling_peak - noise_floor)).clamp(0.0, 1.0)
                    }
                };

                let decay_rate = if is_mic { 0.65 } else { 0.72 };
                decay_brightness = (decay_brightness * decay_rate).max(raw_ratio);
                if decay_brightness < 0.03 {
                    decay_brightness = 0.0;
                }

                let curved = decay_brightness * decay_brightness * decay_brightness;
                let level = (curved * 100.0).round().clamp(0.0, 100.0) as u8;

                output_atomic.store(level, Ordering::Relaxed);
            }

            let _ = child.kill();
            let _ = child.wait();
        });

        self.thread_handle = Some(handle);
        self.current_source = Some(source.to_string());
    }

    /// Automatically find the cleanest microphone available on the system.
    /// Priority: ghelper-audio -> clean/clean-fx -> hush/hushmic -> rnnoise/filter -> default analog mic.
    pub fn find_preferred_mic_source() -> String {
        if let Ok(output) = std::process::Command::new("pactl")
            .args(["list", "short", "sources"])
            .output()
        {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                let mut candidates: Vec<String> = Vec::new();
                for line in text.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let name = parts[1];
                        if !name.ends_with(".monitor") && !name.contains("bluez_input") {
                            candidates.push(name.to_string());
                        }
                    }
                }
                // Priority 1: ghelper-audio (single persistent virtual mic)
                if let Some(src) = candidates.iter().find(|s| s.contains("ghelper-audio")) {
                    return src.clone();
                }
                // Priority 2: clean-fx / clean
                if let Some(src) = candidates.iter().find(|s| s.contains("clean")) {
                    return src.clone();
                }
                // Priority 3: hush / hushmic
                if let Some(src) = candidates.iter().find(|s| s.contains("hush")) {
                    return src.clone();
                }
                // Priority 4: rnnoise / noisetorch / deepfilter
                if let Some(src) = candidates.iter().find(|s| {
                    s.contains("rnnoise")
                        || s.contains("noisetorch")
                        || s.contains("deepfilter")
                        || s.contains("filter")
                }) {
                    return src.clone();
                }
                // Priority 5: analog mic input
                if let Some(src) = candidates
                    .iter()
                    .find(|s| s.contains("analog-stereo") || s.contains("input"))
                {
                    return src.clone();
                }
            }
        }
        "@DEFAULT_SOURCE@".to_string()
    }

    pub fn stop(&mut self) {
        if self.running.load(Ordering::Relaxed) {
            self.running.store(false, Ordering::Relaxed);
            if let Some(h) = self.thread_handle.take() {
                let _ = h.join();
            }
        }
        self.current_source = None;
        self.mic_level.store(0, Ordering::Relaxed);
        self.audio_level.store(0, Ordering::Relaxed);
    }

    pub fn get_mic_level(&self) -> u8 {
        self.mic_level.load(Ordering::Relaxed)
    }

    pub fn get_audio_level(&self) -> u8 {
        self.audio_level.load(Ordering::Relaxed)
    }
}

impl Drop for AudioCaptureService {
    fn drop(&mut self) {
        self.stop();
    }
}

pub struct AmbientCaptureService {
    running: Arc<AtomicBool>,
    latest_color: Arc<Mutex<Option<RgbColor>>>,
    thread_handle: Option<JoinHandle<()>>,
}

impl Default for AmbientCaptureService {
    fn default() -> Self {
        Self::new()
    }
}

impl AmbientCaptureService {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            latest_color: Arc::new(Mutex::new(None)),
            thread_handle: None,
        }
    }

    pub fn start(&mut self) {
        if self.running.load(Ordering::Relaxed) {
            return;
        }
        self.running.store(true, Ordering::Relaxed);
        let running_clone = self.running.clone();
        let color_arc = self.latest_color.clone();

        let handle = std::thread::spawn(move || {
            let mut smoothed: Option<RgbColor> = None;

            while running_clone.load(Ordering::Relaxed) {
                // G-Helper bounds: crop top 1/3 and bottom panel
                // grim -g "0,400 1920x770" -s 0.05 -t ppm -
                let output = std::process::Command::new("grim")
                    .args(["-g", "0,400 1920x770", "-s", "0.05", "-t", "ppm", "-"])
                    .output();

                if let Ok(out) = output {
                    if out.status.success() && out.stdout.len() >= 15 {
                        if let Some(raw_avg) = parse_ppm_dominant(&out.stdout) {
                            // G-Helper UpSaturation (+0.3)
                            let saturated = up_saturation(raw_avg, 0.3);
                            // G-Helper exponential smooth (0.65)
                            let final_color = match smoothed {
                                Some(prev) => {
                                    let s = smooth_color(saturated, prev, 0.65);
                                    smoothed = Some(s);
                                    s
                                }
                                None => {
                                    smoothed = Some(saturated);
                                    saturated
                                }
                            };
                            if let Ok(mut lock) = color_arc.lock() {
                                *lock = Some(final_color);
                            }
                        }
                    }
                }

                std::thread::sleep(std::time::Duration::from_millis(160));
            }
        });

        self.thread_handle = Some(handle);
    }

    pub fn stop(&mut self) {
        if self.running.load(Ordering::Relaxed) {
            self.running.store(false, Ordering::Relaxed);
            if let Some(h) = self.thread_handle.take() {
                let _ = h.join();
            }
        }
    }

    pub fn get_color(&self) -> Option<RgbColor> {
        self.latest_color.lock().ok().and_then(|c| *c)
    }
}

impl Drop for AmbientCaptureService {
    fn drop(&mut self) {
        self.stop();
    }
}

fn parse_ppm_dominant(data: &[u8]) -> Option<RgbColor> {
    if !data.starts_with(b"P6") {
        return None;
    }
    let mut idx = 2;
    let mut tokens = 0;
    while idx < data.len() && tokens < 3 {
        while idx < data.len() && data[idx].is_ascii_whitespace() {
            idx += 1;
        }
        while idx < data.len() && !data[idx].is_ascii_whitespace() {
            idx += 1;
        }
        tokens += 1;
    }
    if idx < data.len() && data[idx].is_ascii_whitespace() {
        idx += 1;
    }
    let pixel_data = &data[idx..];
    let num_pixels = pixel_data.len() / 3;
    if num_pixels == 0 {
        return None;
    }
    let mut sum_r: u64 = 0;
    let mut sum_g: u64 = 0;
    let mut sum_b: u64 = 0;
    let (chunks, _) = pixel_data.as_chunks::<3>();
    for &[r, g, b] in chunks {
        sum_r += r as u64;
        sum_g += g as u64;
        sum_b += b as u64;
    }
    let avg_r = (sum_r / num_pixels as u64) as u8;
    let avg_g = (sum_g / num_pixels as u64) as u8;
    let avg_b = (sum_b / num_pixels as u64) as u8;
    Some(RgbColor::new(avg_r, avg_g, avg_b))
}

fn up_saturation(color: RgbColor, boost: f32) -> RgbColor {
    if color.r == color.g && color.g == color.b {
        return color;
    }
    let (h, s, v) = color.to_hsv();
    let new_s = (s + boost * 100.0).clamp(0.0, 100.0);
    RgbColor::from_hsv(h, new_s, v)
}

fn smooth_color(target: RgbColor, prev: RgbColor, smooth: f32) -> RgbColor {
    let factor = smooth.clamp(0.0, 1.0);
    let inv = 1.0 - factor;
    let r = (target.r as f32 * inv + prev.r as f32 * factor).round() as u8;
    let g = (target.g as f32 * inv + prev.g as f32 * factor).round() as u8;
    let b = (target.b as f32 * inv + prev.b as f32 * factor).round() as u8;
    RgbColor::new(r, g, b)
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
    pub fan_selected_col: usize, // 0 = Fan Speed (PWM %), 1 = Temperature (°C)

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
    pub audio_level_smooth: f32,
    pub audio_wave_hue: f32,
    pub last_ambient_color: Option<RgbColor>,
    pub clouds_phase: f32,
    pub theme_sync_phase: f32,
    pub audio_service: AudioCaptureService,
    pub ambient_service: AmbientCaptureService,

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

    // System Tab field index
    pub system_selected_row: usize,

    // Metrics Logging State (Page 1 toggleable telemetry recorder)
    pub is_logging_metrics: bool,
    pub metrics_log_path: std::path::PathBuf,
    pub metrics_logged_count: usize,

    // Polish additions
    pub saved_software_effect: Option<SoftwareEffect>,
    pub user_custom_color: bool,
    pub color_cycle_progress: f32,
    pub is_full_charge_active: bool,
    pub last_theme_mtime: Option<std::time::SystemTime>,
    pub last_ac_online: Option<bool>,
    pub has_slash_led: bool,
    pub slash_mode: String,
    pub slash_brightness: u32,
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

        let has_slash_led = backend.get_slash_supported().await.unwrap_or(false);
        let slash_mode = if has_slash_led {
            backend.get_slash_mode().await.unwrap_or_else(|_| "Static".to_string())
        } else {
            "Static".to_string()
        };
        let slash_brightness = if has_slash_led {
            backend.get_slash_brightness().await.unwrap_or(2)
        } else {
            2
        };
        let last_ac_online = Some(crate::backend::real::is_ac_online());

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
            fan_selected_col: 0,
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
            audio_level_smooth: 0.0,
            audio_wave_hue: 240.0,
            last_ambient_color: None,
            clouds_phase: 0.0,
            theme_sync_phase: 0.0,
            audio_service: AudioCaptureService::new(),
            ambient_service: AmbientCaptureService::new(),
            armoury_attributes,
            armoury_selected_idx: 0,
            show_unavailable_armoury: false,
            sensors,
            cpu_temp_history,
            gpu_temp_history,
            power_selected_field: 0,
            system_selected_row: 0,
            is_logging_metrics: false,
            metrics_log_path: if let Ok(home) = std::env::var("HOME") {
                std::path::PathBuf::from(home).join(".local/share/rogtui/metrics.csv")
            } else {
                std::path::PathBuf::from("/tmp/rogtui-metrics.csv")
            },
            metrics_logged_count: 0,
            saved_software_effect: None,
            user_custom_color: false,
            color_cycle_progress: 0.0,
            is_full_charge_active: false,
            last_theme_mtime: None,
            last_ac_online,
            has_slash_led,
            slash_mode,
            slash_brightness,
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

    pub fn get_active_hint(&self) -> String {
        match self.current_tab {
            ActiveTab::Dashboard => {
                "Platform profiles balance CPU/GPU wattage, acoustic limits, and battery life.".to_string()
            }
            ActiveTab::Power => {
                match self.power_selected_field {
                    0 => "Sets system performance, thermal thresholds, and fan curves.".to_string(),
                    1 => "Applies when the charger is connected or removed.".to_string(),
                    2 => "Applies when the charger is connected or removed.".to_string(),
                    3 => if self.charge_limit == 100 {
                        "Warning: 100% maximum charge accelerates battery wear.".to_string()
                    } else {
                        "80% or lower extends battery lifespan by reducing cell degradation.".to_string()
                    },
                    4 => if self.is_full_charge_active {
                        "Active, ends when unplugged.".to_string()
                    } else {
                        "Temporarily charge to 100% for travel; resets automatically on disconnect.".to_string()
                    },
                    _ => String::new(),
                }
            }
            ActiveTab::FanCurves => {
                if self.fan_selected_col == 0 {
                    "j/k selects point, h/l adjusts speed, Space toggles temp/speed, w saves.".to_string()
                } else {
                    "j/k selects point, h/l adjusts temp, Space toggles temp/speed, w saves.".to_string()
                }
            }
            ActiveTab::Aura => {
                match self.aura_selected_field {
                    0 => "Adjusts keyboard LED backlight intensity (Off, Low, Med, High).".to_string(),
                    1 => match crate::ui::aura::get_current_mode_index(self) {
                        0 => "Single solid illumination across all keyboard zones.",
                        1 => "Sine-wave brightness breathing between primary and secondary colors.",
                        2 => "Hardware-timed spectrum wave traversing the keyboard matrix.",
                        3 => "Periodic quick flash pulses.",
                        4 => "Smooth continuous 360° spectrum sweep across the keyboard.",
                        5 => "Sequential smooth crossfade transitions through 7 primary colors.",
                        6 => "Organic fireplace flame simulation with shifting amber and red embers.",
                        7 => "Smooth atmospheric drift between soft cyan, sky blue, and cloud white.",
                        8 => "Synchronizes keyboard lighting to active desktop theme from colors.sh.",
                        9 => "High-contrast blinking pulse.",
                        10 => "Speech reactive illumination via internal laptop microphone.",
                        11 => "Real-time system audio playback reactive pulse.",
                        12 => "Multi-harmonic frequency spectrum lighting.",
                        13 => "Wayland display ambient color sync via grim.",
                        14 => "Thermal color gradient reflecting CPU and GPU temperatures.",
                        15 => "Reflects active GPU operating mode in color.",
                        16 => "Visual state-of-charge and charging indicator.",
                        17 => "Visual workload percentage monitor.",
                        _ => "Lighting mode.",
                    }.to_string(),
                    2 => "Primary keyboard backlight color. Press c for 24-bit picker, # to enter hex.".to_string(),
                    3 => "Adjusts Red color channel (0..255).".to_string(),
                    4 => "Adjusts Green color channel (0..255).".to_string(),
                    5 => "Adjusts Blue color channel (0..255).".to_string(),
                    6 => "Secondary breathing color.".to_string(),
                    7 => "Animation speed (Low, Med, High).".to_string(),
                    8 => "Animation traversal direction (Left, Right).".to_string(),
                    9 => "Keep keyboard lit during system boot sequence.".to_string(),
                    10 => "Keep keyboard lit while system is running.".to_string(),
                    11 => "Keep keyboard lit during system suspend.".to_string(),
                    12 => "Keep keyboard lit during system shutdown.".to_string(),
                    13 => "Slash lighting animation mode for Zephyrus lid LED bar.".to_string(),
                    14 => "Slash lighting intensity (Off, Low, Med, High).".to_string(),
                    _ => String::new(),
                }
            }
            ActiveTab::System => {
                let has_auto_bright = self.sensors.screen_auto_brightness.is_some();
                let export_row = if has_auto_bright { 2 } else { 1 };
                if self.system_selected_row == 0 {
                    "Faster pixel response; can cause ghosting in some content.".to_string()
                } else if self.system_selected_row == 1 && has_auto_bright {
                    "Adjusts screen brightness to ambient room light.".to_string()
                } else if self.system_selected_row == export_row {
                    "Exports full hardware and system diagnostics to ~/.local/state/rogtui/report-<timestamp>.md.".to_string()
                } else {
                    String::new()
                }
            }
        }
    }

    pub fn check_and_reload_system_theme(&mut self) -> bool {
        let Ok(home) = std::env::var("HOME") else { return false; };
        let path = std::path::PathBuf::from(home).join(".local/state/theme/colors.sh");
        let Ok(meta) = std::fs::metadata(&path) else { return false; };
        let Ok(mtime) = meta.modified() else { return false; };

        if self.last_theme_mtime != Some(mtime) {
            self.last_theme_mtime = Some(mtime);
            let (_name, theme_rgb) = Theme::read_system_theme();
            self.theme.accent = ratatui::style::Color::Rgb(theme_rgb.r, theme_rgb.g, theme_rgb.b);
            return true;
        }
        false
    }

    pub async fn check_and_apply_power_state_transition(&mut self) -> bool {
        let current_ac = crate::backend::real::is_ac_online();
        let mut changed = false;

        if let Some(prev_ac) = self.last_ac_online {
            if prev_ac != current_ac {
                self.last_ac_online = Some(current_ac);
                if current_ac {
                    // Plugged in to AC adapter
                    if self.auto_ac && self.platform_profile != self.profile_ac {
                        let _ = self.backend.set_platform_profile(self.profile_ac).await;
                        self.platform_profile = self.profile_ac;
                        self.set_status(format!("Switched to {} profile (AC Power)", self.profile_ac.as_str()), false);
                        changed = true;
                    }
                } else {
                    // Running on battery
                    if self.auto_battery && self.platform_profile != self.profile_battery {
                        let _ = self.backend.set_platform_profile(self.profile_battery).await;
                        self.platform_profile = self.profile_battery;
                        self.set_status(format!("Switched to {} profile (Battery)", self.profile_battery.as_str()), false);
                        changed = true;
                    }
                    // Auto-reset travel mode / full charge oneshot when unplugged
                    if self.is_full_charge_active {
                        self.is_full_charge_active = false;
                        let _ = self.backend.set_charge_limit(self.charge_limit).await;
                        changed = true;
                    }
                }
            }
        } else {
            self.last_ac_online = Some(current_ac);
        }

        changed
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

    /// Compute the next step for Color Cycle (7 primary colors) with smooth crossfade and speed control.
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

        let step = match self.aura_data.speed.to_ascii_lowercase().as_str() {
            "low" => 0.02,
            "high" => 0.10,
            _ => 0.05,
        };

        if self.aura_data.direction.eq_ignore_ascii_case("left") {
            self.color_cycle_progress = (self.color_cycle_progress - step + 7.0) % 7.0;
        } else {
            self.color_cycle_progress = (self.color_cycle_progress + step) % 7.0;
        }

        let idx1 = self.color_cycle_progress.floor() as usize % 7;
        let idx2 = (idx1 + 1) % 7;
        let frac = self.color_cycle_progress.fract();

        let c1 = CYCLE_COLORS[idx1];
        let c2 = CYCLE_COLORS[idx2];

        RgbColor::new(
            (c1.r as f32 * (1.0 - frac) + c2.r as f32 * frac).round() as u8,
            (c1.g as f32 * (1.0 - frac) + c2.g as f32 * frac).round() as u8,
            (c1.b as f32 * (1.0 - frac) + c2.b as f32 * frac).round() as u8,
        )
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
                RgbColor::new(0, 255, 255)
            } else {
                self.aura_data.color1
            }
        } else {
            RgbColor::new(0, 0, 0)
        }
    }

    /// Clouds: Smooth harmonic atmospheric drift between soft cyan (#00E5FF),
    /// sky blue (#4FC3F7), and misty cloud white (#E0F7FA).
    pub fn compute_clouds_color(&mut self) -> RgbColor {
        self.clouds_phase = (self.clouds_phase + 0.05) % (2.0 * std::f32::consts::PI);
        let wave1 = (self.clouds_phase.sin() + 1.0) * 0.5;
        let wave2 = ((self.clouds_phase * 0.7).cos() + 1.0) * 0.5;
        let blend = (wave1 * 0.6 + wave2 * 0.4).clamp(0.0, 1.0);

        if blend < 0.7 {
            let t = blend / 0.7;
            let r = (0.0 * (1.0 - t) + 79.0 * t) as u8;
            let g = (229.0 * (1.0 - t) + 195.0 * t) as u8;
            let b = (255.0 * (1.0 - t) + 247.0 * t) as u8;
            RgbColor::new(r, g, b)
        } else {
            let t = (blend - 0.7) / 0.3;
            let r = (79.0 * (1.0 - t) + 224.0 * t) as u8;
            let g = (195.0 * (1.0 - t) + 247.0 * t) as u8;
            let b = (247.0 * (1.0 - t) + 250.0 * t) as u8;
            RgbColor::new(r, g, b)
        }
    }

    /// Theme Sync: Dynamically synchronizes keyboard lighting to the active desktop theme
    /// from ~/.local/state/theme/colors.sh with a subtle breathing pulsation.
    pub fn compute_theme_sync_color(&mut self) -> RgbColor {
        let (_, base_col) = crate::ui::theme::Theme::read_system_theme();
        self.theme_sync_phase = (self.theme_sync_phase + 0.08) % (2.0 * std::f32::consts::PI);
        let intensity = 0.85 + 0.15 * ((self.theme_sync_phase.sin() + 1.0) * 0.5);
        RgbColor::new(
            (base_col.r as f32 * intensity).min(255.0) as u8,
            (base_col.g as f32 * intensity).min(255.0) as u8,
            (base_col.b as f32 * intensity).min(255.0) as u8,
        )
    }

    /// Voice Pulse: Speech reactive using clean microphone (ghelper-audio) via non-blocking worker.
    /// Priority: Defaults to active system desktop theme color; overridden by user-specified custom color.
    /// At brightness 1 (Low), pulses between 0 (off) and 1 (lit) on speech sound.
    pub fn compute_voice_pulse_color(&mut self) -> RgbColor {
        let mic_source = AudioCaptureService::find_preferred_mic_source();
        self.audio_service.start(&mic_source);
        let level = self.audio_service.get_mic_level();
        self.mic_level = level;

        let c = if !self.user_custom_color
            || (self.aura_data.color1.r == 0
                && self.aura_data.color1.g == 0
                && self.aura_data.color1.b == 0)
        {
            crate::ui::theme::Theme::read_system_theme().1
        } else {
            self.aura_data.color1
        };

        if self.aura_brightness == 1 {
            if level == 0 {
                RgbColor::new(0, 0, 0)
            } else {
                let mult = (level as f32 / 100.0).clamp(0.4, 1.0);
                RgbColor::new(
                    (c.r as f32 * mult) as u8,
                    (c.g as f32 * mult) as u8,
                    (c.b as f32 * mult) as u8,
                )
            }
        } else if level == 0 {
            // Resting ambient glow
            RgbColor::new(
                (c.r as f32 * 0.15) as u8,
                (c.g as f32 * 0.15) as u8,
                (c.b as f32 * 0.15) as u8,
            )
        } else {
            let mult = 0.15 + 0.85 * (level as f32 / 100.0);
            RgbColor::new(
                (c.r as f32 * mult).min(255.0) as u8,
                (c.g as f32 * mult).min(255.0) as u8,
                (c.b as f32 * mult).min(255.0) as u8,
            )
        }
    }

    /// Audio Pulse: Sound output reactive visualizer via non-blocking worker.
    /// Priority: Defaults to active system desktop theme color; overridden by user-specified custom color.
    /// At brightness 1 (Low), pulses between 0 (off) and 1 (lit) on beat drops.
    pub fn compute_audio_pulse_color(&mut self) -> RgbColor {
        self.audio_service.start("@DEFAULT_MONITOR@");
        let level = self.audio_service.get_audio_level();
        self.audio_level = level;

        let c = if !self.user_custom_color
            || (self.aura_data.color1.r == 0
                && self.aura_data.color1.g == 0
                && self.aura_data.color1.b == 0)
        {
            crate::ui::theme::Theme::read_system_theme().1
        } else {
            self.aura_data.color1
        };

        if self.aura_brightness == 1 {
            if level == 0 {
                RgbColor::new(0, 0, 0)
            } else {
                let mult = (level as f32 / 100.0).clamp(0.4, 1.0);
                RgbColor::new(
                    (c.r as f32 * mult) as u8,
                    (c.g as f32 * mult) as u8,
                    (c.b as f32 * mult) as u8,
                )
            }
        } else if level == 0 {
            RgbColor::new(
                (c.r as f32 * 0.12) as u8,
                (c.g as f32 * 0.12) as u8,
                (c.b as f32 * 0.12) as u8,
            )
        } else {
            let mult = 0.12 + 0.88 * (level as f32 / 100.0);
            RgbColor::new(
                (c.r as f32 * mult).min(255.0) as u8,
                (c.g as f32 * mult).min(255.0) as u8,
                (c.b as f32 * mult).min(255.0) as u8,
            )
        }
    }

    /// Audio Wave: Dynamic audio spectrum EQ via non-blocking worker.
    /// Smoothly transitions color across the spectrum based on playback intensity.
    pub fn compute_audio_wave_color(&mut self) -> RgbColor {
        self.audio_service.start("@DEFAULT_MONITOR@");
        let level = self.audio_service.get_audio_level();
        self.audio_level = level;

        let raw_level = level as f32;
        // Smooth audio level tracking (attack faster than decay for punchy response without jitter)
        let alpha_level = if raw_level > self.audio_level_smooth { 0.28 } else { 0.10 };
        self.audio_level_smooth = self.audio_level_smooth * (1.0 - alpha_level) + raw_level * alpha_level;

        // Continuous hue mapping across spectrum:
        // 0% -> 240° (Deep blue), 30% -> 180° (Teal/Cyan), 60% -> 110° (Green/Lime), 85% -> 45° (Amber), 100% -> 0° (Red)
        let target_hue = (240.0 - (self.audio_level_smooth / 100.0) * 240.0).clamp(0.0, 240.0);

        // Smooth gradual hue transition so colors change gracefully and musically, not rapidly
        let alpha_hue = 0.12;
        self.audio_wave_hue = self.audio_wave_hue * (1.0 - alpha_hue) + target_hue * alpha_hue;

        let val = if self.aura_brightness == 1 && level == 0 {
            0.0
        } else {
            (25.0 + 75.0 * (self.audio_level_smooth / 100.0)).clamp(15.0, 100.0)
        };

        RgbColor::from_hsv(self.audio_wave_hue, 100.0, val)
    }

    /// Ambient Light: Real-time screen ambient sync on Wayland (ported from G-Helper).
    /// Uses grim with crop "0,400 1920x770", saturation boost +0.3, and smooth factor 0.65.
    pub fn compute_ambient_color(&mut self) -> RgbColor {
        self.ambient_service.start();
        if let Some(col) = self.ambient_service.get_color() {
            self.last_ambient_color = Some(col);
            col
        } else if let Some(last) = self.last_ambient_color {
            last
        } else {
            self.aura_data.color1
        }
    }

    /// Ambient Audio Sync: Merges real-time Wayland display color extraction
    /// with speaker/headphone playback audio reactive dynamics.
    /// Screen captures provide the scene color hue and saturation,
    /// while audio playback levels modulate the brightness and energy rhythm.
    pub fn compute_ambient_audio_sync_color(&mut self) -> RgbColor {
        self.ambient_service.start();
        self.audio_service.start("@DEFAULT_MONITOR@");

        let base_col = self.compute_ambient_color();
        let level = self.audio_service.get_audio_level();
        self.audio_level = level;

        let raw_level = level as f32;
        let alpha = if raw_level > self.audio_level_smooth { 0.35 } else { 0.12 };
        self.audio_level_smooth = self.audio_level_smooth * (1.0 - alpha) + raw_level * alpha;

        if self.aura_brightness == 1 && level == 0 {
            RgbColor::new(0, 0, 0)
        } else {
            let mult = 0.20 + 0.80 * (self.audio_level_smooth / 100.0);
            RgbColor::new(
                (base_col.r as f32 * mult).min(255.0) as u8,
                (base_col.g as f32 * mult).min(255.0) as u8,
                (base_col.b as f32 * mult).min(255.0) as u8,
            )
        }
    }

    /// Apply the active software effect to the keyboard via volatile EC write.
    /// No-op if brightness is Off (0) or effect is None.
    /// Only writes to hardware when the computed color changes
    /// to avoid stressing the system or bus with redundant I/O.
    pub async fn apply_software_effect(&mut self) {
        if self.aura_brightness == 0 || self.active_software_effect == SoftwareEffect::None {
            self.audio_service.stop();
            self.ambient_service.stop();
            return;
        }

        // Clean up services if no longer in audio or ambient modes
        if !matches!(
            self.active_software_effect,
            SoftwareEffect::VoicePulse
                | SoftwareEffect::AudioPulse
                | SoftwareEffect::AudioWave
                | SoftwareEffect::AmbientAudioSync
        ) {
            self.audio_service.stop();
        }
        if !matches!(
            self.active_software_effect,
            SoftwareEffect::AmbientLight | SoftwareEffect::AmbientAudioSync
        ) {
            self.ambient_service.stop();
        }

        let color = match self.active_software_effect {
            SoftwareEffect::RainbowWave => self.compute_rainbow_wave_color(),
            SoftwareEffect::ColorCycle => self.compute_color_cycle_color(),
            SoftwareEffect::FireFlicker => self.compute_fire_flicker_color(),
            SoftwareEffect::Clouds => self.compute_clouds_color(),
            SoftwareEffect::ThemeSync => self.compute_theme_sync_color(),
            SoftwareEffect::Strobe => self.compute_strobe_color(),
            SoftwareEffect::VoicePulse => self.compute_voice_pulse_color(),
            SoftwareEffect::AudioPulse => self.compute_audio_pulse_color(),
            SoftwareEffect::AudioWave => self.compute_audio_wave_color(),
            SoftwareEffect::AmbientLight => self.compute_ambient_color(),
            SoftwareEffect::AmbientAudioSync => self.compute_ambient_audio_sync_color(),
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

        // Deduplicate effects: skip hardware write if color has not changed
        if self.last_effect_color == Some(color) {
            return;
        }
        self.last_effect_color = Some(color);

        // Ensure Static mode (0) is the active firmware mode so the color shows
        if self.aura_mode != 0 {
            let _ = self.backend.set_aura_mode(0).await;
            self.aura_mode = 0;
            self.aura_data.mode = 0;
        }

        // Write volatile frame color directly to hardware without mutating base color1
        let _ = self.backend.write_volatile_rgb(color).await;
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
