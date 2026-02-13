//! Config picker popup module
//!
//! Provides a Ctrl+P popup for editing default WPM, theme, and ghost words.
//! Uses inline editors with arrow-key cycling and live theme preview.

mod handler;
mod render;
mod state;

pub use handler::{handle_popup_key, PopupAction};
pub use render::render_config_popup;
pub use state::ConfigPopupState;

// Re-export from central theme definition
pub use crate::config::themes::{theme_index, THEME_NAMES};

// Backwards compatibility alias
#[allow(dead_code)]
#[deprecated(note = "Use THEME_NAMES from config::themes instead")]
pub const THEMES: &[&str] = THEME_NAMES;
