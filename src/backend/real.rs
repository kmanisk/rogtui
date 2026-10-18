use std::fs;
use zbus::proxy;

use crate::backend::types::{
    ArmouryAttribute, AttributeKind, AuraModeData, AuraPowerState, FanCurve,
    GpuMode, GpuPowerState, PendingUserAction, PlatformProfile, RgbColor, SensorData,
};

#[proxy(
    interface = "xyz.ljones.Platform",
    default_service = "xyz.ljones.Asusd",
    default_path = "/xyz/ljones"
)]
pub trait Platform {
    #[zbus(property)]
    fn platform_profile(&self) -> zbus::Result<u32>;

    #[zbus(property)]
    fn set_platform_profile(&self, value: u32) -> zbus::Result<()>;

    #[zbus(property)]
    fn platform_profile_choices(&self) -> zbus::Result<Vec<u32>>;

    #[zbus(property)]
    fn platform_profile_on_ac(&self) -> zbus::Result<u32>;

    #[zbus(property)]
    fn set_platform_profile_on_ac(&self, value: u32) -> zbus::Result<()>;

    #[zbus(property)]
    fn platform_profile_on_battery(&self) -> zbus::Result<u32>;

    #[zbus(property)]
    fn set_platform_profile_on_battery(&self, value: u32) -> zbus::Result<()>;

    #[zbus(property)]
    fn change_platform_profile_on_ac(&self) -> zbus::Result<bool>;

    #[zbus(property)]
    fn set_change_platform_profile_on_ac(&self, value: bool) -> zbus::Result<()>;

    #[zbus(property)]
    fn change_platform_profile_on_battery(&self) -> zbus::Result<bool>;

    #[zbus(property)]
    fn set_change_platform_profile_on_battery(&self, value: bool) -> zbus::Result<()>;

    #[zbus(property)]
    fn charge_control_end_threshold(&self) -> zbus::Result<u8>;

    #[zbus(property)]
    fn set_charge_control_end_threshold(&self, value: u8) -> zbus::Result<()>;

    fn one_shot_full_charge(&self) -> zbus::Result<()>;

    #[zbus(property)]
    fn disable_nvidia_powerd_on_battery(&self) -> zbus::Result<bool>;

    #[zbus(property)]
    fn set_disable_nvidia_powerd_on_battery(&self, value: bool) -> zbus::Result<()>;

    #[zbus(property)]
    fn platform_profile_linked_epp(&self) -> zbus::Result<bool>;

    #[zbus(property)]
    fn set_platform_profile_linked_epp(&self, value: bool) -> zbus::Result<()>;

    fn next_platform_profile(&self) -> zbus::Result<()>;

    #[zbus(property)]
    fn version(&self) -> zbus::Result<String>;
}

type FanCurveRaw = (
    String,
    (u8, u8, u8, u8, u8, u8, u8, u8),
    (u8, u8, u8, u8, u8, u8, u8, u8),
    bool,
);

#[proxy(
    interface = "xyz.ljones.FanCurves",
    default_service = "xyz.ljones.Asusd",
    default_path = "/xyz/ljones"
)]
pub trait FanCurves {
    fn fan_curve_data(&self, profile: u32) -> zbus::Result<Vec<FanCurveRaw>>;

    fn set_fan_curve(&self, profile: u32, curve: FanCurveRaw) -> zbus::Result<()>;

    fn set_fan_curves_enabled(&self, profile: u32, enabled: bool) -> zbus::Result<()>;

    fn set_profile_fan_curve_enabled(&self, profile: u32, fan: &str, enabled: bool) -> zbus::Result<()>;

    fn set_curves_to_defaults(&self, profile: u32) -> zbus::Result<()>;
}

type AuraModeRaw = (u32, u32, (u8, u8, u8), (u8, u8, u8), String, String);
type AuraPowerRaw = (Vec<(u32, bool, bool, bool, bool)>,);

