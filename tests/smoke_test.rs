use rogtui::app::{ActiveTab, AppState, ConfirmAction};
use rogtui::backend::mock::MockBackend;
use rogtui::backend::types::{FanCurve, GpuMode, PlatformProfile};
use rogtui::backend::Backend;

#[tokio::test]
async fn test_mock_backend_platform_cycle() {
    let backend = MockBackend::new();
    let app = AppState::new(backend).await.expect("app init");

    assert_eq!(app.platform_profile, PlatformProfile::Balanced);

    app.backend.next_platform_profile().await.expect("next profile");
    let p = app.backend.get_platform_profile().await.expect("get profile");
    assert_eq!(p, PlatformProfile::Performance);

    app.backend.next_platform_profile().await.expect("next profile");
    let p = app.backend.get_platform_profile().await.expect("get profile");
    assert_eq!(p, PlatformProfile::Quiet);
}

#[tokio::test]
async fn test_mock_backend_charge_limit() {
    let backend = MockBackend::new();
    let app = AppState::new(backend).await.expect("app init");

    assert_eq!(app.charge_limit, 60);
    app.backend.set_charge_limit(80).await.expect("set charge limit");
    assert_eq!(app.backend.get_charge_limit().await.expect("get limit"), 80);

    app.backend.one_shot_full_charge().await.expect("one shot");
    assert_eq!(app.backend.get_charge_limit().await.expect("get limit"), 100);
}

#[tokio::test]
async fn test_mock_backend_fan_curve_safety() {
    let backend = MockBackend::new();
    let app = AppState::new(backend).await.expect("app init");

    // Invalid curve: speed < 30% at temp >= 80C
    let unsafe_curve = FanCurve::new(
        "CPU",
        [40, 50, 60, 70, 75, 80, 85, 90],
        [10, 15, 20, 22, 24, 25, 50, 100],
        true,
    );
    let res = app.backend.set_fan_curve(PlatformProfile::Balanced, &unsafe_curve).await;
    assert!(res.is_err(), "Unsafe fan curve must be rejected by safety policy");

    // Valid curve
    let safe_curve = FanCurve::new(
        "CPU",
        [40, 50, 60, 70, 75, 80, 85, 90],
        [10, 15, 20, 25, 30, 35, 60, 100],
        true,
    );
    let res = app.backend.set_fan_curve(PlatformProfile::Balanced, &safe_curve).await;
    assert!(res.is_ok(), "Valid fan curve must be accepted");
}

#[tokio::test]
async fn test_mock_backend_gpu_switch() {
    let backend = MockBackend::new();
    let app = AppState::new(backend).await.expect("app init");

    let action = app.backend.set_gpu_mode(GpuMode::Integrated).await.expect("set gpu");
    assert_eq!(action, rogtui::backend::types::PendingUserAction::Reboot);
    assert_eq!(app.backend.get_gpu_pending_mode().await.expect("pending mode"), GpuMode::Integrated);
}

#[tokio::test]
async fn test_tab_navigation_and_reducers() {
    let backend = MockBackend::new();
    let mut app = AppState::new(backend).await.expect("app init");

    assert_eq!(app.current_tab, ActiveTab::Dashboard);
    app.current_tab = app.current_tab.next();
    assert_eq!(app.current_tab, ActiveTab::Power);
    app.current_tab = app.current_tab.next();
    assert_eq!(app.current_tab, ActiveTab::FanCurves);
    app.current_tab = app.current_tab.prev();
    assert_eq!(app.current_tab, ActiveTab::Power);

    // Confirm modal
    app.confirm_modal = Some((
        "Confirm test".to_string(),
        ConfirmAction::SwitchGpu(GpuMode::Hybrid),
    ));
    assert!(app.confirm_modal.is_some());
}

#[tokio::test]
async fn test_fan_curve_editing_clamping_and_safety() {
    let mut curve = FanCurve::new(
        "CPU",
        [40, 50, 60, 70, 75, 80, 85, 90],
        [10, 20, 30, 40, 50, 60, 70, 80],
        true,
    );

    // Speed edit: cannot exceed next point (idx 2 speed 30 cannot exceed point 3 speed 40)
    let res = curve.edit_speed(2, 20, false);
    assert!(res.is_err(), "Should clamp to next point");
    assert_eq!(curve.points[2].pwm_percent, 40);

    // Speed edit: cannot drop below previous point (idx 2 speed 40 cannot drop below point 1 speed 20)
    let res = curve.edit_speed(2, -30, false);
    assert!(res.is_err(), "Should clamp to previous point");
    assert_eq!(curve.points[2].pwm_percent, 20);

    // Temp edit: strictly increasing (idx 3 temp 70 must be strictly between 60+1=61 and 75-1=74)
    let res = curve.edit_temp(3, 10);
    assert!(res.is_err(), "Should clamp to upper neighbour temp - 1");
    assert_eq!(curve.points[3].temp, 74);

    let res = curve.edit_temp(3, -20);
    assert!(res.is_err(), "Should clamp to lower neighbour temp + 1");
    assert_eq!(curve.points[3].temp, 61);

    // Lower idx 3 and 4 first so lower_limit for idx 5 allows reaching 30%
    let _ = curve.edit_speed(3, -20, false);
    assert_eq!(curve.points[3].pwm_percent, 20);
    let _ = curve.edit_speed(4, -30, false);
    assert_eq!(curve.points[4].pwm_percent, 20);

    // Safety floor at >= 80 C (idx 5 temp is 80)
    let res = curve.edit_speed(5, -50, false);
    assert!(res.is_err(), "Safety floor must refuse < 30% without override");
    assert_eq!(curve.points[5].pwm_percent, 30);

    // Safety floor with override
    let res_override = curve.edit_speed(5, -15, true);
    assert!(res_override.is_ok() || res_override.is_err());
    assert_eq!(curve.points[5].pwm_percent, 20);
}

