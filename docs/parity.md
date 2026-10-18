# ROG Control Center (ROGCC) Feature Parity Matrix

This document maps all features from the official ASUS ROG Control Center (`rog-control-center`) GUI application to `rogtui` on the ASUS TUF Gaming F16 (FX608JH) running CachyOS.

---

## 1. System Control -> Dash & Power Tabs

| ROGCC GUI Element | Backend Source / DBus Endpoint | rogtui Location | Parity Status | Notes |
|---|---|---|---|---|
| CPU Temperature Gauge | `/sys/class/hwmon` (coretemp) | Tab 1: Dash (Thermals panel) | Full Parity | + 60-sample sparkline history |
| CPU Usage % | `/proc/stat` deltas | Tab 1: Dash (Hardware Monitor) | Full Parity | Computed per 1 Hz active tick |
| CPU Frequency (MHz) | `/sys/devices/system/cpu/cpufreq` | Tab 1: Dash (Hardware Monitor) | Full Parity | Live package average frequency |
| CPU Model String | `/proc/cpuinfo` | Tab 1: Dash (Hardware Monitor) | Full Parity | Intel Core i5-13450HX |
| iGPU Name & Temp | `/sys/class/drm/card1` | Tab 1: Dash (Hardware Monitor) | Full Parity | Intel UHD Graphics (iHD) |
| dGPU Name & Temp | `/sys/class/drm/card0` / NVML | Tab 1: Dash (Hardware Monitor) | Full Parity | NVIDIA GeForce RTX 5050 Mobile |
| dGPU Usage % | NVML / sysfs | Tab 1: Dash (Hardware Monitor) | Full Parity | Guarded: Never wakes D3cold dGPU |
| Memory (RAM) Usage % | `/proc/meminfo` | Tab 1: Dash (Hardware Monitor) | Full Parity | MemTotal vs MemAvailable |
| CPU Fan RPM | `xyz.ljones.Asusd` / hwmon | Tab 1: Dash (Thermals panel) | Full Parity | Real-time gauge |
| GPU Fan RPM | `xyz.ljones.Asusd` / hwmon | Tab 1: Dash (Thermals panel) | Full Parity | Real-time gauge |
| Middle (Aux) Fan RPM | hwmon `fan3_input` | Tab 1: Dash (Thermals panel) | Not Supported Here | Hardware has dual fans only |
| Platform Profile Selector | `xyz.ljones.Platform.PlatformProfile` | Tab 1: Dash & Tab 2: Power | Full Parity | Quiet / Balanced / Performance |
| AC Profile Auto-Binding | `xyz.ljones.Platform.PlatformProfileOnAc` | Tab 2: Power (Profiles group) | Full Parity | Configurable per AC plug state |
| Battery Profile Auto-Binding | `xyz.ljones.Platform.PlatformProfileOnBattery` | Tab 2: Power (Profiles group) | Full Parity | Configurable per AC unplug state |
| Advanced Power Limits Modal | `xyz.ljones.AsusArmoury` / `ppt_pl1_spl` | Tab 1: Dash (Key 'A') | Full Parity | SPL, sPPT, TGP power tuning |
| Panel Overdrive Toggle | `xyz.ljones.AsusArmoury/panel_overdrive` | Tab 1: Dash & Tab 6: Armoury | Full Parity | Quick toggle on Dash |
| Screen Auto-Brightness (ALS) | `xyz.ljones.AsusArmoury/screen_auto_brightness` | Tab 1: Dash & Tab 6: Armoury | Full Parity | Quick toggle on Dash |

---

## 2. Keyboard Aura -> RGB Tab