#[proxy(
    interface = "xyz.ljones.Aura",
    default_service = "xyz.ljones.Asusd",
    default_path = "/xyz/ljones/aura/tuf"
)]
pub trait Aura {
    #[zbus(property)]
    fn brightness(&self) -> zbus::Result<u32>;

    #[zbus(property)]
    fn set_brightness(&self, value: u32) -> zbus::Result<()>;

    #[zbus(property)]
    fn supported_brightness(&self) -> zbus::Result<Vec<u32>>;

    #[zbus(property)]
    fn led_mode(&self) -> zbus::Result<u32>;

    #[zbus(property)]
    fn set_led_mode(&self, value: u32) -> zbus::Result<()>;

    #[zbus(property)]
    fn supported_basic_modes(&self) -> zbus::Result<Vec<u32>>;

    #[zbus(property)]
    fn led_mode_data(&self) -> zbus::Result<AuraModeRaw>;

    #[zbus(property)]
    fn set_led_mode_data(&self, value: AuraModeRaw) -> zbus::Result<()>;

    #[zbus(property)]
    fn led_power(&self) -> zbus::Result<AuraPowerRaw>;

    #[zbus(property)]
    fn set_led_power(&self, value: AuraPowerRaw) -> zbus::Result<()>;

    fn all_mode_data(&self) -> zbus::Result<std::collections::HashMap<u32, AuraModeRaw>>;
}

#[proxy(
    interface = "org.supergfxctl.Daemon",
    default_service = "org.supergfxctl.Daemon",
    default_path = "/org/supergfxctl/Gfx"
)]
pub trait SuperGfx {
    fn mode(&self) -> zbus::Result<u32>;

    fn supported(&self) -> zbus::Result<Vec<u32>>;

    fn power(&self) -> zbus::Result<u32>;

    fn pending_mode(&self) -> zbus::Result<u32>;

    fn pending_user_action(&self) -> zbus::Result<u32>;

    fn vendor(&self) -> zbus::Result<String>;

    fn version(&self) -> zbus::Result<String>;

    fn set_mode(&self, mode: u32) -> zbus::Result<u32>;
}

#[proxy(
    interface = "xyz.ljones.AsusArmoury",
    default_service = "xyz.ljones.Asusd"
)]
pub trait AsusArmoury {
    #[zbus(property)]
    fn name(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn current_value(&self) -> zbus::Result<i32>;

    #[zbus(property)]
    fn set_current_value(&self, value: i32) -> zbus::Result<()>;

    #[zbus(property)]
    fn default_value(&self) -> zbus::Result<i32>;

    #[zbus(property)]
    fn min_value(&self) -> zbus::Result<i32>;

    #[zbus(property)]
    fn max_value(&self) -> zbus::Result<i32>;

    #[zbus(property)]
    fn possible_values(&self) -> zbus::Result<Vec<i32>>;

    #[zbus(property)]
    fn available_attrs(&self) -> zbus::Result<Vec<String>>;

    #[zbus(property)]
    fn scalar_increment(&self) -> zbus::Result<i32>;

    fn restore_default(&self) -> zbus::Result<()>;

    fn apply_queued_gpu_value(&self) -> zbus::Result<bool>;
}

pub struct RealBackend {
    conn: zbus::Connection,
    platform: PlatformProxy<'static>,
    fan_curves: FanCurvesProxy<'static>,
    aura: AuraProxy<'static>,
    supergfx: SuperGfxProxy<'static>,
    armoury_nodes: Vec<String>,
    cpu_prev_stat: std::sync::Mutex<Option<(u64, u64)>>,
    nvml: crate::backend::nvml::NvmlReader,
}

impl RealBackend {
    pub async fn connect() -> anyhow::Result<Self> {
        let conn = zbus::Connection::system().await?;
        let platform = PlatformProxy::new(&conn).await?;
        let fan_curves = FanCurvesProxy::new(&conn).await?;
        let aura = AuraProxy::new(&conn).await?;
        let supergfx = SuperGfxProxy::new(&conn).await?;

        let armoury_nodes = vec![
            "charge_mode".to_string(),
            "panel_overdrive".to_string(),
            "dgpu_disable".to_string(),
            "gpu_mux_mode".to_string(),
            "screen_auto_brightness".to_string(),
            "nv_base_tgp".to_string(),
            "nv_dynamic_boost".to_string(),
            "nv_temp_target".to_string(),
            "nv_tgp".to_string(),
            "ppt_pl1_spl".to_string(),
            "ppt_pl2_sppt".to_string(),
        ];

        let nvml = crate::backend::nvml::NvmlReader::new();

        Ok(Self {
            conn,
            platform,
            fan_curves,
            aura,
            supergfx,
            armoury_nodes,
            cpu_prev_stat: std::sync::Mutex::new(None),
            nvml,
        })
    }

