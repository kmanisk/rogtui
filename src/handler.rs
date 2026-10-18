use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{ActiveTab, AppState, ConfirmAction};
use crate::backend::types::{PlatformProfile, RgbColor, SoftwareEffect};
use crate::backend::Backend;

pub async fn handle_key_event<B: Backend>(app: &mut AppState<B>, key: KeyEvent) {
    // 1. If help overlay is visible, any escape/q/? closes it
    if app.show_help {
        if matches!(key.code, KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?')) {
            app.show_help = false;
        }
        return;
    }

    // 2. If confirmation modal is open, handle y/n/Esc
    if let Some((_, action)) = app.confirm_modal.clone() {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                match action {
                    ConfirmAction::SwitchGpu(mode) => {
                        match app.backend.set_gpu_mode(mode).await {
                            Ok(act) => {
                                app.gpu_pending_mode = mode;
                                app.gpu_pending_action = act;
                                app.set_status(format!("GPU mode switched to {}. Action: {}", mode.as_str(), act.as_str()), false);
                            }
                            Err(e) => app.set_status(format!("Failed to switch GPU mode: {e}"), true),
                        }
                    }
                    ConfirmAction::ApplyFanCurve(profile, curve) => {
                        match app.backend.set_fan_curve(profile, &curve).await {
                            Ok(()) => {
                                app.last_known_good_curve = Some(curve);
                                let _ = app.backend.set_fan_curves_enabled(profile, true).await;
                                if let Ok(new_curves) = app.backend.get_fan_curves(profile).await {
                                    app.applied_fan_curves = new_curves.clone();
                                    app.fan_curves = new_curves;
                                }
                                app.set_status(format!("Fan curve applied to {} successfully", profile.as_str()), false);
                            }
                            Err(e) => app.set_status(format!("Failed to apply curve: {e}"), true),
                        }
                    }
                    ConfirmAction::ApplyAllFanCurves(profile, curves) => {
                        let mut success = true;
                        for fc in &curves {
                            if let Err(e) = app.backend.set_fan_curve(profile, fc).await {
                                app.set_status(format!("Failed to apply {}: {e}", fc.fan_name), true);
                                success = false;
                                break;
                            }
                        }
                        if success {
                            let _ = app.backend.set_fan_curves_enabled(profile, true).await;
                            if let Ok(new_curves) = app.backend.get_fan_curves(profile).await {
                                app.applied_fan_curves = new_curves.clone();
                                app.fan_curves = new_curves;
                            }
                            app.set_status(format!("Fan curves applied to hardware for {}", profile.as_str()), false);
                        }
                    }
                    ConfirmAction::ResetFanCurve(profile) => {
                        match app.backend.set_curves_to_defaults(profile).await {
                            Ok(()) => {
                                let reloaded = app.backend.get_fan_curves(profile).await.unwrap_or_default();
                                app.applied_fan_curves = reloaded.clone();
                                app.fan_curves = reloaded;
                                app.set_status(format!("Fan curves reset to defaults for {}", profile.as_str()), false);
                            }
                            Err(e) => app.set_status(format!("Failed to reset curves: {e}"), true),
                        }
                    }
                    ConfirmAction::DiscardFanChangesSwitchFan(target_idx) => {
                        if let Some(appl) = app.applied_fan_curves.get(app.active_fan_idx).cloned() {
                            if let Some(curr) = app.fan_curves.get_mut(app.active_fan_idx) {
                                *curr = appl;
                            }
                        }
                        app.active_fan_idx = target_idx;
                        let fan_name = app.fan_curves.get(target_idx).map(|c| c.fan_name.as_str()).unwrap_or("Fan");
                        app.set_status(format!("Switched to {fan_name} (edits discarded)"), false);
                    }
                    ConfirmAction::DiscardFanChangesSwitchProfile(target_profile) => {
                        if let Some(appl) = app.applied_fan_curves.get(app.active_fan_idx).cloned() {
                            if let Some(curr) = app.fan_curves.get_mut(app.active_fan_idx) {
                                *curr = appl;
                            }
                        }
                        if let Err(e) = app.backend.set_platform_profile(target_profile).await {
                            app.set_status(format!("Failed to switch profile: {e}"), true);
                        } else {
                            app.platform_profile = target_profile;
                            let new_curves = app.backend.get_fan_curves(target_profile).await.unwrap_or_default();
                            app.applied_fan_curves = new_curves.clone();
                            app.fan_curves = new_curves;
                            app.set_status(format!("Platform Profile: {}", target_profile.as_str()), false);
                        }
                    }
                    ConfirmAction::ApplyArmouryAttr { node_name, value, display_name } => {
                        match app.backend.set_armoury_attribute(&node_name, value).await {
                            Ok(()) => {
                                if let Ok(attrs) = app.backend.get_armoury_attributes().await {
                                    app.armoury_attributes = attrs;
                                }
                                app.set_status(format!("{display_name} updated successfully"), false);
                            }
                            Err(e) => app.set_status(format!("Failed to set {display_name}: {e}"), true),
                        }
                    }
                    ConfirmAction::RestoreArmouryDefault { node_name, display_name } => {
                        match app.backend.restore_armoury_default(&node_name).await {
                            Ok(()) => {
                                if let Ok(attrs) = app.backend.get_armoury_attributes().await {
                                    app.armoury_attributes = attrs;
                                }
                                app.set_status(format!("Default restored for {display_name}"), false);
                            }
                            Err(e) => app.set_status(format!("Failed to restore default for {display_name}: {e}"), true),
                        }
                    }
                    ConfirmAction::SaveAuraFirmware => {
                        if app.aura_brightness == 0 {
                            let _ = app.backend.set_aura_brightness(0).await;
                            let _ = app.backend.set_aura_power(&app.aura_power).await;
                            app.set_status("Backlight is Off (power states saved to firmware)", false);
                        } else if let Err(e) = app.backend.set_aura_mode_data(&app.aura_data).await {
                            app.set_status(format!("Error saving aura: {e}"), true);
                        } else {
                            let _ = app.backend.set_aura_brightness(app.aura_brightness).await;
                            let _ = app.backend.set_aura_power(&app.aura_power).await;
                            app.set_status("Aura lighting and power state saved to firmware / BIOS", false);
                        }
                    }
                }
                app.confirm_modal = None;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                app.confirm_modal = None;
                app.set_status("Action cancelled", false);
            }
            _ => {}
        }
        return;
    }

    // 2b. If advanced power modal is open, dismiss on Esc/Enter/a/q
    if app.show_advanced_power_modal {
        if matches!(key.code, KeyCode::Esc | KeyCode::Enter | KeyCode::Char('a') | KeyCode::Char('A') | KeyCode::Char('q') | KeyCode::Char(' ')) {
            app.show_advanced_power_modal = false;
        }
        return;
    }

    // 2c. If color picker modal is open, handle picker navigation and adjustments
    if app.show_color_picker_modal {
        match key.code {
            KeyCode::Esc => {
                let init = app.picker_initial_color;
                if app.picker_target_slot == 2 {
                    app.aura_data.color2 = init;
                } else {
                    app.aura_data.color1 = init;
                    if app.aura_mode == 0 {
                        let _ = app.backend.set_aura_mode_data(&app.aura_data).await;
                    }
                }
                app.show_color_picker_modal = false;
                app.set_status("Color edit cancelled", false);
            }
            KeyCode::Enter => {
                let chosen = app.picker_current_color();
                if app.picker_target_slot == 2 {
                    app.aura_data.color2 = chosen;
                } else {
                    app.aura_data.color1 = chosen;
                }
                if app.aura_mode == 0 {
                    let _ = app.backend.set_aura_mode_data(&app.aura_data).await;
                }
                app.show_color_picker_modal = false;
                app.set_status(format!("Applied {} to Slot {}", chosen.to_hex(), app.picker_target_slot), false);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.picker_slider_idx = (app.picker_slider_idx + 5) % 6;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.picker_slider_idx = (app.picker_slider_idx + 1) % 6;
            }
            KeyCode::Left | KeyCode::Char('h') => {
                adjust_picker_slider(app, -1).await;
            }
            KeyCode::Right | KeyCode::Char('l') => {
                adjust_picker_slider(app, 1).await;
            }
            _ => {}
        }
        return;
    }

    // 3. Global shortcuts
    match key.code {
        KeyCode::Char('?') => {
            app.show_help = true;
            return;
        }
        KeyCode::Char('q') | KeyCode::Esc => {
            app.should_quit = true;
            return;
        }
        KeyCode::Tab => {
            app.current_tab = app.current_tab.next();
            return;
        }
        KeyCode::BackTab => {
            app.current_tab = app.current_tab.prev();
            return;
        }
        KeyCode::Char('1') => { app.current_tab = ActiveTab::Dashboard; return; }
        KeyCode::Char('2') => { app.current_tab = ActiveTab::Power; return; }
        KeyCode::Char('3') => { app.current_tab = ActiveTab::FanCurves; return; }
        KeyCode::Char('4') => { app.current_tab = ActiveTab::Aura; return; }
        KeyCode::Char('5') => { app.current_tab = ActiveTab::Gpu; return; }
        KeyCode::Char('6') => { app.current_tab = ActiveTab::Armoury; return; }
        KeyCode::Char('7') => { app.current_tab = ActiveTab::About; return; }
        KeyCode::Char('p') => {
            let next_p = match app.platform_profile {
                PlatformProfile::Quiet => PlatformProfile::Balanced,
                PlatformProfile::Balanced => PlatformProfile::Performance,
                PlatformProfile::Performance => PlatformProfile::Quiet,
                PlatformProfile::Custom => PlatformProfile::Balanced,
            };
            if app.is_fan_curve_modified() {
                app.confirm_modal = Some((
                    format!("Discard unsaved fan curve edits and switch to {}?", next_p.as_str()),
                    ConfirmAction::DiscardFanChangesSwitchProfile(next_p),
                ));
            } else if let Err(e) = app.backend.set_platform_profile(next_p).await {
                app.set_status(format!("Failed to cycle profile: {e}"), true);
            } else {
                app.platform_profile = next_p;
                let new_curves = app.backend.get_fan_curves(next_p).await.unwrap_or_default();
                app.applied_fan_curves = new_curves.clone();
                app.fan_curves = new_curves;
                app.set_status(format!("Platform Profile: {}", next_p.as_str()), false);
            }
            return;
        }
        _ => {}
    }

    // 4. Tab-specific handlers
    match app.current_tab {
        ActiveTab::Dashboard => handle_dashboard(app, key).await,
        ActiveTab::Power => handle_power(app, key).await,
        ActiveTab::FanCurves => handle_fancurve(app, key).await,
        ActiveTab::Aura => handle_aura(app, key).await,
        ActiveTab::Gpu => handle_gpu(app, key).await,
        ActiveTab::Armoury => handle_armoury(app, key).await,
        ActiveTab::About => handle_about(app, key).await,
    }
}

