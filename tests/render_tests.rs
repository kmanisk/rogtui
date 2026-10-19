use ratatui::backend::TestBackend;
use ratatui::Terminal;
use rogtui::app::{ActiveTab, AppState};
use rogtui::backend::mock::MockBackend;
use rogtui::ui::render_ui;

#[tokio::test]
async fn test_render_all_tabs_80x24() {
    let backend = MockBackend::new();
    let mut app = AppState::new(backend).await.expect("app init");
    let test_backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(test_backend).expect("terminal creation");

    let tabs = [
        ActiveTab::Dashboard,
        ActiveTab::Power,
        ActiveTab::FanCurves,
        ActiveTab::Aura,
        ActiveTab::System,
    ];

    for tab in tabs {
        app.current_tab = tab;
        terminal.draw(|f| render_ui(f, &app)).expect("draw succeeded");
        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|c| c.symbol()).collect();
        assert!(content.contains("Dash"), "Tab bar must contain Dash on tab {tab:?}");
        assert!(content.contains("Power"), "Tab bar must contain Power on tab {tab:?}");
        assert!(content.contains("help"), "Footer must contain help on tab {tab:?}");
        assert!(content.contains("quit"), "Footer must contain quit on tab {tab:?}");
    }
}

#[tokio::test]
async fn test_render_all_tabs_104x30() {
    let backend = MockBackend::new();
    let mut app = AppState::new(backend).await.expect("app init");
    let test_backend = TestBackend::new(104, 30);
    let mut terminal = Terminal::new(test_backend).expect("terminal creation");

    let tabs = [
        ActiveTab::Dashboard,
        ActiveTab::Power,
        ActiveTab::FanCurves,
        ActiveTab::Aura,
        ActiveTab::System,
    ];

    for tab in tabs {
        app.current_tab = tab;
        terminal.draw(|f| render_ui(f, &app)).expect("draw succeeded");
        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|c| c.symbol()).collect();
        assert!(content.contains("Dash"), "Tab bar must contain Dash on tab {tab:?}");
        assert!(content.contains("System"), "Tab bar must contain System on tab {tab:?}");
        assert!(content.contains("help"), "Footer must contain help on tab {tab:?}");
        assert!(content.contains("quit"), "Footer must contain quit on tab {tab:?}");
    }
}

#[tokio::test]
async fn test_render_all_tabs_200x50() {
    let backend = MockBackend::new();
    let mut app = AppState::new(backend).await.expect("app init");
    let test_backend = TestBackend::new(200, 50);
    let mut terminal = Terminal::new(test_backend).expect("terminal creation");

    let tabs = [
        ActiveTab::Dashboard,
        ActiveTab::Power,
        ActiveTab::FanCurves,
        ActiveTab::Aura,
        ActiveTab::System,
    ];

    for tab in tabs {
        app.current_tab = tab;
        terminal.draw(|f| render_ui(f, &app)).expect("draw succeeded");
        let buffer = terminal.backend().buffer();
        let content: String = buffer.content().iter().map(|c| c.symbol()).collect();
        assert!(content.contains("Dash"), "Tab bar must contain Dash on tab {tab:?}");
        assert!(content.contains("RGB"), "Tab bar must contain RGB on tab {tab:?}");
        assert!(content.contains("help"), "Footer must contain help on tab {tab:?}");
        assert!(content.contains("quit"), "Footer must contain quit on tab {tab:?}");
    }
}

#[tokio::test]
async fn test_terminal_too_small_warning() {
    let backend = MockBackend::new();
    let app = AppState::new(backend).await.expect("app init");
    let test_backend = TestBackend::new(60, 15);
    let mut terminal = Terminal::new(test_backend).expect("terminal creation");

    terminal.draw(|f| render_ui(f, &app)).expect("draw succeeded");
    let buffer = terminal.backend().buffer();
    let content: String = buffer.content().iter().map(|c| c.symbol()).collect();
    assert!(content.contains("Terminal Too Small"), "Must show warning for terminal < 80x24");
    assert!(content.contains("Required: at least 80x24"), "Must indicate required dimensions");
}
