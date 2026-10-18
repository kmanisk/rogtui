use std::io::{self, BufRead, Write};
use std::path::Path;
use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;

pub async fn run_probe() -> anyhow::Result<()> {
    println!("rogtui - Keyboard LED Aura Mode Prober");
    println!("======================================");
    println!("Hardware: ASUS TUF Gaming F16 (FX608JH)");
    println!("Target interface: /sys/class/leds/asus::kbd_backlight/kbd_rgb_mode");
    println!("Mode parameter: cmd=0 (volatile, non-persistent, set in EC RAM)");
    println!();
    println!("This prober will sweep modes 0 through 12, holding each for 2 seconds with cyan color.");
    println!("For each mode, observe your physical keyboard and report what you see.");
    println!();

    let sysfs_path = "/sys/class/leds/asus::kbd_backlight/kbd_rgb_mode";
    if !Path::new(sysfs_path).exists() {
        eprintln!("Error: {sysfs_path} not found on this system.");
        return Ok(());
    }

    let stdin = io::stdin();
    let mut reader = stdin.lock();

    let mut results = Vec::new();

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
        (9, "Mode 9 (kernel maps to 10)"),
        (10, "Pulse"),
        (11, "Comet / Warning"),
        (12, "Flash (kernel maps to 10)"),
    ];

    for (mode, default_name) in mode_names {
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

        print!("Mode {mode}: Visible effect on physical keyboard? [y/n + name/notes] (default: y {default_name}): ");
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

    // Restore safe default (Mode 0 Static Cyan)
    println!("--------------------------------------------------");
    println!("Restoring default static lighting...");
    let _ = Command::new("sudo")
        .args(["tee", sysfs_path])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .spawn()
        .and_then(|mut child| {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(b"0 0 0 200 255 1");
            }
            child.wait()
        });

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