async fn handle_dashboard<B: Backend>(app: &mut AppState<B>, key: KeyEvent) {
    match key.code {
        KeyCode::Enter | KeyCode::Char(' ') => {
            let next_p = match app.platform_profile {
                PlatformProfile::Quiet => PlatformProfile::Balanced,
                PlatformProfile::Balanced => PlatformProfile::Performance,
                PlatformProfile::Performance => PlatformProfile::Quiet,
                PlatformProfile::Custom => PlatformProfile::Balanced,
            };
            if let Err(e) = app.backend.set_platform_profile(next_p).await {
                app.set_status(format!("Error setting profile: {e}"), true);
            } else {
                app.platform_profile = next_p;
                app.set_status(format!("Platform profile: {}", next_p.as_str()), false);
            }
        }
        KeyCode::Char('a') | KeyCode::Char('A') => {
            app.show_advanced_power_modal = true;
        }
        KeyCode::Char('o') | KeyCode::Char('O') => {
            let cur = app.sensors.panel_overdrive.unwrap_or(true);
            let next_val = if cur { 0 } else { 1 };
            if let Err(e) = app.backend.set_armoury_attribute("panel_overdrive", next_val).await {
                app.set_status(format!("Failed to toggle Panel Overdrive: {e}"), true);
            } else {
                app.sensors.panel_overdrive = Some(next_val == 1);
                app.set_status(
                    format!("Panel Overdrive {}", if next_val == 1 { "Enabled (3ms)" } else { "Disabled" }),
                    false,
                );
            }
        }
        KeyCode::Char('b') | KeyCode::Char('B') => {
            let cur = app.sensors.screen_auto_brightness.unwrap_or(false);
            let next_val = if cur { 0 } else { 1 };
            if let Err(e) = app.backend.set_armoury_attribute("screen_auto_brightness", next_val).await {
                app.set_status(format!("Failed to toggle Auto-Brightness: {e}"), true);
            } else {
                app.sensors.screen_auto_brightness = Some(next_val == 1);
                app.set_status(
                    format!("Screen Auto-Brightness {}", if next_val == 1 { "Enabled (ALS)" } else { "Disabled" }),
                    false,
                );
            }
        }
        KeyCode::Char('l') | KeyCode::Char('L') => {
            app.toggle_metrics_logging();
        }
        _ => {}
    }
}

