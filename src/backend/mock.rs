use std::sync::Mutex;

use crate::backend::types::{
    ArmouryAttribute, AttributeKind, AuraModeData, AuraPowerState, FanCurve,
    GpuMode, GpuPowerState, PendingUserAction, PlatformProfile, RgbColor, SensorData,
};
use crate::backend::Backend;

struct MockState {
    platform_profile: PlatformProfile,
    profile_ac: PlatformProfile,
    profile_battery: PlatformProfile,
    auto_ac: bool,
    auto_battery: bool,
    charge_limit: u8,
    linked_epp: bool,
    disable_powerd_battery: bool,
    fan_curves: Vec<FanCurve>,
    aura_brightness: u32,
    aura_mode: u32,
    aura_data: AuraModeData,
    aura_power: AuraPowerState,
    gpu_mode: GpuMode,
    gpu_supported: Vec<GpuMode>,
    gpu_power: GpuPowerState,
    gpu_pending_mode: GpuMode,
    gpu_pending_action: PendingUserAction,
    armoury_attributes: Vec<ArmouryAttribute>,
    sensors: SensorData,
    volatile_rgb: Option<RgbColor>,
}

pub struct MockBackend {
    state: Mutex<MockState>,
}

impl Default for MockBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl MockBackend {
    pub fn new() -> Self {
        let default_curves = vec![
            FanCurve::new(
                "CPU",
                [45, 55, 65, 72, 78, 84, 90, 96],
                [5, 15, 30, 50, 65, 78, 88, 100],
                true,
            ),
            FanCurve::new(
                "GPU",
                [45, 55, 65, 72, 78, 84, 90, 96],
                [5, 15, 30, 50, 65, 78, 88, 100],
                true,
            ),
        ];

        let aura_data = AuraModeData {
            mode: 0,
            zone: 0,
            color1: RgbColor::new(0, 255, 200),
            color2: RgbColor::new(255, 0, 128),
            speed: "Med".to_string(),
            direction: "Right".to_string(),
        };

        let aura_power = AuraPowerState {
            boot: true,
            awake: true,
            sleep: false,
            shutdown: false,
        };

        let armoury_attributes = vec![
            ArmouryAttribute {
                node_name: "charge_mode".to_string(),
                display_name: "Charge Mode (Fast/Standard/Eco)".to_string(),
                current_value: 1,
                default_value: 1,
                min_value: -1,
                max_value: -1,
                possible_values: vec![0, 1, 2],
                scalar_increment: 1,
                kind: AttributeKind::Enum,
                is_available: true,
            },
            ArmouryAttribute {
                node_name: "panel_overdrive".to_string(),
                display_name: "Panel Overdrive (3ms Response)".to_string(),
                current_value: 1,
                default_value: 1,
                min_value: -1,
                max_value: -1,
                possible_values: vec![0, 1],
                scalar_increment: 1,
                kind: AttributeKind::Toggle,
                is_available: true,
            },
            ArmouryAttribute {
                node_name: "gpu_mux_mode".to_string(),
                display_name: "Display MUX Switch (Discrete Direct)".to_string(),
                current_value: 0,
                default_value: 0,
                min_value: -1,
                max_value: -1,
                possible_values: vec![0, 1],
                scalar_increment: 1,
                kind: AttributeKind::Toggle,
                is_available: true,
            },
            ArmouryAttribute {
                node_name: "screen_auto_brightness".to_string(),
                display_name: "Screen Auto-Brightness (ALS)".to_string(),
                current_value: 0,
                default_value: 0,
                min_value: -1,
                max_value: -1,
                possible_values: vec![0, 1],
                scalar_increment: 1,
                kind: AttributeKind::Toggle,
                is_available: true,
            },
            ArmouryAttribute {
                node_name: "ppt_pl1_spl".to_string(),
                display_name: "CPU Sustained Power Limit PL1 (W)".to_string(),
                current_value: -1,
                default_value: -1,
                min_value: -1,
                max_value: -1,
                possible_values: vec![],
                scalar_increment: 1,
                kind: AttributeKind::ReadOnly,
                is_available: false,
            },
        ];

        let sensors = SensorData {
            cpu_temp_c: Some(54),
            gpu_temp_c: Some(46),
            cpu_fan_rpm: Some(2700),
            gpu_fan_rpm: Some(2500),
            battery_charge_percent: Some(60),
            battery_charge_limit: 60,
            panel_overdrive: Some(true),
            gpu_mux_mode: Some(false),
            cpu_model: "13th Gen Intel(R) Core(TM) i5-13450HX".to_string(),
            cpu_freq_mhz: Some(3900),
            cpu_usage_percent: Some(18),
            cpu_power_w: Some(18.5),
            ram_used_mb: Some(10129),
            ram_total_mb: Some(15584),
            ram_usage_percent: Some(64),
            igpu_name: "Intel UHD Graphics (iHD)".to_string(),
            igpu_temp_c: Some(51),
            dgpu_name: "NVIDIA GeForce RTX 5050 Mobile".to_string(),
            dgpu_status: "Active (46 C)".to_string(),
            dgpu_power_state: "Active".to_string(),
            gpu_usage_percent: Some(7),
            gpu_power_w: Some(8.2),
            battery_status: "Not charging".to_string(),
            battery_power_w: Some(0.0),
            battery_health_percent: Some(98),
            battery_cycle_count: Some(0),
            screen_auto_brightness: Some(false),
        };

        Self {
            state: Mutex::new(MockState {
                platform_profile: PlatformProfile::Balanced,
                profile_ac: PlatformProfile::Performance,
                profile_battery: PlatformProfile::Quiet,
                auto_ac: true,
                auto_battery: true,
                charge_limit: 60,
                linked_epp: true,
                disable_powerd_battery: true,
                fan_curves: default_curves,
                aura_brightness: 2,
                aura_mode: 0,
                aura_data,
                aura_power,
                gpu_mode: GpuMode::AsusMuxDgpu,
                gpu_supported: vec![GpuMode::Integrated, GpuMode::Hybrid, GpuMode::AsusMuxDgpu],
                gpu_power: GpuPowerState::AsusMuxDiscreet,
                gpu_pending_mode: GpuMode::AsusMuxDgpu,
                gpu_pending_action: PendingUserAction::Nothing,
                armoury_attributes,
                sensors,
                volatile_rgb: None,
            }),
        }
    }
}