    pub fn connection(&self) -> &zbus::Connection {
        &self.conn
    }
}

impl super::Backend for RealBackend {
    async fn get_platform_profile(&self) -> anyhow::Result<PlatformProfile> {
        let raw = self.platform.platform_profile().await?;
        Ok(PlatformProfile::from_u32(raw))
    }

    async fn set_platform_profile(&self, profile: PlatformProfile) -> anyhow::Result<()> {
        self.platform.set_platform_profile(profile.to_u32()).await?;
        Ok(())
    }

    async fn next_platform_profile(&self) -> anyhow::Result<()> {
        self.platform.next_platform_profile().await?;
        Ok(())
    }

    async fn get_platform_profile_choices(&self) -> anyhow::Result<Vec<PlatformProfile>> {
        let raw = self.platform.platform_profile_choices().await?;
        Ok(raw.into_iter().map(PlatformProfile::from_u32).collect())
    }

    async fn get_profile_ac(&self) -> anyhow::Result<PlatformProfile> {
        let raw = self.platform.platform_profile_on_ac().await?;
        Ok(PlatformProfile::from_u32(raw))
    }

    async fn set_profile_ac(&self, profile: PlatformProfile) -> anyhow::Result<()> {
        self.platform.set_platform_profile_on_ac(profile.to_u32()).await?;
        Ok(())
    }

    async fn get_profile_battery(&self) -> anyhow::Result<PlatformProfile> {
        let raw = self.platform.platform_profile_on_battery().await?;
        Ok(PlatformProfile::from_u32(raw))
    }

    async fn set_profile_battery(&self, profile: PlatformProfile) -> anyhow::Result<()> {
        self.platform.set_platform_profile_on_battery(profile.to_u32()).await?;
        Ok(())
    }

    async fn get_auto_ac(&self) -> anyhow::Result<bool> {
        Ok(self.platform.change_platform_profile_on_ac().await?)
    }

    async fn set_auto_ac(&self, enabled: bool) -> anyhow::Result<()> {
        self.platform.set_change_platform_profile_on_ac(enabled).await?;
        Ok(())
    }

    async fn get_auto_battery(&self) -> anyhow::Result<bool> {
        Ok(self.platform.change_platform_profile_on_battery().await?)
    }

    async fn set_auto_battery(&self, enabled: bool) -> anyhow::Result<()> {
        self.platform.set_change_platform_profile_on_battery(enabled).await?;
        Ok(())
    }

    async fn get_charge_limit(&self) -> anyhow::Result<u8> {
        Ok(self.platform.charge_control_end_threshold().await?)
    }

    async fn set_charge_limit(&self, limit: u8) -> anyhow::Result<()> {
        self.platform.set_charge_control_end_threshold(limit).await?;
        Ok(())
    }

    async fn one_shot_full_charge(&self) -> anyhow::Result<()> {
        self.platform.one_shot_full_charge().await?;
        Ok(())
    }

    async fn get_linked_epp(&self) -> anyhow::Result<bool> {
        Ok(self.platform.platform_profile_linked_epp().await?)
    }

    async fn set_linked_epp(&self, enabled: bool) -> anyhow::Result<()> {
        self.platform.set_platform_profile_linked_epp(enabled).await?;
        Ok(())
    }