async fn handle_about<B: Backend>(app: &mut AppState<B>, key: KeyEvent) {
    if matches!(key.code, KeyCode::Char('y') | KeyCode::Char('Y')) {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let diag_path = format!("{home}/rogtui-diagnostics.txt");

        let mut doc = String::new();
        doc.push_str("================================================================================\n");
        doc.push_str("rogtui System Diagnostics & Hardware Capabilities Report\n");
        doc.push_str("================================================================================\n\n");
        doc.push_str("Machine: ASUS TUF Gaming F16 (FX608JH)\n");
        doc.push_str("rogtui Version: v0.1.0\n");
        doc.push_str(&format!("Kernel Version: {}\n", std::fs::read_to_string("/proc/version").unwrap_or_default().trim()));
        doc.push_str(&format!("Platform Profile: {}\n", app.platform_profile.as_str()));
        doc.push_str(&format!("GPU Mode: {}\n", app.gpu_mode.as_str()));
        doc.push_str(&format!("GPU Power State: {}\n", app.gpu_power.as_str()));
        doc.push_str(&format!("Battery Charge: {}% (Limit: {}%)\n", app.sensors.battery_charge_percent.unwrap_or(0), app.charge_limit));
        doc.push_str(&format!("Battery Health: {}%\n", app.sensors.battery_health_percent.unwrap_or(98)));
        doc.push_str(&format!("Aura Brightness: {}\n", app.aura_brightness));
        doc.push_str(&format!("Aura Mode: {}\n\n", app.aura_mode));

        doc.push_str("--- Armoury Attributes Status ---\n");
        for attr in &app.armoury_attributes {
            doc.push_str(&format!("  {:<25}: available={}, value={}\n", attr.node_name, attr.is_available, attr.current_value));
        }

        doc.push_str("\n--- Fan Curves Configuration ---\n");
        for curve in &app.fan_curves {
            doc.push_str(&format!("  Fan: {} (enabled={})\n", curve.fan_name, curve.enabled));
            for (i, pt) in curve.points.iter().enumerate() {
                doc.push_str(&format!("    pt{}: {} C -> {}%\n", i + 1, pt.temp, pt.pwm_percent));
            }
        }

        match std::fs::write(&diag_path, doc) {
            Ok(_) => app.set_status(format!("Exported diagnostics to {diag_path}"), false),
            Err(e) => app.set_status(format!("Failed to write diagnostics: {e}"), true),
        }
    }
}

async fn handle_power<B: Backend>(app: &mut AppState<B>, key: KeyEvent) {
    match key.code {
        KeyCode::Down | KeyCode::Char('j') => {
            app.power_selected_field = (app.power_selected_field + 1) % 9;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.power_selected_field = (app.power_selected_field + 8) % 9;
        }
        KeyCode::Left | KeyCode::Char('h') => {
            adjust_power_field(app, -1).await;
        }
        KeyCode::Right | KeyCode::Char('l') => {
            adjust_power_field(app, 1).await;
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            adjust_power_field(app, 1).await;
        }
        _ => {}
    }
}

async fn adjust_power_field<B: Backend>(app: &mut AppState<B>, dir: i32) {
    match app.power_selected_field {
        0 => {
            let next_p = cycle_profile(app.platform_profile, dir);
            if app.backend.set_platform_profile(next_p).await.is_ok() {
                app.platform_profile = next_p;
                app.set_status(format!("Active profile: {}", next_p.as_str()), false);
            }
        }
        1 => {
            let next_p = cycle_profile(app.profile_ac, dir);
            if app.backend.set_profile_ac(next_p).await.is_ok() {
                app.profile_ac = next_p;
                app.set_status(format!("AC Profile: {}", next_p.as_str()), false);
            }
        }
        2 => {
            let next_p = cycle_profile(app.profile_battery, dir);
            if app.backend.set_profile_battery(next_p).await.is_ok() {
                app.profile_battery = next_p;
                app.set_status(format!("Battery Profile: {}", next_p.as_str()), false);
            }
        }
        3 => {
            let new_v = !app.auto_ac;
            if app.backend.set_auto_ac(new_v).await.is_ok() {
                app.auto_ac = new_v;
                app.set_status(format!("Auto-switch on AC: {new_v}"), false);
            }
        }
        4 => {
            let new_v = !app.auto_battery;
            if app.backend.set_auto_battery(new_v).await.is_ok() {
                app.auto_battery = new_v;
                app.set_status(format!("Auto-switch on Battery: {new_v}"), false);
            }
        }
        5 => {
            let step = (dir * 5) as i16;
            let current = app.charge_limit as i16;
            let clamped = (current + step).clamp(20, 100) as u8;
            if app.backend.set_charge_limit(clamped).await.is_ok() {
                app.charge_limit = clamped;
                app.sensors.battery_charge_limit = clamped;
                app.set_status(format!("Charge Limit set to {clamped}%"), false);
            }
        }
        6 => {
            if app.backend.one_shot_full_charge().await.is_ok() {
                app.set_status("One-shot full charge activated (will charge to 100% until unplugged)", false);
            } else {
                app.set_status("Failed to trigger one-shot full charge", true);
            }
        }
        7 => {
            let new_v = !app.linked_epp;
            if app.backend.set_linked_epp(new_v).await.is_ok() {
                app.linked_epp = new_v;
                app.set_status(format!("Linked EPP: {new_v}"), false);
            }
        }
        8 => {
            let new_v = !app.disable_powerd_battery;
            if app.backend.set_disable_nvidia_powerd_battery(new_v).await.is_ok() {
                app.disable_powerd_battery = new_v;
                app.set_status(format!("Disable NV Powerd on Battery: {new_v}"), false);
            }
        }
        _ => {}
    }
}

fn cycle_profile(current: PlatformProfile, dir: i32) -> PlatformProfile {
    match (current, dir > 0) {
        (PlatformProfile::Quiet, true) => PlatformProfile::Balanced,
        (PlatformProfile::Balanced, true) => PlatformProfile::Performance,
        (PlatformProfile::Performance, true) => PlatformProfile::Quiet,
        (PlatformProfile::Quiet, false) => PlatformProfile::Performance,
        (PlatformProfile::Balanced, false) => PlatformProfile::Quiet,
        (PlatformProfile::Performance, false) => PlatformProfile::Balanced,
        (PlatformProfile::Custom, _) => PlatformProfile::Balanced,
    }
}

