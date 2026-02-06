// Configuration for Speedy engine and UI components
// All values derived from PRD specifications with defaults as documented
use std::ops::RangeInclusive;

/// Timing Constants per PRD Section 3.2
pub const DEFAULT_WPM: u32 = 300;
pub const MIN_WPM: u32 = 50;
pub const MAX_WPM: u32 = 1000;
pub const WPM_ADJUSTMENT_STEP: i32 = 50;
pub const MILLISECONDS_PER_MINUTE: u32 = 60_000;

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
pub const FONT_SIZE_FALLBACK: f32 = 24.0;
pub const FONT_SIZE_MULTIPLIER: f32 = 5.0;

/// Rendering Constants
pub const RENDER_FPS: u32 = 60;
pub const PROGRESS_BAR_MARGIN_PX: u32 = 10;
pub const PROGRESS_BAR_WIDTH_PCT: f64 = 0.5;
pub const PROGRESS_BAR_HEIGHT: u32 = 2;

/// Cache Constants
pub const DEFAULT_CACHE_CAPACITY: usize = 1000;
pub const DEFAULT_MEMORY_CAP_BYTES: u64 = 100 * 1024 * 1024; // 100MB

/// Viewport Constants
pub const READING_ZONE_CENTER_PCT: f32 = 0.42;
pub const MIN_TERMINAL_COLS: u16 = 80;
pub const MIN_TERMINAL_ROWS: u16 = 24;

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
    pub period_multiplier: f64,
    pub comma_multiplier: f64,
    pub question_multiplier: f64,
    pub exclamation_multiplier: f64,
    pub newline_multiplier: f64,
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
