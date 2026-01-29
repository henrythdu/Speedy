mod app;
mod audio;
mod engine;
mod input;
mod reading;
mod rendering;
mod storage;
mod ui;

use crate::app::App;
use crate::rendering::capability::{
    get_tui_fallback_warning, CapabilityDetector, GraphicsCapability,
};
use crate::rendering::font::{get_font, get_font_metrics};
use crate::ui::TuiManager;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse CLI arguments for force flags
    let args: Vec<String> = env::args().collect();
    let force_kitty = args.contains(&"--force-kitty".to_string());
    let force_tui = args.contains(&"--force-tui".to_string());

    // Check for mutually exclusive flags
    if force_kitty && force_tui {
        eprintln!("Error: --force-kitty and --force-tui are mutually exclusive");
        std::process::exit(1);
    }

    // Detect terminal capability
    let detector = CapabilityDetector::new();
    let capability =
        if let Some(override_cap) = detector.detect_from_override(force_kitty, force_tui) {
            override_cap
        } else {
            detector.detect()
        };

    // Show warning if in TUI fallback mode
    if capability == GraphicsCapability::None {
        eprintln!("{}", get_tui_fallback_warning());
    }

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
