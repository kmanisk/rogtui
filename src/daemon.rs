use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::app::AppState;
use crate::backend::real::RealBackend;
use crate::backend::types::{RgbColor, SoftwareEffect};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RgbDaemonState {
    pub effect: SoftwareEffect,
    pub brightness: u32,
    pub speed: String,
    pub color1: RgbColor,
    pub color2: RgbColor,
    pub voice_sensitivity: u8,
}

pub fn get_state_dir() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".local/state/rogtui")
    } else {
        PathBuf::from("/tmp/rogtui")
    }
}

pub fn get_state_file_path() -> PathBuf {
    get_state_dir().join("rgb_state.toml")
}

pub fn get_pid_file_path() -> PathBuf {
    get_state_dir().join("rgb_daemon.pid")
}

pub fn is_daemon_running() -> Option<u32> {
    let pid_file = get_pid_file_path();
    if let Ok(content) = fs::read_to_string(&pid_file) {
        if let Ok(pid) = content.trim().parse::<u32>() {
            let proc_path = PathBuf::from(format!("/proc/{pid}"));
            if proc_path.exists() {
                return Some(pid);
            }
        }
    }
    None
}

pub fn stop_daemon() -> Result<bool> {
    if let Some(pid) = is_daemon_running() {
        unsafe {
            libc::kill(pid as libc::pid_t, libc::SIGTERM);
        }
        std::thread::sleep(Duration::from_millis(60));
        let _ = fs::remove_file(get_pid_file_path());
        let _ = fs::remove_file(get_state_file_path());
        Ok(true)
    } else {
        let _ = fs::remove_file(get_pid_file_path());
        let _ = fs::remove_file(get_state_file_path());
        Ok(false)
    }
}

pub fn save_state<B: crate::backend::Backend>(app: &AppState<B>) -> Result<()> {
    let state = RgbDaemonState {
        effect: app.active_software_effect,
        brightness: app.aura_brightness,
        speed: app.aura_data.speed.clone(),
        color1: app.aura_data.color1,
        color2: app.aura_data.color2,
        voice_sensitivity: app.voice_sensitivity,
    };
    let dir = get_state_dir();
    let _ = fs::create_dir_all(&dir);
    let serialized = toml::to_string(&state).context("Failed to serialize RGB daemon state")?;
    fs::write(get_state_file_path(), serialized)?;
    Ok(())
}

pub fn load_state() -> Option<RgbDaemonState> {
    let path = get_state_file_path();
    if let Ok(content) = fs::read_to_string(path) {
        toml::from_str(&content).ok()
    } else {
        None
    }
}

pub fn spawn_daemon_process() {
    let current_exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("rogtui"));
    let _ = std::process::Command::new(current_exe)
        .arg("--daemon")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}

/// Run headless RGB daemon loop
pub async fn run_daemon() -> Result<()> {
    // If another daemon is already running, stop it first
    if let Some(pid) = is_daemon_running() {
        if pid != std::process::id() {
            unsafe {
                libc::kill(pid as libc::pid_t, libc::SIGTERM);
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    let state = match load_state() {
        Some(s) => s,
        None => return Ok(()),
    };

    if state.effect == SoftwareEffect::None || state.brightness == 0 {
        return Ok(());
    }

    // Write own PID
    let dir = get_state_dir();
    let _ = fs::create_dir_all(&dir);
    let my_pid = std::process::id();
    fs::write(get_pid_file_path(), my_pid.to_string())?;

    let backend = RealBackend::connect().await?;
    let mut app = AppState::new(backend).await?;

    app.active_software_effect = state.effect;
    app.aura_brightness = state.brightness;
    app.aura_data.speed = state.speed;
    app.aura_data.color1 = state.color1;
    app.aura_data.color2 = state.color2;
    app.voice_sensitivity = state.voice_sensitivity;
    app.user_custom_color = state.color1 != crate::ui::aura::SENTINEL_DYNAMIC_THEME;

    let effect_interval = match app.active_software_effect {
        SoftwareEffect::VoicePulse
        | SoftwareEffect::AudioPulse
        | SoftwareEffect::AudioWave
        | SoftwareEffect::AmbientAudioSync => Duration::from_millis(50),
        SoftwareEffect::RainbowWave
        | SoftwareEffect::ColorCycle
        | SoftwareEffect::FireFlicker
        | SoftwareEffect::Clouds
        | SoftwareEffect::ThemeSync
        | SoftwareEffect::Strobe => Duration::from_millis(70),
        SoftwareEffect::AmbientLight => Duration::from_millis(100),
        SoftwareEffect::Heatmap
        | SoftwareEffect::GpuMode
        | SoftwareEffect::BatteryStatus
        | SoftwareEffect::CpuLoad => Duration::from_millis(1000),
        SoftwareEffect::None => Duration::from_millis(1000),
    };

    let mut ticker = tokio::time::interval(effect_interval);
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    let mut last_sensor_time = std::time::Instant::now() - Duration::from_secs(5);

    use tokio::signal::unix::{signal, SignalKind};
    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;

    loop {
        tokio::select! {
            _ = sigterm.recv() => break,
            _ = sigint.recv() => break,
            _ = ticker.tick() => {
                let now = std::time::Instant::now();
                let needs_sensors = matches!(
                    app.active_software_effect,
                    SoftwareEffect::Heatmap
                        | SoftwareEffect::GpuMode
                        | SoftwareEffect::BatteryStatus
                        | SoftwareEffect::CpuLoad
                );

                if needs_sensors && now.duration_since(last_sensor_time) >= Duration::from_millis(1000) {
                    app.refresh_sensors().await;
                    last_sensor_time = now;
                }

                let is_dynamic_theme = app.active_software_effect == SoftwareEffect::ThemeSync
                    || app.aura_data.color1 == crate::ui::aura::SENTINEL_DYNAMIC_THEME
                    || !app.user_custom_color;

                if is_dynamic_theme {
                    let reloaded = app.check_and_reload_system_theme();
                    if reloaded {
                        app.apply_software_effect().await;
                    }
                }

                app.apply_software_effect().await;
            }
        }
    }

    let _ = fs::remove_file(get_pid_file_path());
    Ok(())
}
