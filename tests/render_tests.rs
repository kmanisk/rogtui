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

#[tokio::test]
async fn test_dashboard_two_temperature_graphs_and_single_line_footer() {
    let backend = MockBackend::new();
    let mut app = AppState::new(backend).await.expect("app init");
    app.current_tab = ActiveTab::Dashboard;
    app.cpu_temp_history = std::collections::VecDeque::from(vec![52, 53, 54, 55, 54]);
    app.gpu_temp_history = std::collections::VecDeque::from(vec![44, 45, 46, 45, 44]);

    let test_backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(test_backend).expect("terminal creation");
    terminal.draw(|f| render_ui(f, &app)).expect("draw succeeded");

    let buffer = terminal.backend().buffer();

    // Verify row 23 (last row, 0-indexed) has the single-line footer
    let mut last_row = String::new();
    for x in 0..80 {
        last_row.push_str(buffer[(x, 23)].symbol());
    }
    assert!(last_row.contains("p cycle profile"), "Row 24 must contain p cycle profile: {last_row}");
    assert!(last_row.contains("help"), "Row 24 must contain help");
    assert!(last_row.contains("quit"), "Row 24 must contain quit");

    // Verify row 22 is the dim rule, not an extra hint row
    let mut rule_row = String::new();
    for x in 0..80 {
        rule_row.push_str(buffer[(x, 22)].symbol());
    }
    assert!(rule_row.contains('─'), "Row 23 must be separator rule");
    assert!(!rule_row.contains("Platform profiles"), "Row 23 must not be two-row verbose hint");

    // Verify both CPU and GPU graphs exist side by side
    let content: String = buffer.content().iter().map(|c| c.symbol()).collect();
    assert!(content.contains("CPU"), "Must contain CPU chart");
    assert!(content.contains("GPU"), "Must contain GPU chart");
    assert!(content.contains("90°C"), "Must contain 90°C y-axis");
    assert!(content.contains("60°C"), "Must contain 60°C y-axis");
    assert!(content.contains("30°C"), "Must contain 30°C y-axis");
}
