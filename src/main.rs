mod app;
mod audio;
mod engine;
mod input;
mod storage;
mod ui;

use crate::app::App;
use crate::ui::TuiManager;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new();
    let mut tui = TuiManager::new()?;

    // Run the main TUI event loop
    // The TUI will handle all user input including file loading commands
    tui.run_event_loop(&mut app)?;

    Ok(())
}