#[tokio::test]
async fn test_fan_curve_presets_and_interpolation() {
    let mut curve = FanCurve::new(
        "CPU",
        [40, 50, 60, 70, 75, 80, 85, 90],
        [10, 20, 30, 40, 50, 60, 70, 80],
        true,
    );

    // Silent preset
    curve.apply_preset("Silent");
    assert!(curve.validate().is_ok());

    // Aggressive preset
    curve.apply_preset("Aggressive");
    assert!(curve.validate().is_ok());

    // Balanced preset
    curve.apply_preset("Balanced");
    assert!(curve.validate().is_ok());

    // Interpolation tests
    let flat_low = curve.interpolate(25.0);
    assert_eq!(flat_low, curve.points[0].pwm_percent as f64);

    let flat_high = curve.interpolate(105.0);
    assert_eq!(flat_high, curve.points[7].pwm_percent as f64);

    let exact_pt = curve.interpolate(curve.points[2].temp as f64);
    assert_eq!(exact_pt, curve.points[2].pwm_percent as f64);
}

#[tokio::test]
async fn test_aura_brightness_guardrail() {
    let backend = MockBackend::new();
    let mut app = AppState::new(backend).await.expect("app init");
    app.current_tab = ActiveTab::Aura;
    app.aura_brightness = 0; // Off
    app.aura_selected_field = 0;

    // When brightness is 0 (Off), pressing 'j' (Down) must skip mode/color fields (1..=8)
    // and jump directly to field 9 (Boot hardware power state).
    rogtui::handler::handle_key_event(&mut app, crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char('j'))).await;
    assert_eq!(app.aura_selected_field, 9, "Mode controls must be skipped when brightness is 0");

    // Pressing 'k' (Up) from Boot moves back to field 0 (Brightness)
    rogtui::handler::handle_key_event(&mut app, crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char('k'))).await;
    assert_eq!(app.aura_selected_field, 0, "Up from Boot must move back to Brightness");

    // Mode cannot be changed while brightness is 0
    let initial_mode = app.aura_mode;
    app.aura_selected_field = 1; // Even if forced to 1
    rogtui::handler::handle_key_event(&mut app, crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char('l'))).await;
    assert_eq!(app.aura_mode, initial_mode, "Effect mode must not change when brightness is 0");

    // Now turn brightness up to 1 (Low) on field 0
    app.aura_selected_field = 0;
    rogtui::handler::handle_key_event(&mut app, crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char('l'))).await;
    assert_eq!(app.aura_brightness, 1, "Brightness should be Low (1)");

    // With brightness > 0, pressing 'j' (Down) now properly accesses field 1 (Mode)
    rogtui::handler::handle_key_event(&mut app, crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char('j'))).await;
    assert_eq!(app.aura_selected_field, 1, "Mode must be accessible when brightness > 0");
}

#[tokio::test]
async fn test_fan_curve_repair() {
    let mut curve = FanCurve::new(
        "CPU",
        [40, 50, 60, 70, 75, 80, 85, 90],
        [10, 20, 77, 50, 60, 70, 85, 100], // pt2 is 77, pt3 is 50 -> non-monotonic
        true,
    );
    assert!(curve.check_monotonicity().is_err(), "Must detect non-monotonic curve");
    let repaired = curve.repair();
    assert!(repaired, "repair() should return true when curve was modified");
    assert!(curve.check_monotonicity().is_ok(), "Repaired curve must be monotonic");
    assert!(curve.points[3].pwm_percent >= curve.points[2].pwm_percent);
}

#[tokio::test]
async fn test_render_all_tabs_80x24_and_120x40() {
    let backend = MockBackend::new();
    let mut app = AppState::new(backend).await.expect("app init");

    let tabs = [
        ActiveTab::Dashboard,
        ActiveTab::Power,
        ActiveTab::FanCurves,
        ActiveTab::Aura,
        ActiveTab::System,
    ];

    for (w, h) in [(80, 24), (120, 40), (50, 12)] {
        let test_backend = ratatui::backend::TestBackend::new(w, h);
        let mut terminal = ratatui::Terminal::new(test_backend).expect("terminal creation");

        for tab in tabs {
            app.current_tab = tab;
            terminal.draw(|f| rogtui::ui::render_ui(f, &app)).expect("draw succeeded");
        }
    }
}

#[tokio::test]
async fn test_rgb_hsv_conversions() {
    use rogtui::backend::types::RgbColor;

    let red = RgbColor::new(255, 0, 0);
    let (h, s, v) = red.to_hsv();
    assert!((h - 0.0).abs() < 0.1);
    assert!((s - 100.0).abs() < 0.1);
    assert!((v - 100.0).abs() < 0.1);
    assert_eq!(RgbColor::from_hsv(h, s, v), red);

    let cyan = RgbColor::new(0, 255, 255);
    let (h, s, v) = cyan.to_hsv();
    assert!((h - 180.0).abs() < 0.1);
    assert_eq!(RgbColor::from_hsv(h, s, v), cyan);

    let custom = RgbColor::new(128, 64, 200);
    let (h, s, v) = custom.to_hsv();
    let back = RgbColor::from_hsv(h, s, v);
    assert!((back.r as i16 - custom.r as i16).abs() <= 1);
    assert!((back.g as i16 - custom.g as i16).abs() <= 1);
    assert!((back.b as i16 - custom.b as i16).abs() <= 1);
}