    async fn get_disable_nvidia_powerd_battery(&self) -> anyhow::Result<bool> {
        Ok(self.platform.disable_nvidia_powerd_on_battery().await?)
    }

    async fn set_disable_nvidia_powerd_battery(&self, disable: bool) -> anyhow::Result<()> {
        self.platform.set_disable_nvidia_powerd_on_battery(disable).await?;
        Ok(())
    }

    async fn get_fan_curves(&self, profile: PlatformProfile) -> anyhow::Result<Vec<FanCurve>> {
        let raw = self.fan_curves.fan_curve_data(profile.to_u32()).await?;
        let mut curves = Vec::new();
        for (name, pwms, temps, enabled) in raw {
            let p_arr = [pwms.0, pwms.1, pwms.2, pwms.3, pwms.4, pwms.5, pwms.6, pwms.7];
            let t_arr = [temps.0, temps.1, temps.2, temps.3, temps.4, temps.5, temps.6, temps.7];
            curves.push(FanCurve::new(name, t_arr, p_arr, enabled));
        }
        Ok(curves)
    }

    async fn set_fan_curve(&self, profile: PlatformProfile, curve: &FanCurve) -> anyhow::Result<()> {
        curve.validate().map_err(|e| anyhow::anyhow!(e))?;
        let raw = curve.to_dbus_raw();
        self.fan_curves.set_fan_curve(profile.to_u32(), raw).await?;
        Ok(())
    }

    async fn set_fan_curves_enabled(&self, profile: PlatformProfile, enabled: bool) -> anyhow::Result<()> {
        self.fan_curves.set_fan_curves_enabled(profile.to_u32(), enabled).await?;
        Ok(())
    }

    async fn set_curves_to_defaults(&self, profile: PlatformProfile) -> anyhow::Result<()> {
        self.fan_curves.set_curves_to_defaults(profile.to_u32()).await?;
        Ok(())
    }

    async fn get_aura_brightness(&self) -> anyhow::Result<u32> {
        Ok(self.aura.brightness().await?)
    }

    async fn set_aura_brightness(&self, level: u32) -> anyhow::Result<()> {
        self.aura.set_brightness(level).await?;
        Ok(())
    }

    async fn get_aura_mode(&self) -> anyhow::Result<u32> {
        Ok(self.aura.led_mode().await?)
    }

    async fn set_aura_mode(&self, mode: u32) -> anyhow::Result<()> {
        self.aura.set_led_mode(mode).await?;
        Ok(())
    }

    async fn get_aura_mode_data(&self) -> anyhow::Result<AuraModeData> {
        let (mode, zone, c1, c2, speed, dir) = self.aura.led_mode_data().await?;
        Ok(AuraModeData {
            mode,
            zone,
            color1: RgbColor::new(c1.0, c1.1, c1.2),
            color2: RgbColor::new(c2.0, c2.1, c2.2),
            speed,
            direction: dir,
        })
    }

    async fn set_aura_mode_data(&self, data: &AuraModeData) -> anyhow::Result<()> {
        let raw: AuraModeRaw = (
            data.mode,
            data.zone,
            (data.color1.r, data.color1.g, data.color1.b),
            (data.color2.r, data.color2.g, data.color2.b),
            data.speed.clone(),
            data.direction.clone(),
        );
        self.aura.set_led_mode_data(raw).await?;
        Ok(())
    }

    async fn get_aura_power(&self) -> anyhow::Result<AuraPowerState> {
        let (zones,) = self.aura.led_power().await?;
        if let Some((_zone, boot, awake, sleep, shutdown)) = zones.into_iter().next() {
            Ok(AuraPowerState {
                boot,
                awake,
                sleep,
                shutdown,
            })
        } else {
            Ok(AuraPowerState {
                boot: true,
                awake: true,
                sleep: false,
                shutdown: false,
            })
        }
    }