| ROGCC GUI Element | Backend Source / DBus Endpoint | rogtui Location | Parity Status | Notes |
|---|---|---|---|---|
| Backlight Brightness | `xyz.ljones.Aura.Brightness` | Tab 4: RGB (Effect panel) | Full Parity | Off, Low, Med, High steppers |
| Aura Effect Mode | `xyz.ljones.Aura.LedMode` | Tab 4: RGB (Effect panel) | Full Parity | Static, Breathe, Rainbow, Pulse |
| Primary Color Chooser | `xyz.ljones.Aura.LedModeData` | Tab 4: RGB (Colours panel) | Enhanced Parity | Custom RGB/HSV 24-bit picker modal |
| Secondary Color Chooser | `xyz.ljones.Aura.LedModeData` | Tab 4: RGB (Colours panel) | Enhanced Parity | Active for Breathe / Pulse |
| Speed Selector | `xyz.ljones.Aura.LedModeData` | Tab 4: RGB (Effect panel) | Full Parity | Slow, Med, Fast |
| Direction Selector | `xyz.ljones.Aura.LedModeData` | Tab 4: RGB (Effect panel) | Full Parity | Left, Right (for supported modes) |
| Multi-Zone RGB Mapping | `xyz.ljones.Aura` multi-zone | Tab 4: RGB | Not Supported Here | FX608JH is SingleZone hardware |
| Per-Key RGB Customization | ROG Strix / Scar HID addressing | Tab 4: RGB | Not Supported Here | TUF series uses single-zone matrix |
| Hardware Power States | `xyz.ljones.Aura.LedPower` | Tab 4: RGB (Power States) | Full Parity | Boot, Awake, Sleep, Shutdown |
| Lighting Presets | `~/.config/rogtui/lighting.toml` | Tab 4: RGB (Key 'p') | New Feature | Save/apply named lighting setups |
| Screen Ambient Sync | wlr-screencopy / X11 GetImage | Tab 4: RGB (Effects panel) | New Feature | Opt-in foreground loop |
| Audio Sync (FFT Reactive) | PipeWire / PulseAudio monitor sink | Tab 4: RGB (Effects panel) | New Feature | Opt-in foreground loop |
| Thermal Color Gradient | CPU/GPU hwmon temp stream | Tab 4: RGB (Effects panel) | New Feature | Blue -> Green -> Yellow -> Red |

---

## 3. Fan Curves -> Fan Tab

| ROGCC GUI Element | Backend Source / DBus Endpoint | rogtui Location | Parity Status | Notes |
|---|---|---|---|---|
| Profile Selector | `xyz.ljones.Platform.PlatformProfile` | Tab 3: Fan (Header row) | Full Parity | Quiet / Balanced / Performance |
| Fan Selector (CPU/GPU/Mid) | `xyz.ljones.FanCurves.FanCurveData` | Tab 3: Fan (Header row) | Full Parity | CPU Fan (1/2), GPU Fan (2/2) |
| Enable Custom Curves Toggle | `xyz.ljones.FanCurves.SetFanCurvesEnabled` | Tab 3: Fan (Key 'e') | Full Parity | Hardware ACPI curve bypass |
| 8-Point Calibration Curve | `xyz.ljones.FanCurves.SetFanCurve` | Tab 3: Fan (Chart + Table) | Enhanced Parity | Labelled axes, Braille line, points |
| Live Sensor Overlay | hwmon sensor stream | Tab 3: Fan (Vertical line) | Full Parity | Real-time marker on curve |
| Apply Edits | `xyz.ljones.FanCurves.SetFanCurve` | Tab 3: Fan (Key 'w') | Full Parity | Writes validated 8-point curve |
| Cancel / Discard Edits | Reverts to `applied_fan_curves` | Tab 3: Fan (Key 'u') | Full Parity | Restores last applied state |
| Factory BIOS Defaults | `xyz.ljones.FanCurves.SetCurvesToDefaults` | Tab 3: Fan (Key 'r') | Full Parity | Full factory curve reset (confirm) |
| Non-Monotonic Repair Flow | Curve monotonicity validator | Tab 3: Fan (Key 'x') | Enhanced Parity | Fixes inverted PWM values |

---

## 4. GPU Configuration -> GPU Tab

