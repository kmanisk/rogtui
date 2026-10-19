# rogtui

Minimal, zero-overhead Terminal User Interface (TUI) for ASUS ROG and TUF gaming laptops running Linux.

Built in pure Rust (`ratatui`, `crossterm`, `tokio`, `zbus`) to directly interface with `asusd` and `supergfxd` via system DBus. Designed as a lean, responsive replacement for heavy GUI control centers like Armoury Crate or G-Helper.

---

## Technical Specifications

- **Startup Latency:** ~2 milliseconds
- **Idle CPU Utilization:** 0.0%
- **Memory Footprint (RSS):** ~4.0 MiB
- **Binary Size (Stripped):** 2.4 MiB
- **Rendering Architecture:** 100% event-driven; redraws occur exclusively on user input, terminal resize, or DBus state events. No fixed-FPS game loops. Telemetry sensors are sampled at 1.0 Hz strictly when telemetry views are active.
- **Protocol:** System DBus async proxies (`xyz.ljones.Asusd`, `org.supergfxctl.Daemon`) plus native sysfs/hwmon telemetry. No background daemons or shell forks during runtime.

---

## Target Platforms & Compatibility

- **Primary Validation:** ASUS TUF Gaming F16 (FX608JH) running CachyOS Linux (x86-64-v3, BORE kernel).
- **Supported Daemon Versions:**
  - `asusd` v6.5.0+ (interface `xyz.ljones.*`)
  - `supergfxd` v5.2.7+ (interface `org.supergfxctl.Daemon`)
- **Compatibility:** All ASUS ROG Zephyrus, Strix, Flow, and TUF gaming series supported by `asus-linux.org`.

---

## Features

### 1. Dashboard (Tab 1)
- Live system telemetry: CPU package temperature (C), GPU core temperature (C), CPU fan RPM, GPU fan RPM.
- Real-time status: Active platform profile, GPU mode, dGPU power gate state, battery limit, and current battery percentage.
- Quick profile cycling shortcut (`p`).

### 2. Power Profiles & Charging (Tab 2)
- Platform profile selector: Quiet, Balanced, Performance.
- AC and Battery binding profiles (automatic profile switching upon power plug/unplug).
- Battery charge end threshold limiter (slider from 20% to 100% in 5% increments).
- One-shot full charge override: Charges battery to 100% for travel, resetting automatically upon disconnect.
- Energy Performance Preference (EPP) linkage and NVIDIA Dynamic Boost battery gating.

### 3. Fan Curve Editor (Tab 3)
- Per-fan interactive curve tuning (CPU and GPU fans) across all power profiles.
- Sub-pixel resolution Braille graph rendering 141 interpolated points (0.5 C steps) with live crosshairs and operating point indicator.
- 8 hardware control points per curve with monotonic non-decreasing safety validation.
- Safety policy enforcement: Refuses curves that set fan speed below 30% at or above 80 C.
- Factory BIOS reset (`r`) and one-key undo / revert to last-known-good curve (`u`).

### 4. Aura Keyboard Lighting (Tab 4)
- Backlight brightness control (Off, Low, Medium, High).
- 18 Lighting Modes across 4 distinct operational categories:
  - **Hardware Firmware:** Static, Breathe, Rainbow, Pulse (Zero host CPU overhead, executed in Embedded Controller).
  - **Software Animations:** RainbowFlow, BreathingColor, FireFlicker, StarryNight, SpectrumCycle, Clouds.
  - **Audio & Visual Synchronization:**
    - `AmbientSync`: Ported G-Helper `ApplyAmbient` Wayland screen sync (`0,400 1920x770` geometry crop, 0.05 downscale, +0.3 HSV saturation boost, 0.65 exponential smoothing).
    - `VoicePulse`: Real-time microphone reactive pulse with dynamic Automatic Gain Control (AGC), noise floor filter, and cubic easing.
    - `AudioPulse`: Real-time system audio playback reactive pulse.
    - `AudioWave`: Multi-harmonic audio frequency split lighting.
    - `ThemeSync`: Dynamic synchronization with active system desktop color palette (`~/.local/state/theme/colors.sh`).
  - **System Telemetry:** Real-time thermal and load reactive modes (CpuTemp, GpuTemp, BatteryLevel, CpuLoad).
- Dual-color RGB configuration with hex code representation and interactive 24-bit RGB/HSV color picker modal (`c`).
- Keyboard power state toggles: Boot, Awake, Sleep, and Shutdown.

