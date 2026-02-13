// Configuration for Speedy engine and UI components
// All values derived from PRD specifications with defaults as documented
use serde::{Deserialize, Serialize};

/// Timing Constants per PRD Section 3.2
pub const DEFAULT_WPM: u32 = 300;
pub const MIN_WPM: u32 = 50;
pub const MAX_WPM: u32 = 1000;

/// Word Length Penalty Constants
pub const LONG_WORD_THRESHOLD: usize = 10;
pub const LONG_WORD_PENALTY: f64 = 1.15;

/// Punctuation Multipliers per PRD Section 3.2
pub const PERIOD_MULTIPLIER: f64 = 3.0;
pub const COMMA_MULTIPLIER: f64 = 1.5;
pub const QUESTION_MULTIPLIER: f64 = 3.0;
pub const EXCLAMATION_MULTIPLIER: f64 = 3.0;
pub const NEWLINE_MULTIPLIER: f64 = 4.0;

/// Font Constants
pub const DEFAULT_FONT_SIZE: f32 = 24.0;

/// Rendering Constants
pub const PROGRESS_BAR_MARGIN_PX: u32 = 10;
pub const PROGRESS_BAR_WIDTH_PCT: f64 = 0.3;
pub const PROGRESS_BAR_HEIGHT: u32 = 2;

/// Cache Constants
pub const DEFAULT_CACHE_CAPACITY: usize = 1000;
pub const DEFAULT_MEMORY_CAP_BYTES: u64 = 100 * 1024 * 1024; // 100MB

/// Theme Colors (matching PRD color scheme)
/// Progress bar and gutter colors - bright for read, dim for unread
pub const PROGRESS_COLOR_R: u8 = 169;
pub const PROGRESS_COLOR_G: u8 = 177;
pub const PROGRESS_COLOR_B: u8 = 214;
pub const PROGRESS_BRIGHT_ALPHA: u8 = 255; // Full opacity for read portion
pub const PROGRESS_DIM_ALPHA: u8 = 50; // 20% opacity for unread portion

/// Default functions for serde deserialization
fn default_wpm() -> u32 {
    DEFAULT_WPM
}

fn default_long_word_threshold() -> usize {
    LONG_WORD_THRESHOLD
}

fn default_long_word_penalty() -> f64 {
    LONG_WORD_PENALTY
}

fn default_period_multiplier() -> f64 {
    PERIOD_MULTIPLIER
}

fn default_comma_multiplier() -> f64 {
    COMMA_MULTIPLIER
}

fn default_question_multiplier() -> f64 {
    QUESTION_MULTIPLIER
}

fn default_exclamation_multiplier() -> f64 {
    EXCLAMATION_MULTIPLIER
}

fn default_newline_multiplier() -> f64 {
    NEWLINE_MULTIPLIER
}

/// Timing configuration per PRD Section 3.2
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TimingConfig {
    /// Words per minute reading speed (default 300)
    #[serde(default = "default_wpm")]
    pub wpm: u32,

    /// Word length threshold for penalty (default 10 chars)
    #[serde(default = "default_long_word_threshold")]
    pub long_word_threshold: usize,

    /// Word length penalty multiplier for words > threshold (default 1.15x)
    #[serde(default = "default_long_word_penalty")]
    pub long_word_penalty: f64,

    /// Punctuation multipliers per PRD Section 3.2
    #[serde(default = "default_period_multiplier")]
    pub period_multiplier: f64,

    #[serde(default = "default_comma_multiplier")]
    pub comma_multiplier: f64,

    #[serde(default = "default_question_multiplier")]
    pub question_multiplier: f64,

    #[serde(default = "default_exclamation_multiplier")]
    pub exclamation_multiplier: f64,

    #[serde(default = "default_newline_multiplier")]
    pub newline_multiplier: f64,
}

impl TimingConfig {
    /// Validates and normalizes configuration values.
    /// Clamps wpm to valid range [MIN_WPM, MAX_WPM].
    pub fn validate(&mut self) {
        self.wpm = self.wpm.clamp(MIN_WPM, MAX_WPM);
    }

    /// Returns the valid WPM range.
    /// Note: RangeInclusive is not stored in the struct (serde incompatible),
    /// but can be derived from constants for API compatibility.
    pub fn wpm_range(&self) -> std::ops::RangeInclusive<u32> {
        MIN_WPM..=MAX_WPM
    }

    /// Returns the word length threshold for penalty.
    pub fn long_word_threshold(&self) -> usize {
        self.long_word_threshold
    }

    /// Returns the word length penalty multiplier.
    pub fn long_word_penalty(&self) -> f64 {
        self.long_word_penalty
    }

    /// Returns the period/full stop multiplier.
    pub fn period_multiplier(&self) -> f64 {
        self.period_multiplier
    }

    /// Returns the comma multiplier.
    pub fn comma_multiplier(&self) -> f64 {
        self.comma_multiplier
    }

    /// Returns the question mark multiplier.
    pub fn question_multiplier(&self) -> f64 {
        self.question_multiplier
    }

    /// Returns the exclamation mark multiplier.
    pub fn exclamation_multiplier(&self) -> f64 {
        self.exclamation_multiplier
    }

    /// Returns the newline multiplier.
    pub fn newline_multiplier(&self) -> f64 {
        self.newline_multiplier
    }
}

impl Default for TimingConfig {
    fn default() -> Self {
        Self {
            wpm: DEFAULT_WPM,
            long_word_threshold: LONG_WORD_THRESHOLD,
            long_word_penalty: LONG_WORD_PENALTY,
            period_multiplier: PERIOD_MULTIPLIER,
            comma_multiplier: COMMA_MULTIPLIER,
            question_multiplier: QUESTION_MULTIPLIER,
            exclamation_multiplier: EXCLAMATION_MULTIPLIER,
            newline_multiplier: NEWLINE_MULTIPLIER,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_timing_config() {
        let config = TimingConfig::default();
        assert_eq!(config.wpm, DEFAULT_WPM);
        assert_eq!(config.long_word_threshold(), LONG_WORD_THRESHOLD);
    }

    #[test]
    fn test_validate_clamps_wpm() {
        let mut config = TimingConfig {
            wpm: 0, // Below minimum
            ..Default::default()
        };
        config.validate();
        assert_eq!(config.wpm, MIN_WPM);

        config.wpm = 5000; // Above maximum
        config.validate();
        assert_eq!(config.wpm, MAX_WPM);
    }

    #[test]
    fn test_deserialize_from_toml() {
        let toml_str = r#"
            wpm = 500
            long_word_threshold = 8
        "#;
        let config: TimingConfig = toml::from_str(toml_str).expect("Failed to deserialize");
        assert_eq!(config.wpm, 500);
        assert_eq!(config.long_word_threshold, 8);
        // Defaults should apply for missing fields
        assert_eq!(config.long_word_penalty, LONG_WORD_PENALTY);
    }

    #[test]
    fn test_deserialize_with_defaults() {
        let toml_str = "";
        let config: TimingConfig = toml::from_str(toml_str).expect("Failed to deserialize");
        assert_eq!(config.wpm, DEFAULT_WPM);
        assert_eq!(config.long_word_threshold, LONG_WORD_THRESHOLD);
    }
}
