mod app;
mod audio;
mod cli;
mod config;
mod engine;
mod input;
mod reading;
mod rendering;
mod storage;
mod ui;

use crate::app::App;
use crate::cli::Args;
use crate::config::Config;
use crate::rendering::font::{get_font, get_font_metrics};
use crate::ui::TuiManager;
use clap::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse CLI arguments
    let args = Args::parse();

    // Handle --list-themes early exit
    if args.list_themes {
        crate::cli::list_themes();
        return Ok(());
    }

    // Initialize tracing subscriber with env-based filtering
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("✓ Kitty Graphics Protocol detected - pixel-perfect mode enabled");

    // Save config path for later (needed for save)
    let config_path = args.config.clone();
    
    // Load configuration (from file or use defaults)
    let config = match crate::config::load(args.config) {
        Ok(mut cfg) => {
            cfg.validate();
            tracing::info!("Configuration loaded successfully");
            cfg
        }
        Err(e) => {
            tracing::warn!("Failed to load config (using defaults): {}", e);
            Config::default()
        }
    };

    // Initialize font
    match get_font() {
        Some(font) => {
            let metrics = get_font_metrics(&font, speedy::engine::config::DEFAULT_FONT_SIZE);
            tracing::debug!("Font loaded: height={:.1}", metrics.height);
        }
        None => {
            tracing::error!("Failed to load embedded font");
            std::process::exit(1);
        }
    }

    let mut app = App::with_config(config, config_path);
    let mut tui = TuiManager::new()?;

    // Run the main TUI event loop
    // The TUI will handle all user input including file loading commands
    tui.run_event_loop(&mut app)?;

    Ok(())
}