    async fn set_aura_power(&self, power: &AuraPowerState) -> anyhow::Result<()> {
        let val: AuraPowerRaw = (vec![(1, power.boot, power.awake, power.sleep, power.shutdown)],);
        self.aura.set_led_power(val).await?;
        Ok(())
    }

    async fn write_volatile_rgb(&self, color: RgbColor) -> anyhow::Result<()> {
        const SYSFS_PATH: &str = "/sys/class/leds/asus::kbd_backlight/kbd_rgb_mode";
        if std::path::Path::new(SYSFS_PATH).exists() {
            let payload = format!("0 0 {} {} {} 1\n", color.r, color.g, color.b);
            match std::fs::write(SYSFS_PATH, payload.as_bytes()) {
                Ok(()) => return Ok(()),
                Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
                    let status = std::process::Command::new("sudo")
                        .args(["-n", "tee", SYSFS_PATH])
                        .stdin(std::process::Stdio::piped())
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .spawn()
                        .and_then(|mut child| {
                            if let Some(mut stdin) = child.stdin.take() {
                                use std::io::Write;
                                let _ = stdin.write_all(payload.as_bytes());
                            }
                            child.wait()
                        });
                    let _ = std::process::Command::new("sudo")
                        .args(["-n", "chmod", "0666", SYSFS_PATH])
                        .spawn();
                    if let Ok(st) = status {
                        if st.success() {
                            return Ok(());
                        }
                    }
                }
                Err(e) => return Err(e.into()),
            }
        }

        let raw: AuraModeRaw = (
            0,
            0,
            (color.r, color.g, color.b),
            (0, 0, 0),
            "Med".to_string(),
            "Right".to_string(),
        );
        let _ = self.aura.set_led_mode(0).await;
        self.aura.set_led_mode_data(raw).await?;
        Ok(())
    }

    async fn get_gpu_mode(&self) -> anyhow::Result<GpuMode> {
        let raw = self.supergfx.mode().await?;
        Ok(GpuMode::from_u32(raw))
    }

    async fn get_gpu_supported(&self) -> anyhow::Result<Vec<GpuMode>> {
        let raw = self.supergfx.supported().await?;
        Ok(raw.into_iter().map(GpuMode::from_u32).collect())
    }

    async fn get_gpu_power(&self) -> anyhow::Result<GpuPowerState> {
        let raw = self.supergfx.power().await?;
        Ok(GpuPowerState::from_u32(raw))
    }

    async fn get_gpu_pending_mode(&self) -> anyhow::Result<GpuMode> {
        let raw = self.supergfx.pending_mode().await?;
        Ok(GpuMode::from_u32(raw))
    }

    async fn get_gpu_pending_action(&self) -> anyhow::Result<PendingUserAction> {
        let raw = self.supergfx.pending_user_action().await?;
        Ok(PendingUserAction::from_u32(raw))
    }

    async fn set_gpu_mode(&self, mode: GpuMode) -> anyhow::Result<PendingUserAction> {
        let res = self.supergfx.set_mode(mode.to_u32()).await?;
        Ok(PendingUserAction::from_u32(res))
    }

