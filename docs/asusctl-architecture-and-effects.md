# ASUS Architecture, DBus Services, and Software Effects in rogtui

This document provides a comprehensive technical overview of the ASUS hardware control architecture, system interfaces, DBus services, keyboard lighting pipelines, and performance overhead metrics implemented in `rogtui`.

---

## 1. Machine Architecture & System Environment

- **Target Machine:** ASUS TUF Gaming F16 (FX608JH)
- **Operating System:** CachyOS rolling (x86-64-v3, Linux BORE / EEVDF kernel)
- **Processor:** Intel Core i5-13450HX (6 P-cores + 4 E-cores / 16 threads, up to 4.60 GHz)
- **Graphics Pipeline:**
  - Integrated GPU: Intel UHD Graphics (`LIBVA_DRIVER_NAME=iHD`)
  - Discrete GPU: NVIDIA GeForce RTX 5050 Mobile (Optimus hybrid / Direct MUX)
- **Display Server:** Wayland compositor (Sway / wlroots) @ 1920x1200 165Hz
- **System Daemons:** `asusd` v6.5.0, `supergfxd` v5.2.7, `pipewire` v1.4.x with WirePlumber

---

## 2. Hardware Control Architecture & Interfaces

`rogtui` communicates with laptop hardware through three distinct tiers:

```
+-------------------------------------------------------------------------+
|                                rogtui                                   |
|   (Fast, event-driven Ratatui TUI with 1 Hz opt-in foreground ticker)   |
+-------------------+--------------------+--------------------------------+
                    |                    |
        (Direct Linux Sysfs)        (System DBus)
                    |                    |
         +----------v---------+  +-------v-------------------------+
         | /sys/class/leds/   |  | xyz.ljones.Asusd                |
         | asus::kbd_backlight|  |  - xyz.ljones.Platform          |
         |                    |  |  - xyz.ljones.FanCurves         |
         | /sys/class/hwmon/  |  |  - xyz.ljones.Aura              |
         | /proc/stat, etc.   |  |  - xyz.ljones.AsusArmoury       |
         +----------+---------+  +-------+-------------------------+
                    |                    |
                    |            +-------v-------------------------+
                    |            | org.supergfxctl.Daemon          |
                    |            |  - /org/supergfxctl             |
                    |            +-------+-------------------------+
                    |                    |
+-------------------v--------------------v--------------------------------+
|                 Linux Kernel & Embedded Controller (EC)                 |
|   drivers/platform/x86/asus-wmi.c + drivers/platform/x86/asus-nb-wmi.c   |
+-------------------------------------------------------------------------+
```

### 2.1 Direct Sysfs vs System DBus Separation

1. **Hardware Power & Profile Management (via DBus):**
   - Service: `xyz.ljones.Asusd` at `/xyz/ljones`
   - Platform Profile interface: `xyz.ljones.Platform`
   - Fan Curve interface: `xyz.ljones.FanCurves`
   - Armoury attributes: `xyz.ljones.AsusArmoury`
2. **GPU Switching & MUX Control (via DBus):**
   - Service: `org.supergfxctl.Daemon` at `/org/supergfxctl`
   - Interface: `org.supergfxctl.Daemon`
3. **Telemetry & Sensors (Direct Sysfs / Kernel IPC):**
   - Temperatures & Fan RPM: Direct traversal of `/sys/class/hwmon/hwmon*/`
   - CPU Usage: Delta calculation from `/proc/stat`
   - CPU Frequency: Package core average from `/sys/devices/system/cpu/cpufreq/policy*/scaling_cur_freq`
   - Memory Usage: Direct parsing of `/proc/meminfo`
   - Battery Health & Discharge: Direct parsing of `/sys/class/power_supply/BAT0/`
   - dGPU Power State: Direct sysfs inspection to prevent waking D3cold sleeping dGPU

---

## 3. Keyboard Lighting Architecture on ASUS TUF

