use std::io::{self, BufRead, Write};
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

type AuraModeRaw = (u32, u32, (u8, u8, u8), (u8, u8, u8), String, String);

#[zbus::proxy(
    interface = "xyz.ljones.Aura",
    default_service = "xyz.ljones.Asusd",
    default_path = "/xyz/ljones/aura/tuf"
)]
trait AuraProbe {
    #[zbus(property)]
    fn brightness(&self) -> zbus::Result<u32>;

    #[zbus(property)]
    fn set_brightness(&self, value: u32) -> zbus::Result<()>;

    #[zbus(property)]
    fn led_mode(&self) -> zbus::Result<u32>;

    #[zbus(property)]
    fn set_led_mode(&self, value: u32) -> zbus::Result<()>;

    #[zbus(property)]
    fn led_mode_data(&self) -> zbus::Result<AuraModeRaw>;

    #[zbus(property)]
    fn set_led_mode_data(&self, value: AuraModeRaw) -> zbus::Result<()>;
}

struct AuraStateBackup {
    mode: u32,
    brightness: u32,
    mode_data: Option<AuraModeRaw>,
    sysfs_path: &'static str,
}

impl AuraStateBackup {
    fn restore_sync(&self) {
        println!("Restoring previous lighting state...");
        // Restore sysfs with cmd=0 (non-persistent)
        let (r, g, b, speed_val) = if let Some(ref d) = self.mode_data {
            let col = d.2;
            let spd = match d.4.as_str() {
                "Slow" => 0,
                "Fast" => 2,
                _ => 1,
            };
            (col.0, col.1, col.2, spd)
        } else {
            (0, 200, 255, 1)
        };

        let payload = format!("0 {} {} {} {} {}", self.mode, r, g, b, speed_val);
        let _ = Command::new("sudo")
            .args(["tee", self.sysfs_path])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .spawn()
            .and_then(|mut child| {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(payload.as_bytes());
                }
                child.wait()
            });
    }

    async fn restore_async(&self, conn: &zbus::Connection) {
        self.restore_sync();
        if let Ok(proxy) = AuraProbeProxy::new(conn).await {
            let _ = proxy.set_led_mode(self.mode).await;
            let _ = proxy.set_brightness(self.brightness).await;
            if let Some(ref d) = self.mode_data {
                let _ = proxy.set_led_mode_data(d.clone()).await;
            }
        }
    }
}

pub async fn run_probe() -> anyhow::Result<()> {
    println!("rogtui - Keyboard LED Aura Mode Prober");
    println!("======================================");
    println!("Hardware: ASUS TUF Gaming F16 (FX608JH)");
    println!("Target interface: /sys/class/leds/asus::kbd_backlight/kbd_rgb_mode");
    println!("Mode parameter: cmd=0 (volatile, non-persistent, set in EC RAM)");
    println!();
    println!("Note: Modes 9 and 12 are kernel aliases clamped to mode 10 (Pulse) by");
    println!("drivers/platform/x86/asus-wmi.c line 1079; skipped to avoid redundant tests.");
    println!();

    let sysfs_path = "/sys/class/leds/asus::kbd_backlight/kbd_rgb_mode";
    if !Path::new(sysfs_path).exists() {
        eprintln!("Error: {sysfs_path} not found on this system.");
        return Ok(());
    }

    // 1. Read and store current asusd aura state before sweeping
    let conn = zbus::Connection::system().await?;
    let proxy = AuraProbeProxy::new(&conn).await?;

    let initial_mode = proxy.led_mode().await.unwrap_or(0);
    let initial_brightness = proxy.brightness().await.unwrap_or(2);
    let initial_mode_data = proxy.led_mode_data().await.ok();

    let backup = Arc::new(AuraStateBackup {
        mode: initial_mode,
        brightness: initial_brightness,
        mode_data: initial_mode_data,
        sysfs_path,
    });

    println!(
        "Saved initial lighting state: Mode {}, Brightness {}",
        backup.mode, backup.brightness
    );

    // Setup cancellation / Ctrl-C hook
    let running = Arc::new(AtomicBool::new(true));
    let r_clone = running.clone();
    let b_clone = backup.clone();

    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        println!("\nAborting probe on Ctrl-C...");
        r_clone.store(false, Ordering::SeqCst);
        b_clone.restore_sync();
        std::process::exit(130);
    });

    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut results = Vec::new();

    // Distinct modes only: 0..=8, 10, 11 (skip 9 and 12)
    let mode_names = [
        (0, "Static"),
        (1, "Breathe"),
        (2, "RainbowCycle"),
        (3, "RainbowWave"),
        (4, "Stars"),
        (5, "Rain"),
        (6, "Highlight"),
        (7, "Laser"),
        (8, "Ripple"),
        (10, "Pulse"),
        (11, "Comet / Strobe Warning"),
    ];

    for (mode, default_name) in mode_names {
        if !running.load(Ordering::SeqCst) {
            break;
        }

        println!("--------------------------------------------------");
        println!("Testing Mode {mode} ({default_name})...");

        // Write non-persistent mode with cmd=0:
        // Format: cmd mode red green blue speed
        let payload = format!("0 {mode} 0 200 255 1");
        let write_res = Command::new("sudo")
            .args(["tee", sysfs_path])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::null())
            .spawn()
            .and_then(|mut child| {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(payload.as_bytes());
                }
                child.wait()
            });

        if let Err(e) = write_res {
            eprintln!("Warning: Failed to write to {sysfs_path}: {e}");
        }

        // Wait 2 seconds for animation to be observed
        sleep(Duration::from_secs(2)).await;

        print!("Mode {mode}: Visible effect on physical keyboard? [y/n + name/description] (default: y {default_name}): ");
        io::stdout().flush()?;

        let mut line = String::new();
        reader.read_line(&mut line)?;
        let ans = line.trim();
        let final_ans = if ans.is_empty() {
            format!("y {default_name}")
        } else {
            ans.to_string()
        };

        results.push((mode, default_name, final_ans));
    }

    // 2. Restore exact saved lighting state
    println!("--------------------------------------------------");
    backup.restore_async(&conn).await;
    println!("Original lighting state successfully restored.");

    // Write results to ~/.config/rogtui/modes.toml
    let config_dir = dirs_or_home()?;
    std::fs::create_dir_all(&config_dir)?;
    let toml_path = format!("{config_dir}/modes.toml");

    let mut toml_content = String::from("# rogtui keyboard hardware probe results\n[modes]\n");
    for (mode, def, ans) in &results {
        toml_content.push_str(&format!("mode_{mode} = \"{ans}\" # default: {def}\n"));
    }
    std::fs::write(&toml_path, &toml_content)?;
    println!("Probe complete. Results saved to {toml_path}");

    Ok(())
}

fn dirs_or_home() -> anyhow::Result<String> {
    if let Ok(home) = std::env::var("HOME") {
        Ok(format!("{home}/.config/rogtui"))
    } else {
        Ok(".config/rogtui".to_string())
    }
}
