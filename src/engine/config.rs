// Configuration for Speedy engine and UI components
// All values derived from PRD specifications with defaults as documented
use std::ops::RangeInclusive;

/// UI Constants per PRD and Design Specifications
pub const COMMAND_DECK_LINES: u16 = 5; // Fixed 5-line command deck per PRD Section 4.3
pub const READING_ZONE_CENTER_PCT: f32 = 0.42; // 42% of reading zone height for OVP per PRD
pub const KITTY_CHUNK_SIZE: usize = 4096; // Kitty protocol max chunk size
pub const DEFAULT_WPM: u32 = 300; // Default reading speed per PRD Section 3.2
pub const MIN_TERMINAL_COLS: u16 = 80; // Minimum supported terminal width
pub const MIN_TERMINAL_ROWS: u16 = 24; // Minimum supported terminal height
pub const FONT_SIZE_MULTIPLIER: f32 = 5.0; // Font size = cell_height * 5 per design
pub const RENDER_FPS: u32 = 60; // Target render rate for smooth UI
pub const FONT_SIZE_FALLBACK: f32 = 24.0; // Fallback font size if viewport query fails

/// UI configuration for layout and rendering
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiConfig {
    pub command_deck_lines: u16,
    pub reading_zone_center_pct: f32,
    pub render_fps: u32,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            command_deck_lines: COMMAND_DECK_LINES,
            reading_zone_center_pct: READING_ZONE_CENTER_PCT,
            render_fps: RENDER_FPS,
        }
    }
}

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
            wpm: DEFAULT_WPM,
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
