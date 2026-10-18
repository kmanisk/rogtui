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

    // Press 'j' (Down) -> must skip fields 1..8 and jump straight to field 9
    rogtui::handler::handle_key_event(&mut app, crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char('j'))).await;
    assert_eq!(app.aura_selected_field, 9, "Should skip effect fields when brightness is 0");

    // Press 'k' (Up) -> jumps back to field 0
    rogtui::handler::handle_key_event(&mut app, crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char('k'))).await;
    assert_eq!(app.aura_selected_field, 0, "Should jump back to field 0 from field 9");

    // Force selection to field 1 (Effect Mode) while brightness is 0 and try to adjust with 'l'
    app.aura_selected_field = 1;
    let initial_mode = app.aura_mode;
    rogtui::handler::handle_key_event(&mut app, crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char('l'))).await;
    assert_eq!(app.aura_mode, initial_mode, "Effect mode must NOT change when brightness is 0");
    assert!(app.status_message.is_some(), "Status warning must be shown");

    // Now turn brightness up to 1 (Low) on field 0
    app.aura_selected_field = 0;
    rogtui::handler::handle_key_event(&mut app, crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char('l'))).await;
    assert_eq!(app.aura_brightness, 1, "Brightness should be Low (1)");

    // Press 'j' (Down) -> now moves into field 1 (Effect Mode)
    rogtui::handler::handle_key_event(&mut app, crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char('j'))).await;
    assert_eq!(app.aura_selected_field, 1, "Should enter field 1 when brightness > 0");

    // Adjust effect mode with 'l' -> now it should change
    rogtui::handler::handle_key_event(&mut app, crossterm::event::KeyEvent::from(crossterm::event::KeyCode::Char('l'))).await;
    assert_ne!(app.aura_mode, initial_mode, "Effect mode should change when brightness > 0");
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
        ActiveTab::Gpu,
        ActiveTab::Armoury,
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

    // 4. App state effect apply & deduplication
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

    // Brightness 0 guardrail -> no-op
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