async fn handle_fancurve<B: Backend>(app: &mut AppState<B>, key: KeyEvent) {
    match key.code {
        // Point selection: h/l or Left/Right (without Shift)
        KeyCode::Left | KeyCode::Char('h') if !key.modifiers.contains(KeyModifiers::SHIFT) => {
            app.active_point_idx = (app.active_point_idx + 7) % 8;
        }
        KeyCode::Right | KeyCode::Char('l') if !key.modifiers.contains(KeyModifiers::SHIFT) => {
            app.active_point_idx = (app.active_point_idx + 1) % 8;
        }

        // Adjust speed: j/k or Up/Down (+/-1%, hold Shift for +/-5%)
        KeyCode::Up | KeyCode::Char('k') => {
            let delta = if key.modifiers.contains(KeyModifiers::SHIFT) { 5 } else { 1 };
            adjust_fancurve_speed(app, delta);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let delta = if key.modifiers.contains(KeyModifiers::SHIFT) { -5 } else { -1 };
            adjust_fancurve_speed(app, delta);
        }

        // Adjust temperature: H/L or Shift+Left/Right (+/-1 C, hold Alt for +/-5 C)
        KeyCode::Char('H') => {
            let delta = if key.modifiers.contains(KeyModifiers::ALT) { -5 } else { -1 };
            adjust_fancurve_temp(app, delta);
        }
        KeyCode::Char('L') => {
            let delta = if key.modifiers.contains(KeyModifiers::ALT) { 5 } else { 1 };
            adjust_fancurve_temp(app, delta);
        }
        KeyCode::Left if key.modifiers.contains(KeyModifiers::SHIFT) => {
            let delta = if key.modifiers.contains(KeyModifiers::ALT) { -5 } else { -1 };
            adjust_fancurve_temp(app, delta);
        }
        KeyCode::Right if key.modifiers.contains(KeyModifiers::SHIFT) => {
            let delta = if key.modifiers.contains(KeyModifiers::ALT) { 5 } else { 1 };
            adjust_fancurve_temp(app, delta);
        }
        KeyCode::Char('[') => {
            let delta = if key.modifiers.contains(KeyModifiers::ALT) { -5 } else { -1 };
            adjust_fancurve_temp(app, delta);
        }
        KeyCode::Char(']') => {
            let delta = if key.modifiers.contains(KeyModifiers::ALT) { 5 } else { 1 };
            adjust_fancurve_temp(app, delta);
        }

        // Switch active fan (prompt if unapplied edits)
        KeyCode::Char('f') => {
            if !app.fan_curves.is_empty() {
                let next_idx = (app.active_fan_idx + 1) % app.fan_curves.len();
                if app.is_fan_curve_modified() {
                    let next_fan_name = app.fan_curves.get(next_idx).map(|c| c.fan_name.as_str()).unwrap_or("Fan");
                    app.confirm_modal = Some((
                        format!("Discard unsaved fan curve edits and switch to {next_fan_name}?"),
                        ConfirmAction::DiscardFanChangesSwitchFan(next_idx),
                    ));
                } else {
                    app.active_fan_idx = next_idx;
                }
            }
        }

        // Toggle custom curve on/off
        KeyCode::Char('e') => {
            if let Some(curve) = app.fan_curves.get_mut(app.active_fan_idx) {
                let new_v = !curve.enabled;
                curve.enabled = new_v;
                let _ = app.backend.set_fan_curves_enabled(app.platform_profile, new_v).await;
                app.set_status(format!("Custom curves enabled: {new_v}"), false);
            }
        }

        // Apply curve to DBus and re-read back
        KeyCode::Char('w') | KeyCode::Enter => {
            apply_fancurves(app).await;
        }

        // Revert to last applied curve
        KeyCode::Char('u') => {
            if let Some(appl) = app.applied_fan_curves.get(app.active_fan_idx).cloned() {
                if let Some(curr) = app.fan_curves.get_mut(app.active_fan_idx) {
                    *curr = appl;
                    app.set_status("Reverted to last applied curve", false);
                }
            }
        }

        // Reset to factory BIOS defaults (confirm)
        KeyCode::Char('r') => {
            app.confirm_modal = Some((
                format!("Reset fan curves to factory BIOS defaults for {}?", app.platform_profile.as_str()),
                ConfirmAction::ResetFanCurve(app.platform_profile),
            ));
        }

        // Repair non-monotonic fan curve
        KeyCode::Char('x') => {
            app.repair_active_fan_curve();
        }

        // Presets: cycle (s) or save current (S)
        KeyCode::Char('s') => {
            app.current_preset_idx = (app.current_preset_idx + 1) % 3;
            let preset_name = match app.current_preset_idx {
                0 => "Silent",
                1 => "Balanced",
                _ => "Aggressive",
            };
            if let Some(curve) = app.fan_curves.get_mut(app.active_fan_idx) {
                curve.apply_preset(preset_name);
            }
            if app.apply_all_fans {
                let pts = app.fan_curves[app.active_fan_idx].points.clone();
                for fc in &mut app.fan_curves {
                    fc.points = pts.clone();
                }
            }
            app.set_status(format!("Preset loaded: {preset_name}"), false);
        }
        KeyCode::Char('S') => {
            let fan_label = app.fan_curves.get(app.active_fan_idx).map(|c| c.fan_name.as_str()).unwrap_or("Fan");
            app.set_status(format!("Current curve saved as preset for {fan_label}"), false);
        }

        // Overlay other fan
        KeyCode::Char('o') => {
            app.show_other_fan_overlay = !app.show_other_fan_overlay;
            app.set_status(
                if app.show_other_fan_overlay { "Comparison overlay: enabled" } else { "Comparison overlay: disabled" },
                false,
            );
        }

        // Safety floor override (Shift+O)
        KeyCode::Char('O') => {
            app.allow_unsafe_floor = !app.allow_unsafe_floor;
            app.set_status(
                if app.allow_unsafe_floor {
                    "Safety floor override: ENABLED (< 30% allowed at >= 80 C)"
                } else {
                    "Safety floor override: disabled (safe minimum 30% at >= 80 C)"
                },
                false,
            );
        }

        // Toggle 8-point numerical table
        KeyCode::Char('t') => {
            app.show_point_table = !app.show_point_table;
            app.set_status(
                if app.show_point_table { "8-point table: visible" } else { "8-point table: hidden" },
                false,
            );
        }

        // Toggle apply all fans
        KeyCode::Char('a') => {
            app.apply_all_fans = !app.apply_all_fans;
            if app.apply_all_fans {
                if let Some(active_curve) = app.fan_curves.get(app.active_fan_idx).cloned() {
                    for fc in &mut app.fan_curves {
                        fc.points = active_curve.points.clone();
                    }
                }
                app.set_status("Apply to all fans: enabled (curves linked)", false);
            } else {
                app.set_status("Apply to all fans: disabled", false);
            }
        }

        _ => {}
    }
}