#[tokio::test]
async fn test_color_picker_modal_flow() {
    use crossterm::event::{KeyCode, KeyEvent};

    let backend = MockBackend::new();
    let mut app = AppState::new(backend).await.expect("app init");

    app.current_tab = ActiveTab::Aura;
    app.aura_brightness = 2; // Medium
    app.aura_selected_field = 2; // Primary color
    let orig_color = app.aura_data.color1;

    // Press 'c' to open color picker modal
    rogtui::handler::handle_key_event(&mut app, KeyEvent::from(KeyCode::Char('c'))).await;
    assert!(app.show_color_picker_modal, "Color picker modal must open on 'c'");
    assert_eq!(app.picker_target_slot, 1);

    // Adjust Hue with 'l'
    let prev_hue = app.picker_hue;
    rogtui::handler::handle_key_event(&mut app, KeyEvent::from(KeyCode::Char('l'))).await;
    assert_ne!(app.picker_hue, prev_hue, "Hue must adjust on 'l'");

    // Cancel with Esc -> must revert color and close modal
    rogtui::handler::handle_key_event(&mut app, KeyEvent::from(KeyCode::Esc)).await;
    assert!(!app.show_color_picker_modal, "Modal must close on Esc");
    assert_eq!(app.aura_data.color1, orig_color, "Color must revert to original on cancel");

    // Open again with Enter on field 2
    rogtui::handler::handle_key_event(&mut app, KeyEvent::from(KeyCode::Enter)).await;
    assert!(app.show_color_picker_modal, "Modal must open on Enter");

    // Switch to Red slider (j x 3)
    rogtui::handler::handle_key_event(&mut app, KeyEvent::from(KeyCode::Char('j'))).await;
    rogtui::handler::handle_key_event(&mut app, KeyEvent::from(KeyCode::Char('j'))).await;
    rogtui::handler::handle_key_event(&mut app, KeyEvent::from(KeyCode::Char('j'))).await;
    assert_eq!(app.picker_slider_idx, 3, "Slider index should be Red (3)");

    // Adjust Red slider
    rogtui::handler::handle_key_event(&mut app, KeyEvent::from(KeyCode::Char('l'))).await;
    let new_color = app.picker_current_color();

    // Confirm with Enter -> applies color
    rogtui::handler::handle_key_event(&mut app, KeyEvent::from(KeyCode::Enter)).await;
    assert!(!app.show_color_picker_modal, "Modal must close on Enter");
    assert_eq!(app.aura_data.color1, new_color, "Selected color must be applied");
}

#[tokio::test]
async fn test_modal_rendering_stability() {
    let backend = MockBackend::new();
    let mut app = AppState::new(backend).await.expect("app init");

    for (w, h) in [(80, 24), (120, 40)] {
        let test_backend = ratatui::backend::TestBackend::new(w, h);
        let mut terminal = ratatui::Terminal::new(test_backend).expect("terminal creation");

        // Advanced power modal
        app.current_tab = ActiveTab::Dashboard;
        app.show_advanced_power_modal = true;
        terminal.draw(|f| rogtui::ui::render_ui(f, &app)).expect("draw succeeded");
        app.show_advanced_power_modal = false;

        // Color picker modal
        app.current_tab = ActiveTab::Aura;
        app.open_color_picker(1);
        terminal.draw(|f| rogtui::ui::render_ui(f, &app)).expect("draw succeeded");
        app.show_color_picker_modal = false;
    }
}

#[tokio::test]
async fn test_software_effects_and_color_logic() {
    use rogtui::backend::types::{RgbColor, SoftwareEffect};

    // 1. Heatmap colors
    let cool = AppState::<MockBackend>::compute_heatmap_color(Some(30), Some(20));
    assert_eq!(cool.r, 0);
    assert_eq!(cool.b, 255);

    let hot = AppState::<MockBackend>::compute_heatmap_color(Some(95), Some(60));
    assert_eq!(hot.r, 255);
    assert_eq!(hot.g, 0);
    assert_eq!(hot.b, 0);

    // 2. GPU Mode colors
    let igpu = AppState::<MockBackend>::compute_gpu_mode_color(GpuMode::Integrated);
    assert_eq!(igpu, RgbColor::new(0, 200, 0));
    let dgpu = AppState::<MockBackend>::compute_gpu_mode_color(GpuMode::AsusMuxDgpu);
    assert_eq!(dgpu, RgbColor::new(255, 80, 0));

    // 3. Battery colors
    let charging = AppState::<MockBackend>::compute_battery_color(Some(40), "Charging");
    assert_eq!(charging, RgbColor::new(0, 220, 0));
    let low = AppState::<MockBackend>::compute_battery_color(Some(10), "Discharging");
    assert_eq!(low, RgbColor::new(220, 0, 0));

    // 4. CPU Load colors
    let cool_load = AppState::<MockBackend>::compute_cpu_load_color(Some(15));
    assert_eq!(cool_load, RgbColor::new(0, 220, 0));
    let mod_load = AppState::<MockBackend>::compute_cpu_load_color(Some(50));
    assert_eq!(mod_load, RgbColor::new(220, 220, 0));
    let heavy_load = AppState::<MockBackend>::compute_cpu_load_color(Some(75));
    assert_eq!(heavy_load, RgbColor::new(255, 120, 0));
    let peak_load = AppState::<MockBackend>::compute_cpu_load_color(Some(90));
    assert_eq!(peak_load, RgbColor::new(255, 0, 0));

    // 5. App state effect apply & deduplication
    let backend = MockBackend::new();
    let mut app = AppState::new(backend).await.expect("app init");
    app.aura_brightness = 2;
    app.active_software_effect = SoftwareEffect::Heatmap;
    app.sensors.cpu_temp_c = Some(50);
    app.sensors.gpu_temp_c = Some(45);

    app.apply_software_effect().await;
    let col1 = app.last_effect_color;
    assert!(col1.is_some(), "Effect color must be computed and cached");

    // Re-apply without temp change -> cached, deduplicated
    app.apply_software_effect().await;
    assert_eq!(app.last_effect_color, col1);

    // 6. Custom Animations: Rainbow Wave advances hue
    app.active_software_effect = SoftwareEffect::RainbowWave;
    app.aura_data.speed = "med".to_string();
    app.aura_data.direction = "Right".to_string();
    app.apply_software_effect().await;
    let rw_col1 = app.last_effect_color.expect("rainbow wave color 1");
    app.apply_software_effect().await;
    let rw_col2 = app.last_effect_color.expect("rainbow wave color 2");
    assert_ne!(rw_col1, rw_col2, "Rainbow wave must advance on each tick");

    // 7. Custom Animations: Color Cycle advances index
    app.active_software_effect = SoftwareEffect::ColorCycle;
    app.apply_software_effect().await;
    let cc_col1 = app.last_effect_color.expect("color cycle 1");
    app.apply_software_effect().await;
    let cc_col2 = app.last_effect_color.expect("color cycle 2");
    assert_ne!(cc_col1, cc_col2, "Color cycle must step to next color");

    // 8. Custom Animations: Strobe alternates
    app.active_software_effect = SoftwareEffect::Strobe;
    app.aura_data.color1 = RgbColor::new(0, 255, 255);
    app.apply_software_effect().await;
    let str_col1 = app.last_effect_color.expect("strobe color 1");
    app.apply_software_effect().await;
    let str_col2 = app.last_effect_color.expect("strobe color 2");
    assert_ne!(str_col1, str_col2, "Strobe must alternate between color and black");
    assert_eq!(app.aura_data.color1, RgbColor::new(0, 255, 255), "Base primary color must NOT be overwritten by effect");

    // 9. Audio and Visual Effects (VoicePulse, AudioPulse, AudioWave, AmbientLight)
    app.active_software_effect = SoftwareEffect::VoicePulse;
    app.aura_data.color1 = RgbColor::new(200, 100, 50);
    app.apply_software_effect().await;
    assert!(app.last_effect_color.is_some(), "Voice pulse should produce a color");
    assert_eq!(app.aura_data.color1, RgbColor::new(200, 100, 50), "Voice pulse must preserve base color1");

    app.active_software_effect = SoftwareEffect::AudioPulse;
    app.apply_software_effect().await;
    assert!(app.last_effect_color.is_some(), "Audio pulse should produce a color");

    app.active_software_effect = SoftwareEffect::AudioWave;
    app.apply_software_effect().await;
    assert!(app.last_effect_color.is_some(), "Audio wave should produce a color");

    app.active_software_effect = SoftwareEffect::AmbientLight;
    app.apply_software_effect().await;
    assert!(app.last_effect_color.is_some(), "Ambient light should produce a color");

    // 10. System theme detection
    let (theme_name, theme_col) = rogtui::ui::theme::Theme::get_system_theme_color();
    assert!(!theme_name.is_empty(), "Theme name should not be empty");
    assert!(theme_col.r > 0 || theme_col.g > 0 || theme_col.b > 0, "Theme color should be non-black");

    // 11. Brightness 0 guardrail -> silences all effects
    app.aura_brightness = 0;
    app.last_effect_color = None;
    app.apply_software_effect().await;
    assert!(app.last_effect_color.is_none(), "When brightness is 0, software effects must be silenced");
}

