use rogtui::{app, backend, handler, ui};
use rogtui::backend::Backend;
use rogtui::backend::types::SoftwareEffect;

use std::env;
use std::io::{self, stdout};
use std::time::Duration;

use crossterm::{
    event::{Event, EventStream},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures_lite::StreamExt;
use ratatui::{backend::CrosstermBackend, Terminal};

use app::{ActiveTab, AppState};
use backend::mock::MockBackend;
use backend::real::RealBackend;
use ui::theme::Theme;

fn print_usage() {
    println!("rogtui - Minimal, fast hardware control center for ASUS ROG & TUF laptops");
    println!();
    println!("Usage: rogtui [OPTIONS] [COMMAND]");
    println!();
    println!("Commands:");
    println!("  probe-modes    Sweep keyboard RGB modes 0-12 interactively to audit physical presets");
    println!("  aura <action>  Control RGB lighting directly (ideal for WM/DE keybindings)");
    println!("      next-mode      Cycle to next RGB effect / mode");
    println!("      prev-mode      Cycle to previous RGB effect / mode");
    println!("      next-preset    Cycle to next RGB color preset");
    println!("      prev-preset    Cycle to previous RGB color preset");
    println!("      toggle         Toggle keyboard backlight on/off");
    println!("      brightness <+|-|0-3> Adjust keyboard backlight brightness");
    println!("      status         Print current RGB mode, preset, and brightness");
    println!();
    println!("Options:");
    println!("  --check       Validate system support, daemons, and exit with diagnostic report");
    println!("  --mock        Run with simulated mock hardware (for testing without daemons)");
    println!("  --no-color    Disable ANSI color sequences (monochrome mode)");
    println!("  --ascii       Use pure ASCII box-drawing characters");
    println!("  --daemon      Run active software RGB effect headless in background");
    println!("  --close, -c   Stop persistent background RGB daemon and reset keyboard");
    println!("  -h, --help    Display this help message");
    println!("  -v, -V, --version Display version information");
}

async fn run_check() -> anyhow::Result<()> {
    println!("rogtui System Compatibility Audit");
    println!("─────────────────────────────────");

    let model = ui::system::detect_model();
    println!("  [OK] Model: {}", model);

    let kernel = ui::system::detect_kernel();
    println!("  [OK] Kernel: {}", kernel);

    let session = ui::system::detect_session();
    println!("  [OK] Session: {}", session);

    let mut all_ok = true;

    // Check system D-Bus
    let dbus_res = zbus::Connection::system().await;
    match &dbus_res {
        Ok(conn) => {
            println!("  [OK] D-Bus: system bus connected");

            // Check asusd
            let asusd_reply = conn
                .call_method(
                    Some("org.freedesktop.DBus"),
                    "/org/freedesktop/DBus",
                    Some("org.freedesktop.DBus"),
                    "GetNameOwner",
                    &("xyz.ljones.Asusd",),
                )
                .await;
            if asusd_reply.is_ok() {
                println!("  [OK] asusd: running (active)");
            } else {
                println!("  [WARN] asusd: not detected on D-Bus (may need: sudo systemctl start asusd)");
            }

            // Check supergfxd
            let supergfxd_reply = conn
                .call_method(
                    Some("org.freedesktop.DBus"),
                    "/org/freedesktop/DBus",
                    Some("org.freedesktop.DBus"),
                    "GetNameOwner",
                    &("org.supergfxctl.Daemon",),
                )
                .await;
            if supergfxd_reply.is_ok() {
                println!("  [OK] supergfxd: running (active)");
            } else {
                println!("  [WARN] supergfxd: not detected on D-Bus (optional for GPU switching)");
            }
        }
        Err(e) => {
            println!("  [FAIL] D-Bus: unable to connect to system bus: {e}");
            all_ok = false;
        }
    }

    // Check ASUS sysfs
    let asus_wmi = std::path::Path::new("/sys/devices/platform/asus-nb-wmi");
    if asus_wmi.exists() {
        println!("  [OK] Hardware: asus-nb-wmi platform driver loaded");
    } else {
        println!("  [INFO] Hardware: asus-nb-wmi not found (generic laptop mode)");
    }

    println!();
    if all_ok {
        println!("All checks passed. System is compatible with rogtui.");
        Ok(())
    } else {
        println!("Compatibility checks failed. See diagnostics above.");
        std::process::exit(1);
    }
}

async fn run_aura_cli(subargs: &[String]) -> anyhow::Result<()> {
    let backend = match RealBackend::connect().await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("[error] Failed to connect to ASUS hardware backend: {e}");
            std::process::exit(1);
        }
    };
    let mut app = AppState::new(backend).await?;
    if let Some(state) = rogtui::daemon::load_state() {
        app.active_software_effect = state.effect;
        app.aura_mode = state.aura_mode;
        if app.aura_brightness > 0 {
            app.aura_brightness = state.brightness;
        }
        app.aura_data.speed = state.speed;
        app.aura_data.color1 = state.color1;
        app.aura_data.color2 = state.color2;
        app.voice_sensitivity = state.voice_sensitivity;
        app.user_custom_color = state.color1 != rogtui::ui::aura::SENTINEL_DYNAMIC_THEME
            && state.color1 != rogtui::ui::aura::SENTINEL_SPECTRUM_CYCLE;
    }

    let is_ui_active = rogtui::daemon::is_ui_running();
    let cmd = if subargs[0] == "aura" || subargs[0] == "rgb" {
        subargs.get(1).map(|s| s.as_str()).unwrap_or("status")
    } else {
        subargs[0].as_str()
    };

    match cmd {
        "next-mode" | "next-effect" | "next" => {
            rogtui::handler::cycle_aura_mode(&mut app, 1).await;
            if !is_ui_active {
                if app.active_software_effect != rogtui::backend::types::SoftwareEffect::None && app.aura_brightness > 0 {
                    rogtui::daemon::spawn_daemon_process();
                } else {
                    let _ = rogtui::daemon::stop_daemon();
                }
            }
        }
        "prev-mode" | "prev-effect" | "prev" => {
            rogtui::handler::cycle_aura_mode(&mut app, -1).await;
            if !is_ui_active {
                if app.active_software_effect != rogtui::backend::types::SoftwareEffect::None && app.aura_brightness > 0 {
                    rogtui::daemon::spawn_daemon_process();
                } else {
                    let _ = rogtui::daemon::stop_daemon();
                }
            }
        }
        "next-preset" => {
            rogtui::handler::cycle_aura_preset(&mut app, 1).await;
            if !is_ui_active && app.active_software_effect != rogtui::backend::types::SoftwareEffect::None && app.aura_brightness > 0 {
                rogtui::daemon::spawn_daemon_process();
            }
        }
        "prev-preset" => {
            rogtui::handler::cycle_aura_preset(&mut app, -1).await;
            if !is_ui_active && app.active_software_effect != rogtui::backend::types::SoftwareEffect::None && app.aura_brightness > 0 {
                rogtui::daemon::spawn_daemon_process();
            }
        }
        "toggle" | "toggle-rgb" => {
            let next_b = if app.aura_brightness == 0 { 3 } else { 0 };
            let dir = next_b as i32 - app.aura_brightness as i32;
            rogtui::handler::cycle_aura_brightness(&mut app, dir).await;
            if !is_ui_active {
                if app.aura_brightness == 0 {
                    let _ = rogtui::daemon::stop_daemon();
                } else if app.active_software_effect != rogtui::backend::types::SoftwareEffect::None {
                    rogtui::daemon::spawn_daemon_process();
                }
            }
        }
        "brightness" => {
            let dir = match subargs.get(2).map(|s| s.as_str()) {
                Some("up") | Some("+") => 1,
                Some("down") | Some("-") => -1,
                Some("off") | Some("0") => 0 - app.aura_brightness as i32,
                Some("low") | Some("1") => 1 - app.aura_brightness as i32,
                Some("med") | Some("2") => 2 - app.aura_brightness as i32,
                Some("high") | Some("3") => 3 - app.aura_brightness as i32,
                _ => 1,
            };
            rogtui::handler::cycle_aura_brightness(&mut app, dir).await;
            if !is_ui_active {
                if app.aura_brightness == 0 {
                    let _ = rogtui::daemon::stop_daemon();
                } else if app.active_software_effect != rogtui::backend::types::SoftwareEffect::None {
                    rogtui::daemon::spawn_daemon_process();
                }
            }
        }
        "status" => {
            let mode_name = match app.active_software_effect {
                rogtui::backend::types::SoftwareEffect::None => match app.aura_mode {
                    1 => "Breathe",
                    2 => "Rainbow",
                    10 => "Pulse",
                    _ => "Static",
                },
                e => e.label(),
            };
            let preset_name = rogtui::ui::aura::get_active_preset_name(app.aura_data.color1);
            let b_names = ["Off", "Low", "Med", "High"];
            let b_str = b_names[app.aura_brightness.clamp(0, 3) as usize];
            println!("Aura Lighting: Mode = {mode_name} | Preset = {preset_name} | Brightness = {b_str}");
            return Ok(());
        }
        other => {
            eprintln!("[error] Unknown aura command '{other}'. Available: next-mode, prev-mode, next-preset, prev-preset, toggle, brightness, status");
            std::process::exit(1);
        }
    }

    if let Some((msg, _, _)) = app.status_message {
        println!("[ok] {msg}");
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_usage();
        return Ok(());
    }
    if args.iter().any(|a| a == "-v" || a == "-V" || a == "--version") {
        println!("rogtui v{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if args.iter().any(|a| a == "--ascii") {
        env::set_var("ROGTUI_ASCII", "1");
    }

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    if args.iter().any(|a| a == "--close" || a == "-c" || a == "close") {
        let stopped = rogtui::daemon::stop_daemon().unwrap_or(false);
        rt.block_on(async {
            if let Ok(backend) = RealBackend::connect().await {
                let _ = backend.write_volatile_rgb(rogtui::backend::types::RgbColor::new(0, 0, 0)).await;
            }
        });
        if stopped {
            println!("[ok] Persistent background RGB daemon stopped. Keyboard lighting reset.");
        } else {
            println!("[ok] No background RGB daemon was running.");
        }
        return Ok(());
    }
    if args.iter().any(|a| a == "--daemon") {
        return rt.block_on(rogtui::daemon::run_daemon());
    }

    if args.iter().any(|a| a == "--check") {
        return rt.block_on(run_check());
    }
    if args.iter().any(|a| a == "probe-modes") {
        return rt.block_on(rogtui::probe::run_probe());
    }
    if args.len() > 1
        && (args[1] == "aura"
            || args[1] == "rgb"
            || args[1] == "next-effect"
            || args[1] == "prev-effect"
            || args[1] == "next-preset"
            || args[1] == "prev-preset"
            || args[1] == "toggle-rgb")
    {
        return rt.block_on(run_aura_cli(&args[1..]));
    }

    let use_mock = args.iter().any(|a| a == "--mock");
    let no_color = args.iter().any(|a| a == "--no-color");

    rt.block_on(async move {
        if use_mock {
            let backend = MockBackend::new();
            run_app(backend, no_color).await
        } else {
            match RealBackend::connect().await {
                Ok(backend) => run_app(backend, no_color).await,
                Err(err) => {
                    eprintln!("Error: Unable to connect to system DBus services (asusd / supergfxd):");
                    eprintln!("  {err}");
                    eprintln!();
                    eprintln!("Troubleshooting:");
                    eprintln!("  1. Ensure asusd is active: sudo systemctl status asusd");
                    eprintln!("  2. Ensure supergfxd is active: sudo systemctl status supergfxd");
                    eprintln!("  3. To test the interface without hardware daemons, pass: rogtui --mock");
                    std::process::exit(1);
                }
            }
        }
    })
}

