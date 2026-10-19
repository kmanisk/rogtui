use ratatui::backend::TestBackend;
use ratatui::Terminal;
use rogtui::app::{ActiveTab, AppState};
use rogtui::backend::mock::MockBackend;
use rogtui::ui::render_ui;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let backend = MockBackend::new();
    let mut app = AppState::new(backend).await?;
    app.cpu_temp_history = std::collections::VecDeque::from(vec![52, 53, 53, 54, 55, 56, 55, 54, 54]);
    app.gpu_temp_history = std::collections::VecDeque::from(vec![44, 44, 45, 45, 46, 46, 45, 46]);

    let tabs = [
        (ActiveTab::Dashboard, "dash"),
        (ActiveTab::Power, "power"),
        (ActiveTab::FanCurves, "fan"),
        (ActiveTab::Aura, "rgb"),
        (ActiveTab::System, "system"),
    ];

    let sizes = [(80, 24), (100, 30), (160, 45)];

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

        // Capture Aura Color Picker Modal
        {
            app.current_tab = ActiveTab::Aura;
            app.open_color_picker(1);
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
            let file_path = format!("target/captures/rgb_color_picker_{w}x{h}.txt");
            std::fs::write(&file_path, content)?;
            println!("Saved capture: {file_path}");
            app.show_color_picker_modal = false;
        }
    }

    Ok(())
}
