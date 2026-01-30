// Configuration for Speedy engine and UI components
// All values derived from PRD specifications with defaults as documented
use std::ops::RangeInclusive;

/// Timing configuration per PRD Section 3.2
#[derive(Debug, Clone, PartialEq)]
pub struct TimingConfig {
    /// Words per minute reading speed (default 300)
    pub wpm: u32,

    /// Minimum and maximum allowed WPM
    pub wpm_range: RangeInclusive<u32>,

    /// Word length threshold for penalty (default 10 chars)
    pub long_word_threshold: usize,

    /// Word length penalty multiplier for words > threshold (default 1.15x)
    pub long_word_penalty: f64,

    /// Punctuation multipliers per PRD Section 3.2
    pub period_multiplier: f64, // default 3.0x
    pub comma_multiplier: f64,       // default 1.5x
    pub question_multiplier: f64,    // default 3.0x
    pub exclamation_multiplier: f64, // default 3.0x
    pub newline_multiplier: f64,     // default 4.0x
}

impl Default for TimingConfig {
    fn default() -> Self {
        Self {
            wpm: 300,
            wpm_range: 50..=1000,
            long_word_threshold: 10,
            long_word_penalty: 1.15,
            period_multiplier: 3.0,
            comma_multiplier: 1.5,
            question_multiplier: 3.0,
            exclamation_multiplier: 3.0,
            newline_multiplier: 4.0,
        }
    }
}