impl Backend for MockBackend {
    async fn get_platform_profile(&self) -> anyhow::Result<PlatformProfile> {
        let lock = self.state.lock().unwrap();
        Ok(lock.platform_profile)
    }

    async fn set_platform_profile(&self, profile: PlatformProfile) -> anyhow::Result<()> {
        let mut lock = self.state.lock().unwrap();
        lock.platform_profile = profile;
        Ok(())
    }

    async fn next_platform_profile(&self) -> anyhow::Result<()> {
        let mut lock = self.state.lock().unwrap();
        lock.platform_profile = match lock.platform_profile {
            PlatformProfile::Quiet => PlatformProfile::Balanced,
            PlatformProfile::Balanced => PlatformProfile::Performance,
            PlatformProfile::Performance => PlatformProfile::Quiet,
            PlatformProfile::Custom => PlatformProfile::Balanced,
        };
        Ok(())
    }

    async fn get_platform_profile_choices(&self) -> anyhow::Result<Vec<PlatformProfile>> {
        Ok(vec![
            PlatformProfile::Quiet,
            PlatformProfile::Balanced,
            PlatformProfile::Performance,
        ])
    }

    async fn get_profile_ac(&self) -> anyhow::Result<PlatformProfile> {
        let lock = self.state.lock().unwrap();
        Ok(lock.profile_ac)
    }

    async fn set_profile_ac(&self, profile: PlatformProfile) -> anyhow::Result<()> {
        let mut lock = self.state.lock().unwrap();
        lock.profile_ac = profile;
        Ok(())
    }

    async fn get_profile_battery(&self) -> anyhow::Result<PlatformProfile> {
        let lock = self.state.lock().unwrap();
        Ok(lock.profile_battery)
    }

    async fn set_profile_battery(&self, profile: PlatformProfile) -> anyhow::Result<()> {
        let mut lock = self.state.lock().unwrap();
        lock.profile_battery = profile;
        Ok(())
    }

    async fn get_auto_ac(&self) -> anyhow::Result<bool> {
        let lock = self.state.lock().unwrap();
        Ok(lock.auto_ac)
    }

