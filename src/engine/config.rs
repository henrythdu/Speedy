// Configuration for Speedy engine and UI components
// All values derived from PRD specifications with defaults as documented
use std::ops::RangeInclusive;

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

/// Timing configuration per PRD Section 3.2
#[derive(Debug, Clone, PartialEq)]
pub struct TimingConfig {
    /// Words per minute reading speed (default 300)
    wpm: u32,

    /// Minimum and maximum allowed WPM
    wpm_range: RangeInclusive<u32>,

    /// Word length threshold for penalty (default 10 chars)
    long_word_threshold: usize,

    /// Word length penalty multiplier for words > threshold (default 1.15x)
    long_word_penalty: f64,

    /// Punctuation multipliers per PRD Section 3.2
    period_multiplier: f64,
    comma_multiplier: f64,
    question_multiplier: f64,
    exclamation_multiplier: f64,
    newline_multiplier: f64,
}

impl TimingConfig {
    /// Returns the current WPM setting.
    /// Currently unused but reserved for future WPM control UI.
    #[allow(dead_code)]
    pub fn wpm(&self) -> u32 {
        self.wpm
    }

    /// Sets the WPM, clamped to the valid range.
    /// Currently unused but reserved for future WPM control UI.
    #[allow(dead_code)]
    pub fn set_wpm(&mut self, wpm: u32) {
        self.wpm = wpm.clamp(*self.wpm_range.start(), *self.wpm_range.end());
    }

    /// Returns the valid WPM range.
    pub fn wpm_range(&self) -> &RangeInclusive<u32> {
        &self.wpm_range
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
            wpm_range: MIN_WPM..=MAX_WPM,
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
