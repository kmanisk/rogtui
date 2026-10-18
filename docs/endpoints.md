# DBus Endpoints and Hardware Mapping for rogtui

Target Machine: ASUS TUF Gaming F16 (FX608JH)  
asusd version: 6.5.0  
supergfxd version: 5.2.7  

---

## 1. System DBus Services

### Service: `xyz.ljones.Asusd`

#### Interface: `xyz.ljones.Platform`
Object Path: `/xyz/ljones`  
Mapped UI Tab: **Dashboard (Tab 1)** & **Power (Tab 2)**

| Member | Type | Signature | Description | UI Location |
| :--- | :--- | :--- | :--- | :--- |
| `PlatformProfile` | Property | `u` (rw) | Current active power profile (0=Balanced, 1=Performance, 2=Quiet) | Tab 1, Tab 2 |
| `PlatformProfileChoices` | Property | `au` (ro) | Supported profile indices (e.g. `[2, 0, 1]`) | Tab 2 |
| `PlatformProfileOnAc` | Property | `u` (rw) | Profile assigned when connected to AC | Tab 2 |
| `PlatformProfileOnBattery` | Property | `u` (rw) | Profile assigned when running on battery | Tab 2 |
| `ChangePlatformProfileOnAc` | Property | `b` (rw) | Auto-switch profile on AC connect | Tab 2 |
| `ChangePlatformProfileOnBattery` | Property | `b` (rw) | Auto-switch profile on battery switch | Tab 2 |
| `ChargeControlEndThreshold` | Property | `y` (rw) | Battery charge limit percentage (20..=100) | Tab 1, Tab 2 |
| `OneShotFullCharge` | Method | `() -> ()` | Temporarily charge to 100% until next unplug | Tab 2 |
| `DisableNvidiaPowerdOnBattery` | Property | `b` (rw) | Disable dynamic boost daemon on battery | Tab 2 |
| `PlatformProfileLinkedEpp` | Property | `b` (rw) | Link CPU EPP to platform profile | Tab 2 |
| `ProfileQuietEpp` | Property | `u` (rw) | CPU EPP value for Quiet profile | Tab 2 |
| `ProfileBalancedEpp` | Property | `u` (rw) | CPU EPP value for Balanced profile | Tab 2 |
| `ProfilePerformanceEpp` | Property | `u` (rw) | CPU EPP value for Performance profile | Tab 2 |
| `NextPlatformProfile` | Method | `() -> ()` | Cycle to next profile | Tab 1, Tab 2 |
| `SupportedProperties` | Method | `() -> as` | Properties supported by current hardware | Internal / Tab 2 |
| `Version` | Property | `s` (ro) | Daemon version string (`6.5.0`) | Dashboard footer |

---

#### Interface: `xyz.ljones.FanCurves`
Object Path: `/xyz/ljones`  
Mapped UI Tab: **Fan Curves (Tab 3)**

| Member | Type | Signature | Description | UI Location |
| :--- | :--- | :--- | :--- | :--- |
| `FanCurveData` | Method | `u -> a(s(yyyyyyyy)(yyyyyyyy)b)` | Query curves for profile `u` (returns fan name, PWM 8-tuple, Temp 8-tuple, enabled bool) | Tab 3 |
| `SetFanCurve` | Method | `u(s(yyyyyyyy)(yyyyyyyy)b) -> ()` | Write custom curve for profile `u` and specific fan | Tab 3 |
| `SetFanCurvesEnabled` | Method | `ub -> ()` | Enable/disable all custom curves for profile `u` | Tab 3 |
| `SetProfileFanCurveEnabled` | Method | `usb -> ()` | Enable/disable curve for profile `u` and fan name `s` | Tab 3 |
| `SetCurvesToDefaults` | Method | `u -> ()` | Reset profile `u` fan curves to factory default | Tab 3 |

---

#### Interface: `xyz.ljones.Aura`
Object Path: `/xyz/ljones/aura/tuf` (or `/xyz/ljones/aura/*` dynamically detected)  
Mapped UI Tab: **RGB (Aura) (Tab 4)**

| Member | Type | Signature | Description | UI Location |
| :--- | :--- | :--- | :--- | :--- |
| `Brightness` | Property | `u` (rw) | Backlight level (`0=Off, 1=Low, 2=Med, 3=High`) | Tab 4 |
| `SupportedBrightness` | Property | `au` (ro) | Available brightness levels (`[0, 1, 2, 3]`) | Tab 4 |
| `LedMode` | Property | `u` (rw) | Active RGB animation mode index | Tab 4 |
| `SupportedBasicModes` | Property | `au` (ro) | Supported modes (`0=Static, 1=Breathe, 2=Rainbow, 10=Pulse, ...`) | Tab 4 |
| `LedModeData` | Property | `(uu(yyy)(yyy)ss)` (rw) | `(mode, zone, rgb1, rgb2, speed, direction)` | Tab 4 |
| `LedPower` | Property | `(a(ubbbb))` (rw) | Power state flags: `(zone, boot, awake, sleep, shutdown)` | Tab 4 |
| `AllModeData` | Method | `() -> a{u(uu(yyy)(yyy)ss)}` | Map of all mode configurations | Tab 4 |