    async fn get_armoury_attributes(&self) -> anyhow::Result<Vec<ArmouryAttribute>> {
        let mut list = Vec::new();
        for node in &self.armoury_nodes {
            let path = format!("/xyz/ljones/asus_armoury/{node}");
            let proxy = AsusArmouryProxy::builder(&self.conn)
                .path(path)?
                .build()
                .await?;

            let available_attrs = proxy.available_attrs().await.unwrap_or_default();
            let is_available = available_attrs.iter().any(|s| s == "current_value");

            let current_value = if is_available {
                proxy.current_value().await.unwrap_or(-1)
            } else {
                -1
            };

            let default_value = proxy.default_value().await.unwrap_or(-1);
            let min_value = proxy.min_value().await.unwrap_or(-1);
            let max_value = proxy.max_value().await.unwrap_or(-1);
            let possible_values = proxy.possible_values().await.unwrap_or_default();
            let scalar_increment = proxy.scalar_increment().await.unwrap_or(1);

            let kind = if possible_values.len() == 2 && possible_values.contains(&0) && possible_values.contains(&1) {
                AttributeKind::Toggle
            } else if !possible_values.is_empty() {
                AttributeKind::Enum
            } else if min_value >= 0 && max_value > min_value {
                AttributeKind::Slider
            } else {
                AttributeKind::ReadOnly
            };

            let display_name = match node.as_str() {
                "charge_mode" => "Charge Mode (Fast/Standard/Eco)".to_string(),
                "panel_overdrive" => "Panel Overdrive (3ms Response)".to_string(),
                "dgpu_disable" => "dGPU Hardware Gate (ACPI Disable)".to_string(),
                "gpu_mux_mode" => "Display MUX Switch (Discrete Direct)".to_string(),
                "screen_auto_brightness" => "Screen Auto-Brightness (ALS)".to_string(),
                "nv_base_tgp" => "NVIDIA Base TGP (W)".to_string(),
                "nv_dynamic_boost" => "NVIDIA Dynamic Boost (W)".to_string(),
                "nv_temp_target" => "NVIDIA Thermal Target (C)".to_string(),
                "nv_tgp" => "NVIDIA Total Graphics Power (W)".to_string(),
                "ppt_pl1_spl" => "CPU Sustained Power Limit PL1 (W)".to_string(),
                "ppt_pl2_sppt" => "CPU Burst Power Limit PL2 (W)".to_string(),
                other => other.replace('_', " "),
            };

            list.push(ArmouryAttribute {
                node_name: node.clone(),
                display_name,
                current_value,
                default_value,
                min_value,
                max_value,
                possible_values,
                scalar_increment,
                kind,
                is_available,
            });
        }
        Ok(list)
    }

    async fn set_armoury_attribute(&self, node_name: &str, value: i32) -> anyhow::Result<()> {
        let path = format!("/xyz/ljones/asus_armoury/{node_name}");
        let proxy = AsusArmouryProxy::builder(&self.conn)
            .path(path)?
            .build()
            .await?;
        proxy.set_current_value(value).await?;
        if node_name == "gpu_mux_mode" || node_name == "dgpu_disable" {
            let _ = proxy.apply_queued_gpu_value().await;
        }
        Ok(())
    }

    async fn restore_armoury_default(&self, node_name: &str) -> anyhow::Result<()> {
        let path = format!("/xyz/ljones/asus_armoury/{node_name}");
        let proxy = AsusArmouryProxy::builder(&self.conn)
            .path(path)?
            .build()
            .await?;
        proxy.restore_default().await?;
        Ok(())
    }