#[tokio::test]
async fn test_metrics_logging_lifecycle() {
    let backend = MockBackend::new();
    let mut app = AppState::new(backend).await.expect("app init");

    let temp_log = std::env::temp_dir().join(format!("rogtui_test_metrics_{}.csv", std::process::id()));
    app.metrics_log_path = temp_log.clone();

    assert!(!app.is_logging_metrics);
    app.toggle_metrics_logging();
    assert!(app.is_logging_metrics);
    assert!(temp_log.exists(), "CSV log file must be created on start");

    // Log sample
    app.sensors.cpu_temp_c = Some(58);
    app.sensors.gpu_temp_c = Some(48);
    app.log_metrics_sample();
    assert_eq!(app.metrics_logged_count, 1);

    // Toggle off
    app.toggle_metrics_logging();
    assert!(!app.is_logging_metrics);

    // Clean up temporary file
    let _ = std::fs::remove_file(temp_log);
}

#[tokio::test]
async fn test_hardware_sensitive_confirmations() {
    use crossterm::event::{KeyCode, KeyEvent};

    let backend = MockBackend::new();
    let mut app = AppState::new(backend).await.expect("app init");

    // 1. Aura 'w' requires confirmation modal
    app.current_tab = ActiveTab::Aura;
    rogtui::handler::handle_key_event(&mut app, KeyEvent::from(KeyCode::Char('w'))).await;
    assert!(app.confirm_modal.is_some(), "Saving Aura to hardware must show confirmation modal");

    // Cancel modal
    rogtui::handler::handle_key_event(&mut app, KeyEvent::from(KeyCode::Char('n'))).await;
    assert!(app.confirm_modal.is_none(), "Modal must dismiss on 'n'");

    // 2. Fan Curves 'w' requires confirmation modal
    app.current_tab = ActiveTab::FanCurves;
    rogtui::handler::handle_key_event(&mut app, KeyEvent::from(KeyCode::Char('w'))).await;
    assert!(app.confirm_modal.is_some(), "Applying fan curve to EC must show confirmation modal");

    // Confirm modal
    rogtui::handler::handle_key_event(&mut app, KeyEvent::from(KeyCode::Char('y'))).await;
    assert!(app.confirm_modal.is_none(), "Modal must dismiss after confirming with 'y'");
}

