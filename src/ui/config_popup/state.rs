use super::{theme_index, THEME_NAMES};
use crate::config::Config;
use crate::engine::config::{MAX_WPM, MIN_WPM};

/// State for the config picker popup
pub struct ConfigPopupState {
    pub is_open: bool,
    pub selected_row: usize,
    pub temp_default_wpm: u32,
    pub temp_theme_index: usize,
    pub temp_ghost_words: bool,
    original_theme_index: usize,
    original_ghost_words: bool,
}

impl ConfigPopupState {
    /// Create a new closed popup state
    pub fn new() -> Self {
        Self {
            is_open: false,
            selected_row: 0,
            temp_default_wpm: 300,
            temp_theme_index: 0,
            temp_ghost_words: false,
            original_theme_index: 0,
            original_ghost_words: false,
        }
    }

    /// Open popup with current config values
    pub fn open(&mut self, config: &Config) {
        self.is_open = true;
        self.selected_row = 0;
        self.temp_default_wpm = config.default_wpm;
        self.temp_theme_index = theme_index(&config.theme);
        self.temp_ghost_words = config.ghost_words;
        self.original_theme_index = self.temp_theme_index;
        self.original_ghost_words = config.ghost_words;
    }

    /// Close popup
    pub fn close(&mut self) {
        self.is_open = false;
    }

    /// Move selection up
    pub fn move_up(&mut self) {
        if self.selected_row > 0 {
            self.selected_row -= 1;
        }
    }

    /// Move selection down
    pub fn move_down(&mut self) {
        if self.selected_row < 2 {
            self.selected_row += 1;
        }
    }

    /// Cycle current row value left (decrease/prev)
    pub fn cycle_left(&mut self) {
        match self.selected_row {
            0 => self.temp_default_wpm = self.temp_default_wpm.saturating_sub(50).max(MIN_WPM),
            1 => {
                self.temp_theme_index = if self.temp_theme_index == 0 {
                    THEME_NAMES.len() - 1
                } else {
                    self.temp_theme_index - 1
                }
            }
            2 => self.temp_ghost_words = !self.temp_ghost_words,
            _ => {}
        }
    }

    /// Cycle current row value right (increase/next)
    pub fn cycle_right(&mut self) {
        match self.selected_row {
            0 => self.temp_default_wpm = (self.temp_default_wpm + 50).min(MAX_WPM),
            1 => self.temp_theme_index = (self.temp_theme_index + 1) % THEME_NAMES.len(),
            2 => self.temp_ghost_words = !self.temp_ghost_words,
            _ => {}
        }
    }

    /// Apply changes to config (does not save to disk - caller handles persistence)
    pub fn apply_to_config(&self, config: &mut Config) {
        config.default_wpm = self.temp_default_wpm;
        config.theme = THEME_NAMES[self.temp_theme_index].to_string();
        config.ghost_words = self.temp_ghost_words;
    }

    /// Revert theme to original (for Esc flow)
    pub fn original_theme(&self) -> &'static str {
        THEME_NAMES[self.original_theme_index]
    }

    /// Revert ghost_words to original (for Esc flow)
    pub fn original_ghost_words(&self) -> bool {
        self.original_ghost_words
    }

    /// Get current temp theme name
    pub fn current_theme(&self) -> &'static str {
        THEME_NAMES[self.temp_theme_index]
    }
}

impl Default for ConfigPopupState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cycle_wpm_bounds() {
        let mut state = ConfigPopupState::new();
        state.temp_default_wpm = 75;
        state.cycle_left();
        assert_eq!(state.temp_default_wpm, MIN_WPM);
        state.cycle_left(); // Should stay at MIN_WPM
        assert_eq!(state.temp_default_wpm, MIN_WPM);

        state.temp_default_wpm = 975;
        state.cycle_right();
        assert_eq!(state.temp_default_wpm, MAX_WPM);
        state.cycle_right(); // Should stay at MAX_WPM
        assert_eq!(state.temp_default_wpm, MAX_WPM);
    }

    #[test]
    fn test_cycle_theme_wraps() {
        let mut state = ConfigPopupState::new();
        state.selected_row = 1; // Theme row
        state.temp_theme_index = 0;
        state.cycle_left();
        assert_eq!(state.temp_theme_index, THEME_NAMES.len() - 1);

        state.temp_theme_index = THEME_NAMES.len() - 1;
        state.cycle_right();
        assert_eq!(state.temp_theme_index, 0);
    }

    #[test]
    fn test_row_navigation_bounds() {
        let mut state = ConfigPopupState::new();
        state.move_up(); // Should stay at 0
        assert_eq!(state.selected_row, 0);

        state.move_down();
        state.move_down();
        assert_eq!(state.selected_row, 2);

        state.move_down(); // Should stay at 2
        assert_eq!(state.selected_row, 2);
    }
}
