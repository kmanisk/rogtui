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

### 1. Dash (Tab 1)
- Clean read-only system telemetry: cleaned CPU name, RTX 5050 dGPU usage/temp/fan (or "Idle" if suspended), RAM usage in GiB, and 60-second Unicode temperature sparklines.
- Battery summary: percentage, charge limit, health, and clean power state.
- Quick profile cycling shortcut (`p`).

### 2. Power (Tab 2)
- Platform profile selector: Quiet, Balanced, Performance.
- Merged AC and Battery binding profiles (`When plugged in: ‹...›`, `On battery: ‹...›` with "Off" option).
- Battery charge limit slider (`━━━━━────  60%`, 20% to 100% in 5% increments).
- One-shot full charge override action to temporarily charge to 100% for travel.
- Real battery health calculation and dynamic cycle count display (hidden if unpopulated).

### 3. Fan (Tab 3)
- Clean header: `Fan ‹ CPU ›   Preset ‹ Balanced ›   Custom ● Off/On`.
- Strictly linear x (30-100°C) and y (0-100%) axes with evenly spaced labels.
- Braille fan curve passing exactly through the 8 hardware points.
- Highlighted selected point marker, dim unselected markers, and live operating point crosshair.
- Monotonic non-decreasing safety validation and minimum 30% PWM thermal clamp at >= 80°C.

### 4. RGB (Tab 4)
- Backlight brightness slider (Off, Low, Medium, High).
- Contextual control filtering: only controls supported by the active mode are shown (hides animation speed, direction, and secondary color when in Static mode).
- Interactive 24-bit RGB/HSV color picker modal (`c`) with half-block hue strip and live swatch preview.
- Hardware power states folded into a single compact line (Boot, Awake, Sleep, Shutdown).
- Bottom contextual hint line explaining selected field or effect.

### 5. System (Tab 5)
- Consolidated single view combining Display, Graphics, and About information.
- Display controls: Panel Overdrive (3ms toggle) and Auto-Brightness.
- Graphics: Plain text GPU mode display with dynamic support detection.
- About: System model, BIOS version, kernel version, active session (`Sway · Wayland`), display mode (`1920×1200 · 165 Hz`), and ROGTUI version.

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
| `1` .. `5` | Global | Direct jump to tab (Dash, Power, Fan, RGB, System) |
| `j` / `k` or `Up` / `Down` | List Views | Move selection up / down |
| `h` / `l` or `Left` / `Right` | Controls | Decrease / increase value, toggle, or cycle options |
| `Enter` / `Space` | Controls | Execute action, toggle state, or open color picker |
| `c` | RGB | Open interactive 24-bit RGB/HSV color picker modal |
| `p` | Global | Quick cycle active Platform Profile |
| `w` | Fan / RGB | Apply and write staged configuration to hardware / BIOS |
| `s` | Fan | Cycle fan presets (Quiet, Balanced, Performance) |
| `f` | Fan | Toggle CPU / GPU fan curve selection |
| `y` | System | Export full system diagnostics report to `~/rogtui-diagnostics.txt` |
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