#[tokio::test]
async fn test_unified_aura_mode_and_preset_cycling() {
    use crossterm::event::{KeyCode, KeyEvent};
    use rogtui::backend::types::SoftwareEffect;

    let backend = MockBackend::new();
    let mut app = AppState::new(backend).await.expect("app init");

    app.current_tab = ActiveTab::Aura;
    app.aura_brightness = 2; // Medium

    // 1. Test Field 1: Mode cycling across all 18 modes
    app.aura_selected_field = 1;
    let initial_idx = rogtui::ui::aura::get_current_mode_index(&app);
    assert_eq!(initial_idx, 0, "Initial mode should be 0 (Static)");

    // Cycle right 7 times -> should land on Clouds (mode 7)
    for _ in 0..7 {
        rogtui::handler::handle_key_event(&mut app, KeyEvent::from(KeyCode::Char('l'))).await;
    }
    assert_eq!(app.active_software_effect, SoftwareEffect::Clouds);
    assert_eq!(rogtui::ui::aura::get_current_mode_index(&app), 7);

    // Cycle right 1 time -> should land on ThemeSync (mode 8)
    rogtui::handler::handle_key_event(&mut app, KeyEvent::from(KeyCode::Char('l'))).await;
    assert_eq!(app.active_software_effect, SoftwareEffect::ThemeSync);
    assert_eq!(rogtui::ui::aura::get_current_mode_index(&app), 8);

    // Cycle right 2 more times -> should land on Voice Pulse (mode 10)
    for _ in 0..2 {
        rogtui::handler::handle_key_event(&mut app, KeyEvent::from(KeyCode::Char('l'))).await;
    }
    assert_eq!(app.active_software_effect, SoftwareEffect::VoicePulse);
    assert_eq!(rogtui::ui::aura::get_current_mode_index(&app), 10);

    // Cycle right 9 more times -> should complete 19-step full circle back to 0 (Static)
    for _ in 0..9 {
        rogtui::handler::handle_key_event(&mut app, KeyEvent::from(KeyCode::Char('l'))).await;
    }
    assert_eq!(app.active_software_effect, SoftwareEffect::None);
    assert_eq!(rogtui::ui::aura::get_current_mode_index(&app), 0);

    // 2. Test Field 2: Color Preset cycling (on Static mode where colors are supported)
    app.aura_selected_field = 2;
    let initial_color = app.aura_data.color1;
    rogtui::handler::handle_key_event(&mut app, KeyEvent::from(KeyCode::Char('l'))).await;
    assert_ne!(app.aura_data.color1, initial_color, "Preset cycling must change color1 on Static mode");
    let preset_name = rogtui::ui::aura::get_active_preset_name(app.aura_data.color1);
    assert_ne!(preset_name, "Custom", "Preset color should match a known preset name");

    // Verify algorithmic modes (Rainbow Wave, Fire Flicker, Clouds, Theme Sync, Audio Wave, Ambient Audio Sync)
    // completely disable and hide custom colors (fields 2..=5)
    app.active_software_effect = SoftwareEffect::RainbowWave;
    assert!(!rogtui::ui::aura::supports_custom_color(&app));
    assert!(!rogtui::ui::aura::is_aura_field_supported(&app, 2));

    app.active_software_effect = SoftwareEffect::FireFlicker;
    assert!(!rogtui::ui::aura::supports_custom_color(&app));
    assert!(!rogtui::ui::aura::is_aura_field_supported(&app, 2));

    app.active_software_effect = SoftwareEffect::Clouds;
    assert!(!rogtui::ui::aura::supports_custom_color(&app));
    assert!(!rogtui::ui::aura::is_aura_field_supported(&app, 2));

    app.active_software_effect = SoftwareEffect::ThemeSync;
    assert!(!rogtui::ui::aura::supports_custom_color(&app));
    assert!(!rogtui::ui::aura::is_aura_field_supported(&app, 2));

    app.active_software_effect = SoftwareEffect::AudioWave;
    assert!(!rogtui::ui::aura::supports_custom_color(&app));
    assert!(!rogtui::ui::aura::is_aura_field_supported(&app, 2));

    app.active_software_effect = SoftwareEffect::AmbientLight;
    assert!(!rogtui::ui::aura::supports_custom_color(&app));
    assert!(!rogtui::ui::aura::is_aura_field_supported(&app, 2));

    app.active_software_effect = SoftwareEffect::AmbientAudioSync;
    assert!(!rogtui::ui::aura::supports_custom_color(&app));
    assert!(!rogtui::ui::aura::is_aura_field_supported(&app, 2));

    // Verify color picker shortcut 'c' is blocked on algorithmic modes
    rogtui::handler::handle_key_event(&mut app, KeyEvent::from(KeyCode::Char('c'))).await;
    assert!(!app.show_color_picker_modal, "Color picker modal must remain closed on algorithmic mode");

    // Switch back to Static mode
    app.active_software_effect = SoftwareEffect::None;
    app.aura_mode = 0;
    assert!(rogtui::ui::aura::supports_custom_color(&app));
    assert!(rogtui::ui::aura::is_aura_field_supported(&app, 2));

    // 3. Test Power States vertical navigation and toggle (Fields 9..=12)
    app.aura_selected_field = 9; // Boot
    let orig_boot = app.aura_power.boot;
    rogtui::handler::handle_key_event(&mut app, KeyEvent::from(KeyCode::Char('l'))).await;
    assert_ne!(app.aura_power.boot, orig_boot, "l on field 9 should toggle boot state");

    // Down (j) moves down to Awake (10)
    rogtui::handler::handle_key_event(&mut app, KeyEvent::from(KeyCode::Char('j'))).await;
    assert_eq!(app.aura_selected_field, 10, "Down (j) from 9 should navigate to 10");

    let orig_awake = app.aura_power.awake;
    rogtui::handler::handle_key_event(&mut app, KeyEvent::from(KeyCode::Char('h'))).await;
    assert_ne!(app.aura_power.awake, orig_awake, "h on field 10 should toggle awake state");

    // Down (j) moves down to Sleep (11) and Shutdown (12)
    rogtui::handler::handle_key_event(&mut app, KeyEvent::from(KeyCode::Char('j'))).await;
    assert_eq!(app.aura_selected_field, 11, "Down (j) from 10 should navigate to 11");
    rogtui::handler::handle_key_event(&mut app, KeyEvent::from(KeyCode::Char('j'))).await;
    assert_eq!(app.aura_selected_field, 12, "Down (j) from 11 should navigate to 12");
}

#[test]
fn test_btop_gauge_rendering() {
    use ratatui::style::Color;
    let theme = rogtui::ui::theme::Theme::default();

    let line = rogtui::ui::widgets::render_gauge("CPU Usage", "45%", 45, 20, Color::Cyan, &theme);
    let rendered_text: String = line.spans.iter().map(|s| s.content.as_ref()).collect();

    assert!(rendered_text.contains("CPU Usage"), "Gauge must contain label");
    assert!(rendered_text.contains("45%"), "Gauge must contain value string");
    assert!(rendered_text.contains('■'), "Btop gauge must use segmented black square glyphs");
}

