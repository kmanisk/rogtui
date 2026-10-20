# rogtui

[![Crates.io](https://img.shields.io/crates/v/rogtui.svg)](https://crates.io/crates/rogtui)
[![License: MIT / Apache-2.0](https://img.shields.io/badge/license-MIT%20%2F%20Apache--2.0-blue.svg)](LICENSE)
[![Platform: Linux](https://img.shields.io/badge/platform-Linux%20x86__64-lightgrey.svg)](https://kernel.org)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()
[![Idle CPU: 0.0%](https://img.shields.io/badge/idle%20cpu-0.0%25-brightgreen.svg)]()

Zero-overhead, keyboard-driven hardware control center for ASUS ROG and TUF gaming laptops running Linux.

Built in pure Rust (`ratatui`, `crossterm`, `tokio`, `zbus`) to directly communicate with `asusd` and `supergfxd` via system D-Bus. Designed as a high-performance, resource-minimal replacement for heavy GUI control centers like Armoury Crate, ASUS Control Center, or G-Helper.

![rogtui demonstration](assets/demo.gif)

> For full-resolution 60 FPS video demonstration, see [assets/demo.mp4](assets/demo.mp4).

---

## Technical Architecture & Invariants

- **Startup Latency:** Less than 2 milliseconds from invocation to initial render.
- **Idle CPU Footprint:** 0.0% CPU usage while running. Redraws are strictly event-driven (terminal events, user keystrokes, or incoming D-Bus property changes). No polling game loops.
- **Memory Footprint:** Approximately 4.0 MiB RSS resident memory.
- **Binary Footprint:** ~2.4 MiB compiled stripped ELF binary.
- **Asynchronous IPC:** Native system D-Bus interfaces (`xyz.ljones.Asusd`, `org.supergfxctl.Daemon`) without subprocess forks or shell exec wrappers during normal operation.
- **Safety Clamps:** Monotonic non-decreasing fan curve validation prevents thermal runaway; minimum 30% PWM floor enforced at temperatures above 80°C.
- **Zero Text Clipping:** Responsive layout architecture adapts across terminal sizes (tested from minimal 80x24 up to full 4K ultra-wide).

---

## Hardware Compatibility

`rogtui` supports all ASUS gaming laptops powered by the Linux kernel ASUS WMI drivers and the `asus-linux.org` daemon stack (`asusd` / `supergfxd`):

| Product Family | Supported Models |
| :--- | :--- |
| **ROG Zephyrus** | G14, G15, G16, M16, Duo 15/16 (SE/GX series), Slash LED models |
| **ROG Strix / SCAR** | Strix G15/G16/G17/G18, SCAR 15/16/17/18 (G533, G733, G634, G834) |
| **TUF Gaming** | A15, A16, F15, F16, Dash F15 (FA506, FA507, FX506, FX507, FX607, FX608) |
| **ROG Flow** | X13, X16, Z13 tablet |
| **ROG Handhelds** | ROG Ally (RC71L), ROG Ally X |

### System Prerequisites
- Linux Kernel 6.1+ with `asus_wmi` and `asus_nb_wmi` modules loaded
- `asusd` v6.5.0 or newer (`systemctl is-active asusd`)
- `supergfxd` v5.2.7 or newer (`systemctl is-active supergfxd`) for dynamic GPU switching

---

## Tab Overview & Feature Matrix

### 1. Dash (Overview & Live Telemetry)
- **Live Wattage Tracking:** Aligned CPU package power consumption via Intel RAPL microjoule delta counter (`intel-rapl:0/energy_uj`), dGPU board power via dynamic NVML query (reporting `D3cold (0 W)` when offloaded GPU is suspended), and live battery charge/discharge rate.
- **Dual Temperature History:** Full-width dual Braille graphs displaying recent 2-minute temperature history for CPU and GPU over a linear 30-100°C scale.
- **Aligned Specifications:** Precise vertical alignment of CPU model, dGPU model, and battery capacity indicators.
- **Persistent Profile Indicator:** Quick profile switcher indicator (`p`) remains visible across view changes.

### 2. Power (Profiles, Triggers & Battery Lifespan)
- **Platform Profiles:** Instant switching between Quiet, Balanced, and Performance modes.
- **AC/DC Automatic Profile Transitions:** Define custom platform profiles triggered automatically on AC connection (`When plugged in`) vs disconnection (`On battery`).
- **Battery Health Limiter:** Stepped slider from 40% to 100% charge threshold with optimal lifespan indicators at 60% and 80%.
- **One-Shot Travel Override:** `[ Start ↵ ]` triggers a temporary charge to 100% for travel, automatically disengaging and reverting to configured lifespan limit upon charger disconnect.

### 3. Fan (8-Point Curve Calibration Studio)
- **True 8-Point EC Curves:** Calibrate independent PWM-to-temperature response curves for CPU and GPU fans.
- **Operating State Marker:** Real-time indicator (`◆`) plots current thermal and fan operating coordinates along the active curve.
- **Validation Engine:** Enforces non-decreasing monotonicity across points and prevents unsafe low fan speeds at high temperatures.
- **Hardware Integration:** Toggle custom curves on/off (`c`), apply staged changes to hardware EC (`w`), or restore factory BIOS curves (`r`).

### 4. RGB (Firmware Modes, Software Animations & Audio Reactive)
- **4-Stage Brightness:** Off, Low, Medium, and High states with instant blackout guardrails.
- **Firmware Modes:** Static, Breathe (with independent Primary & Secondary color synchronization, speed, and direction), Rainbow Cycle, and Pulse.
- **Software Animations:** Rainbow Wave (360-degree spectrum sweep), Color Cycle, Fireplace Embers, Clouds, Strobe / Flash, Heatmap (temperature-reactive palette), GPU Mode, Battery Status, and CPU Load.
- **Audio Pulse & Reactive Spectrum:**
  - **Voice Pulse:** Speech-reactive lighting via the internal microphone (`ghelper-audio` / clean audio inputs prioritized) with an adjustable sensitivity ratio meter.
  - **Audio Pulse:** Amplitude-reactive illumination driven by speaker output.
  - **Audio Wave:** Continuous Cava-style frequency spectrum EQ lighting with dynamic hue interpolation.
- **Desktop Synchronization:**
  - **Ambient Screen Sync:** Wayland desktop screen color sampling via `grim`.
  - **Ambient Audio Sync:** Merges desktop color sampling with live audio amplitude dynamics.
  - **Dynamic Theme Sync:** Dynamically inspects desktop theme and terminal colors (`~/.local/state/theme/colors.sh`), prioritizing vibrant accent colors.
- **Color Tools:** 24-bit RGB/HSV color picker modal (`c`) and direct 6-digit hex input buffer (`#`).
- **Power State Control:** Discrete toggles for Boot, Awake, Sleep, and Shutdown lighting states.
- **Slash LED Support:** Automatically detected for Zephyrus lids with dedicated animation and brightness controls.

### 5. System (Display, Graphics & Diagnostics)
- **Display Configuration:** Panel Overdrive (3ms response overdrive toggle) and Screen Auto-Brightness.
- **Graphics MUX Integration:** View and trigger dedicated GPU MUX switching modes (Integrated, Hybrid, AsusMuxDgpu).
- **Diagnostics Export:** One-key diagnostic generation (`y` or `[ Run ↵ ]`) writing full system health, sysfs, and D-Bus reports to `~/.local/state/rogtui/report-<timestamp>.md`.

---

## Visual Showcase

| Dashboard & Telemetry | Fan Curve Studio |
| :---: | :---: |
| ![Dashboard](assets/screenshot_dashboard.png) | ![Fan Curves](assets/screenshot_fan.png) |

| Power & Battery Lifespan | RGB Lighting & Reactive Voice |
| :---: | :---: |
| ![Power Management](assets/screenshot_power.png) | ![RGB Lighting](assets/screenshot_rgb.png) |

---

## Installation

### Arch Linux / CachyOS (AUR)

Install via your preferred AUR helper:

```bash
# Using yay
yay -S rogtui-git

# Using paru
paru -S rogtui-git
```

Or build locally with `makepkg`:

```bash
git clone https://github.com/manisk/rogtui.git
cd rogtui
makepkg -si
```

### Cargo (Crates.io)

```bash
cargo install rogtui
```

### Build from Source

Ensure Rust and Cargo (version 1.80 or later) are installed:

```bash
git clone https://github.com/manisk/rogtui.git
cd rogtui

# Compile release binary with Link-Time Optimization (LTO)
cargo build --release

# Install binary to local user bin
install -Dm755 target/release/rogtui ~/.local/bin/rogtui
```

---

## Background Daemon (`rogtui-close` / `--daemon`)

Software RGB animations, reactive microphone meters, and screen color sync can persist in the background after closing the TUI interface with negligible resource utilization:

- **Resource Consumption:** Consistently less than 0.2% CPU and ~6 MiB RSS.
- **Launching Daemon:** Run `rogtui --daemon` or trigger background detach upon exit.
- **Terminating Daemon:** Run `rogtui-close` or `pkill -f "rogtui --daemon"`.

---

## CLI Options

```bash
rogtui [OPTIONS] [COMMAND]

Commands:
  probe-modes       Sweep keyboard RGB modes 0-12 interactively to audit physical presets

Options:
  --daemon          Run active software RGB effect in the background headlessly
  --check           Validate system support, daemons, and exit with diagnostic report
  --mock            Run with simulated mock hardware (for testing without ASUS daemons)
  --no-color        Disable ANSI color sequences (monochrome mode)
  --ascii           Use pure ASCII box-drawing characters
  -h, --help        Display help message
  -v, -V, --version Display version information
```

---

## Keyboard Navigation

| Key | Context | Description |
| :--- | :--- | :--- |
| `Tab` / `Shift+Tab` | Global | Cycle to next / previous tab |
| `1` .. `5` | Global | Direct jump to tab (Dash, Power, Fan, RGB, System) |
| `j` / `k` or `Up` / `Down` | List Views | Move selection up / down |
| `h` / `l` or `Left` / `Right` | Controls | Decrease / increase value, toggle option, or cycle presets |
| `Enter` / `Space` | Controls | Execute action, confirm modal, toggle state, or open color picker |
| `c` | RGB / Fan | Open 24-bit RGB color picker (RGB) / toggle custom curve (Fan) |
| `#` | RGB | Enter 6-digit hex color directly (e.g. `ff4080`) |
| `p` | Global | Quick cycle active Platform Profile |
| `w` | Fan | Write staged fan curve to hardware EC |
| `r` | Fan | Reset selected fan curve to factory BIOS defaults |
| `s` | Fan | Cycle fan presets (Quiet, Balanced, Performance) |
| `f` | Fan | Toggle CPU / GPU fan curve selection |
| `y` | System | Export full system diagnostics report |
| `?` | Global | Open / close keyboard shortcuts overlay |
| `q` / `Esc` | Global | Close overlay / cancel confirmation modal / exit application |

---

## Troubleshooting

### Built-in System Check
Run the comprehensive self-check to verify permissions, kernel modules, and D-Bus interfaces:
```bash
rogtui --check
```

### Daemon Connection Failure
If `rogtui` reports `Error: Unable to connect to system DBus services`:
1. Verify `asusd` and `supergfxd` service states:
   ```bash
   sudo systemctl status asusd.service supergfxd.service
   ```
2. Enable and start missing services:
   ```bash
   sudo systemctl enable --now asusd.service supergfxd.service
   ```

### Running in Mock Mode
To test `rogtui` on non-ASUS systems, continuous integration runners, or virtual machines:
```bash
rogtui --mock
```

---

## Documentation

- [`docs/rogtui-hardware-control.md`](docs/rogtui-hardware-control.md): Deep-dive architecture of the 18 Aura modes, G-Helper screen & audio algorithms, fan curve mathematics, and power management.
- [`docs/asusctl-architecture-and-effects.md`](docs/asusctl-architecture-and-effects.md): Complete ASUS hardware architecture, kernel WMI sysfs ABI, D-Bus endpoints, and performance overhead metrics.
- [`docs/parity.md`](docs/parity.md): Full feature parity matrix comparing `rogtui` with ROG Control Center (`rog-control-center`).
- [`docs/keyboard-probe.md`](docs/keyboard-probe.md): Hardware verification and probe results for keyboard LED modes.
- [`docs/endpoints.md`](docs/endpoints.md): Complete mapping of all discovered D-Bus properties, methods, and sysfs paths to their UI components.

---

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
