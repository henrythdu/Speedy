//! Theme presets for Speedy.
//!
//! Provides pre-built color themes that can be selected by name.
//! Unknown theme names fall back to `tokyo-night` (the default).

mod catppuccin;
mod dracula;
mod gruvbox;
mod light;
mod nord;
mod tokyo_night;

/// Available theme names in order.
/// This is the single source of truth for theme names across the application.
pub const THEME_NAMES: &[&str] = &[
    "tokyo-night",
    "dracula",
    "gruvbox",
    "catppuccin-mocha",
    "nord",
    "light",
];

/// Get theme index by name, defaults to 0 if not found.
pub fn theme_index(name: &str) -> usize {
    THEME_NAMES.iter().position(|&t| t == name).unwrap_or(0)
}
