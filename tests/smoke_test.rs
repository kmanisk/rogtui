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