async fn run_app<B: backend::Backend>(backend: B, no_color: bool) -> anyhow::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen, crossterm::cursor::Hide)?;
    let term_backend = CrosstermBackend::new(out);
    let mut terminal = Terminal::new(term_backend)?;
    terminal.clear()?;

    let _ = rogtui::daemon::register_ui_process();

    let res = app_loop(&mut terminal, backend, no_color).await;

    rogtui::daemon::unregister_ui_process();

    // Restore terminal safely
    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen, crossterm::cursor::Show);
    let _ = terminal.show_cursor();

    res
}

async fn app_loop<B: backend::Backend>(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    backend: B,
    no_color: bool,
) -> anyhow::Result<()> {
    let mut app = AppState::new(backend).await?;
    if no_color {
        app.theme = Theme::monochrome();
    }

    // Restore persistent RGB effect and settings from saved state (whether daemon was running or not)
    if let Some(state) = rogtui::daemon::load_state() {
        app.active_software_effect = state.effect;
        app.aura_mode = state.aura_mode;
        if app.aura_brightness > 0 {
            app.aura_brightness = state.brightness;
        }
        app.aura_data.speed = state.speed;
        app.aura_data.color1 = state.color1;
        app.aura_data.color2 = state.color2;
        app.voice_sensitivity = state.voice_sensitivity;
        app.user_custom_color = state.color1 != rogtui::ui::aura::SENTINEL_DYNAMIC_THEME
            && state.color1 != rogtui::ui::aura::SENTINEL_SPECTRUM_CYCLE;
        if app.aura_brightness == 0 && app.active_software_effect != SoftwareEffect::None {
            app.saved_software_effect = Some(app.active_software_effect);
        }
    }
    if rogtui::daemon::is_daemon_running().is_some() {
        let _ = rogtui::daemon::stop_daemon();
    }

    let mut event_reader = EventStream::new();
    let mut ticker = tokio::time::interval(Duration::from_millis(50));
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    let mut last_sensor_time = std::time::Instant::now() - Duration::from_secs(5);
    let mut last_effect_time = std::time::Instant::now() - Duration::from_secs(5);
    let mut last_draw_time = std::time::Instant::now() - Duration::from_secs(5);
    let mut last_known_state_mtime = rogtui::daemon::get_state_file_mtime();

    // Initial draw
    terminal.draw(|f| ui::render_ui(f, &app))?;

    while !app.should_quit {
        tokio::select! {
            maybe_event = event_reader.next() => {
                match maybe_event {
                    Some(Ok(Event::Key(key))) => {
                        handler::handle_key_event(&mut app, key).await;
                        last_known_state_mtime = rogtui::daemon::get_state_file_mtime();
                        terminal.draw(|f| ui::render_ui(f, &app))?;
                        last_draw_time = std::time::Instant::now();
                    }
                    Some(Ok(Event::Resize(_, _))) => {
                        terminal.draw(|f| ui::render_ui(f, &app))?;
                        last_draw_time = std::time::Instant::now();
                    }
                    Some(Err(_)) => break,
                    None => break,
                    _ => {}
                }
            }
            _ = ticker.tick() => {
                let now = std::time::Instant::now();
                let had_expired = app.clear_expired_messages();

                // Hot reload external state if modified by CLI shortcut or external tool
                let mut did_sync_external_rgb = false;
                if let Some(mtime) = rogtui::daemon::get_state_file_mtime() {
                    let is_newer = match last_known_state_mtime {
                        Some(last) => mtime > last,
                        None => true,
                    };
                    if is_newer {
                        last_known_state_mtime = Some(mtime);
                        if let Some(state) = rogtui::daemon::load_state() {
                            let changed = app.active_software_effect != state.effect
                                || app.aura_mode != state.aura_mode
                                || app.aura_brightness != state.brightness
                                || app.aura_data.speed != state.speed
                                || app.aura_data.color1 != state.color1
                                || app.aura_data.color2 != state.color2
                                || app.voice_sensitivity != state.voice_sensitivity;

                            if changed {
                                app.active_software_effect = state.effect;
                                app.aura_mode = state.aura_mode;
                                app.aura_brightness = state.brightness;
                                app.aura_data.speed = state.speed;
                                app.aura_data.color1 = state.color1;
                                app.aura_data.color2 = state.color2;
                                app.voice_sensitivity = state.voice_sensitivity;
                                app.user_custom_color = state.color1 != rogtui::ui::aura::SENTINEL_DYNAMIC_THEME
                                    && state.color1 != rogtui::ui::aura::SENTINEL_SPECTRUM_CYCLE;
                                app.last_effect_color = None;

                                if rogtui::daemon::is_daemon_running().is_some() {
                                    let _ = rogtui::daemon::stop_daemon();
                                }

                                let mode_name = match app.active_software_effect {
                                    SoftwareEffect::None => match app.aura_mode {
                                        1 => "Breathe",
                                        2 => "Rainbow",
                                        10 => "Pulse",
                                        _ => "Static",
                                    },
                                    e => e.label(),
                                };
                                let preset_name = rogtui::ui::aura::get_active_preset_name(app.aura_data.color1);
                                app.set_status(format!("RGB Synced: {mode_name} ({preset_name})"), false);
                                did_sync_external_rgb = true;
                            }
                        }
                    }
                }

                let is_telemetry_tab = matches!(app.current_tab, ActiveTab::Dashboard | ActiveTab::FanCurves);
                let needs_sensors = is_telemetry_tab
                    || app.active_software_effect != SoftwareEffect::None
                    || app.is_logging_metrics;

                let mut did_refresh_sensors = false;
                if needs_sensors && now.duration_since(last_sensor_time) >= Duration::from_millis(1000) {
                    app.refresh_sensors().await;
                    last_sensor_time = now;
                    did_refresh_sensors = true;
                    if app.is_logging_metrics {
                        app.log_metrics_sample();
                    }
                }

                // Hot reload dynamic theme from colors.sh
                let did_reload_theme = if !no_color {
                    let reloaded = app.check_and_reload_system_theme();
                    let is_dynamic_active = app.active_software_effect == SoftwareEffect::ThemeSync
                        || app.aura_data.color1 == rogtui::ui::aura::SENTINEL_DYNAMIC_THEME;
                    if reloaded && is_dynamic_active {
                        app.apply_software_effect().await;
                    }
                    reloaded
                } else {
                    false
                };

                let effect_interval = match app.active_software_effect {
                    SoftwareEffect::VoicePulse | SoftwareEffect::AudioPulse | SoftwareEffect::AudioWave | SoftwareEffect::AmbientAudioSync => Duration::from_millis(50),
                    SoftwareEffect::RainbowWave | SoftwareEffect::ColorCycle | SoftwareEffect::FireFlicker | SoftwareEffect::Clouds | SoftwareEffect::ThemeSync | SoftwareEffect::Strobe => Duration::from_millis(70),
                    SoftwareEffect::AmbientLight => Duration::from_millis(100),
                    SoftwareEffect::Heatmap | SoftwareEffect::GpuMode | SoftwareEffect::BatteryStatus | SoftwareEffect::CpuLoad => Duration::from_millis(1000),
                    SoftwareEffect::None => Duration::from_millis(1000),
                };

                let mut did_apply_effect = false;
                if app.active_software_effect != SoftwareEffect::None && now.duration_since(last_effect_time) >= effect_interval {
                    app.apply_software_effect().await;
                    last_effect_time = now;
                    did_apply_effect = true;
                }

                let did_power_transition = app.check_and_apply_power_state_transition().await;

                let should_redraw = did_sync_external_rgb
                    || did_refresh_sensors
                    || had_expired
                    || did_reload_theme
                    || did_power_transition
                    || (did_apply_effect && matches!(app.current_tab, ActiveTab::Aura) && now.duration_since(last_draw_time) >= Duration::from_millis(70));

                if should_redraw {
                    terminal.draw(|f| ui::render_ui(f, &app))?;
                    last_draw_time = now;
                }
            }
        }
    }

    // Always persist state so next launch restores user's selected mode and colors
    let _ = rogtui::daemon::save_state(&app);

    // If a software RGB effect was active and backlight is on, run headless daemon in background
    if app.active_software_effect != SoftwareEffect::None && app.aura_brightness > 0 {
        rogtui::daemon::spawn_daemon_process();
    } else {
        let _ = rogtui::daemon::stop_daemon();
    }

    Ok(())
}
