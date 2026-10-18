use std::future::Future;

pub mod mock;
pub mod nvml;
pub mod real;
pub mod types;

use types::{
    ArmouryAttribute, AuraModeData, AuraPowerState, FanCurve, GpuMode, GpuPowerState,
    PendingUserAction, PlatformProfile, SensorData,
};

pub trait Backend: Send + Sync + 'static {
    fn get_platform_profile(&self) -> impl Future<Output = anyhow::Result<PlatformProfile>> + Send;
    fn set_platform_profile(&self, profile: PlatformProfile) -> impl Future<Output = anyhow::Result<()>> + Send;
    fn next_platform_profile(&self) -> impl Future<Output = anyhow::Result<()>> + Send;
    fn get_platform_profile_choices(&self) -> impl Future<Output = anyhow::Result<Vec<PlatformProfile>>> + Send;
    fn get_profile_ac(&self) -> impl Future<Output = anyhow::Result<PlatformProfile>> + Send;
    fn set_profile_ac(&self, profile: PlatformProfile) -> impl Future<Output = anyhow::Result<()>> + Send;
    fn get_profile_battery(&self) -> impl Future<Output = anyhow::Result<PlatformProfile>> + Send;
    fn set_profile_battery(&self, profile: PlatformProfile) -> impl Future<Output = anyhow::Result<()>> + Send;
    fn get_auto_ac(&self) -> impl Future<Output = anyhow::Result<bool>> + Send;
    fn set_auto_ac(&self, enabled: bool) -> impl Future<Output = anyhow::Result<()>> + Send;
    fn get_auto_battery(&self) -> impl Future<Output = anyhow::Result<bool>> + Send;
    fn set_auto_battery(&self, enabled: bool) -> impl Future<Output = anyhow::Result<()>> + Send;
    fn get_charge_limit(&self) -> impl Future<Output = anyhow::Result<u8>> + Send;
    fn set_charge_limit(&self, limit: u8) -> impl Future<Output = anyhow::Result<()>> + Send;
    fn one_shot_full_charge(&self) -> impl Future<Output = anyhow::Result<()>> + Send;
    fn get_linked_epp(&self) -> impl Future<Output = anyhow::Result<bool>> + Send;
    fn set_linked_epp(&self, enabled: bool) -> impl Future<Output = anyhow::Result<()>> + Send;
    fn get_disable_nvidia_powerd_battery(&self) -> impl Future<Output = anyhow::Result<bool>> + Send;
    fn set_disable_nvidia_powerd_battery(&self, disable: bool) -> impl Future<Output = anyhow::Result<()>> + Send;

    fn get_fan_curves(&self, profile: PlatformProfile) -> impl Future<Output = anyhow::Result<Vec<FanCurve>>> + Send;
    fn set_fan_curve(&self, profile: PlatformProfile, curve: &FanCurve) -> impl Future<Output = anyhow::Result<()>> + Send;
    fn set_fan_curves_enabled(&self, profile: PlatformProfile, enabled: bool) -> impl Future<Output = anyhow::Result<()>> + Send;
    fn set_curves_to_defaults(&self, profile: PlatformProfile) -> impl Future<Output = anyhow::Result<()>> + Send;

    fn get_aura_brightness(&self) -> impl Future<Output = anyhow::Result<u32>> + Send;
    fn set_aura_brightness(&self, level: u32) -> impl Future<Output = anyhow::Result<()>> + Send;
    fn get_aura_mode(&self) -> impl Future<Output = anyhow::Result<u32>> + Send;
    fn set_aura_mode(&self, mode: u32) -> impl Future<Output = anyhow::Result<()>> + Send;
    fn get_aura_mode_data(&self) -> impl Future<Output = anyhow::Result<AuraModeData>> + Send;
    fn set_aura_mode_data(&self, data: &AuraModeData) -> impl Future<Output = anyhow::Result<()>> + Send;
    fn get_aura_power(&self) -> impl Future<Output = anyhow::Result<AuraPowerState>> + Send;
    fn set_aura_power(&self, power: &AuraPowerState) -> impl Future<Output = anyhow::Result<()>> + Send;

    fn get_gpu_mode(&self) -> impl Future<Output = anyhow::Result<GpuMode>> + Send;
    fn get_gpu_supported(&self) -> impl Future<Output = anyhow::Result<Vec<GpuMode>>> + Send;
    fn get_gpu_power(&self) -> impl Future<Output = anyhow::Result<GpuPowerState>> + Send;
    fn get_gpu_pending_mode(&self) -> impl Future<Output = anyhow::Result<GpuMode>> + Send;
    fn get_gpu_pending_action(&self) -> impl Future<Output = anyhow::Result<PendingUserAction>> + Send;
    fn set_gpu_mode(&self, mode: GpuMode) -> impl Future<Output = anyhow::Result<PendingUserAction>> + Send;

    fn get_armoury_attributes(&self) -> impl Future<Output = anyhow::Result<Vec<ArmouryAttribute>>> + Send;
    fn set_armoury_attribute(&self, node_name: &str, value: i32) -> impl Future<Output = anyhow::Result<()>> + Send;
    fn restore_armoury_default(&self, node_name: &str) -> impl Future<Output = anyhow::Result<()>> + Send;

    fn read_sensors(&self) -> impl Future<Output = anyhow::Result<SensorData>> + Send;
}
