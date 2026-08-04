use crate::app::mode::AppMode;
use crate::config::{save, Config};
use crate::reading::{tokenize_text, ReadingState};
use crate::ui::config_popup::state::ConfigPopupState;
use std::path::PathBuf;

pub struct App {
    mode: AppMode,
    reading_state: Option<ReadingState>,
    error_message: Option<String>,
    pub config: Config,
    config_path: Option<PathBuf>,
    pub config_popup: ConfigPopupState,
    command_buffer: String,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            mode: AppMode::default(),
            reading_state: None,
            error_message: None,
            config: Config::default(),
            config_path: None,
            config_popup: ConfigPopupState::new(),
            command_buffer: String::new(),
        }
    }

    /// Create App with a specific configuration and optional path
    pub fn with_config(config: Config, config_path: Option<PathBuf>) -> Self {
        Self {
            mode: AppMode::default(),
            reading_state: None,
            error_message: None,
            config,
            config_path,
            config_popup: ConfigPopupState::new(),
            command_buffer: String::new(),
        }
    }

    /// Set an error message to be displayed in the UI
    pub fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
    }

    /// Get the current error message if any
    pub fn get_error(&self) -> Option<&str> {
        self.error_message.as_deref()
    }

    pub fn start_reading(&mut self, text: &str, wpm: u32) {
        let tokens = tokenize_text(text);
        // Use config's TimingConfig for reading state
        let timing_config = self.config.timing().clone();
        self.reading_state = Some(ReadingState::new(tokens, wpm, timing_config));
        self.mode = AppMode::Reading;
    }

    /// Advances to the next word in the reading stream.
    ///
    /// Used by TuiManager for auto-advancement in Reading mode.
    /// Returns `true` if advanced, `false` if at end or no reading state.
    pub fn advance_reading(&mut self) -> bool {
        match self.reading_state.as_mut() {
            Some(state) => {
                let before = state.current_index();
                state.advance();
                state.current_index() > before
            }
            None => false,
        }
    }

    pub fn toggle_pause(&mut self) {
        match self.mode {
            AppMode::Reading => {
                self.mode = AppMode::Paused;
            }
            AppMode::Paused => {
                self.mode = AppMode::Reading;
            }
            _ => {}
        }
    }

    /// Quit the application (Ctrl+C).
    pub fn quit(&mut self) {
        self.mode = AppMode::Quit;
    }

    /// Toggle the config popup (Ctrl+P).
    /// Opens from Reading/Paused; closes (discarding changes) from Popup.
    pub fn toggle_popup(&mut self) {
        match self.mode {
            AppMode::Popup => {
                self.config_popup.close();
                self.mode = AppMode::Reading;
            }
            AppMode::Reading | AppMode::Paused => {
                self.config_popup.open(&self.config);
                self.mode = AppMode::Popup;
            }
            _ => {}
        }
    }

    /// Get current word for rendering
    ///
    /// Returns the word at current reading position with punctuation attached, or None if no reading state.
    pub fn get_current_word(&self) -> Option<String> {
        self.reading_state
            .as_ref()
            .and_then(|s| s.current_token())
            .map(|t| {
                let mut word = t.text().to_string();
                for p in t.punctuation() {
                    word.push(*p);
                }
                word
            })
    }

    pub fn mode(&self) -> AppMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: AppMode) {
        self.mode = mode;
    }

    /// Get a reference to the reading state
    pub fn reading_state(&self) -> Option<&ReadingState> {
        self.reading_state.as_ref()
    }

    /// Get a mutable reference to the reading state
    pub fn reading_state_mut(&mut self) -> Option<&mut ReadingState> {
        self.reading_state.as_mut()
    }

    /// Get current WPM from reading state
    ///
    /// Returns the current words-per-minute setting, or 0 if no reading state.
    pub fn get_wpm(&self) -> u32 {
        self.reading_state.as_ref().map(|s| s.wpm()).unwrap_or(0)
    }

    /// Check if ghost words are enabled
    pub fn ghost_words_enabled(&self) -> bool {
        self.config.ghost_words
    }

    /// Get the default WPM for new reading sessions
    pub fn default_wpm(&self) -> u32 {
        self.config.timing().wpm
    }

    /// Get the current theme name
    pub fn theme_name(&self) -> &str {
        &self.config.theme
    }

    /// Get the duration for the current token in milliseconds
    ///
    /// Returns the calculated duration for the current token, including
    /// punctuation multipliers and word length penalties, or 0 if no reading state.
    pub fn get_current_token_duration(&self) -> u64 {
        self.reading_state
            .as_ref()
            .map(|state| state.current_token_duration())
            .unwrap_or(0)
    }

    /// Save the current configuration to disk.
    ///
    /// Persists the current config state (theme, default_wpm, ghost_words, etc.)
    /// to the XDG-compliant config file location.
    ///
    /// # Errors
    /// Returns an error if the config cannot be serialized or written to disk.
    pub fn save_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        save(&self.config, self.config_path.clone())?;
        Ok(())
    }

    // Command buffer methods for Command mode

    /// Get a reference to the command buffer
    pub fn command_buffer(&self) -> &str {
        &self.command_buffer
    }

    /// Get a mutable reference to the command buffer
    pub fn command_buffer_mut(&mut self) -> &mut String {
        &mut self.command_buffer
    }

    /// Clear the command buffer
    pub fn clear_command_buffer(&mut self) {
        self.command_buffer.clear();
    }

    /// Push a character to the command buffer
    pub fn push_command_char(&mut self, c: char) {
        self.command_buffer.push(c);
    }

    /// Pop a character from the command buffer (backspace)
    pub fn pop_command_char(&mut self) -> Option<char> {
        self.command_buffer.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_current_word_no_reading() {
        let app = App::new();
        assert!(app.get_current_word().is_none());
    }

    #[test]
    fn test_get_current_word_reading() {
        let mut app = App::new();
        app.start_reading("hello world", 300);
        assert_eq!(app.get_current_word(), Some("hello".to_string()));
    }

    #[test]
    fn test_advance_reading_moves_to_next_word() {
        let mut app = App::new();
        app.start_reading("hello world test", 300);
        assert_eq!(app.reading_state().unwrap().current_index(), 0);

        let advanced = app.advance_reading();
        assert!(advanced);
        assert_eq!(app.reading_state().unwrap().current_index(), 1);
    }

    #[test]
    fn test_advance_reading_returns_false_at_end() {
        let mut app = App::new();
        app.start_reading("hello", 300);
        assert_eq!(app.reading_state().unwrap().current_index(), 0);

        let advanced = app.advance_reading();
        assert!(!advanced);
        assert_eq!(app.reading_state().unwrap().current_index(), 0);
    }

    #[test]
    fn test_advance_reading_returns_false_with_no_state() {
        let mut app = App::new();
        let advanced = app.advance_reading();
        assert!(!advanced);
    }

    #[test]
    fn test_get_current_token_duration_with_punctuation() {
        let mut app = App::new();
        app.start_reading("Hello, world.", 300);

        assert_eq!(app.get_current_token_duration(), 300);

        app.advance_reading();

        assert_eq!(app.get_current_token_duration(), 600);
    }

    #[test]
    fn test_get_current_token_duration_long_word() {
        let mut app = App::new();
        app.start_reading("extraordinarily", 300);

        assert_eq!(app.get_current_token_duration(), 230);
    }

    #[test]
    fn test_get_current_token_duration_no_reading_state() {
        let app = App::new();
        assert_eq!(app.get_current_token_duration(), 0);
    }

    #[test]
    fn test_advance_reading_stays_false_at_end() {
        let mut app = App::new();
        app.start_reading("hello", 300);
        assert_eq!(app.reading_state().unwrap().current_index(), 0);
        assert_eq!(app.mode, AppMode::Reading);

        let advanced = app.advance_reading();
        assert!(!advanced);
        assert_eq!(app.reading_state().unwrap().current_index(), 0);

        let advanced = app.advance_reading();
        assert!(!advanced);
        assert_eq!(app.reading_state().unwrap().current_index(), 0);
    }
}
