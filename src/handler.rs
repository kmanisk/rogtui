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

    // 2. If confirmation modal is open, handle y/n/Esc/Enter
    if let Some((_, action)) = app.confirm_modal.clone() {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
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

    // 2b. If hex input is active, process hex digits, backspace, Enter, Esc
    if let Some(mut buf) = app.hex_input_buffer.take() {
        match key.code {
            KeyCode::Esc => {
                app.hex_input_buffer = None;
                app.set_status("Hex entry cancelled", false);
            }
            KeyCode::Enter => {
                match RgbColor::from_hex(&buf) {
                    Ok(col) => {
                        let slot = if app.aura_selected_field == 6 { 2 } else { 1 };
                        if slot == 2 {
                            app.aura_data.color2 = col;
                        } else {
                            app.aura_data.color1 = col;
                        }
                        app.user_custom_color = true;
                        if app.aura_brightness > 0 {
                            if app.aura_mode == 0 && app.active_software_effect == SoftwareEffect::None {
                                let _ = app.backend.write_volatile_rgb(col).await;
                            } else if app.active_software_effect == SoftwareEffect::None {
                                let _ = app.backend.set_aura_mode_data(&app.aura_data).await;
                            } else {
                                app.apply_software_effect().await;
                            }
                        }
                        app.set_status(format!("Set color to {}", col.to_hex()), false);
                    }
                    Err(e) => {
                        app.set_status(format!("Invalid hex: {e}"), true);
                    }
                }
                app.hex_input_buffer = None;
            }
            KeyCode::Backspace => {
                buf.pop();
                app.hex_input_buffer = Some(buf);
            }
            KeyCode::Char(c) if c.is_ascii_hexdigit() && buf.len() < 6 => {
                buf.push(c);
                app.hex_input_buffer = Some(buf);
            }
            _ => {
                app.hex_input_buffer = Some(buf);
            }
        }
        return;
    }

    // 2c. If advanced power modal is open, dismiss on Esc/Enter/a/q
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
                }
                if app.aura_mode == 0 && app.aura_brightness > 0 {
                    let _ = app.backend.write_volatile_rgb(app.aura_data.color1).await;
                } else if app.active_software_effect == SoftwareEffect::None && app.aura_brightness > 0 {
                    let _ = app.backend.set_aura_mode_data(&app.aura_data).await;
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
                if app.aura_mode == 0 && app.aura_brightness > 0 {
                    let _ = app.backend.write_volatile_rgb(app.aura_data.color1).await;
                } else if app.active_software_effect == SoftwareEffect::None && app.aura_brightness > 0 {
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
        KeyCode::Char('J') => {
            app.current_tab = app.current_tab.next();
            return;
        }
        KeyCode::Char('K') => {
            app.current_tab = app.current_tab.prev();
            return;
        }
        KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::SHIFT) => {
            app.current_tab = app.current_tab.next();
            return;
        }
        KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::SHIFT) => {
            app.current_tab = app.current_tab.prev();
            return;
        }
        KeyCode::Char('1') => { app.current_tab = ActiveTab::Dashboard; return; }
        KeyCode::Char('2') => { app.current_tab = ActiveTab::Power; return; }
        KeyCode::Char('3') => { app.current_tab = ActiveTab::FanCurves; return; }
        KeyCode::Char('4') => { app.current_tab = ActiveTab::Aura; return; }
        KeyCode::Char('5') => { app.current_tab = ActiveTab::System; return; }
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
                app.set_status(format!("Profile set to {}", next_p.as_str()), false);
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
        ActiveTab::System => handle_system(app, key).await,
    }
}

async fn handle_dashboard<B: Backend>(app: &mut AppState<B>, key: KeyEvent) {
    if matches!(key.code, KeyCode::Enter | KeyCode::Char(' ')) {
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
            app.set_status(format!("Profile set to {}", next_p.as_str()), false);
        }
    }
}