    async fn read_sensors(&self) -> anyhow::Result<SensorData> {
        let mut sensors = SensorData::default();

        // 1. Battery capacity & limit
        if let Ok(cap_s) = fs::read_to_string("/sys/class/power_supply/BAT0/capacity") {
            sensors.battery_charge_percent = cap_s.trim().parse::<u8>().ok();
        }
        sensors.battery_charge_limit = self.get_charge_limit().await.unwrap_or(80);

        // Extended battery telemetry
        if let Ok(st) = fs::read_to_string("/sys/class/power_supply/BAT0/status") {
            sensors.battery_status = st.trim().to_string();
        }
        if let Ok(pw) = fs::read_to_string("/sys/class/power_supply/BAT0/power_now") {
            if let Ok(u_w) = pw.trim().parse::<f32>() {
                sensors.battery_power_w = Some(u_w / 1_000_000.0);
            }
        }
        if let (Ok(efull), Ok(edesign)) = (
            fs::read_to_string("/sys/class/power_supply/BAT0/energy_full"),
            fs::read_to_string("/sys/class/power_supply/BAT0/energy_full_design"),
        ) {
            if let (Ok(ef), Ok(ed)) = (efull.trim().parse::<u64>(), edesign.trim().parse::<u64>()) {
                sensors.battery_health_percent = (ef * 100).checked_div(ed).map(|pct| pct.min(100) as u8);
            }
        }
        if let Ok(cyc) = fs::read_to_string("/sys/class/power_supply/BAT0/cycle_count") {
            sensors.battery_cycle_count = cyc.trim().parse::<u32>().ok();
        }

        // 2. CPU Model
        if let Ok(cpuinfo) = fs::read_to_string("/proc/cpuinfo") {
            for line in cpuinfo.lines() {
                if line.starts_with("model name") {
                    if let Some((_, model)) = line.split_once(':') {
                        sensors.cpu_model = model.trim().to_string();
                        break;
                    }
                }
            }
        }

        // 3. CPU Frequency (average across active cores in MHz)
        let mut freq_sum = 0u64;
        let mut freq_count = 0u64;
        if let Ok(entries) = fs::read_dir("/sys/devices/system/cpu/cpufreq") {
            for entry in entries.flatten() {
                let name = entry.file_name();
                if name.to_string_lossy().starts_with("policy") {
                    let freq_path = entry.path().join("scaling_cur_freq");
                    if let Ok(freq_s) = fs::read_to_string(freq_path) {
                        if let Ok(khz) = freq_s.trim().parse::<u64>() {
                            freq_sum += khz;
                            freq_count += 1;
                        }
                    }
                }
            }
        }
        sensors.cpu_freq_mhz = freq_sum.checked_div(freq_count).map(|khz| (khz / 1000) as u32);

        // 4. CPU Usage % deltas from /proc/stat
        if let Ok(stat) = fs::read_to_string("/proc/stat") {
            if let Some(first_line) = stat.lines().next() {
                let parts: Vec<&str> = first_line.split_whitespace().collect();
                if parts.len() >= 5 && parts[0] == "cpu" {
                    let user: u64 = parts[1].parse().unwrap_or(0);
                    let nice: u64 = parts[2].parse().unwrap_or(0);
                    let system: u64 = parts[3].parse().unwrap_or(0);
                    let idle_raw: u64 = parts[4].parse().unwrap_or(0);
                    let iowait: u64 = parts.get(5).and_then(|p| p.parse().ok()).unwrap_or(0);
                    let irq: u64 = parts.get(6).and_then(|p| p.parse().ok()).unwrap_or(0);
                    let softirq: u64 = parts.get(7).and_then(|p| p.parse().ok()).unwrap_or(0);
                    let steal: u64 = parts.get(8).and_then(|p| p.parse().ok()).unwrap_or(0);

                    let idle = idle_raw + iowait;
                    let total = user + nice + system + idle + irq + softirq + steal;

                    let mut prev_guard = self.cpu_prev_stat.lock().unwrap();
                    if let Some((prev_idle, prev_total)) = *prev_guard {
                        let delta_idle = idle.saturating_sub(prev_idle);
                        let delta_total = total.saturating_sub(prev_total);
                        sensors.cpu_usage_percent = (delta_total.saturating_sub(delta_idle) * 100)
                            .checked_div(delta_total)
                            .map(|usage| usage.min(100) as u8);
                    }
                    *prev_guard = Some((idle, total));
                }
            }
        }

        // 5. RAM Usage from /proc/meminfo
        if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
            let mut total_kb = 0u64;
            let mut avail_kb = 0u64;
            for line in meminfo.lines() {
                if line.starts_with("MemTotal:") {
                    total_kb = line.split_whitespace().nth(1).and_then(|v| v.parse().ok()).unwrap_or(0);
                } else if line.starts_with("MemAvailable:") {
                    avail_kb = line.split_whitespace().nth(1).and_then(|v| v.parse().ok()).unwrap_or(0);
                }
            }
            if total_kb > 0 {
                let total_mb = total_kb / 1024;
                let used_mb = total_kb.saturating_sub(avail_kb) / 1024;
                let usage_pct = ((used_mb * 100) / total_mb) as u8;
                sensors.ram_total_mb = Some(total_mb);
                sensors.ram_used_mb = Some(used_mb);
                sensors.ram_usage_percent = Some(usage_pct.min(100));
            }
        }

