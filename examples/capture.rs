use ratatui::backend::TestBackend;
use ratatui::Terminal;
use rogtui::app::{ActiveTab, AppState};
use rogtui::backend::mock::MockBackend;
use rogtui::ui::render_ui;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let backend = MockBackend::new();
    let mut app = AppState::new(backend).await?;

    let tabs = [
        (ActiveTab::Dashboard, "dashboard"),
        (ActiveTab::Power, "power"),
        (ActiveTab::FanCurves, "fancurves"),
        (ActiveTab::Aura, "aura"),
        (ActiveTab::Gpu, "gpu"),
        (ActiveTab::Armoury, "armoury"),
    ];

    let sizes = [(80, 24), (120, 40)];

    std::fs::create_dir_all("target/captures")?;

    for (w, h) in sizes {
        for (tab, name) in &tabs {
            app.current_tab = *tab;
            let test_backend = TestBackend::new(w, h);
            let mut terminal = Terminal::new(test_backend)?;
            terminal.draw(|f| render_ui(f, &app))?;

            let buffer = terminal.backend().buffer();
            let mut content = String::new();
            for y in 0..h {
                let mut line = String::new();
                for x in 0..w {
                    line.push_str(buffer[(x, y)].symbol());
                }
                content.push_str(line.trim_end());
                content.push('\n');
            }

            let file_path = format!("target/captures/{name}_{w}x{h}.txt");
            std::fs::write(&file_path, content)?;
            println!("Saved capture: {file_path}");
        }
    }

    Ok(())
}
