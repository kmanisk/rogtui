use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{ActiveTab, AppState, ConfirmAction};
use crate::backend::types::PlatformProfile;
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
                                app.set_status(format!("Fan curve applied to {} successfully", profile.as_str()), false);
                            }
                            Err(e) => app.set_status(format!("Failed to apply curve: {e}"), true),
                        }
                    }
                    ConfirmAction::ResetFanCurve(profile) => {
                        match app.backend.set_curves_to_defaults(profile).await {
                            Ok(()) => {
                                app.fan_curves = app.backend.get_fan_curves(profile).await.unwrap_or_default();
                                app.set_status(format!("Fan curves reset to defaults for {}", profile.as_str()), false);
                            }
                            Err(e) => app.set_status(format!("Failed to reset curves: {e}"), true),
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
        KeyCode::Char('p') => {
            let next_p = match app.platform_profile {
                PlatformProfile::Quiet => PlatformProfile::Balanced,
                PlatformProfile::Balanced => PlatformProfile::Performance,
                PlatformProfile::Performance => PlatformProfile::Quiet,
                PlatformProfile::Custom => PlatformProfile::Balanced,
            };
            if let Err(e) = app.backend.set_platform_profile(next_p).await {
                app.set_status(format!("Failed to cycle profile: {e}"), true);
            } else {
                app.platform_profile = next_p;
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
            app.set_status(format!("Platform profile: {}", next_p.as_str()), false);
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
        KeyCode::Char('f') => {
            if !app.fan_curves.is_empty() {
                app.active_fan_idx = (app.active_fan_idx + 1) % app.fan_curves.len();
            }
        }
        KeyCode::Left | KeyCode::Char('h') => {
            app.active_point_idx = (app.active_point_idx + 7) % 8;
        }
        KeyCode::Right | KeyCode::Char('l') => {
            app.active_point_idx = (app.active_point_idx + 1) % 8;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            let delta = if key.modifiers.contains(KeyModifiers::SHIFT) { 5 } else { 1 };
            adjust_curve_speed(app, delta);
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let delta = if key.modifiers.contains(KeyModifiers::SHIFT) { -5 } else { -1 };
            adjust_curve_speed(app, delta);
        }
        KeyCode::Char('[') => {
            adjust_curve_temp(app, -1);
        }
        KeyCode::Char(']') => {
            adjust_curve_temp(app, 1);
        }
        KeyCode::Char('e') => {
            if let Some(curve) = app.fan_curves.get_mut(app.active_fan_idx) {
                let new_v = !curve.enabled;
                curve.enabled = new_v;
                let _ = app.backend.set_fan_curves_enabled(app.platform_profile, new_v).await;
                app.set_status(format!("Custom curves enabled: {new_v}"), false);
            }
        }
        KeyCode::Char('r') => {
            app.confirm_modal = Some((
                format!("Reset fan curves to factory BIOS defaults for {}?", app.platform_profile.as_str()),
                ConfirmAction::ResetFanCurve(app.platform_profile),
            ));
        }
        KeyCode::Char('w') => {
            if let Some(curve) = app.fan_curves.get(app.active_fan_idx) {
                if let Err(err) = curve.validate() {
                    app.set_status(format!("Cannot apply: {err}"), true);
                } else {
                    app.confirm_modal = Some((
                        format!("Apply custom fan curve to {} Fan in {} profile?", curve.fan_name, app.platform_profile.as_str()),
                        ConfirmAction::ApplyFanCurve(app.platform_profile, curve.clone()),
                    ));
                }
            }
        }
        KeyCode::Char('u') => {
            if let Some(good) = &app.last_known_good_curve {
                if let Some(curr) = app.fan_curves.get_mut(app.active_fan_idx) {
                    *curr = good.clone();
                    app.set_status("Curve reverted to last known good state", false);
                }
            }
        }
        _ => {}
    }
}

fn adjust_curve_speed<B: Backend>(app: &mut AppState<B>, delta: i16) {
    if let Some(curve) = app.fan_curves.get_mut(app.active_fan_idx) {
        if let Some(pt) = curve.points.get_mut(app.active_point_idx) {
            let cur = pt.pwm_percent as i16;
            pt.pwm_percent = (cur + delta).clamp(0, 100) as u8;
        }
    }
}

fn adjust_curve_temp<B: Backend>(app: &mut AppState<B>, delta: i16) {
    if let Some(curve) = app.fan_curves.get_mut(app.active_fan_idx) {
        if let Some(pt) = curve.points.get_mut(app.active_point_idx) {
            let cur = pt.temp as i16;
            pt.temp = (cur + delta).clamp(30, 110) as u8;
        }
    }
}

async fn handle_aura<B: Backend>(app: &mut AppState<B>, key: KeyEvent) {
    match key.code {
        KeyCode::Down | KeyCode::Char('j') => {
            app.aura_selected_field = (app.aura_selected_field + 1) % 13;
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.aura_selected_field = (app.aura_selected_field + 12) % 13;
        }
        KeyCode::Left | KeyCode::Char('h') => {
            adjust_aura_field(app, -1).await;
        }
        KeyCode::Right | KeyCode::Char('l') => {
            adjust_aura_field(app, 1).await;
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            adjust_aura_field(app, 1).await;
        }
        KeyCode::Char('w') => {
            if let Err(e) = app.backend.set_aura_mode_data(&app.aura_data).await {
                app.set_status(format!("Error applying aura: {e}"), true);
            } else {
                let _ = app.backend.set_aura_brightness(app.aura_brightness).await;
                let _ = app.backend.set_aura_power(&app.aura_power).await;
                app.set_status("Aura lighting and power state applied to hardware", false);
            }
        }
        _ => {}
    }
}

async fn adjust_aura_field<B: Backend>(app: &mut AppState<B>, dir: i32) {
    match app.aura_selected_field {
        0 => {
            let b = (app.aura_brightness as i32 + dir).clamp(0, 3) as u32;
            app.aura_brightness = b;
            let _ = app.backend.set_aura_brightness(b).await;
        }
        1 => {
            let modes = [0, 1, 2, 10];
            let cur_idx = modes.iter().position(|m| *m == app.aura_mode).unwrap_or(0);
            let next_idx = ((cur_idx as i32 + dir + 4) % 4) as usize;
            app.aura_mode = modes[next_idx];
            app.aura_data.mode = modes[next_idx];
            let _ = app.backend.set_aura_mode(modes[next_idx]).await;
        }
        3 => {
            let r = (app.aura_data.color1.r as i16 + (dir * 5) as i16).clamp(0, 255) as u8;
            app.aura_data.color1.r = r;
        }
        4 => {
            let g = (app.aura_data.color1.g as i16 + (dir * 5) as i16).clamp(0, 255) as u8;
            app.aura_data.color1.g = g;
        }
        5 => {
            let b = (app.aura_data.color1.b as i16 + (dir * 5) as i16).clamp(0, 255) as u8;
            app.aura_data.color1.b = b;
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
        9 => app.aura_power.boot = !app.aura_power.boot,
        10 => app.aura_power.awake = !app.aura_power.awake,
        11 => app.aura_power.sleep = !app.aura_power.sleep,
        12 => app.aura_power.shutdown = !app.aura_power.shutdown,
        _ => {}
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
        KeyCode::Down | KeyCode::Char('j') => {
            app.armoury_selected_idx = (app.armoury_selected_idx + 1) % app.armoury_attributes.len();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.armoury_selected_idx = (app.armoury_selected_idx + app.armoury_attributes.len() - 1) % app.armoury_attributes.len();
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
                    let node = attr.node_name.clone();
                    if app.backend.restore_armoury_default(&node).await.is_ok() {
                        if let Some(a) = app.armoury_attributes.get_mut(app.armoury_selected_idx) {
                            a.current_value = a.default_value;
                        }
                        app.set_status(format!("Default restored for {}", attr.display_name), false);
                    }
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