    async fn set_auto_ac(&self, enabled: bool) -> anyhow::Result<()> {
        let mut lock = self.state.lock().unwrap();
        lock.auto_ac = enabled;
        Ok(())
    }

    async fn get_auto_battery(&self) -> anyhow::Result<bool> {
        let lock = self.state.lock().unwrap();
        Ok(lock.auto_battery)
    }

    async fn set_auto_battery(&self, enabled: bool) -> anyhow::Result<()> {
        let mut lock = self.state.lock().unwrap();
        lock.auto_battery = enabled;
        Ok(())
    }

    async fn get_charge_limit(&self) -> anyhow::Result<u8> {
        let lock = self.state.lock().unwrap();
        Ok(lock.charge_limit)
    }

    async fn set_charge_limit(&self, limit: u8) -> anyhow::Result<()> {
        let mut lock = self.state.lock().unwrap();
        lock.charge_limit = limit;
        lock.sensors.battery_charge_limit = limit;
        Ok(())
    }

    async fn one_shot_full_charge(&self) -> anyhow::Result<()> {
        let mut lock = self.state.lock().unwrap();
        lock.charge_limit = 100;
        lock.sensors.battery_charge_limit = 100;
        Ok(())
    }

    async fn get_linked_epp(&self) -> anyhow::Result<bool> {
        let lock = self.state.lock().unwrap();
        Ok(lock.linked_epp)
    }

    async fn set_linked_epp(&self, enabled: bool) -> anyhow::Result<()> {
        let mut lock = self.state.lock().unwrap();
        lock.linked_epp = enabled;
        Ok(())
    }

    async fn get_disable_nvidia_powerd_battery(&self) -> anyhow::Result<bool> {
        let lock = self.state.lock().unwrap();
        Ok(lock.disable_powerd_battery)
    }

    async fn set_disable_nvidia_powerd_battery(&self, disable: bool) -> anyhow::Result<()> {
        let mut lock = self.state.lock().unwrap();
        lock.disable_powerd_battery = disable;
        Ok(())
    }

    async fn get_fan_curves(&self, _profile: PlatformProfile) -> anyhow::Result<Vec<FanCurve>> {
        let lock = self.state.lock().unwrap();
        Ok(lock.fan_curves.clone())
    }

    async fn set_fan_curve(&self, _profile: PlatformProfile, curve: &FanCurve) -> anyhow::Result<()> {
        curve.validate().map_err(|e| anyhow::anyhow!(e))?;
        let mut lock = self.state.lock().unwrap();
        if let Some(pos) = lock.fan_curves.iter().position(|c| c.fan_name == curve.fan_name) {
            lock.fan_curves[pos] = curve.clone();
        } else {
            lock.fan_curves.push(curve.clone());
        }
        Ok(())
    }

    async fn set_fan_curves_enabled(&self, _profile: PlatformProfile, enabled: bool) -> anyhow::Result<()> {
        let mut lock = self.state.lock().unwrap();
        for curve in &mut lock.fan_curves {
            curve.enabled = enabled;
        }
        Ok(())
    }

    async fn set_curves_to_defaults(&self, _profile: PlatformProfile) -> anyhow::Result<()> {
        let mut lock = self.state.lock().unwrap();
        lock.fan_curves = vec![
            FanCurve::new("CPU", [45, 55, 65, 72, 78, 84, 90, 96], [5, 15, 30, 50, 65, 78, 88, 100], true),
            FanCurve::new("GPU", [45, 55, 65, 72, 78, 84, 90, 96], [5, 15, 30, 50, 65, 78, 88, 100], true),
        ];
        Ok(())
    }

    async fn get_aura_brightness(&self) -> anyhow::Result<u32> {
        let lock = self.state.lock().unwrap();
        Ok(lock.aura_brightness)
    }

    async fn set_aura_brightness(&self, level: u32) -> anyhow::Result<()> {
        let mut lock = self.state.lock().unwrap();
        lock.aura_brightness = level;
        Ok(())
    }

    async fn get_aura_mode(&self) -> anyhow::Result<u32> {
        let lock = self.state.lock().unwrap();
        Ok(lock.aura_mode)
    }

