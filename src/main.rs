mod app;
mod audio;
mod engine;
mod input;
mod reading;
mod rendering;
mod storage;
mod ui;

use crate::app::App;
use crate::rendering::font::{get_font, get_font_metrics};
use crate::ui::TuiManager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("✓ Kitty Graphics Protocol detected - pixel-perfect mode enabled");

    // Initialize font
    match get_font() {
        Some(font) => {
            let metrics = get_font_metrics(&font, speedy::engine::config::DEFAULT_FONT_SIZE);
            eprintln!("Font loaded: height={:.1}", metrics.height);
        }
        None => {
            eprintln!("Warning: Failed to load embedded font");
            std::process::exit(1);
        }
    }

    let mut app = App::new();
    let mut tui = TuiManager::new()?;

    // Run the main TUI event loop
    // The TUI will handle all user input including file loading commands
    tui.run_event_loop(&mut app)?;

    Ok(())
}