async fn handle_system<B: Backend>(app: &mut AppState<B>, key: KeyEvent) {
    let has_auto_bright = app.sensors.screen_auto_brightness.is_some();
    let max_rows = if has_auto_bright { 3 } else { 2 };
    let export_row = if has_auto_bright { 2 } else { 1 };

    match key.code {
        KeyCode::Down | KeyCode::Char('j') => {
            app.system_selected_row = (app.system_selected_row + 1) % max_rows;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.system_selected_row = (app.system_selected_row + max_rows - 1) % max_rows;
        }
        KeyCode::Left | KeyCode::Char('h') | KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter | KeyCode::Char(' ') => {
            if app.system_selected_row == export_row {
                export_system_report(app);
            } else {
                toggle_system_row(app).await;
            }
        }
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            export_system_report(app);
        }
        _ => {}
    }
}

async fn toggle_system_row<B: Backend>(app: &mut AppState<B>) {
    let has_auto_bright = app.sensors.screen_auto_brightness.is_some();
    match app.system_selected_row {
        0 => {
            let cur = app.sensors.panel_overdrive.unwrap_or(true);
            let next_val = if cur { 0 } else { 1 };
            if let Err(e) = app.backend.set_armoury_attribute("panel_overdrive", next_val).await {
                app.set_status(format!("Failed to toggle Panel Overdrive: {e}"), true);
            } else {
                app.sensors.panel_overdrive = Some(next_val == 1);
                app.set_status(format!("Panel Overdrive {}", if next_val == 1 { "enabled" } else { "disabled" }), false);
            }
        }
        1 if has_auto_bright => {
            let cur = app.sensors.screen_auto_brightness.unwrap_or(false);
            let next_val = if cur { 0 } else { 1 };
            if let Err(e) = app.backend.set_armoury_attribute("screen_auto_brightness", next_val).await {
                let msg = e.to_string();
                if msg.contains("Input/output error") || msg.contains("os error 5") {
                    app.sensors.screen_auto_brightness = None;
                    app.system_selected_row = 0;
                    app.set_status("Auto-Brightness is not supported on this hardware", true);
                } else {
                    app.set_status(format!("Failed to toggle Auto-Brightness: {e}"), true);
                }
            } else {
                app.sensors.screen_auto_brightness = Some(next_val == 1);
                app.set_status(format!("Auto-Brightness {}", if next_val == 1 { "enabled" } else { "disabled" }), false);
            }
        }
        _ => {}
    }
}