        // 6. iGPU Name
        sensors.igpu_name = "Intel UHD Graphics (iHD)".to_string();

        // 7. Scan hwmon for CPU package temp and ASUS fan RPMs
        if let Ok(entries) = fs::read_dir("/sys/class/hwmon") {
            for entry in entries.flatten() {
                let p = entry.path();
                let name_file = p.join("name");
                if let Ok(name) = fs::read_to_string(&name_file) {
                    let name = name.trim();
                    if name == "coretemp" {
                        if let Ok(t) = fs::read_to_string(p.join("temp1_input")) {
                            if let Ok(val) = t.trim().parse::<u32>() {
                                sensors.cpu_temp_c = Some(val / 1000);
                                sensors.igpu_temp_c = Some(val / 1000);
                            }
                        }
                    } else if name == "asus" {
                        if let Ok(r) = fs::read_to_string(p.join("fan1_input")) {
                            sensors.cpu_fan_rpm = r.trim().parse::<u32>().ok();
                        }
                        if let Ok(r) = fs::read_to_string(p.join("fan2_input")) {
                            sensors.gpu_fan_rpm = r.trim().parse::<u32>().ok();
                        }
                    }
                }
            }
        }

        // 8. Dynamic dGPU Discovery: vendor 0x10de, class prefix 0x03 under /sys/bus/pci/devices
        sensors.dgpu_name = "NVIDIA GeForce RTX 5050 Mobile".to_string();
        let mut dgpu_found = false;
        if let Ok(entries) = fs::read_dir("/sys/bus/pci/devices") {
            for entry in entries.flatten() {
                let p = entry.path();
                let vendor_file = p.join("vendor");
                let class_file = p.join("class");
                if let (Ok(v), Ok(c)) = (fs::read_to_string(&vendor_file), fs::read_to_string(&class_file)) {
                    if v.trim() == "0x10de" && c.trim().starts_with("0x03") {
                        dgpu_found = true;
                        let runtime_status = fs::read_to_string(p.join("power/runtime_status"))
                            .unwrap_or_default();
                        let status_clean = runtime_status.trim();
                        sensors.dgpu_power_state = status_clean.to_string();

                        if status_clean == "active" {
                            // Discrete GPU is active: safe to read without waking D3cold
                            let (gpu_temp, gpu_util) = self.nvml.query_gpu_metrics();
                            sensors.gpu_temp_c = gpu_temp;
                            match (gpu_temp, gpu_util) {
                                (Some(t), Some(u)) => {
                                    sensors.dgpu_status = format!("Active ({t} C, {u}% util)");
                                }
                                (Some(t), None) => {
                                    sensors.dgpu_status = format!("Active ({t} C)");
                                }
                                _ => {
                                    sensors.dgpu_status = "Active (n/a)".to_string();
                                }
                            }
                        } else if status_clean == "suspended" {
                            // Discrete GPU is in low-power D3cold: DO NOT wake it
                            sensors.gpu_temp_c = None;
                            sensors.dgpu_status = "Suspended (D3cold)".to_string();
                        } else {
                            sensors.gpu_temp_c = None;
                            sensors.dgpu_status = format!("Power: {status_clean}");
                        }
                        break;
                    }
                }
            }
        }
        if !dgpu_found {
            sensors.dgpu_status = "Not detected / Off".to_string();
            sensors.dgpu_power_state = "Off".to_string();
        }

        // Quick check for panel_overdrive and screen_auto_brightness from Armoury
        if let Ok(attrs) = self.get_armoury_attributes().await {
            for attr in attrs {
                if attr.node_name == "panel_overdrive" && attr.is_available {
                    sensors.panel_overdrive = Some(attr.current_value == 1);
                } else if attr.node_name == "screen_auto_brightness" && attr.is_available {
                    sensors.screen_auto_brightness = Some(attr.current_value == 1);
                }
            }
        }

        Ok(sensors)
    }
}
