use rogtui::{app, backend, handler, ui};
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

fn print_usage() {
    println!("rogtui - Minimal, fast hardware control center for ASUS ROG & TUF laptops");
    println!();
    println!("Usage: rogtui [OPTIONS] [COMMAND]");
    println!();
    println!("Commands:");
    println!("  probe-modes   Sweep keyboard RGB modes 0-12 interactively to audit physical presets");
    println!();
    println!("Options:");
    println!("  --mock        Run with simulated mock hardware (for testing without daemons)");
    println!("  -h, --help    Display this help message");
    println!("  -v, --version Display version information");
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_usage();
        return Ok(());
    }
    if args.iter().any(|a| a == "-v" || a == "--version") {
        println!("rogtui v0.1.0 (asusd v6.5.0 + supergfxd v5.2.7)");
        return Ok(());
    }
    if args.iter().any(|a| a == "probe-modes") {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        return rt.block_on(rogtui::probe::run_probe());
    }

    let use_mock = args.iter().any(|a| a == "--mock");

    // Construct single-threaded Tokio current_thread runtime
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    rt.block_on(async move {
        if use_mock {
            let backend = MockBackend::new();
            run_app(backend).await
        } else {
            match RealBackend::connect().await {
                Ok(backend) => run_app(backend).await,
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

async fn run_app<B: backend::Backend>(backend: B) -> anyhow::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen, crossterm::cursor::Hide)?;
    let term_backend = CrosstermBackend::new(out);
    let mut terminal = Terminal::new(term_backend)?;
    terminal.clear()?;

    let res = app_loop(&mut terminal, backend).await;

    // Restore terminal safely
    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen, crossterm::cursor::Show);
    let _ = terminal.show_cursor();

    res
}

async fn app_loop<B: backend::Backend>(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    backend: B,
) -> anyhow::Result<()> {
    let mut app = AppState::new(backend).await?;

    let mut event_reader = EventStream::new();
    let mut ticker = tokio::time::interval(Duration::from_millis(50));
    ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    let mut last_sensor_time = std::time::Instant::now() - Duration::from_secs(5);
    let mut last_effect_time = std::time::Instant::now() - Duration::from_secs(5);
    let mut last_draw_time = std::time::Instant::now() - Duration::from_secs(5);

    // Initial draw
    terminal.draw(|f| ui::render_ui(f, &app))?;

    while !app.should_quit {
        tokio::select! {
            maybe_event = event_reader.next() => {
                match maybe_event {
                    Some(Ok(Event::Key(key))) => {
                        handler::handle_key_event(&mut app, key).await;
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

                let effect_interval = match app.active_software_effect {
                    SoftwareEffect::VoicePulse | SoftwareEffect::AudioPulse | SoftwareEffect::AudioWave => Duration::from_millis(50),
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

                let should_redraw = did_refresh_sensors
                    || had_expired
                    || (did_apply_effect && matches!(app.current_tab, ActiveTab::Aura) && now.duration_since(last_draw_time) >= Duration::from_millis(70));

                if should_redraw {
                    terminal.draw(|f| ui::render_ui(f, &app))?;
                    last_draw_time = now;
                }
            }
        }
    }

    Ok(())
}
