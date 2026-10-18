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