fn export_system_report<B: Backend>(app: &mut AppState<B>) {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let state_dir = std::path::PathBuf::from(&home).join(".local/state/rogtui");
    let _ = std::fs::create_dir_all(&state_dir);

    let mut buf = [0u8; 64];
    let time = unsafe { libc::time(std::ptr::null_mut()) };
    let tm = unsafe { libc::localtime(&time) };
    let len = unsafe {
        libc::strftime(
            buf.as_mut_ptr() as *mut libc::c_char,
            buf.len(),
            c"%Y-%m-%d-%H%M%S".as_ptr(),
            tm,
        )
    };
    let timestamp = if len > 0 {
        String::from_utf8_lossy(&buf[..len]).to_string()
    } else {
        "report".to_string()
    };

    let filename = format!("report-{timestamp}.md");
    let file_path = state_dir.join(&filename);

    let mut doc = String::new();
    doc.push_str("# rogtui System Diagnostics & Hardware Capabilities Report\n\n");
    doc.push_str(&format!("- Date: {}\n", timestamp));
    doc.push_str(&format!("- Model: {}\n", crate::ui::system::detect_model()));
    doc.push_str(&format!("- Processor: {}\n", crate::ui::system::detect_cpu()));
    doc.push_str(&format!("- Graphics: {}\n", crate::ui::system::detect_gpu()));
    doc.push_str(&format!("- Memory: {}\n", crate::ui::system::detect_ram()));
    doc.push_str(&format!("- NVIDIA Driver: {}\n", crate::ui::system::detect_nvidia_driver()));
    doc.push_str(&format!("- BIOS: {}\n", crate::ui::system::detect_bios()));
    doc.push_str(&format!("- Kernel: {}\n", crate::ui::system::detect_kernel()));
    doc.push_str(&format!("- Architecture: {}\n", crate::ui::system::detect_arch()));
    doc.push_str(&format!("- Platform Driver: {}\n", crate::ui::system::detect_platform_driver()));
    doc.push_str(&format!("- Session: {}\n", crate::ui::system::detect_session()));
    doc.push_str(&format!("- Display: {}\n", crate::ui::system::detect_display()));
    doc.push_str(&format!("- ROGTUI Version: {}\n\n", env!("CARGO_PKG_VERSION")));

    doc.push_str("## Power & Platform\n\n");
    doc.push_str(&format!("- Platform Profile: {}\n", app.platform_profile.as_str()));
    doc.push_str(&format!("- Auto AC Profile: {}\n", if app.auto_ac { app.profile_ac.as_str() } else { "Off" }));
    doc.push_str(&format!("- Auto Battery Profile: {}\n", if app.auto_battery { app.profile_battery.as_str() } else { "Off" }));
    doc.push_str(&format!("- Charge Limit: {}%\n", app.charge_limit));
    doc.push_str(&format!("- Battery Status: {}\n", app.sensors.battery_status));
    doc.push_str(&format!("- Battery Health: {}%\n", app.sensors.battery_health_percent.unwrap_or(98)));
    if let Some(cycles) = app.sensors.battery_cycle_count {
        doc.push_str(&format!("- Battery Cycle Count: {}\n", cycles));
    }
    if let Some(w) = app.sensors.battery_power_w {
        doc.push_str(&format!("- Power Draw: {:.1} W\n", w));
    }

    doc.push_str("\n## GPU Architecture\n\n");
    doc.push_str(&format!("- Current GPU Mode: {}\n", app.gpu_mode.as_str()));
    doc.push_str(&format!("- GPU Power State: {}\n", app.gpu_power.as_str()));

    doc.push_str("\n## Fan Curves\n\n");
    for fc in &app.fan_curves {
        doc.push_str(&format!("### {} Fan Curve (Enabled: {})\n", fc.fan_name, fc.enabled));
        for (i, pt) in fc.points.iter().enumerate() {
            doc.push_str(&format!("  - Point {}: {}°C -> {}%\n", i + 1, pt.temp, pt.pwm_percent));
        }
    }

    doc.push_str("\n## Aura RGB Lighting\n\n");
    doc.push_str(&format!("- Brightness: {}\n", app.aura_brightness));
    doc.push_str(&format!("- Mode: {}\n", app.aura_mode));
    doc.push_str(&format!("- Primary Color: {}\n", app.aura_data.color1.to_hex()));
    doc.push_str(&format!("- Secondary Color: {}\n", app.aura_data.color2.to_hex()));

    match std::fs::write(&file_path, doc) {
        Ok(_) => app.set_status(format!("Report saved to ~/.local/state/rogtui/{filename}"), false),
        Err(e) => app.set_status(format!("Failed to write report: {e}"), true),
    }
}