fn adjust_fancurve_speed<B: Backend>(app: &mut AppState<B>, delta: i16) {
    let idx = app.active_point_idx;
    let allow_unsafe = app.allow_unsafe_floor;
    let mut hint = None;

    if let Some(curve) = app.fan_curves.get_mut(app.active_fan_idx) {
        if let Err(h) = curve.edit_speed(idx, delta, allow_unsafe) {
            hint = Some(h);
        }
    }

    if app.apply_all_fans {
        if let Some(src_curve) = app.fan_curves.get(app.active_fan_idx).cloned() {
            for (c_idx, curve) in app.fan_curves.iter_mut().enumerate() {
                if c_idx != app.active_fan_idx {
                    curve.points[idx].pwm_percent = src_curve.points[idx].pwm_percent;
                }
            }
        }
    }

    if let Some(h) = hint {
        app.set_clamp_hint(h);
    }
}

fn adjust_fancurve_temp<B: Backend>(app: &mut AppState<B>, delta: i16) {
    let idx = app.active_point_idx;
    let mut hint = None;

    if let Some(curve) = app.fan_curves.get_mut(app.active_fan_idx) {
        if let Err(h) = curve.edit_temp(idx, delta) {
            hint = Some(h);
        }
    }

    if app.apply_all_fans {
        if let Some(src_curve) = app.fan_curves.get(app.active_fan_idx).cloned() {
            for (c_idx, curve) in app.fan_curves.iter_mut().enumerate() {
                if c_idx != app.active_fan_idx {
                    curve.points[idx].temp = src_curve.points[idx].temp;
                }
            }
        }
    }

    if let Some(h) = hint {
        app.set_clamp_hint(h);
    }
}

async fn apply_fancurves<B: Backend>(app: &mut AppState<B>) {
    if let Some(curve) = app.fan_curves.get(app.active_fan_idx).cloned() {
        if let Err(err) = curve.validate() {
            app.set_status(format!("Cannot apply: {err}"), true);
            return;
        }

        let fans_to_apply = if app.apply_all_fans {
            app.fan_curves.clone()
        } else {
            vec![curve.clone()]
        };

        let target = if app.apply_all_fans {
            "All Fans".to_string()
        } else {
            format!("{} Fan", curve.fan_name)
        };

        app.confirm_modal = Some((
            format!("Apply custom fan curve ({target}) to hardware EC for {}?", app.platform_profile.as_str()),
            ConfirmAction::ApplyAllFanCurves(app.platform_profile, fans_to_apply),
        ));
    }
}

fn next_aura_field(current: usize, dir: i32, brightness: u32) -> usize {
    let accessible: &[usize] = if brightness == 0 {
        &[0, 9, 10, 11, 12]
    } else {
        &[
            0, 1, 7, 8,                     // Panel 1: Brightness, Mode, Speed, Direction
            2, 3, 4, 5, 6,                  // Panel 2: Color 1, R, G, B, Color 2
            13, 14, 15, 16,                 // Panel 3, Col 1: Animations
            17, 18, 19, 20,                 // Panel 3, Col 2: Audio & Visual Sync
            21, 22, 23, 24,                 // Panel 3, Col 3: Telemetry
            9, 10, 11, 12,                  // Panel 4: Hardware Power States
        ]
    };

    let cur_pos = accessible.iter().position(|&f| f == current).unwrap_or(0);
    let count = accessible.len() as i32;
    let next_pos = ((cur_pos as i32 + dir + count) % count) as usize;
    accessible[next_pos]
}

async fn handle_aura<B: Backend>(app: &mut AppState<B>, key: KeyEvent) {
    match key.code {
        KeyCode::Down | KeyCode::Char('j') => {
            app.aura_selected_field = next_aura_field(app.aura_selected_field, 1, app.aura_brightness);
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.aura_selected_field = next_aura_field(app.aura_selected_field, -1, app.aura_brightness);
        }
        KeyCode::Left | KeyCode::Char('h') => {
            if (17..=20).contains(&app.aura_selected_field) {
                app.aura_selected_field -= 4; // Move from Col 2 to Col 1
            } else if (21..=24).contains(&app.aura_selected_field) {
                app.aura_selected_field -= 4; // Move from Col 3 to Col 2
            } else {
                adjust_aura_field(app, -1).await;
            }
        }
        KeyCode::Right | KeyCode::Char('l') => {
            if (13..=16).contains(&app.aura_selected_field) {
                app.aura_selected_field += 4; // Move from Col 1 to Col 2
            } else if (17..=20).contains(&app.aura_selected_field) {
                app.aura_selected_field += 4; // Move from Col 2 to Col 3
            } else {
                adjust_aura_field(app, 1).await;
            }
        }
        KeyCode::Char('c') => {
            if app.aura_brightness > 0 {
                let slot = if app.aura_selected_field == 6 { 2 } else { 1 };
                app.open_color_picker(slot);
            } else {
                app.set_status("Backlight is Off. Increase brightness to access colors.", true);
            }
        }
        KeyCode::Enter => {
            if app.aura_brightness > 0 {
                if app.aura_selected_field == 2 || (3..=5).contains(&app.aura_selected_field) {
                    app.open_color_picker(1);
                    return;
                } else if app.aura_selected_field == 6 {
                    app.open_color_picker(2);
                    return;
                }
            }
            adjust_aura_field(app, 1).await;
        }
        KeyCode::Char(' ') => {
            adjust_aura_field(app, 1).await;
        }
        KeyCode::Char('w') => {
            app.confirm_modal = Some((
                "Save Aura lighting and power configuration to hardware firmware / BIOS?".to_string(),
                ConfirmAction::SaveAuraFirmware,
            ));
        }
        _ => {}
    }
}