### 5. GPU Mode Control (Tab 5)
- Native `supergfxctl` mode switching: Integrated, Hybrid, Dedicated, and AsusMuxDgpu (Direct hardware MUX bypass).
- Verification of hardware-supported modes to prevent unsupported mode configurations.
- Pending action indicators: Displays if session logout or system reboot is required.
- Confirmation prompt before triggering graphics mode switching.

### 6. Armoury & Firmware Attributes (Tab 6)
- Dynamic auto-discovery of all firmware attributes exposed by `xyz.ljones.AsusArmoury`:
  - Panel Overdrive (3ms response time toggle)
  - Charge Mode (Fast, Standard, Eco)
  - dGPU ACPI Hardware Disable
  - Display MUX Switch
  - Screen Auto-Brightness (ALS)
- Hardware defaults restoration per attribute (`r`).
- Graceful degradation: Unavailable attributes on specific boards are cleanly marked and shielded from accidental writes.

---

## Installation

### Prerequisites

Ensure `asusd` and `supergfxd` are installed and running:

```bash
systemctl is-active asusd supergfxd
```

### Build from Source

```bash
# Clone the repository
git clone https://github.com/manisk/rogtui.git
cd rogtui

# Compile release binary with fat LTO and full optimizations
cargo build --release

# Install binary to user PATH
install -Dm755 target/release/rogtui ~/.local/bin/rogtui
```

### Arch Linux / CachyOS (PKGBUILD)

A standalone `PKGBUILD` is provided in the repository root:

```bash
makepkg -si
```

---

## Keyboard Navigation

| Key | Context | Description |
| :--- | :--- | :--- |
| `Tab` / `Shift+Tab` | Global | Cycle to next / previous tab |
| `1` .. `7` | Global | Direct jump to tab |
| `j` / `k` or `Up` / `Down` | List Views | Move selection up / down |
| `h` / `l` or `Left` / `Right` | Controls | Decrease / increase value, toggle, or navigate effect columns |
| `Enter` / `Space` | Controls | Execute action, confirm, cycle, or open color picker |
| `c` | Aura | Open interactive 24-bit RGB/HSV color picker modal |
| `p` | Global | Quick cycle active Platform Profile |
| `l` | Dashboard | Toggle real-time telemetry metrics logging to CSV |
| `A` | Dashboard | Open Advanced Power Limits modal (SPL, sPPT, TGP) |
| `w` | Fan / Aura | Apply and write staged configuration to hardware / BIOS |
| `r` | Fan / Armoury | Restore factory default settings |
| `u` | Fan | Undo / revert to last-known-good curve |
| `y` | About | Export full system diagnostics report to `~/rogtui-diagnostics.txt` |
| `?` | Global | Open / close keyboard shortcuts help overlay |
| `q` / `Esc` | Global | Close overlay / cancel confirmation / exit |

---

## Troubleshooting

### Daemon Connection Failure
If `rogtui` reports `Error: Unable to connect to system DBus services`:
1. Check daemon status:
   ```bash
   sudo systemctl status asusd.service
   sudo systemctl status supergfxd.service
   ```
2. Start or enable the services:
   ```bash
   sudo systemctl enable --now asusd.service supergfxd.service
   ```

### Running Without Hardware (Mock Mode)
To inspect or test the interface on non-ASUS machines or in containerized environments:

```bash
rogtui --mock
```

---

## Documentation

- [`docs/rogtui-hardware-control.md`](docs/rogtui-hardware-control.md): Comprehensive guide to rogtui architecture, 18 Aura modes, G-Helper screen & audio algorithms, fan curve controls, and power profiles.
- [`docs/asusctl-architecture-and-effects.md`](docs/asusctl-architecture-and-effects.md): Complete ASUS hardware architecture, kernel WMI sysfs ABI, DBus endpoints, and performance overhead metrics.
- [`docs/parity.md`](docs/parity.md): Full feature parity matrix comparing `rogtui` with ROG Control Center (`rog-control-center`).
- [`docs/keyboard-probe.md`](docs/keyboard-probe.md): Hardware verification and interactive probe results for TUF keyboard LED modes.
- [`docs/endpoints.md`](docs/endpoints.md): Complete mapping of all discovered DBus properties, methods, and sysfs paths to their UI counterparts.
- [`docs/dbus-introspection.txt`](docs/dbus-introspection.txt): Introspection dump of `xyz.ljones.Asusd` and `org.supergfxctl.Daemon`.