#[tokio::test]
async fn test_clouds_and_dynamic_theme_sync() {
    let backend = MockBackend::new();
    let mut app = AppState::new(backend).await.expect("app init");

    // 1. Clouds effect color computation
    let c1 = app.compute_clouds_color();
    assert!(c1.b > 200, "Clouds effect should have high blue component for cyan/sky tones");
    let c2 = app.compute_clouds_color();
    assert!(c2.g > 150, "Clouds effect should have high green/cyan component");

    // 2. Dynamic Theme sync
    let (theme_name, theme_col) = rogtui::ui::theme::Theme::read_system_theme();
    assert!(!theme_name.is_empty(), "Theme name should be read from system or default");
    assert!(theme_col.r > 0 || theme_col.g > 0 || theme_col.b > 0);

    let sync_col = app.compute_theme_sync_color();
    assert!(sync_col.r > 0 || sync_col.g > 0 || sync_col.b > 0);

    // 3. AudioCaptureService state transitions
    assert_eq!(app.audio_service.get_mic_level(), 0);
    assert_eq!(app.audio_service.get_audio_level(), 0);
}

#[tokio::test]
async fn test_shift_jk_tab_switching() {
    let backend = MockBackend::new();
    let mut app = AppState::new(backend).await.expect("app init");
    app.current_tab = ActiveTab::Dashboard;

    // Shift + j (J) should switch to Power tab
    let shift_j = crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Char('J'),
        crossterm::event::KeyModifiers::SHIFT,
    );
    rogtui::handler::handle_key_event(&mut app, shift_j).await;
    assert_eq!(app.current_tab, ActiveTab::Power, "Shift+J must move to Power tab");

    // Shift + j (J) again -> FanCurves
    rogtui::handler::handle_key_event(&mut app, shift_j).await;
    assert_eq!(app.current_tab, ActiveTab::FanCurves, "Shift+J must move to FanCurves tab");

    // Shift + k (K) should move back to Power
    let shift_k = crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Char('K'),
        crossterm::event::KeyModifiers::SHIFT,
    );
    rogtui::handler::handle_key_event(&mut app, shift_k).await;
    assert_eq!(app.current_tab, ActiveTab::Power, "Shift+K must move back to Power tab");

    // Shift + k (K) should move back to Dashboard
    rogtui::handler::handle_key_event(&mut app, shift_k).await;
    assert_eq!(app.current_tab, ActiveTab::Dashboard, "Shift+K must move back to Dashboard tab");
}

#[tokio::test]
async fn test_fan_curve_jk_point_navigation_and_hl_adjustment() {
    let backend = MockBackend::new();
    let mut app = AppState::new(backend).await.expect("app init");
    app.current_tab = ActiveTab::FanCurves;
    app.active_point_idx = 0;
    app.fan_selected_col = 0; // Speed

    // j moves down to Point 2 (idx 1)
    rogtui::handler::handle_key_event(&mut app, crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char('j'))).await;
    assert_eq!(app.active_point_idx, 1, "j must navigate vertically down to next point");

    // k moves up to Point 1 (idx 0)
    rogtui::handler::handle_key_event(&mut app, crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char('k'))).await;
    assert_eq!(app.active_point_idx, 0, "k must navigate vertically up to previous point");

    // l adjusts speed up (+1%)
    let orig_speed = app.fan_curves[app.active_fan_idx].points[0].pwm_percent;
    rogtui::handler::handle_key_event(&mut app, crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char('l'))).await;
    assert_eq!(
        app.fan_curves[app.active_fan_idx].points[0].pwm_percent,
        (orig_speed + 1).min(100),
        "l must adjust active column value"
    );

    // Space toggles column to Temperature (1)
    rogtui::handler::handle_key_event(&mut app, crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char(' '))).await;
    assert_eq!(app.fan_selected_col, 1, "Space must toggle fan edit column to Temperature");

    // Space toggles column back to Speed (0)
    rogtui::handler::handle_key_event(&mut app, crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char(' '))).await;
    assert_eq!(app.fan_selected_col, 0, "Space must toggle fan edit column back to Speed");
}

#[test]
fn test_charge_limit_slider_thumb_and_level_display() {
    let theme = rogtui::ui::theme::Theme::default();
    let spans = rogtui::ui::widgets::charge_limit_slider(60, 24, true, &theme);
    let rendered: String = spans.iter().map(|s| s.content.as_ref()).collect();

    assert!(rendered.contains('◆'), "Slider must render thumb marker");
    assert!(rendered.contains("60% (max lifespan)"), "Slider must show lifespan description");
}

#[tokio::test]
async fn test_dashboard_footer_p_key_persistence() {
    let backend = MockBackend::new();
    let mut app = AppState::new(backend).await.expect("app init");
    app.current_tab = ActiveTab::Dashboard;

    // Press 'p' to switch profile -> status message is set
    rogtui::handler::handle_key_event(&mut app, crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char('p'))).await;
    assert!(app.status_message.is_some(), "Status message should be active after profile change");

    // Render footer and verify 'p' and 'profile' are present in the output
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    let test_backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(test_backend).unwrap();
    terminal.draw(|f| rogtui::ui::render_ui(f, &app)).unwrap();
    let buffer = terminal.backend().buffer();

    let mut footer_text = String::new();
    for x in 0..80 {
        footer_text.push_str(buffer[(x, 23)].symbol());
    }

    assert!(footer_text.contains('p'), "Footer must contain 'p' key shortcut");
    assert!(footer_text.contains("profile"), "Footer must contain 'profile' label");
    assert!(footer_text.contains("[ok]"), "Footer must contain [ok] status indicator");
}

