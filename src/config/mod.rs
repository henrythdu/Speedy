//! Configuration module for Speedy.
//!
//! Provides TOML-based configuration loading with XDG-compliant paths.
//! Supports theme presets and timing customization.

mod file;
pub mod theme;
pub mod themes;

pub use file::load;
pub use file::save;
// Note: config_path and ConfigLoadError available via file module if needed
// Note: ThemeColors and get_theme available via submodules if needed

use serde::{Deserialize, Serialize};

// Re-export timing config from engine (consolidated - single source of truth)
pub use crate::engine::config::TimingConfig;

// Re-export timing config from engine (consolidated - single source of truth)
// Note: Timing constants available via crate::engine::config if needed externally

/// Default theme name.
const DEFAULT_THEME: &str = "tokyo-night";

/// Default WPM for new reading sessions.
const DEFAULT_WPM: u32 = 300;

/// Minimum allowed WPM value.
const MIN_DEFAULT_WPM: u32 = 50;

/// Maximum allowed WPM value.
const MAX_DEFAULT_WPM: u32 = 1000;

/// Default value function for default_wpm serde default
fn default_wpm() -> u32 {
    DEFAULT_WPM
}

/// Main configuration structure.
///
/// Supports partial configuration - missing fields use defaults.
/// Loaded from `~/.config/speedy/config.toml` (XDG-compliant).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Theme preset name (e.g., "tokyo-night", "dracula", "gruvbox")
    /// See docs for available themes. Unknown themes fall back to default.
    #[serde(default = "default_theme")]
    pub theme: String,

    /// Default WPM for new reading sessions (50-1000).
    /// Used as starting speed when loading text.
    #[serde(default = "default_wpm")]
    pub default_wpm: u32,

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
            default_wpm: DEFAULT_WPM,
            timing: TimingConfig::default(),
            ghost_words: false,
        }
    }
}

impl Config {
    /// Validates and clamps configuration values to valid ranges.
    /// Called after loading to ensure safe values.
    pub fn validate(&mut self) {
        self.default_wpm = self.default_wpm.clamp(MIN_DEFAULT_WPM, MAX_DEFAULT_WPM);
        self.timing.validate();
    }

    /// Get a reference to the timing configuration
    pub fn timing(&self) -> &TimingConfig {
        &self.timing
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

    // Task 1: default_wpm field tests
    #[test]
    fn test_default_wpm_field() {
        let config = Config::default();
        assert!(config.default_wpm >= 50, "default_wpm should be >= 50");
        assert!(config.default_wpm <= 1000, "default_wpm should be <= 1000");
    }

    #[test]
    fn test_default_wpm_clamped() {
        // Test below minimum
        let mut config = Config {
            default_wpm: 25,
            ..Default::default()
        };
        config.validate();
        assert_eq!(config.default_wpm, 50, "Should clamp to minimum 50");

        // Test above maximum
        config.default_wpm = 2000;
        config.validate();
        assert_eq!(config.default_wpm, 1000, "Should clamp to maximum 1000");
    }
}