async fn activate_software_mode<B: Backend>(app: &mut AppState<B>, effect: SoftwareEffect, status_msg: &str) {
    app.active_software_effect = effect;
    app.last_effect_color = None;
    app.aura_mode = 0;
    app.aura_data.mode = 0;
    let _ = app.backend.set_aura_mode(0).await;
    app.apply_software_effect().await;
    app.set_status(status_msg, false);
}

async fn toggle_custom_effect<B: Backend>(app: &mut AppState<B>, effect: SoftwareEffect, desc: &str) {
    if app.active_software_effect == effect {
        app.active_software_effect = SoftwareEffect::None;
        app.last_effect_color = None;
        app.set_status(format!("{} deactivated", effect.label()), false);
    } else {
        activate_software_mode(app, effect, &format!("{} active: {}", effect.label(), desc)).await;
    }
}

async fn adjust_aura_field<B: Backend>(app: &mut AppState<B>, dir: i32) {
    // Guardrail: if brightness is Off (0), block all effect and color configuration
    if app.aura_brightness == 0 && ((1..=8).contains(&app.aura_selected_field) || (13..=24).contains(&app.aura_selected_field)) {
        app.set_status("Backlight is Off. Increase brightness to access effects.", true);
        return;
    }

    let (theme_name, theme_col) = crate::ui::theme::Theme::get_system_theme_color();
    let presets = [
        theme_col,                     // Dynamic System Theme
        RgbColor::new(235, 219, 178),  // Gruvbox (#ebdbb2)
        RgbColor::new(245, 194, 231),  // Catppuccin (#f5c2e7)
        RgbColor::new(0, 255, 170),    // Cyberpunk (#00ffaa)
        RgbColor::new(136, 192, 208),  // Nord (#88c0d0)
        RgbColor::new(255, 122, 0),    // Sunset Amber (#ff7a00)
        RgbColor::new(255, 0, 0),      // Red
        RgbColor::new(0, 255, 0),      // Green
        RgbColor::new(0, 136, 255),    // Blue
        RgbColor::new(255, 255, 255),  // White
    ];

    match app.aura_selected_field {
        0 => {
            let b = (app.aura_brightness as i32 + dir).clamp(0, 3) as u32;
            app.aura_brightness = b;
            let _ = app.backend.set_aura_brightness(b).await;
            if b == 0 {
                // Brightness off: move focus back to brightness row and stop any software effect
                if (1..=24).contains(&app.aura_selected_field) {
                    app.aura_selected_field = 0;
                }
                app.active_software_effect = SoftwareEffect::None;
                app.last_effect_color = None;
            }
        }
        1 => {
            let current_idx = match app.active_software_effect {
                SoftwareEffect::RainbowWave => 4,
                SoftwareEffect::ColorCycle => 5,
                SoftwareEffect::FireFlicker => 6,
                SoftwareEffect::Strobe => 7,
                SoftwareEffect::VoicePulse => 8,
                SoftwareEffect::AudioPulse => 9,
                SoftwareEffect::AudioWave => 10,
                SoftwareEffect::AmbientLight => 11,
                SoftwareEffect::Heatmap => 12,
                SoftwareEffect::GpuMode => 13,
                SoftwareEffect::BatteryStatus => 14,
                SoftwareEffect::CpuLoad => 15,
                SoftwareEffect::None => match app.aura_mode {
                    1 => 1,
                    2 => 2,
                    10 => 3,
                    _ => 0,
                },
            };
            let next_idx = ((current_idx + dir + 16) % 16) as usize;
            match next_idx {
                0 => {
                    app.active_software_effect = SoftwareEffect::None;
                    app.last_effect_color = None;
                    app.aura_mode = 0;
                    app.aura_data.mode = 0;
                    let _ = app.backend.set_aura_mode(0).await;
                    let _ = app.backend.set_aura_mode_data(&app.aura_data).await;
                    app.set_status("Aura Mode: Static", false);
                }
                1 => {
                    app.active_software_effect = SoftwareEffect::None;
                    app.last_effect_color = None;
                    app.aura_mode = 1;
                    app.aura_data.mode = 1;
                    let _ = app.backend.set_aura_mode(1).await;
                    let _ = app.backend.set_aura_mode_data(&app.aura_data).await;
                    app.set_status("Aura Mode: Breathe", false);
                }
                2 => {
                    app.active_software_effect = SoftwareEffect::None;
                    app.last_effect_color = None;
                    app.aura_mode = 2;
                    app.aura_data.mode = 2;
                    let _ = app.backend.set_aura_mode(2).await;
                    let _ = app.backend.set_aura_mode_data(&app.aura_data).await;
                    app.set_status("Aura Mode: Rainbow Cycle", false);
                }
                3 => {
                    app.active_software_effect = SoftwareEffect::None;
                    app.last_effect_color = None;
                    app.aura_mode = 10;
                    app.aura_data.mode = 10;
                    let _ = app.backend.set_aura_mode(10).await;
                    let _ = app.backend.set_aura_mode_data(&app.aura_data).await;
                    app.set_status("Aura Mode: Pulse", false);
                }
                4 => activate_software_mode(app, SoftwareEffect::RainbowWave, "Rainbow Wave active (360° spectrum sweep)").await,
                5 => activate_software_mode(app, SoftwareEffect::ColorCycle, "Color Cycle active (7 primary colors)").await,
                6 => activate_software_mode(app, SoftwareEffect::FireFlicker, "Fire Flicker active (fireplace embers)").await,
                7 => activate_software_mode(app, SoftwareEffect::Strobe, "Strobe / Flash active (pulsing pulse)").await,
                8 => activate_software_mode(app, SoftwareEffect::VoicePulse, "Voice Pulse active (speech reactive via clean mic)").await,
                9 => activate_software_mode(app, SoftwareEffect::AudioPulse, "Audio Pulse active (speaker output reactive)").await,
                10 => activate_software_mode(app, SoftwareEffect::AudioWave, "Audio Wave active (frequency spectrum EQ)").await,
                11 => activate_software_mode(app, SoftwareEffect::AmbientLight, "Ambient Light active (Wayland screen sync)").await,
                12 => activate_software_mode(app, SoftwareEffect::Heatmap, "Heatmap active (CPU/GPU temp -> color)").await,
                13 => activate_software_mode(app, SoftwareEffect::GpuMode, "GPU Mode active (GPU mode -> color)").await,
                14 => activate_software_mode(app, SoftwareEffect::BatteryStatus, "Battery Status active (charge state -> color)").await,
                15 => activate_software_mode(app, SoftwareEffect::CpuLoad, "CPU Load active (load % -> color)").await,
                _ => {}
            }
        }
        2 => {
            let cur_pos = presets.iter().position(|p| *p == app.aura_data.color1).unwrap_or(0);
            let count = presets.len() as i32;
            let next_pos = ((cur_pos as i32 + dir + count) % count) as usize;
            app.aura_data.color1 = presets[next_pos];
            if app.aura_mode == 0 {
                let _ = app.backend.set_aura_mode_data(&app.aura_data).await;
            }
            let label = match next_pos {
                0 => format!("Preset applied: {} ({})", theme_name, theme_col.to_hex()),
                1 => "Preset applied: Gruvbox (#EBDBB2)".to_string(),
                2 => "Preset applied: Catppuccin (#F5C2E7)".to_string(),
                3 => "Preset applied: Cyberpunk (#00FFAA)".to_string(),
                4 => "Preset applied: Nord (#88C0D0)".to_string(),
                5 => "Preset applied: Sunset Amber (#FF7A00)".to_string(),
                6 => "Preset applied: Red (#FF0000)".to_string(),
                7 => "Preset applied: Green (#00FF00)".to_string(),
                8 => "Preset applied: Blue (#0088FF)".to_string(),
                _ => "Preset applied: White (#FFFFFF)".to_string(),
            };
            app.set_status(label, false);
        }
        3 => {
            let r = (app.aura_data.color1.r as i16 + (dir * 5) as i16).clamp(0, 255) as u8;
            app.aura_data.color1.r = r;
            if app.aura_mode == 0 {
                let _ = app.backend.set_aura_mode_data(&app.aura_data).await;
            }
        }
        4 => {
            let g = (app.aura_data.color1.g as i16 + (dir * 5) as i16).clamp(0, 255) as u8;
            app.aura_data.color1.g = g;
            if app.aura_mode == 0 {
                let _ = app.backend.set_aura_mode_data(&app.aura_data).await;
            }
        }
        5 => {
            let b = (app.aura_data.color1.b as i16 + (dir * 5) as i16).clamp(0, 255) as u8;
            app.aura_data.color1.b = b;
            if app.aura_mode == 0 {
                let _ = app.backend.set_aura_mode_data(&app.aura_data).await;
            }
        }
        6 => {
            let cur_pos = presets.iter().position(|p| *p == app.aura_data.color2).unwrap_or(0);
            let count = presets.len() as i32;
            let next_pos = ((cur_pos as i32 + dir + count) % count) as usize;
            app.aura_data.color2 = presets[next_pos];
        }
        7 => {
            let speeds = ["low", "med", "high"];
            let idx = speeds.iter().position(|s| s.eq_ignore_ascii_case(&app.aura_data.speed)).unwrap_or(1);
            let n = ((idx as i32 + dir + 3) % 3) as usize;
            app.aura_data.speed = speeds[n].to_string();
        }
        8 => {
            app.aura_data.direction = if app.aura_data.direction.eq_ignore_ascii_case("right") {
                "Left".to_string()
            } else {
                "Right".to_string()
            };
        }
        9 => {
            app.aura_power.boot = !app.aura_power.boot;
            let _ = app.backend.set_aura_power(&app.aura_power).await;
        }
        10 => {
            app.aura_power.awake = !app.aura_power.awake;
            let _ = app.backend.set_aura_power(&app.aura_power).await;
        }
        11 => {
            app.aura_power.sleep = !app.aura_power.sleep;
            let _ = app.backend.set_aura_power(&app.aura_power).await;
        }
        12 => {
            app.aura_power.shutdown = !app.aura_power.shutdown;
            let _ = app.backend.set_aura_power(&app.aura_power).await;
        }
        13 => toggle_custom_effect(app, SoftwareEffect::RainbowWave, "360° spectrum sweep").await,
        14 => toggle_custom_effect(app, SoftwareEffect::ColorCycle, "7 primary colors").await,
        15 => toggle_custom_effect(app, SoftwareEffect::FireFlicker, "fireplace embers").await,
        16 => toggle_custom_effect(app, SoftwareEffect::Strobe, "pulsing pulse").await,
        17 => toggle_custom_effect(app, SoftwareEffect::VoicePulse, "speech reactive via clean mic").await,
        18 => toggle_custom_effect(app, SoftwareEffect::AudioPulse, "sound output reactive").await,
        19 => toggle_custom_effect(app, SoftwareEffect::AudioWave, "frequency spectrum EQ").await,
        20 => toggle_custom_effect(app, SoftwareEffect::AmbientLight, "Wayland screen ambient sync").await,
        21 => toggle_custom_effect(app, SoftwareEffect::Heatmap, "CPU/GPU temp -> color").await,
        22 => toggle_custom_effect(app, SoftwareEffect::GpuMode, "GPU mode -> color").await,
        23 => toggle_custom_effect(app, SoftwareEffect::BatteryStatus, "charge state -> color").await,
        24 => toggle_custom_effect(app, SoftwareEffect::CpuLoad, "load % -> color").await,
        _ => {}
    }
}