    async fn set_aura_mode(&self, mode: u32) -> anyhow::Result<()> {
        let mut lock = self.state.lock().unwrap();
        lock.aura_mode = mode;
        lock.aura_data.mode = mode;
        Ok(())
    }

    async fn get_aura_mode_data(&self) -> anyhow::Result<AuraModeData> {
        let lock = self.state.lock().unwrap();
        Ok(lock.aura_data.clone())
    }

    async fn set_aura_mode_data(&self, data: &AuraModeData) -> anyhow::Result<()> {
        let mut lock = self.state.lock().unwrap();
        lock.aura_data = data.clone();
        lock.aura_mode = data.mode;
        Ok(())
    }

    async fn get_aura_power(&self) -> anyhow::Result<AuraPowerState> {
        let lock = self.state.lock().unwrap();
        Ok(lock.aura_power)
    }

    async fn set_aura_power(&self, power: &AuraPowerState) -> anyhow::Result<()> {
        let mut lock = self.state.lock().unwrap();
        lock.aura_power = *power;
        Ok(())
    }

    async fn write_volatile_rgb(&self, color: RgbColor) -> anyhow::Result<()> {
        let mut lock = self.state.lock().unwrap();
        lock.volatile_rgb = Some(color);
        Ok(())
    }

    async fn get_gpu_mode(&self) -> anyhow::Result<GpuMode> {
        let lock = self.state.lock().unwrap();
        Ok(lock.gpu_mode)
    }

    async fn get_gpu_supported(&self) -> anyhow::Result<Vec<GpuMode>> {
        let lock = self.state.lock().unwrap();
        Ok(lock.gpu_supported.clone())
    }

    async fn get_gpu_power(&self) -> anyhow::Result<GpuPowerState> {
        let lock = self.state.lock().unwrap();
        Ok(lock.gpu_power)
    }

    async fn get_gpu_pending_mode(&self) -> anyhow::Result<GpuMode> {
        let lock = self.state.lock().unwrap();
        Ok(lock.gpu_pending_mode)
    }

    async fn get_gpu_pending_action(&self) -> anyhow::Result<PendingUserAction> {
        let lock = self.state.lock().unwrap();
        Ok(lock.gpu_pending_action)
    }

    async fn set_gpu_mode(&self, mode: GpuMode) -> anyhow::Result<PendingUserAction> {
        let mut lock = self.state.lock().unwrap();
        lock.gpu_pending_mode = mode;
        let action = if mode != lock.gpu_mode {
            PendingUserAction::Reboot
        } else {
            PendingUserAction::Nothing
        };
        lock.gpu_pending_action = action;
        Ok(action)
    }

    async fn get_armoury_attributes(&self) -> anyhow::Result<Vec<ArmouryAttribute>> {
        let lock = self.state.lock().unwrap();
        Ok(lock.armoury_attributes.clone())
    }

    async fn set_armoury_attribute(&self, node_name: &str, value: i32) -> anyhow::Result<()> {
        let mut lock = self.state.lock().unwrap();
        if let Some(attr) = lock.armoury_attributes.iter_mut().find(|a| a.node_name == node_name) {
            attr.current_value = value;
        }
        Ok(())
    }

    async fn restore_armoury_default(&self, node_name: &str) -> anyhow::Result<()> {
        let mut lock = self.state.lock().unwrap();
        if let Some(attr) = lock.armoury_attributes.iter_mut().find(|a| a.node_name == node_name) {
            attr.current_value = attr.default_value;
        }
        Ok(())
    }

    async fn read_sensors(&self) -> anyhow::Result<SensorData> {
        let lock = self.state.lock().unwrap();
        Ok(lock.sensors.clone())
    }

    async fn get_slash_supported(&self) -> anyhow::Result<bool> {
        Ok(false)
    }

    async fn get_slash_mode(&self) -> anyhow::Result<String> {
        Ok("Static".to_string())
    }

    async fn set_slash_mode(&self, _mode: &str) -> anyhow::Result<()> {
        Ok(())
    }

    async fn get_slash_brightness(&self) -> anyhow::Result<u32> {
        Ok(2)
    }

    async fn set_slash_brightness(&self, _level: u32) -> anyhow::Result<()> {
        Ok(())
    }
}
