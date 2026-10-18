# Keyboard Probe Results: ASUS TUF Gaming F16 (FX608JH)

Date: 2026-10-19
Machine: ASUS TUF Gaming F16 (FX608JH)
Kernel: CachyOS Linux (x86-64-v3, BORE/EEVDF)
asusd: v6.5.0
supergfxd: v5.2.7

---

## 1. Kernel Source Verification: `cmd` Field Semantics

- **Source File**: `drivers/platform/x86/asus-wmi.c`
- **Function**: `kbd_rgb_mode_store` (lines 1050–1101)

### Kernel Implementation Code:
```c
/* TUF Laptop Keyboard RGB Modes **********************************************/
static ssize_t kbd_rgb_mode_store(struct device *dev,
				 struct device_attribute *attr,
				 const char *buf, size_t count)
{
	u32 cmd, mode, r, g, b, speed;
	struct led_classdev *led;
	struct asus_wmi *asus;
	int err;

	led = dev_get_drvdata(dev);
	asus = container_of(led, struct asus_wmi, kbd_led);

	if (sscanf(buf, "%d %d %d %d %d %d", &cmd, &mode, &r, &g, &b, &speed) != 6)
		return -EINVAL;

	/* B3 is set and B4 is save to BIOS */
	switch (cmd) {
	case 0:
		cmd = 0xb3;
		break;
	case 1:
		cmd = 0xb4;
		break;
	default:
		return -EINVAL;
	}

	/* These are the known usable modes across all TUF/ROG */
	if (mode >= 12 || mode == 9)
		mode = 10;

	switch (speed) {
	case 0:
		speed = 0xe1;
		break;
	case 1:
		speed = 0xeb;
		break;
	case 2:
		speed = 0xf5;
		break;
	default:
		speed = 0xeb;
	}

	err = asus_wmi_evaluate_method3(ASUS_WMI_METHODID_DEVS, asus->kbd_rgb_dev,
			cmd | (mode << 8) | (r << 16) | (g << 24), b | (speed << 8), NULL);
	if (err)
		return err;

	return count;
}
```

### Verified Facts:
1. **Source Verification & Kernel Tag Audit**:
   - Initial source examination was referenced from a public kernel mirror (`gitlab.elettra.eu`).
   - Subsequently verified directly against the running system kernel (`uname -r`: `7.2.11-1-cachyos`) using the exact upstream CachyOS source repository (`https://github.com/CachyOS/linux`, release tag `cachyos-7.2.11-1`, file `drivers/platform/x86/asus-wmi.c` lines 1050–1101).
   - The implementation is completely identical in the running kernel:
     - `/* B3 is set and B4 is save to BIOS */`
     - `case 0: cmd = 0xb3; break;` (volatile set to EC RAM)
     - `case 1: cmd = 0xb4; break;` (persistent save to BIOS NVRAM)
     - Line 1079: `if (mode >= 12 || mode == 9) mode = 10;`
2. **Distinct Modes vs Aliases**:
   - Because the kernel driver unconditionally normalizes `mode >= 12 || mode == 9` to mode 10 (`Pulse`), modes 9 and 12 are kernel aliases.
   - The real distinct mode numbers accepted by WMI are: `0, 1, 2, 3, 4, 5, 6, 7, 8, 10, 11`. Probing sweeps exclusively these 11 distinct modes.
3. **Reboot Verification Requirement**:
   - Until tested live across a physical reboot, live effects will avoid high-rate `kbd_rgb_mode`.
   - **Reboot Test Procedure**:
     1. Stop and mask asusd to prevent re-applying saved profile on boot:
        `sudo systemctl stop asusd && sudo systemctl mask asusd`
     2. Write distinctive red color with cmd=0:
        `echo "0 0 255 0 0 0" | sudo tee /sys/class/leds/asus::kbd_backlight/kbd_rgb_mode`
     3. Reboot system. Check if keyboard reverts away from red.
     4. Unmask and restart asusd:
        `sudo systemctl unmask asusd && sudo systemctl start asusd`

---

## 2. Sysfs LED Subsystem Probe

### Path: `/sys/class/leds/asus::kbd_backlight/`

```text
total 0
-rw-r--r-- 1 root root 4096 brightness
-r--r--r-- 1 root root 4096 brightness_hw_changed
lrwxrwxrwx 1 root root    0 device -> ../../../asus-nb-wmi
--w------- 1 root root 4096 kbd_rgb_mode
-r--r--r-- 1 root root 4096 kbd_rgb_mode_index
--w------- 1 root root 4096 kbd_rgb_state
-r--r--r-- 1 root root 4096 kbd_rgb_state_index
-r--r--r-- 1 root root 4096 max_brightness
drwxr-xr-x 2 root root    0 power
lrwxrwxrwx 1 root root    0 subsystem -> ../../../../../class/leds
-rw-r--r-- 1 root root    0 trigger
-rw-r--r-- 1 root root 4096 uevent
```

### Parameter Index Attributes

```bash
$ cat /sys/class/leds/asus::kbd_backlight/kbd_rgb_mode_index
cmd mode red green blue speed

$ cat /sys/class/leds/asus::kbd_backlight/kbd_rgb_state_index
cmd boot awake sleep keyboard
```

---

## 3. Upstream `asusd` Support Database Entry

File: `/usr/share/asusd/aura_support.ron` (lines 191–199):

```ron
    (
        device_name: "FX608JH",
        product_id: "",
        layout_name: "fa506i",
        basic_modes: [Static, Breathe, RainbowCycle, Pulse],
        basic_zones: [],
        advanced_type: Zoned([SingleZone]),
        power_zones: [Keyboard],
    ),
```

---

## 4. Mode Verification Status (Visual Probe Required)

> [!IMPORTANT]
> **Status**: UNVERIFIED (AGENT INFERENCE PENDING USER VISUAL AUDIT).
> The agent cannot physically see the keyboard. Any claims regarding how modes 3..8, 11, and 12 render on the physical keyboard must be confirmed through the interactive `rogtui probe-modes` command with the user observing the physical hardware.

### Hardware Probe Plan:
- `rogtui probe-modes` sweeps modes 0 through 12, holding each for 2 seconds with a distinct color, prompting the user for observed visual feedback ("visible effect? y/n + name").
- Answers will be saved to `~/.config/rogtui/modes.toml`.
- Results will be written to `docs/aura-support-snippet.ron` and `docs/asusctl-issue.md` for upstream submission.
