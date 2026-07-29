//! Config picker popup module
//!
//! Provides a Ctrl+P popup for editing default WPM, theme, and ghost words.
//! Uses inline editors with arrow-key cycling and live theme preview.

mod handler;
mod render;
pub mod state;

// Note: handler module is used for popup key handling
pub use render::render_config_popup;

// Re-export from central theme definition
pub use crate::config::themes::THEME_NAMES;

// Backwards compatibility alias
#[allow(dead_code)]
#[deprecated(note = "Use THEME_NAMES from config::themes instead")]
pub const THEMES: &[&str] = THEME_NAMES;
