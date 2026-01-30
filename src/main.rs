mod app;
mod audio;
mod engine;
mod input;
mod reading;
mod rendering;
mod storage;
mod ui;

use crate::app::App;
use crate::rendering::capability::{CapabilityDetector, GraphicsCapability};
use crate::rendering::font::{get_font, get_font_metrics};
use crate::ui::TuiManager;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse CLI arguments for force flags
    let args: Vec<String> = env::args().collect();
    let force_kitty = args.contains(&"--force-kitty".to_string());

    // Detect terminal capability
    let detector = CapabilityDetector::new();
    let capability = if force_kitty {
        detector.detect_from_override(true)
    } else {
        detector.detect()
    };

    eprintln!("✓ Kitty Graphics Protocol detected - pixel-perfect mode enabled");

    // Initialize font
    match get_font() {
        Some(font) => {
            let metrics = get_font_metrics(&font, 24.0);
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