### 3.1 Firmware Architecture (TUF vs ROG)

ASUS laptops handle keyboard backlighting through two divergent hardware architectures:

1. **ASUS ROG (Strix / Zephyrus / Scar):**
   - Utilizes USB-HID direct addressing with multi-zone or per-key RGB controllers.
   - Controlled via raw USB HID packets or `xyz.ljones.Aura.DirectAddressingRaw`.
2. **ASUS TUF (A15, A16, F15, F16):**
   - Employs a single-zone uniform matrix driven by the ASUS Embedded Controller (EC) over ACPI WMI.
   - Handled directly by the Linux kernel module `asus-nb-wmi` via `/sys/class/leds/asus::kbd_backlight/kbd_rgb_mode`.

### 3.2 Kernel Sysfs ABI: `kbd_rgb_mode`

The sysfs interface `/sys/class/leds/asus::kbd_backlight/kbd_rgb_mode` is defined in `drivers/platform/x86/asus-wmi.c`. It accepts space-separated integers formatted as:

```
<cmd> <mode> <red> <green> <blue> <speed>
```

- **`cmd` Field (Command Type):**
  - `0`: Set volatile in EC RAM (`0xb3`). Non-persistent; updates live LEDs immediately without touching BIOS flash NVRAM. Safe for high-frequency animation loops.
  - `1`: Save persistent to BIOS NVRAM (`0xb4`). Writes the configuration into flash memory so it persists across reboots. Intended only for explicit saves.
- **`mode` Field (Firmware Presets):**
  - `0`: Static (Single uniform color)
  - `1`: Breathe (Two-color cycling respiration)
  - `2`: Rainbow Cycle (Hardware color spectrum loop)
  - `10`: Pulse (Periodic warning-style strobe)
  - *(Note: Kernel clamps modes 9 and 12 to mode 10; other modes revert to static on TUF ECs).*
- **`red`, `green`, `blue`:** Unsigned 8-bit integers (`0` to `255`).
- **`speed`:** `0` (Slow), `1` (Medium), `2` (Fast).

### 3.3 Why Direct Volatile Writes Are Mandatory for Software Effects

When calling `set_led_mode_data` on `xyz.ljones.Aura` via DBus:
1. `asusd` persists the new data directly to the NVMe disk at `/etc/asusd/aura_tuf.ron` on every invocation.
2. `asusd` writes to `/sys/class/leds/asus::kbd_backlight/kbd_rgb_mode` using `cmd = 1` (save to BIOS flash).
3. Invoking DBus writes at 1 Hz causes unnecessary disk I/O, potential NVRAM wear, and DBus context-switching overhead.