async fn adjust_picker_slider<B: Backend>(app: &mut AppState<B>, dir: i32) {
    match app.picker_slider_idx {
        0 => {
            app.picker_hue = (app.picker_hue + dir as f32 * 5.0).rem_euclid(360.0);
        }
        1 => {
            app.picker_sat = (app.picker_sat + dir as f32 * 2.0).clamp(0.0, 100.0);
        }
        2 => {
            app.picker_val = (app.picker_val + dir as f32 * 2.0).clamp(0.0, 100.0);
        }
        3 => {
            let mut col = app.picker_current_color();
            col.r = (col.r as i16 + dir as i16 * 5).clamp(0, 255) as u8;
            let (h, s, v) = col.to_hsv();
            app.picker_hue = h;
            app.picker_sat = s;
            app.picker_val = v;
        }
        4 => {
            let mut col = app.picker_current_color();
            col.g = (col.g as i16 + dir as i16 * 5).clamp(0, 255) as u8;
            let (h, s, v) = col.to_hsv();
            app.picker_hue = h;
            app.picker_sat = s;
            app.picker_val = v;
        }
        5 => {
            let mut col = app.picker_current_color();
            col.b = (col.b as i16 + dir as i16 * 5).clamp(0, 255) as u8;
            let (h, s, v) = col.to_hsv();
            app.picker_hue = h;
            app.picker_sat = s;
            app.picker_val = v;
        }
        _ => {}
    }

    if app.picker_target_slot == 1 && app.aura_mode == 0 {
        let cur = app.picker_current_color();
        app.aura_data.color1 = cur;
        let _ = app.backend.set_aura_mode_data(&app.aura_data).await;
    }
}

