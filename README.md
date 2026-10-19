# rogtui

Minimal, zero-overhead Terminal User Interface (TUI) for ASUS ROG and TUF gaming laptops running Linux.

Built in pure Rust (`ratatui`, `crossterm`, `tokio`, `zbus`) to directly interface with `asusd` and `supergfxd` via system DBus. Designed as a lean, responsive replacement for heavy GUI control centers like Armoury Crate or G-Helper.

---

## Technical Specifications

- **Startup Latency:** ~2 milliseconds
- **Idle CPU Utilization:** 0.0%
- **Memory Footprint (RSS):** ~4.0 MiB
- **Binary Size (Stripped):** 2.4 MiB
- **Layout Architecture:** Centered 104-column grid (or up to 132 for Fan) with dim horizontal dividers, fixed-column alignment, and zero clipped text across all terminal sizes (80x24 minimum).
- **Rendering Architecture:** 100% event-driven; redraws occur exclusively on user input, terminal resize, or DBus/theme state events. No fixed-FPS game loops. Telemetry sensors are sampled at 1.0 Hz strictly when telemetry views are active.
- **Protocol:** System DBus async proxies (`xyz.ljones.Asusd`, `org.supergfxctl.Daemon`) plus native sysfs/hwmon telemetry. No background daemons or shell forks during runtime.

---

## Target Platforms & Compatibility

- **Primary Validation:** ASUS TUF Gaming F16 (FX608JH) running CachyOS Linux (x86-64-v3, BORE kernel, Sway on Wayland).
- **Supported Daemon Versions:**
  - `asusd` v6.5.0+ (interface `xyz.ljones.*`)
  - `supergfxd` v5.2.7+ (interface `org.supergfxctl.Daemon`)
- **Compatibility:** All ASUS ROG Zephyrus, Strix, Flow, and TUF gaming series supported by `asus-linux.org`.

---

## Features

### 1. Dash (Tab 1)
- Unified grid rows for Processor, Graphics, Memory, and Battery with identical column alignment across metrics.
- RTX 5050 dGPU usage/temp/fan, displaying dim "Idle" when suspended or at 0% without rendering empty broken bars.
- Battery summary: percentage, current power draw in watts (e.g. `18.4 W`), tick mark (`╵`/`╷`) at charge threshold, and charging state.
- Full-width dual-line Braille temperature chart (last 2 minutes) for CPU and GPU with a linear 30-100°C axis.
- Quick profile cycling shortcut (`p`).

### 2. Power (Tab 2)
- Platform profile selector: Quiet, Balanced, Performance.
- Automatic profile bindings with "Switch automatically" subheading (`When plugged in: ‹...›`, `On battery: ‹...›`).
- Charge limit slider with tick marks at 60% and 80% (`40 ━━━━━━━━╷───────╷─────── 100   60%`) and battery lifespan advice.
- One-shot full charge override action row (`[ Start ↵ ]`) to temporarily charge to 100% for travel.
- Dedicated read-only Status subsection (Health, Cycles, Discharge/Rate) automatically bypassed by j/k navigation.

### 3. Fan (Tab 3)
- Clean header: `Fan ‹ CPU ›  f    Preset ‹ Balanced ›  s    Custom curve ● On  c`.
- Height 17 (`4k+1`) linear y-axis with 0%, 25%, 50%, 75%, 100% landing on exact character rows.
- Linear 30°C to 100°C x-axis tick marks and labels placed using the exact same mathematical formula as point markers: `col = plot_x + ((temp - 30.0) / 70.0 * (plot_w - 1)).round()`.
- Curve points rendered with `●` markers, active point highlighted, and live operating point indicated with `◆`.
- Right-hand calibration table with right-aligned numeric columns and `▌` gutter marker.
- Monotonic non-decreasing safety validation and minimum 30% PWM thermal clamp at >= 80°C.

### 4. RGB (Tab 4)
- 4-state segmented brightness control: `Off · Low · Med · High`.
- Contextual lighting controls: 18 effect modes including Static, Breathe, Color Cycle, Rainbow Wave, Audio Pulse, Voice Pulse, Ambient Screen Sync (Wayland grim), and Dynamic Theme.
- Dynamic Theme hot reloading: monitors `~/.local/state/theme/colors.sh` on each tick and syncs rising accent colors (Void purple `#bb9af7`, Scaryforest teal `#77beb4`, Gruvbox `#ebdbb2`, Catppuccin `#f5c2e7`, Nord `#88c0d0`).
- Color selection: Preset picker, custom hex with live `██` block preview, and direct hex entry modal (`#`).
- Tinted R, G, B channel sliders with high-contrast visible track.
- Compact hardware power state badges: `Keep lit during: [● Boot] [● Awake] [○ Sleep] [○ Shutdown]`.
- Interactive 24-bit RGB/HSV color picker modal (`c`).

### 5. System (Tab 5)
- Consolidated single view combining Display, Graphics, Diagnostics, and About information.
- Display controls: Panel Overdrive (3ms toggle) and Screen Auto-Brightness.
- Graphics: Clean read-only GPU mode display (`‹ Dedicated GPU (MUX) ›`) with reminder that mode changes take effect on next login.
- Diagnostics action: `Export report [ Run ↵ ]` writes comprehensive markdown reports to `~/.local/state/rogtui/report-<timestamp>.md`.
- About section: Model, Processor, Graphics, Memory, NVIDIA Driver version, OS, Session, Display mode, and ROGTUI version.

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

## Command Line Options

```bash
rogtui [OPTIONS] [COMMAND]

Commands:
  probe-modes       Sweep keyboard RGB modes 0-12 interactively to audit physical presets

Options:
  --check           Validate system support, daemons, and exit with diagnostic report
  --mock            Run with simulated mock hardware (for testing without daemons)
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
| `h` / `l` or `Left` / `Right` | Controls | Decrease / increase value, toggle, or cycle options |
| `Enter` / `Space` | Controls | Execute action, toggle state, or open color picker |
| `c` | RGB / Fan | Open 24-bit RGB color picker (RGB) / toggle custom curve (Fan) |
| `#` | RGB | Enter 6-digit hex color directly (e.g. `ff4080`) |
| `p` | Global | Quick cycle active Platform Profile |
| `w` | Fan / RGB | Apply and write staged configuration to hardware / BIOS |
| `s` | Fan | Cycle fan presets (Quiet, Balanced, Performance) |
| `f` | Fan | Toggle CPU / GPU fan curve selection |
| `y` | System | Export full system diagnostics report to `~/.local/state/rogtui/` |
| `?` | Global | Open / close keyboard shortcuts help overlay |
| `q` / `Esc` | Global | Close overlay / cancel confirmation / exit |

---

## Troubleshooting

### Compatibility Audit
Run the built-in system check:
```bash
rogtui --check
```

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