async fn handle_power<B: Backend>(app: &mut AppState<B>, key: KeyEvent) {
    match key.code {
        KeyCode::Down | KeyCode::Char('j') => {
            app.power_selected_field = (app.power_selected_field + 1) % 5;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.power_selected_field = (app.power_selected_field + 4) % 5;
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
                app.set_status(format!("Profile set to {}", next_p.as_str()), false);
            }
        }
        1 => {
            let options = [None, Some(PlatformProfile::Quiet), Some(PlatformProfile::Balanced), Some(PlatformProfile::Performance)];
            let cur_idx = if !app.auto_ac { 0 } else {
                match app.profile_ac {
                    PlatformProfile::Quiet => 1,
                    PlatformProfile::Balanced => 2,
                    PlatformProfile::Performance => 3,
                    _ => 2,
                }
            };
            let next_idx = ((cur_idx + dir + 4) % 4) as usize;
            match options[next_idx] {
                None => {
                    let _ = app.backend.set_auto_ac(false).await;
                    app.auto_ac = false;
                    app.set_status("When plugged in: Off", false);
                }
                Some(p) => {
                    let _ = app.backend.set_auto_ac(true).await;
                    let _ = app.backend.set_profile_ac(p).await;
                    app.auto_ac = true;
                    app.profile_ac = p;
                    app.set_status(format!("When plugged in: {}", p.as_str()), false);
                }
            }
        }
        2 => {
            let options = [None, Some(PlatformProfile::Quiet), Some(PlatformProfile::Balanced), Some(PlatformProfile::Performance)];
            let cur_idx = if !app.auto_battery { 0 } else {
                match app.profile_battery {
                    PlatformProfile::Quiet => 1,
                    PlatformProfile::Balanced => 2,
                    PlatformProfile::Performance => 3,
                    _ => 1,
                }
            };
            let next_idx = ((cur_idx + dir + 4) % 4) as usize;
            match options[next_idx] {
                None => {
                    let _ = app.backend.set_auto_battery(false).await;
                    app.auto_battery = false;
                    app.set_status("On battery: Off", false);
                }
                Some(p) => {
                    let _ = app.backend.set_auto_battery(true).await;
                    let _ = app.backend.set_profile_battery(p).await;
                    app.auto_battery = true;
                    app.profile_battery = p;
                    app.set_status(format!("On battery: {}", p.as_str()), false);
                }
            }
        }
        3 => {
            let step = (dir * 5) as i16;
            let current = app.charge_limit as i16;
            let clamped = (current + step).clamp(20, 100) as u8;
            if app.backend.set_charge_limit(clamped).await.is_ok() {
                app.charge_limit = clamped;
                app.sensors.battery_charge_limit = clamped;
                app.set_status(format!("Charge limit set to {clamped}%"), false);
            }
        }
        4 => {
            if app.backend.one_shot_full_charge().await.is_ok() {
                app.set_status("Full charge override active (100% until disconnect)", false);
            } else {
                app.set_status("Failed to trigger full charge override", true);
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
        // Point selection: j/k or Down/Up (vertical navigation between rows 1..8)
        KeyCode::Down | KeyCode::Char('j') => {
            app.active_point_idx = (app.active_point_idx + 1) % 8;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.active_point_idx = (app.active_point_idx + 7) % 8;
        }

        // Adjust active column value: h/l or Left/Right (horizontal value adjustment)
        KeyCode::Left | KeyCode::Char('h') => {
            let delta = if key.modifiers.contains(KeyModifiers::ALT) { -5 } else { -1 };
            if app.fan_selected_col == 0 {
                adjust_fancurve_speed(app, delta);
            } else {
                adjust_fancurve_temp(app, delta);
            }
        }
        KeyCode::Right | KeyCode::Char('l') => {
            let delta = if key.modifiers.contains(KeyModifiers::ALT) { 5 } else { 1 };
            if app.fan_selected_col == 0 {
                adjust_fancurve_speed(app, delta);
            } else {
                adjust_fancurve_temp(app, delta);
            }
        }

        // Toggle editing between Speed (%) and Temperature (°C)
        KeyCode::Char(' ') | KeyCode::Char('t') => {
            app.fan_selected_col = if app.fan_selected_col == 0 { 1 } else { 0 };
            let col_name = if app.fan_selected_col == 0 { "Speed (%)" } else { "Temperature (°C)" };
            app.set_status(format!("Editing {col_name}"), false);
        }

        // Direct temperature adjustment shortcuts: [ / ] or H / L
        KeyCode::Char('[') | KeyCode::Char('H') => {
            let delta = if key.modifiers.contains(KeyModifiers::ALT) { -5 } else { -1 };
            adjust_fancurve_temp(app, delta);
        }
        KeyCode::Char(']') | KeyCode::Char('L') => {
            let delta = if key.modifiers.contains(KeyModifiers::ALT) { 5 } else { 1 };
            adjust_fancurve_temp(app, delta);
        }

        // Direct speed adjustment shortcuts: - / +
        KeyCode::Char('-') => {
            let delta = if key.modifiers.contains(KeyModifiers::ALT) { -5 } else { -1 };
            adjust_fancurve_speed(app, delta);
        }
        KeyCode::Char('+') | KeyCode::Char('=') => {
            let delta = if key.modifiers.contains(KeyModifiers::ALT) { 5 } else { 1 };
            adjust_fancurve_speed(app, delta);
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
        KeyCode::Char('c') | KeyCode::Char('e') => {
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

fn next_aura_field<B: Backend>(app: &AppState<B>, current: usize, dir: i32) -> usize {
    let mut all: Vec<usize> = vec![0, 1, 7, 8, 2, 3, 4, 5, 6, 9, 10, 11, 12];

    if app.has_slash_led {
        all.push(13);
        all.push(14);
    }

    let accessible: Vec<usize> = all
        .iter()
        .copied()
        .filter(|&f| crate::ui::aura::is_aura_field_supported(app, f))
        .collect();

    let cur_pos = accessible.iter().position(|&f| f == current).unwrap_or(0);
    let count = accessible.len() as i32;
    let next_pos = ((cur_pos as i32 + dir + count) % count) as usize;
    accessible[next_pos]
}

async fn handle_aura<B: Backend>(app: &mut AppState<B>, key: KeyEvent) {
    match key.code {
        KeyCode::Down | KeyCode::Char('j') => {
            app.aura_selected_field = next_aura_field(app, app.aura_selected_field, 1);
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.aura_selected_field = next_aura_field(app, app.aura_selected_field, -1);
        }
        KeyCode::Left | KeyCode::Char('h') => {
            adjust_aura_field(app, -1).await;
        }
        KeyCode::Right | KeyCode::Char('l') => {
            adjust_aura_field(app, 1).await;
        }
        KeyCode::Char('c') => {
            if app.aura_brightness == 0 || !crate::ui::aura::supports_custom_color(app) {
                return;
            }
            let slot = if app.aura_selected_field == 6 { 2 } else { 1 };
            app.open_color_picker(slot);
        }
        KeyCode::Char('#') => {
            if app.aura_brightness == 0 || !crate::ui::aura::supports_custom_color(app) {
                return;
            }
            app.hex_input_buffer = Some(String::new());
            app.set_status("Enter 6 hex digits (e.g. ff4080) and press Enter:", false);
        }
        KeyCode::Enter => {
            if app.aura_selected_field == 2 || (3..=5).contains(&app.aura_selected_field) {
                app.open_color_picker(1);
                return;
            } else if app.aura_selected_field == 6 {
                app.open_color_picker(2);
                return;
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

async fn adjust_aura_field<B: Backend>(app: &mut AppState<B>, dir: i32) {
    if app.aura_brightness == 0 && (1..=8).contains(&app.aura_selected_field) {
        return;
    }
    if (2..=5).contains(&app.aura_selected_field) && !crate::ui::aura::supports_custom_color(app) {
        return;
    }
    match app.aura_selected_field {
        0 => {
            let b = (app.aura_brightness as i32 + dir).clamp(0, 3) as u32;
            app.aura_brightness = b;
            let _ = app.backend.set_aura_brightness(b).await;
            if b == 0 {
                if (1..=8).contains(&app.aura_selected_field) {
                    app.aura_selected_field = 0;
                }
                if app.active_software_effect != SoftwareEffect::None {
                    app.saved_software_effect = Some(app.active_software_effect);
                }
                app.last_effect_color = None;
                let _ = app.backend.write_volatile_rgb(RgbColor::new(0, 0, 0)).await;
            } else {
                if let Some(saved) = app.saved_software_effect.take() {
                    app.active_software_effect = saved;
                }
                if app.active_software_effect == SoftwareEffect::None && app.aura_mode == 0 {
                    let _ = app.backend.write_volatile_rgb(app.aura_data.color1).await;
                } else {
                    app.apply_software_effect().await;
                }
            }
        }
        1 => {
            let current_idx = crate::ui::aura::get_current_mode_index(app) as i32;
            let next_idx = ((current_idx + dir + 19) % 19) as usize;
            match next_idx {
                0 => {
                    app.active_software_effect = SoftwareEffect::None;
                    app.last_effect_color = None;
                    app.aura_mode = 0;
                    app.aura_data.mode = 0;
                    let _ = app.backend.set_aura_mode(0).await;
                    if app.aura_brightness > 0 {
                        let _ = app.backend.write_volatile_rgb(app.aura_data.color1).await;
                    }
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
                7 => activate_software_mode(app, SoftwareEffect::Clouds, "Clouds active (calm cyan & sky blue drift)").await,
                8 => activate_software_mode(app, SoftwareEffect::ThemeSync, "Theme Sync active (desktop palette sync)").await,
                9 => activate_software_mode(app, SoftwareEffect::Strobe, "Strobe / Flash active (pulsing pulse)").await,
                10 => activate_software_mode(app, SoftwareEffect::VoicePulse, "Voice Pulse active (speech reactive via clean mic)").await,
                11 => activate_software_mode(app, SoftwareEffect::AudioPulse, "Audio Pulse active (speaker output reactive)").await,
                12 => activate_software_mode(app, SoftwareEffect::AudioWave, "Audio Wave active (frequency spectrum EQ)").await,
                13 => activate_software_mode(app, SoftwareEffect::AmbientLight, "Ambient Screen Sync active (Wayland screen sync)").await,
                14 => activate_software_mode(app, SoftwareEffect::AmbientAudioSync, "Ambient Audio Sync active (Screen color + Audio dynamics)").await,
                15 => activate_software_mode(app, SoftwareEffect::Heatmap, "Heatmap active (CPU/GPU temp -> color)").await,
                16 => activate_software_mode(app, SoftwareEffect::GpuMode, "GPU Mode active (GPU mode -> color)").await,
                17 => activate_software_mode(app, SoftwareEffect::BatteryStatus, "Battery Status active (charge state -> color)").await,
                18 => activate_software_mode(app, SoftwareEffect::CpuLoad, "CPU Load active (load % -> color)").await,
                _ => {}
            }
        }
        2 => {
            let presets = crate::ui::aura::get_color_presets();
            let cur_pos = presets.iter().position(|(_, p)| *p == app.aura_data.color1).unwrap_or(0);
            let count = presets.len() as i32;
            let next_pos = ((cur_pos as i32 + dir + count) % count) as usize;
            app.aura_data.color1 = presets[next_pos].1;
            app.user_custom_color = next_pos != 0;
            if app.aura_mode == 0 && app.aura_brightness > 0 {
                let _ = app.backend.write_volatile_rgb(app.aura_data.color1).await;
            } else if app.active_software_effect == SoftwareEffect::None && app.aura_brightness > 0 {
                let _ = app.backend.set_aura_mode_data(&app.aura_data).await;
            }
            let preset_name = presets[next_pos].0;
            app.set_status(format!("Preset applied: {} ({})", preset_name, app.aura_data.color1.to_hex()), false);
        }
        3 => {
            let r = (app.aura_data.color1.r as i16 + (dir * 5) as i16).clamp(0, 255) as u8;
            app.aura_data.color1.r = r;
            app.user_custom_color = true;
            if app.aura_mode == 0 && app.aura_brightness > 0 {
                let _ = app.backend.write_volatile_rgb(app.aura_data.color1).await;
            } else if app.active_software_effect == SoftwareEffect::None && app.aura_brightness > 0 {
                let _ = app.backend.set_aura_mode_data(&app.aura_data).await;
            }
        }
        4 => {
            let g = (app.aura_data.color1.g as i16 + (dir * 5) as i16).clamp(0, 255) as u8;
            app.aura_data.color1.g = g;
            app.user_custom_color = true;
            if app.aura_mode == 0 && app.aura_brightness > 0 {
                let _ = app.backend.write_volatile_rgb(app.aura_data.color1).await;
            } else if app.active_software_effect == SoftwareEffect::None && app.aura_brightness > 0 {
                let _ = app.backend.set_aura_mode_data(&app.aura_data).await;
            }
        }
        5 => {
            let b = (app.aura_data.color1.b as i16 + (dir * 5) as i16).clamp(0, 255) as u8;
            app.aura_data.color1.b = b;
            app.user_custom_color = true;
            if app.aura_mode == 0 && app.aura_brightness > 0 {
                let _ = app.backend.write_volatile_rgb(app.aura_data.color1).await;
            } else if app.active_software_effect == SoftwareEffect::None && app.aura_brightness > 0 {
                let _ = app.backend.set_aura_mode_data(&app.aura_data).await;
            }
        }
        6 => {
            let presets = crate::ui::aura::get_color_presets();
            let cur_pos = presets.iter().position(|(_, p)| *p == app.aura_data.color2).unwrap_or(0);
            let count = presets.len() as i32;
            let next_pos = ((cur_pos as i32 + dir + count) % count) as usize;
            app.aura_data.color2 = presets[next_pos].1;
            if app.active_software_effect == SoftwareEffect::None && app.aura_brightness > 0 {
                let _ = app.backend.set_aura_mode_data(&app.aura_data).await;
            }
            let preset_name = presets[next_pos].0;
            app.set_status(format!("Secondary Preset applied: {} ({})", preset_name, app.aura_data.color2.to_hex()), false);
        }
        7 => {
            if app.active_software_effect == SoftwareEffect::VoicePulse {
                let current = app.voice_sensitivity as i32;
                let step = 5;
                let next = (current + dir * step).clamp(10, 100) as u8;
                app.voice_sensitivity = next;
                app.audio_service.set_voice_sensitivity(next);
                app.set_status(format!("Mic sensitivity set to {}%", next), false);
            } else {
                let speeds = ["Low", "Med", "High"];
                let idx = speeds.iter().position(|s| s.eq_ignore_ascii_case(&app.aura_data.speed)).unwrap_or(1);
                let n = ((idx as i32 + dir + 3) % 3) as usize;
                app.aura_data.speed = speeds[n].to_string();
                if app.active_software_effect == SoftwareEffect::None && app.aura_brightness > 0 {
                    let _ = app.backend.set_aura_mode_data(&app.aura_data).await;
                }
            }
        }
        8 => {
            app.aura_data.direction = if app.aura_data.direction.eq_ignore_ascii_case("right") {
                "Left".to_string()
            } else {
                "Right".to_string()
            };
            if app.active_software_effect == SoftwareEffect::None && app.aura_brightness > 0 {
                let _ = app.backend.set_aura_mode_data(&app.aura_data).await;
            }
        }
        9 => {
            app.aura_power.boot = !app.aura_power.boot;
            let state = if app.aura_power.boot { "enabled" } else { "disabled" };
            app.set_status(format!("Boot backlight {state}"), false);
            let _ = app.backend.set_aura_power(&app.aura_power).await;
        }
        10 => {
            app.aura_power.awake = !app.aura_power.awake;
            let state = if app.aura_power.awake { "enabled" } else { "disabled" };
            app.set_status(format!("Awake backlight {state}"), false);
            let _ = app.backend.set_aura_power(&app.aura_power).await;
        }
        11 => {
            app.aura_power.sleep = !app.aura_power.sleep;
            let state = if app.aura_power.sleep { "enabled" } else { "disabled" };
            app.set_status(format!("Sleep backlight {state}"), false);
            let _ = app.backend.set_aura_power(&app.aura_power).await;
        }
        12 => {
            app.aura_power.shutdown = !app.aura_power.shutdown;
            let state = if app.aura_power.shutdown { "enabled" } else { "disabled" };
            app.set_status(format!("Shutdown backlight {state}"), false);
            let _ = app.backend.set_aura_power(&app.aura_power).await;
        }
        13 => {
            let modes = ["Off", "Static", "Bounce", "Slash", "Flow", "Spectrum"];
            let cur_idx = modes.iter().position(|m| m.eq_ignore_ascii_case(&app.slash_mode)).unwrap_or(1);
            let next_idx = ((cur_idx as i32 + dir + modes.len() as i32) % modes.len() as i32) as usize;
            app.slash_mode = modes[next_idx].to_string();
            let _ = app.backend.set_slash_mode(&app.slash_mode).await;
            app.set_status(format!("Slash Mode: {}", app.slash_mode), false);
        }
        14 => {
            let b = (app.slash_brightness as i32 + dir).clamp(0, 3) as u32;
            app.slash_brightness = b;
            let _ = app.backend.set_slash_brightness(b).await;
            let b_str = match b {
                0 => "Off",
                1 => "Low",
                2 => "Med",
                _ => "High",
            };
            app.set_status(format!("Slash Brightness: {b_str}"), false);
        }
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

    let cur = app.picker_current_color();
    if app.picker_target_slot == 2 {
        app.aura_data.color2 = cur;
    } else {
        app.aura_data.color1 = cur;
    }
    if app.aura_mode == 0 && app.aura_brightness > 0 && app.picker_target_slot == 1 {
        let _ = app.backend.write_volatile_rgb(app.aura_data.color1).await;
    } else if app.active_software_effect == SoftwareEffect::None && app.aura_brightness > 0 {
        let _ = app.backend.set_aura_mode_data(&app.aura_data).await;
    }
}