async fn handle_gpu<B: Backend>(app: &mut AppState<B>, key: KeyEvent) {
    match key.code {
        KeyCode::Down | KeyCode::Char('j') => {
            if !app.gpu_supported.is_empty() {
                app.gpu_selected_idx = (app.gpu_selected_idx + 1) % app.gpu_supported.len();
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if !app.gpu_supported.is_empty() {
                app.gpu_selected_idx = (app.gpu_selected_idx + app.gpu_supported.len() - 1) % app.gpu_supported.len();
            }
        }
        KeyCode::Enter => {
            if let Some(&mode) = app.gpu_supported.get(app.gpu_selected_idx) {
                if mode == app.gpu_mode {
                    app.set_status(format!("{} is already the active mode", mode.as_str()), false);
                } else {
                    app.confirm_modal = Some((
                        format!("Switch GPU mode to {}? A session logout or reboot may be required.", mode.as_str()),
                        ConfirmAction::SwitchGpu(mode),
                    ));
                }
            }
        }
        _ => {}
    }
}

async fn handle_armoury<B: Backend>(app: &mut AppState<B>, key: KeyEvent) {
    if app.armoury_attributes.is_empty() {
        return;
    }
    match key.code {
        KeyCode::Char('u') => {
            app.show_unavailable_armoury = !app.show_unavailable_armoury;
            app.set_status(
                if app.show_unavailable_armoury {
                    "Unavailable attributes: shown"
                } else {
                    "Unavailable attributes: collapsed"
                },
                false,
            );
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let visible_indices: Vec<usize> = app
                .armoury_attributes
                .iter()
                .enumerate()
                .filter(|(_, a)| a.is_available || app.show_unavailable_armoury)
                .map(|(i, _)| i)
                .collect();
            if !visible_indices.is_empty() {
                let cur_pos = visible_indices.iter().position(|&i| i == app.armoury_selected_idx).unwrap_or(0);
                let next_pos = (cur_pos + 1) % visible_indices.len();
                app.armoury_selected_idx = visible_indices[next_pos];
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            let visible_indices: Vec<usize> = app
                .armoury_attributes
                .iter()
                .enumerate()
                .filter(|(_, a)| a.is_available || app.show_unavailable_armoury)
                .map(|(i, _)| i)
                .collect();
            if !visible_indices.is_empty() {
                let cur_pos = visible_indices.iter().position(|&i| i == app.armoury_selected_idx).unwrap_or(0);
                let next_pos = (cur_pos + visible_indices.len() - 1) % visible_indices.len();
                app.armoury_selected_idx = visible_indices[next_pos];
            }
        }
        KeyCode::Left | KeyCode::Char('h') => {
            adjust_armoury_attr(app, -1).await;
        }
        KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter | KeyCode::Char(' ') => {
            adjust_armoury_attr(app, 1).await;
        }
        KeyCode::Char('r') => {
            if let Some(attr) = app.armoury_attributes.get(app.armoury_selected_idx).cloned() {
                if attr.is_available {
                    app.confirm_modal = Some((
                        format!("Restore factory BIOS default for {}?", attr.display_name),
                        ConfirmAction::RestoreArmouryDefault {
                            node_name: attr.node_name,
                            display_name: attr.display_name,
                        },
                    ));
                }
            }
        }
        _ => {}
    }
}

async fn adjust_armoury_attr<B: Backend>(app: &mut AppState<B>, dir: i32) {
    if let Some(attr) = app.armoury_attributes.get(app.armoury_selected_idx).cloned() {
        if !attr.is_available {
            return;
        }
        let node = attr.node_name.clone();
        let new_val = match attr.kind {
            crate::backend::types::AttributeKind::Toggle => {
                if attr.current_value == 1 { 0 } else { 1 }
            }
            crate::backend::types::AttributeKind::Enum => {
                if attr.possible_values.is_empty() {
                    return;
                }
                let cur_idx = attr.possible_values.iter().position(|v| *v == attr.current_value).unwrap_or(0);
                let count = attr.possible_values.len() as i32;
                let next_idx = ((cur_idx as i32 + dir + count) % count) as usize;
                attr.possible_values[next_idx]
            }
            crate::backend::types::AttributeKind::Slider => {
                let inc = if attr.scalar_increment > 0 { attr.scalar_increment } else { 1 };
                (attr.current_value + dir * inc).clamp(attr.min_value, attr.max_value)
            }
            crate::backend::types::AttributeKind::ReadOnly => return,
        };

        if new_val == attr.current_value {
            return;
        }

        // Extra confirmation modal for hardware-sensitive switches and power limits
        let is_sensitive = matches!(
            node.as_str(),
            "gpu_mux_mode"
                | "dgpu_disable"
                | "ppt_pl1_spl"
                | "ppt_pl2_sppt"
                | "nv_base_tgp"
                | "nv_tgp"
                | "nv_temp_target"
                | "charge_mode"
        );

        if is_sensitive {
            let msg = if node == "gpu_mux_mode" || node == "dgpu_disable" {
                let label = if node == "gpu_mux_mode" {
                    if new_val == 0 { "dGPU Direct (MUX)" } else { "Optimus / Hybrid" }
                } else if new_val == 1 {
                    "Disabled (dGPU off)"
                } else {
                    "Enabled (dGPU on)"
                };
                format!("Switch {} to {}? This hardware change requires a reboot.", attr.display_name, label)
            } else {
                format!("Apply hardware setting for {}: {} -> {}?", attr.display_name, attr.current_value, new_val)
            };

            app.confirm_modal = Some((
                msg,
                ConfirmAction::ApplyArmouryAttr {
                    node_name: node,
                    value: new_val,
                    display_name: attr.display_name,
                },
            ));
            return;
        }

        match app.backend.set_armoury_attribute(&node, new_val).await {
            Ok(()) => {
                if let Some(a) = app.armoury_attributes.get_mut(app.armoury_selected_idx) {
                    a.current_value = new_val;
                }
                app.set_status(format!("{} updated to {new_val}", attr.display_name), false);
            }
            Err(e) => app.set_status(format!("Failed to write attribute: {e}"), true),
        }
    }
}
