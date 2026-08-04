//! Theme presets for Speedy.
//!
//! Provides theme names selectable by name, plus the name→index mapping used
//! by the config popup and CLI. Unknown names fall back to index 0 (tokyo-night).
//!
//! Note: runtime palette values live in `crate::ui::theme` (single source).

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