`rogtui` solves this by introducing direct volatile writes (`cmd = 0`) via [`RealBackend::write_volatile_rgb`](file:///home/onyx/Projects/rogtui/src/backend/real.rs):
- **Fast Execution:** Updates in under 1 millisecond directly from user space.
- **Zero Disk Writes:** Never writes to `/etc/asusd/aura_tuf.ron` during effect playback.
- **Zero Flash Wear:** Modifies only EC volatile RAM registers.
- **Unprivileged Access:** Backed by `/etc/udev/rules.d/99-asus-kbd-backlight.rules`:
  ```udev
  ACTION=="add|change", SUBSYSTEM=="leds", KERNEL=="asus::kbd_backlight", RUN+="/bin/chmod 0666 /sys/%p/kbd_rgb_mode /sys/%p/kbd_rgb_state"
  ```

---

## 4. Feature Set by Tab

### Tab 1: Dashboard (`Dash`)
- **System Telemetry:** Live processor model, core frequency, CPU usage %, RAM utilization, dual fan RPM gauges, CPU temperature sparkline (60 samples).
- **Dual-GPU Status:** Real-time state of Intel UHD iGPU and NVIDIA RTX 5050 Mobile dGPU without waking sleeping dGPU in D3cold.
- **Battery Diagnostics:** Capacity percentage, active charge threshold limit, wattage rate, battery wear health %, cycle count.
- **Quick Actions:** Instant toggles for Panel Overdrive (3ms response), Screen Auto-Brightness (ALS), and Telemetry CSV Logging ('l').
- **Advanced Power Tuning Modal ('A'):** View and configure CPU SPL (PL1), sPPT (PL2), and GPU Total Graphics Power (TGP).

### Tab 2: Power Management (`Power`)
- **Profile Management:** Quiet, Balanced, Performance platform profile switching.
- **AC / Battery Auto-Binding:** Automatically switches profiles when AC adapter is plugged or unplugged.
- **Battery Charge Limit:** Configurable 20% to 100% threshold slider.
- **One-Shot Full Charge:** Charges battery to 100% once before resetting to threshold.
- **Energy Preference:** Linked EPP toggle and NVIDIA powerd on-battery suppression.

### Tab 3: Fan Curves (`Fan`)
- **Dual Fan Calibration:** Dedicated 8-point curves for CPU and GPU fans.
- **Monotonic Clamping & Repair:** Prevents invalid non-monotonic configurations; repairs malformed loaded curves.
- **Safety Policy:** Hard enforcement that fan speed cannot drop below 30% at or above 80 C.
- **High-Resolution Braille Curve Graph:** Densified continuous line rendering with labelled axes and live temperature tracking marker.
- **Presets & Copy:** Quiet, Balanced, Aggressive curve presets; option to mirror curves across all fans simultaneously.

### Tab 4: Aura Keyboard RGB (`Aura`)
- **Brightness Levels:** Off (0), Low (1), Medium (2), High (3) with hardware guardrails that disable effect execution when brightness is 0.
- **Hardware Modes:** Static, Breathe, Rainbow Cycle, Pulse.
- **Interactive Color Picker Modal ('c' or Enter):** Full 24-bit RGB and HSV interactive color picker with real-time hardware preview.
- **Curated Palettes:** Dynamic System Theme (auto-detected Gruvbox, Catppuccin, Nord, Cyberpunk), Gruvbox, Catppuccin, Cyberpunk, Nord, Sunset Amber, Red, Green, Blue, White.
- **Hardware Power States:** Toggle keyboard backlight for Boot, Awake, Sleep, Shutdown.
- **12 Custom Software Effects:** Full descriptions below.

### Tab 5: GPU Switching (`GPU`)
- **supergfxctl Integration:** Supports Integrated, Hybrid, and AsusMuxDgpu (Direct MUX).
- **Session Indicators:** Reports pending user actions (e.g. Session Logout Required, System Reboot Required, None).
- **dGPU Hardware Gate:** Live display of dGPU ACPI gate and MUX status.

### Tab 6: ASUS Armoury (`Armoury`)
- **Firmware Attributes:** Direct access to ASUS ACPI WMI attributes (`charge_mode`, `panel_overdrive`, `dgpu_disable`, `gpu_mux_mode`, `screen_auto_brightness`).
- **Safety Confirmations:** Interactive confirmation dialog before modifying hardware-sensitive settings.

### Tab 7: Diagnostics & About (`About`)
- **Diagnostics Export ('y'):** Dumps complete hardware and daemon state report to `~/rogtui-diagnostics.txt`.

---

## 5. Software Effects & Performance Overhead

All software effects execute on a 1 Hz ticker loop in the foreground. They do not run in the background when `rogtui` is closed.

### 5.1 Effect Catalog

| Subcategory | Effect Name | Operation & Behavior |
| :--- | :--- | :--- |
| **Custom Animation** | **Rainbow Wave** | Smooth 360-degree color wheel rotation. Speed and direction adjust step delta. |
| **Custom Animation** | **Color Cycle** | Transitions sequentially through 7 primary spectrum colors. |
| **Custom Animation** | **Fire Flicker** | Warm organic fireplace embers using LCG pseudo-random variance across red, orange, and amber. |
| **Custom Animation** | **Strobe / Flash** | High-contrast blinking pulse between primary base color and black. |
| **Audio & Visual Sync** | **Voice Pulse** | Microphone speech-reactive illumination via clean mic (`ghelper-audio`). Brightens when speaking, rests when silent. |
| **Audio & Visual Sync** | **Audio Pulse** | Sound output reactive visualizer. Pulses keyboard illumination with music beat drops and audio volume. |
| **Audio & Visual Sync** | **Audio Wave** | Dynamic audio spectrum equalizer. Shifts keyboard color hue from deep blue to peak red based on playback volume. |
| **Audio & Visual Sync** | **Ambient Light** | Screen ambient sync. Samples center display pixel coordinates via Wayland `grim` to mirror display visuals. |
| **System Telemetry** | **Heatmap** | Maps live CPU and GPU thermals to color (Blue cool -> Green -> Yellow warm -> Red hot). |
| **System Telemetry** | **GPU Mode** | Reflects active GPU operating mode: Green = Integrated, Yellow = Hybrid, Orange = AsusMuxDgpu. |
| **System Telemetry** | **Battery Status** | Reflects charge state: Green = charging, Yellow = discharging, Red = low (<20%). |
| **System Telemetry** | **CPU Load** | Reflects real-time processor load: Green (<30%) -> Yellow (30-65%) -> Orange (65-85%) -> Red (>85%). |

### 5.2 Performance & Overhead Metrics

Empirically measured on an Intel Core i5-13450HX laptop on CachyOS:

| Effect Category | Measurement Mechanism | Wall-Clock Duration | CPU Execution Time | CPU Overhead (1 Hz) | Memory Impact |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Hardware Static / Firmware** | EC internal | 0 ms | 0.000 ms | **0.00%** | None |
| **In-Process Animations** | Internal math + direct sysfs write | < 0.05 ms | < 0.005 ms | **< 0.01%** | None |
| **Telemetry Indicators** | Direct `/proc` + `/sys` parsing | ~0.15 ms | ~0.020 ms | **< 0.01%** | None |
| **AmbientLight (Wayland)** | `grim -g "960,600 1x1" -t ppm -` | 6 to 13 ms | ~0.27 ms | **~0.03%** of 1 core | ~4 MiB transient |
| **VoicePulse / AudioPulse** | `parec` (800 bytes @ 8 kHz mono, 100 ms) | ~140 to 150 ms | ~0.90 ms | **~0.09%** of 1 core | ~5 MiB transient |

#### Detailed Cost Analysis

1. **Wayland Screen Sync (`grim`):**
   - Coordinates: `960,600 1x1`
   - Output: Raw binary PPM (14 bytes total stdout).
   - Because only a 1x1 pixel region is requested through `wlr-screencopy-unstable-v1`, no full-screen bitmap is rendered or kept in memory. Total CPU time is **0.27 ms per second**.
2. **Audio Sync (`parec`):**
   - Parameters: `--raw --latency-msec=20 --process-time-msec=20 --channels=1 --rate=8000 --format=u8`
   - Samples 800 bytes (100 ms slice).
   - PipeWire already mixes and routes active streams; attaching an uncompressed 8 kHz mono monitor socket consumes **~0.90 ms CPU per second**.
3. **Volatile EC Writes:**
   - Writing directly to `/sys/class/leds/asus::kbd_backlight/kbd_rgb_mode` takes **< 1 ms** kernel time with 0 disk writes and 0 NVRAM writes.

---

## 6. Command Line Interface

```bash
rogtui [OPTIONS] [COMMAND]

Commands:
  probe-modes   Sweep keyboard RGB modes 0-12 interactively to audit physical presets

Options:
  --mock        Run with simulated mock hardware (for testing without daemons)
  -h, --help    Display help message
  -v, --version Display version information
```