| ROGCC GUI Element | Backend Source / DBus Endpoint | rogtui Location | Parity Status | Notes |
|---|---|---|---|---|
| GPU Mode Selector | `org.supergfxctl.Daemon.SetMode` | Tab 5: GPU (Selection panel) | Full Parity | Integrated, Hybrid, AsusMuxDgpu |
| Active GPU Status | `org.supergfxctl.Daemon.Mode` | Tab 5: GPU (Status panel) | Full Parity | Current hardware routing |
| dGPU Power State | `org.supergfxctl.Daemon.Power` | Tab 5: GPU (Status panel) | Full Parity | Active, Suspended (D3cold), Off |
| Pending Mode & Action | `org.supergfxctl.Daemon.PendingMode` | Tab 5: GPU (Status panel) | Full Parity | Displays required Logout/Reboot |
| Disable Powerd on Battery | `xyz.ljones.Platform.DisableNvidia...` | Tab 2: Power & Tab 5: GPU | Full Parity | Prevents dGPU wake on battery |
| MUX Switch Hardware Notice | ASUS ACPI WMI `gpu_mux_mode` | Tab 5: GPU & Tab 6: Armoury | Enhanced Parity | Explains internal panel bypass |

---

## 5. Battery Info -> Power Tab (Battery Group)

| ROGCC GUI Element | Backend Source / DBus Endpoint | rogtui Location | Parity Status | Notes |
|---|---|---|---|---|
| Charge Limit Stepper | `xyz.ljones.Platform.ChargeControl...` | Tab 2: Power (Battery group) | Full Parity | 20%..=100% in steps of 5% |
| Charge Limit Visual Gauge | Internal calculation | Tab 2: Power (Battery group) | Full Parity | Horizontal block gauge |
| One-Shot 100% Full Charge | `xyz.ljones.Platform.OneShotFull...` | Tab 2: Power (Battery group) | Full Parity | Uncaps limit until next unplug |
| Battery Health (Capacity %) | `energy_full` / `energy_full_design` | Tab 2: Power (Battery group) | Enhanced Parity | Wear calculation from power_supply |
| Charging State | `/sys/class/power_supply/BAT0/status` | Tab 2: Power (Battery group) | Enhanced Parity | Charging / Discharging / Full |
| Power Draw (Watts) | `power_now` or `current_now * voltage` | Tab 2: Power (Battery group) | Enhanced Parity | Instantaneous discharge rate |
| Cycle Count | `/sys/class/power_supply/BAT0/cycle_count` | Tab 2: Power (Battery group) | Enhanced Parity | EC battery cycle counter |

---

## 6. App Settings -> Not Ported (Replaced by CLI Subcommands)

| ROGCC GUI Setting | Resident Daemon Required? | Porting Decision | rogtui CLI Alternative |
|---|---|---|---|
| Tray Icon & Menu | Yes (Resident GUI process) | Intentionally Not Ported | Fast transient TUI launches in < 50ms |
| Run in Background | Yes (Background thread) | Intentionally Not Ported | Violates strict ~0% idle CPU invariant |
| Start on System Boot | Yes (Desktop autostart) | Intentionally Not Ported | User manages standard systemd services |
| ROG-Key Window Toggle | Yes (Global hotkey listener) | Replaced by Compositor Keybind | `bindsym XF86Launch1 exec kitty --class rogtui rogtui` |
| dGPU Switch Notifications | Yes (Resident event loop) | Replaced by CLI / Libnotify | `rogtui gpu set <mode>` sends desktop notification |

---

## 7. About -> About Tab

| ROGCC GUI Element | Source | rogtui Location | Parity Status | Notes |
|---|---|---|---|---|
| rogtui Version | Crate metadata (`CARGO_PKG_VERSION`) | Tab 7: About | Full Parity | Displays version & release profile |
| asusd Version | `xyz.ljones.Platform.Version` | Tab 7: About | Full Parity | v6.5.0 |
| supergfxd Version | `org.supergfxctl.Daemon.Version` | Tab 7: About | Full Parity | v5.2.7 |
| Linux Kernel Version | `uname -r` | Tab 7: About | Full Parity | Running kernel & architecture |
| Product Family & Board | `/sys/devices/virtual/dmi/id` | Tab 7: About | Full Parity | ASUS TUF Gaming F16 (FX608JH) |
| BIOS Version | `/sys/devices/virtual/dmi/id/bios_version` | Tab 7: About | Full Parity | DMI BIOS release string |
| Hardware Probe Summary | Internal discovery | Tab 7: About | Full Parity | Summarizes detected EC interfaces |
| Export Diagnostics Report | Sysfs + DBus dump script | Tab 7: About (Key 'y') | Enhanced Parity | Writes to `~/rogtui-diagnostics.txt` |
