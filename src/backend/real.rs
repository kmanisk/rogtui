use std::fs;
use std::path::Path;
use std::process::Command;
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

        Ok(Self {
            conn,
            platform,
            fan_curves,
            aura,
            supergfx,
            armoury_nodes,
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

        // Battery capacity & limit
        if let Ok(cap_s) = fs::read_to_string("/sys/class/power_supply/BAT0/capacity") {
            sensors.battery_charge_percent = cap_s.trim().parse::<u8>().ok();
        }
        sensors.battery_charge_limit = self.get_charge_limit().await.unwrap_or(80);

        // Scan hwmon for CPU package temp and ASUS fan RPMs
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

        // Fast NVIDIA GPU temp via nvidia-smi if dGPU is awake
        if Path::new("/proc/driver/nvidia").exists() {
            if let Ok(out) = Command::new("nvidia-smi")
                .args(["--query-gpu=temperature.gpu", "--format=csv,noheader,nounits"])
                .output()
            {
                if out.status.success() {
                    let s = String::from_utf8_lossy(&out.stdout);
                    sensors.gpu_temp_c = s.trim().parse::<u32>().ok();
                }
            }
        }

        Ok(sensors)
    }
}
