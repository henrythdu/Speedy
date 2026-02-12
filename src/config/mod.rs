//! Configuration module for Speedy.
//!
//! Provides TOML-based configuration loading with XDG-compliant paths.
//! Supports theme presets and timing customization.

mod file;
pub mod theme;
pub mod themes;

pub use file::load;
// Note: config_path and ConfigLoadError available via file module if needed
// Note: ThemeColors and get_theme available via submodules if needed

use serde::Deserialize;

// Re-export timing config from engine (consolidated - single source of truth)
pub use crate::engine::config::TimingConfig;

// Re-export timing config from engine (consolidated - single source of truth)
// Note: Timing constants available via crate::engine::config if needed externally

/// Default theme name.
const DEFAULT_THEME: &str = "tokyo-night";

/// Main configuration structure.
///
/// Supports partial configuration - missing fields use defaults.
/// Loaded from `~/.config/speedy/config.toml` (XDG-compliant).
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Theme preset name (e.g., "tokyo-night", "dracula", "gruvbox")
    /// See docs for available themes. Unknown themes fall back to default.
    #[serde(default = "default_theme")]
    pub theme: String,

    /// Timing configuration for reading speed and punctuation pauses.
    #[serde(default)]
    pub timing: TimingConfig,

    /// Enable vertical ghost words (previous word above, next word below).
    /// Provides eye tracking continuity and comprehension preview.
    #[serde(default)]
    pub ghost_words: bool,
}

// Default value functions for serde
fn default_theme() -> String {
    DEFAULT_THEME.to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: DEFAULT_THEME.to_string(),
            timing: TimingConfig::default(),
            ghost_words: false,
        }
    }
}

impl Config {
    /// Validates and clamps configuration values to valid ranges.
    /// Called after loading to ensure safe values.
    pub fn validate(&mut self) {
        self.timing.validate();
    }

    /// Get a reference to the timing configuration
    pub fn timing(&self) -> &TimingConfig {
        &self.timing
    }

    /// Check if ghost words are enabled
    pub fn ghost_words(&self) -> bool {
        self.ghost_words
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::config::{MAX_WPM, MIN_WPM};

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.theme, "tokyo-night");
        assert_eq!(config.timing.wpm, 300);
    }

    #[test]
    fn test_config_validate_clamps_wpm() {
        let mut config = Config::default();
        config.timing.wpm = 10; // Below minimum
        config.validate();
        assert_eq!(config.timing.wpm, MIN_WPM);

        config.timing.wpm = 2000; // Above maximum
        config.validate();
        assert_eq!(config.timing.wpm, MAX_WPM);
    }
}
