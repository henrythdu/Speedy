//! Config picker popup module
//!
//! Provides a Ctrl+P popup for editing default WPM, theme, and ghost words.
//! Uses inline editors with arrow-key cycling and live theme preview.
//!
//! Key handling for the popup lives in the ui-events cell (key_handlers.rs),
//! which drives ConfigPopupState methods directly.

mod render;
pub mod state;

pub use render::render_config_popup;