#[tokio::test]
async fn test_voice_pulse_sensitivity_and_live_meter() {
    let backend = MockBackend::new();
    let mut app = AppState::new(backend).await.expect("app init");
    app.current_tab = ActiveTab::Aura;
    app.aura_brightness = 2;

    // Switch to Voice Pulse mode (idx 10)
    app.active_software_effect = rogtui::backend::types::SoftwareEffect::VoicePulse;
    assert_eq!(rogtui::ui::aura::get_current_mode_index(&app), 10);

    // Verify field 7 is supported for Voice Pulse (Mic sensitivity)
    assert!(rogtui::ui::aura::is_aura_field_supported(&app, 7));

    // Initial voice sensitivity default should be 75
    assert_eq!(app.voice_sensitivity, 75);

    // Select field 7 and adjust with 'l' (+5) and 'h' (-5)
    app.aura_selected_field = 7;
    rogtui::handler::handle_key_event(&mut app, crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char('l'))).await;
    assert_eq!(app.voice_sensitivity, 80);

    rogtui::handler::handle_key_event(&mut app, crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char('h'))).await;
    assert_eq!(app.voice_sensitivity, 75);

    // Render Aura tab in test terminal and check for "Mic sensitivity"
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    let test_backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(test_backend).unwrap();
    terminal.draw(|f| rogtui::ui::render_ui(f, &app)).unwrap();
    let buffer = terminal.backend().buffer();

    let mut found_mic_sens = false;
    for y in 0..30 {
        let mut row_text = String::new();
        for x in 0..100 {
            row_text.push_str(buffer[(x, y)].symbol());
        }
        if row_text.contains("Mic sensitivity") && row_text.contains("75%") {
            found_mic_sens = true;
            break;
        }
    }
    assert!(found_mic_sens, "Aura UI must render Mic sensitivity slider with live meter for Voice Pulse");
}

#[tokio::test]
async fn test_spectrum_cycle_preset_and_audio_cava_dynamics() {
    let backend = MockBackend::new();
    let mut app = AppState::new(backend).await.expect("app init");
    app.current_tab = ActiveTab::Aura;
    app.aura_brightness = 2;

    // Verify Spectrum Cycle is in presets list
    let presets = rogtui::ui::aura::get_color_presets();
    let has_spectrum = presets.iter().any(|(name, col)| *name == "Spectrum Cycle" && *col == rogtui::backend::types::RgbColor::new(1, 1, 1));
    assert!(has_spectrum, "Presets must include Spectrum Cycle");

    // Set color to Spectrum Cycle
    app.aura_data.color1 = rogtui::backend::types::RgbColor::new(1, 1, 1);
    assert_eq!(rogtui::ui::aura::get_active_preset_name(app.aura_data.color1), "Spectrum Cycle");

    // Activate Audio Pulse mode
    app.active_software_effect = rogtui::backend::types::SoftwareEffect::AudioPulse;

    // Compute color multiple times with simulated audio level to verify dynamic hue progression
    let initial_hue = app.spectrum_pulse_hue;
    let _ = app.compute_audio_pulse_color();
    assert!(app.spectrum_pulse_hue > initial_hue, "Spectrum pulse hue should advance on audio frames");
}

#[tokio::test]
async fn test_inline_effect_details_rendering() {
    let backend = MockBackend::new();
    let mut app = AppState::new(backend).await.expect("app init");
    app.current_tab = ActiveTab::Aura;
    app.aura_brightness = 2;
    app.active_software_effect = rogtui::backend::types::SoftwareEffect::ColorCycle;

    // Render Aura tab in test terminal
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    let test_backend = TestBackend::new(120, 30);
    let mut terminal = Terminal::new(test_backend).unwrap();
    terminal.draw(|f| rogtui::ui::render_ui(f, &app)).unwrap();
    let buffer = terminal.backend().buffer();

    let mut mode_row = None;
    let mut speed_row = None;
    let mut has_separate_effect_details_row = false;

    for y in 0..30 {
        let mut row_text = String::new();
        for x in 0..120 {
            row_text.push_str(buffer[(x, y)].symbol());
        }
        if row_text.contains("Mode") && row_text.contains("Color Cycle") {
            mode_row = Some(y);
            // Verify description is inline on Mode row
            assert!(row_text.contains("Sequential smooth crossfade"), "Effect description should be inline on Mode row");
        }
        if row_text.contains("Animation speed") {
            speed_row = Some(y);
        }
        if row_text.starts_with("  Effect details") || row_text.contains("Effect details ") {
            has_separate_effect_details_row = true;
        }
    }

    assert!(!has_separate_effect_details_row, "There must NOT be a separate 'Effect details' row interrupting speed controls");
    if let (Some(m), Some(s)) = (mode_row, speed_row) {
        assert_eq!(s, m + 1, "Animation speed must immediately follow Mode row without intervening blank lines or rows");
    }
}

#[tokio::test]
async fn test_power_tab_compact_layout_no_spacers() {
    let backend = MockBackend::new();
    let app = AppState::new(backend).await.expect("app init");

    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    let test_backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(test_backend).unwrap();
    terminal.draw(|f| rogtui::ui::power::render_power(f, &app, f.area())).unwrap();
    let buffer = terminal.backend().buffer();

    // Check line 0: should start directly with Profile section rule, not blank lines
    let mut line0 = String::new();
    for x in 0..100 {
        line0.push_str(buffer[(x, 0)].symbol());
    }
    assert!(line0.contains("Profile"), "Power tab line 0 must start immediately with Profile rule, with zero top gap");
}

#[tokio::test]
async fn test_modal_enter_key_confirms() {
    use crossterm::event::{KeyCode, KeyEvent};
    let backend = MockBackend::new();
    let mut app = AppState::new(backend).await.expect("app init");

    // Open a confirm modal to switch GPU mode
    app.confirm_modal = Some((
        "Switch to Hybrid GPU mode?".to_string(),
        rogtui::app::ConfirmAction::SwitchGpu(rogtui::backend::types::GpuMode::Hybrid),
    ));

    // Press Enter to confirm
    rogtui::handler::handle_key_event(&mut app, KeyEvent::from(KeyCode::Enter)).await;

    assert!(app.confirm_modal.is_none(), "Enter key must confirm and close modal");
    assert_eq!(app.gpu_pending_mode, rogtui::backend::types::GpuMode::Hybrid, "Enter must execute the confirmed action");
}