---

#### Interface: `xyz.ljones.AsusArmoury`
Object Paths: `/xyz/ljones/asus_armoury/<attribute_name>`  
Mapped UI Tab: **Armoury / Firmware (Tab 6)**

| Attribute Node | Type | Values on FX608JH | Description | UI Location |
| :--- | :--- | :--- | :--- | :--- |
| `charge_mode` | Enum/List | `0, 1, 2` | Battery charging rate / power adapter policy | Tab 6 |
| `panel_overdrive` | Toggle | `0, 1` | 3ms response time LCD overdrive | Tab 1, Tab 6 |
| `dgpu_disable` | Toggle | `0, 1` | ACPI dGPU power gate | Tab 6 |
| `gpu_mux_mode` | Toggle | `0, 1` | Direct display MUX toggle | Tab 1, Tab 6 |
| `screen_auto_brightness` | Toggle | `0, 1` | Ambient light sensor display scaling | Tab 6 |
| `nv_base_tgp` | Integer | `0` (ro) | NVIDIA base thermal limit | Tab 6 |
| `nv_dynamic_boost` | Scalar | Unavailable on FX608JH | NVIDIA dynamic boost offset | Hidden |
| `ppt_pl1_spl` | Scalar | Unavailable on FX608JH | CPU Sustained PL1 (W) | Hidden |
| `ppt_pl2_sppt` | Scalar | Unavailable on FX608JH | CPU Burst PL2 (W) | Hidden |

*All attributes implement methods `RestoreDefault() -> ()` and property `CurrentValue (i, rw)`.*

---

### Service: `org.supergfxctl.Daemon`

#### Interface: `org.supergfxctl.Daemon`
Object Path: `/org/supergfxctl/Gfx`  
Mapped UI Tab: **Dashboard (Tab 1)** & **GPU (Tab 5)**

| Member | Type | Signature | Description | UI Location |
| :--- | :--- | :--- | :--- | :--- |
| `Mode` | Method | `() -> u` | Current mode (`0=Dedicated, 1=Integrated, 2=Hybrid, 3=Vfio, 4=AsusEgpu, 5=AsusMuxDgpu`) | Tab 1, Tab 5 |
| `Supported` | Method | `() -> au` | Supported GPU modes on current platform | Tab 5 |
| `Power` | Method | `() -> u` | dGPU power state (`0=Unknown, 1=Off, 2=Active, 3=Suspended, 4=AsusMuxDiscreet, 5=AsusDisabled`) | Tab 1, Tab 5 |
| `PendingMode` | Method | `() -> u` | Target mode queued across session | Tab 5 |
| `PendingUserAction` | Method | `() -> u` | Required action (`0=Nothing, 1=Logout, 2=Reboot, 3=SwitchToIntegrated`) | Tab 5 |
| `Vendor` | Method | `() -> s` | dGPU vendor name (e.g. `"Nvidia"`) | Tab 5 |
| `Version` | Method | `() -> s` | supergfxd daemon version (`"5.2.7"`) | Tab 5 footer |
| `SetMode` | Method | `u -> u` | Switch graphics mode (returns required user action) | Tab 5 |
| `NotifyAction` | Signal | `u` | Emitted when required user action changes | Tab 5 live sync |
| `NotifyGfx` | Signal | `u` | Emitted when graphics mode transitions | Tab 5 live sync |
| `NotifyGfxStatus` | Signal | `u` | Emitted when dGPU power state changes | Tab 5 live sync |

---

## 2. Hardware Sensors (sysfs / hwmon)

Mapped UI Location: **Dashboard (Tab 1)** & **Fan Curves (Tab 3)**

| Sensor | Path / Source | Parsing Unit | Displayed As |
| :--- | :--- | :--- | :--- |
| CPU Package Temp | `/sys/class/hwmon/hwmon8/temp1_input` (or `name="coretemp"`) | Millidegrees C | `56 C` |
| CPU Fan Speed | `/sys/class/hwmon/hwmon6/fan1_input` (or `name="asus"`) | RPM | `2800 RPM` |
| GPU Fan Speed | `/sys/class/hwmon/hwmon6/fan2_input` (or `name="asus"`) | RPM | `2500 RPM` |
| NVIDIA dGPU Temp | Direct sysfs or `/sys/class/thermal` fallback | Degrees C | `45 C` |
