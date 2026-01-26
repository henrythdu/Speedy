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

/// Theme configuration per PRD Section 4.1 (future implementation)
#[derive(Debug, Clone, PartialEq)]
pub struct ThemeConfig {
    /// Background color (PRD: #1A1B26 - Stormy Dark)
    pub background_color: String,

    /// Text color (PRD: #A9B1D6 - Light Blue)
    pub text_color: String,

    /// Anchor salience color (PRD: #F7768E - Coral Red)
    pub anchor_color: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            background_color: "#1A1B26".to_string(),
            text_color: "#A9B1D6".to_string(),
            anchor_color: "#F7768E".to_string(),
        }
    }
}

/// Gutter configuration per PRD Section 4.2 (future implementation)
#[derive(Debug, Clone, PartialEq)]
pub struct GutterConfig {
    /// Number of words before/after center (PRD: 3 before, 3 after)
    pub words_before: usize,
    pub words_after: usize,

    /// Peripheral opacity when reading (PRD: 20% - Subliminal)
    pub reading_opacity_percent: u8,

    /// Peripheral opacity when paused (PRD: 100% - Active)
    pub paused_opacity_percent: u8,

    /// Opacity levels for distance from center (PRD: 100%, 80%, 60%, 40%)
    pub opacity_levels: Vec<u8>,
}

impl Default for GutterConfig {
    fn default() -> Self {
        Self {
            words_before: 3,
            words_after: 3,
            reading_opacity_percent: 20,
            paused_opacity_percent: 100,
            opacity_levels: vec![100, 80, 60, 40],
        }
    }
}

/// Audio configuration per PRD Section 5.1 (future implementation)
#[derive(Debug, Clone, PartialEq)]
pub struct AudioConfig {
    /// Paragraph "Thump" frequency in Hz (PRD: 100Hz, range 80-120Hz)
    pub thump_frequency_hz: u16,
    pub thump_frequency_range: RangeInclusive<u16>,

    /// Speed glide range (PRD: 440Hz → 880Hz)
    pub speed_glide_range_hz: RangeInclusive<u16>,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            thump_frequency_hz: 100,
            thump_frequency_range: 80..=120,
            speed_glide_range_hz: 440..=880,
        }
    }
}

/// Tactile controls configuration per PRD Section 5.2 (future implementation)
#[derive(Debug, Clone, PartialEq)]
pub struct TactileConfig {
    /// Tactical throttle percentage (PRD: 50% for dense sections)
    pub throttle_percent: u8,

    /// Ocular priming ramp-up duration in seconds (PRD: 5s)
    pub ramp_up_seconds: u8,

    /// Starting WPM percentage for ramp-up (PRD: 70% to 100%)
    pub ramp_start_percent: u8,
    pub ramp_end_percent: u8,
}

impl Default for TactileConfig {
    fn default() -> Self {
        Self {
            throttle_percent: 50,
            ramp_up_seconds: 5,
            ramp_start_percent: 70,
            ramp_end_percent: 100,
        }
    }
}

/// Master configuration combining all Speedy settings
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub timing: TimingConfig,
    pub theme: ThemeConfig,
    pub gutter: GutterConfig,
    pub audio: AudioConfig,
    pub tactile: TactileConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            timing: TimingConfig::default(),
            theme: ThemeConfig::default(),
            gutter: GutterConfig::default(),
            audio: AudioConfig::default(),
            tactile: TactileConfig::default(),
        }
    }
}