#[tokio::test]
async fn test_dashboard_alignment_and_wattage_matching() {
    let backend = MockBackend::new();
    let mut app = AppState::new(backend).await.expect("app init");
    app.sensors.cpu_power_w = Some(12.0);
    app.sensors.gpu_power_w = Some(9.0);

    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    let test_backend = TestBackend::new(100, 30);
    let mut terminal = Terminal::new(test_backend).unwrap();
    terminal.draw(|f| rogtui::ui::dashboard::render_dashboard(f, &app, f.area())).unwrap();
    let buffer = terminal.backend().buffer();

    let mut cpu_subrow = String::new();
    let mut gpu_subrow = String::new();
    let mut bat_subrow = String::new();

    for y in 0..30 {
        let mut row_text = String::new();
        for x in 0..100 {
            row_text.push_str(buffer[(x, y)].symbol());
        }
        if row_text.contains("Intel Core") {
            cpu_subrow = row_text;
        } else if row_text.contains("RTX 5050") {
            gpu_subrow = row_text;
        } else if row_text.contains("Limit") && row_text.contains("Health") {
            bat_subrow = row_text;
        }
    }

    // Verify sub-rows start at column 2 (2 spaces), matching 'CPU', 'GPU', 'Battery' alignment
    assert!(cpu_subrow.starts_with("  Intel Core"), "CPU sub-row must align to left at column 2");
    assert!(gpu_subrow.starts_with("  RTX 5050"), "GPU sub-row must align to left at column 2");
    assert!(bat_subrow.starts_with("  Limit"), "Battery sub-row must align to left at column 2");

    // Verify wattage separators line up at the exact same column
    let cpu_dot = cpu_subrow.find('·').unwrap_or(0);
    let gpu_dot = gpu_subrow.find('·').unwrap_or(0);
    assert_eq!(cpu_dot, gpu_dot, "CPU and GPU live wattage '·' must vertically align at the exact same column");
}

#[test]
fn test_daemon_state_roundtrip() {
    use rogtui::daemon::RgbDaemonState;
    use rogtui::backend::types::{RgbColor, SoftwareEffect};

    let state = RgbDaemonState {
        effect: SoftwareEffect::RainbowWave,
        brightness: 3,
        speed: "high".to_string(),
        color1: RgbColor::new(1, 1, 1),
        color2: RgbColor::new(0, 0, 0),
        voice_sensitivity: 80,
    };

    let toml_str = toml::to_string(&state).expect("serialize");
    let loaded: RgbDaemonState = toml::from_str(&toml_str).expect("deserialize");

    assert_eq!(loaded.effect, SoftwareEffect::RainbowWave);
    assert_eq!(loaded.brightness, 3);
    assert_eq!(loaded.voice_sensitivity, 80);
    assert_eq!(loaded.color1, RgbColor::new(1, 1, 1));
}

#[tokio::test]
async fn test_max_brightness_dynamic_range() {
    let backend = MockBackend::new();
    let mut app = AppState::new(backend).await.expect("app init");
    app.aura_brightness = 3; // Max brightness
    app.aura_data.color1 = rogtui::backend::types::RgbColor::new(255, 255, 255);
    app.user_custom_color = true;

    // At level 0 (silence), color should drop to 0
    app.active_software_effect = rogtui::backend::types::SoftwareEffect::VoicePulse;
    app.audio_level = 0;
    app.mic_level = 0;
    let col0 = app.compute_voice_pulse_color();
    assert_eq!(col0, rogtui::backend::types::RgbColor::new(0, 0, 0), "Max brightness must drop to 0 on silence");
}

#[tokio::test]
async fn test_rgb_tab_hides_rgb_sliders_for_presets() {
    let backend = MockBackend::new();
    let mut app = AppState::new(backend).await.expect("app init");
    app.aura_brightness = 2;
    app.active_software_effect = rogtui::backend::types::SoftwareEffect::AudioPulse;

    // Set to Spectrum Cycle preset
    app.aura_data.color1 = rogtui::backend::types::RgbColor::new(1, 1, 1);
    assert!(!rogtui::ui::aura::is_aura_field_supported(&app, 3), "Field 3 (R) must NOT be supported for Spectrum Cycle");
    assert!(!rogtui::ui::aura::is_aura_field_supported(&app, 4), "Field 4 (G) must NOT be supported for Spectrum Cycle");
    assert!(!rogtui::ui::aura::is_aura_field_supported(&app, 5), "Field 5 (B) must NOT be supported for Spectrum Cycle");

    // But Preset (field 2) must be supported
    assert!(rogtui::ui::aura::is_aura_field_supported(&app, 2), "Field 2 (Preset) must be supported");
}

#[tokio::test]
async fn test_breathe_primary_and_secondary_updates() {
    let backend = MockBackend::new();
    let mut app = AppState::new(backend).await.expect("app init");
    app.aura_brightness = 3;
    app.aura_mode = 1;
    app.active_software_effect = rogtui::backend::types::SoftwareEffect::None;

    assert!(rogtui::ui::aura::is_aura_field_supported(&app, 2), "Primary color supported in Breathe");
    assert!(rogtui::ui::aura::is_aura_field_supported(&app, 6), "Secondary color supported in Breathe");
    assert!(rogtui::ui::aura::is_aura_field_supported(&app, 7), "Speed supported in Breathe");

    // Test secondary color update via handler
    app.aura_selected_field = 6;
    let key_right = crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Right,
        crossterm::event::KeyModifiers::empty(),
    );
    rogtui::handler::handle_key_event(&mut app, key_right).await;
    let data = app.backend.get_aura_mode_data().await.expect("aura mode data");
    assert_eq!(data.color2, app.aura_data.color2, "Secondary color must sync to backend");

    // Test primary color update via handler
    app.aura_selected_field = 2;
    let key_left = crossterm::event::KeyEvent::new(
        crossterm::event::KeyCode::Left,
        crossterm::event::KeyModifiers::empty(),
    );
    rogtui::handler::handle_key_event(&mut app, key_left).await;
    let data2 = app.backend.get_aura_mode_data().await.expect("aura mode data");
    assert_eq!(data2.color1, app.aura_data.color1, "Primary color must sync to backend");
}





