use crate::app::event::AppEvent;
use crate::app::mode::AppMode;
use crate::app::render_state::RenderState;
use crate::engine::{tokenize_text, ReadingState};
use crate::input::{clipboard, epub, pdf, LoadError, LoadedDocument};
use std::path::Path;

pub struct App {
    pub mode: AppMode,
    pub reading_state: Option<ReadingState>,
}

impl App {
    pub fn new() -> Self {
        Self {
            mode: AppMode::default(),
            reading_state: None,
        }
    }

    pub fn start_reading(&mut self, text: &str, wpm: u32) {
        let tokens = tokenize_text(text);
        self.reading_state = Some(ReadingState::new_with_default_config(tokens, wpm));
        self.mode = AppMode::Reading;
    }

    /// Advances to the next word in the reading stream.
    ///
    /// Used by TuiManager for auto-advancement in Reading mode.
    /// Returns `true` if advanced, `false` if at end or no reading state.
    pub fn advance_reading(&mut self) -> bool {
        match self.reading_state.as_mut() {
            Some(state) => {
                let before = state.current_index;
                state.advance();
                state.current_index > before
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

    pub fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Quit => {
                self.mode = AppMode::Quit;
            }
            AppEvent::Help => {
                // TODO: Implement Help screen toggle (Task 2A-3)
            }
            AppEvent::LoadFile(path) => {
                self.handle_load_file(&path);
            }
            AppEvent::LoadClipboard => {
                self.handle_load_clipboard();
            }
            AppEvent::Warning(msg) => {
                eprintln!("Warning: {}", msg);
            }
            AppEvent::InvalidCommand(cmd) => {
                eprintln!("Unknown command: {}", cmd);
            }
            AppEvent::None => {
                // No action required
            }
        }
    }

    fn handle_load_file(&mut self, path: &str) {
        let path = Path::new(path);

        // Detect file extension
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());

        match ext.as_deref() {
            Some("pdf") => match pdf::load(path.to_str().unwrap_or("")) {
                Ok(doc) => self.apply_loaded_document(doc),
                Err(e) => self.handle_load_error(&e),
            },
            Some("epub") => match epub::load(path.to_str().unwrap_or("")) {
                Ok(doc) => self.apply_loaded_document(doc),
                Err(e) => self.handle_load_error(&e),
            },
            Some(_) | None => {
                let filename = path.file_name().map_or_else(
                    || "unknown".to_string(),
                    |n| n.to_string_lossy().to_string(),
                );
                eprintln!("Unsupported format: {}", filename);
                eprintln!("Supported formats: .pdf, .epub");
                eprintln!("For clipboard, use @@ command");
            }
        }
    }

    fn handle_load_clipboard(&mut self) {
        match clipboard::load() {
            Ok(doc) => self.apply_loaded_document(doc),
            Err(e) => self.handle_load_error(&e),
        }
    }

    fn apply_loaded_document(&mut self, doc: LoadedDocument) {
        self.reading_state = Some(ReadingState::new_with_default_config(
            doc.tokens, 300, // Default WPM per PRD Section 3.2
        ));
        self.mode = AppMode::Reading;
        eprintln!(
            "Loaded: {} ({} words)",
            doc.source,
            self.reading_state.as_ref().map_or(0, |s| s.tokens.len())
        );
    }

    pub fn resume_reading(&mut self) -> Result<(), String> {
        if self.reading_state.is_some() {
            self.mode = AppMode::Reading;
            Ok(())
        } else {
            Err("No reading session to resume".to_string())
        }
    }

    pub fn get_render_state(&self) -> RenderState {
        match &self.reading_state {
            Some(state) => RenderState::from_reading_state(
                self.mode.clone(),
                state.tokens.clone(),
                state.current_index,
                3, // context_window
            ),
            None => RenderState::empty(self.mode.clone()),
        }
    }

    pub fn mode(&self) -> AppMode {
        self.mode.clone()
    }

    pub fn set_mode(&mut self, mode: AppMode) {
        self.mode = mode;
    }

    pub fn get_wpm(&self) -> u32 {
        self.reading_state
            .as_ref()
            .map(|state| state.wpm)
            .unwrap_or(300) // Default 300 WPM
    }

    fn handle_load_error(&self, error: &LoadError) {
        match error {
            LoadError::FileNotFound(path) => {
                eprintln!("Error: File not found: {}", path.display());
            }
            LoadError::PdfParse(msg) => {
                eprintln!("Error: PDF parse error: {}", msg);
            }
            LoadError::EpubParse(msg) => {
                eprintln!("Error: EPUB parse error: {}", msg);
            }
            LoadError::Clipboard(msg) => {
                eprintln!("Error: Clipboard error: {}", msg);
            }
            LoadError::UnsupportedFormat(fmt) => {
                eprintln!("Error: Unsupported format: {}", fmt);
            }
        }
    }

    /// Handle keyboard input in Reading mode.
    /// PRD Section 7.2: j/k for sentence navigation, [ / ] for WPM, Space for pause.
    pub fn handle_keypress(&mut self, key: char) -> bool {
        // Only handle keys in Reading or Paused mode
        if !matches!(self.mode, AppMode::Reading | AppMode::Paused) {
            return false;
        }

        if self.reading_state.is_none() {
            return false;
        }

        let reading_state = self.reading_state.as_mut().unwrap();

        match key {
            // Navigation: j is LEFT on keyboard → go BACKWARD to previous sentence
            'j' | 'J' => {
                reading_state.jump_to_previous_sentence();
                true
            }
            // Navigation: k is RIGHT on keyboard → go FORWARD to next sentence
            'k' | 'K' => {
                reading_state.jump_to_next_sentence();
                true
            }
            // WPM: Decrease (PRD Section 7.2)
            '[' => {
                reading_state.adjust_wpm(-50);
                true
            }
            // WPM: Increase (PRD Section 7.2)
            ']' => {
                reading_state.adjust_wpm(50);
                true
            }
            // Pause/Resume (PRD Section 7.2)
            ' ' => {
                self.toggle_pause();
                true
            }
            // Quit to REPL (PRD Section 7.2)
            'q' | 'Q' => {
                self.mode = AppMode::Command;
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reading::token::Token;

    #[test]
    fn test_apply_loaded_document() {
        let mut app = App::new();
        let doc = LoadedDocument {
            tokens: vec![
                Token {
                    text: "hello".to_string(),
                    punctuation: vec![],
                    is_sentence_start: true,
                },
                Token {
                    text: "world".to_string(),
                    punctuation: vec![],
                    is_sentence_start: false,
                },
            ],
            source: "test.pdf".to_string(),
        };

        assert_eq!(app.mode, AppMode::Command);
        assert!(app.reading_state.is_none());

        app.apply_loaded_document(doc);

        assert_eq!(app.mode, AppMode::Reading);
        assert!(app.reading_state.is_some());
        assert_eq!(app.reading_state.as_ref().unwrap().tokens.len(), 2);
        assert_eq!(app.reading_state.as_ref().unwrap().current_index, 0);
    }

    #[test]
    fn test_handle_load_nonexistent_pdf() {
        let mut app = App::new();
        app.handle_load_file("/nonexistent.pdf");

        // Should stay in Repl mode, not Reading
        assert_eq!(app.mode, AppMode::Command);
        assert!(app.reading_state.is_none());
    }

    #[test]
    fn test_handle_load_unsupported_format() {
        let mut app = App::new();
        app.handle_load_file("/document.txt");

        // Should stay in Repl mode, not Reading
        assert_eq!(app.mode, AppMode::Command);
        assert!(app.reading_state.is_none());
    }

    #[test]
    fn test_get_render_state_no_reading() {
        let app = App::new();
        let render = app.get_render_state();

        assert_eq!(render.mode, AppMode::Command);
        assert!(render.current_word.is_none());
    }

    #[test]
    fn test_get_render_state_reading() {
        let mut app = App::new();
        let doc = LoadedDocument {
            tokens: vec![Token {
                text: "hello".to_string(),
                punctuation: vec![],
                is_sentence_start: true,
            }],
            source: "test.pdf".to_string(),
        };
        app.apply_loaded_document(doc);

        let render = app.get_render_state();

        assert_eq!(render.mode, AppMode::Reading);
        assert_eq!(render.current_word, Some("hello".to_string()));
    }

    #[test]
    fn test_quit_event() {
        let mut app = App::new();
        app.handle_event(AppEvent::Quit);
        assert_eq!(app.mode, AppMode::Quit);
    }

    #[test]
    fn test_invalid_command_event() {
        let mut app = App::new();
        // This should just print to stderr, not change state
        app.handle_event(AppEvent::InvalidCommand("unknown".to_string()));
        assert_eq!(app.mode, AppMode::Command);
    }

    #[test]
    fn test_keypress_j_backward_sentence() {
        let mut app = App::new();
        // Create document with multiple sentences
        let doc = LoadedDocument {
            tokens: vec![
                Token {
                    text: "First".to_string(),
                    punctuation: vec![],
                    is_sentence_start: true,
                },
                Token {
                    text: "sentence".to_string(),
                    punctuation: vec!['.'],
                    is_sentence_start: false,
                },
                Token {
                    text: "Second".to_string(),
                    punctuation: vec![],
                    is_sentence_start: true,
                },
                Token {
                    text: "sentence".to_string(),
                    punctuation: vec!['.'],
                    is_sentence_start: false,
                },
            ],
            source: "test.pdf".to_string(),
        };
        app.apply_loaded_document(doc);

        // Initially at index 0
        assert_eq!(app.reading_state.as_ref().unwrap().current_index, 0);

        // Press 'k' to jump forward (k is right on keyboard) - should go to index 2 (Second sentence)
        let result = app.handle_keypress('k');
        assert!(result);
        assert_eq!(app.reading_state.as_ref().unwrap().current_index, 2);

        // Press 'j' to jump backward (j is left on keyboard) - should go back to index 0
        let result = app.handle_keypress('j');
        assert!(result);
        assert_eq!(app.reading_state.as_ref().unwrap().current_index, 0);
    }

    #[test]
    fn test_keypress_k_forward_sentence() {
        let mut app = App::new();
        // Create document with multiple sentences
        let doc = LoadedDocument {
            tokens: vec![
                Token {
                    text: "First".to_string(),
                    punctuation: vec![],
                    is_sentence_start: true,
                },
                Token {
                    text: "sentence".to_string(),
                    punctuation: vec!['.'],
                    is_sentence_start: false,
                },
                Token {
                    text: "Second".to_string(),
                    punctuation: vec![],
                    is_sentence_start: true,
                },
            ],
            source: "test.pdf".to_string(),
        };
        app.apply_loaded_document(doc);

        // Initially at index 0
        assert_eq!(app.reading_state.as_ref().unwrap().current_index, 0);

        // Press 'k' to jump forward (k is right on keyboard) - should go to index 2
        let result = app.handle_keypress('k');
        assert!(result);
        assert_eq!(app.reading_state.as_ref().unwrap().current_index, 2);
    }

    #[test]
    fn test_keypress_bracket_increase_wpm() {
        let mut app = App::new();
        let doc = LoadedDocument {
            tokens: vec![Token {
                text: "test".to_string(),
                punctuation: vec![],
                is_sentence_start: true,
            }],
            source: "test.pdf".to_string(),
        };
        app.apply_loaded_document(doc);

        let initial_wpm = app.reading_state.as_ref().unwrap().wpm;

        // Press ']' to increase WPM
        let result = app.handle_keypress(']');
        assert!(result);
        assert_eq!(app.reading_state.as_ref().unwrap().wpm, initial_wpm + 50);
    }

    #[test]
    fn test_keypress_bracket_decrease_wpm() {
        let mut app = App::new();
        let doc = LoadedDocument {
            tokens: vec![Token {
                text: "test".to_string(),
                punctuation: vec![],
                is_sentence_start: true,
            }],
            source: "test.pdf".to_string(),
        };
        app.apply_loaded_document(doc);

        let initial_wpm = app.reading_state.as_ref().unwrap().wpm;

        // Press '[' to decrease WPM
        let result = app.handle_keypress('[');
        assert!(result);
        assert_eq!(app.reading_state.as_ref().unwrap().wpm, initial_wpm - 50);
    }

    #[test]
    fn test_keypress_space_toggle_pause() {
        let mut app = App::new();
        let doc = LoadedDocument {
            tokens: vec![Token {
                text: "test".to_string(),
                punctuation: vec![],
                is_sentence_start: true,
            }],
            source: "test.pdf".to_string(),
        };
        app.apply_loaded_document(doc);

        // Initially Reading
        assert_eq!(app.mode, AppMode::Reading);

        // Press Space to pause
        let result = app.handle_keypress(' ');
        assert!(result);
        assert_eq!(app.mode, AppMode::Paused);

        // Press Space again to resume
        let result = app.handle_keypress(' ');
        assert!(result);
        assert_eq!(app.mode, AppMode::Reading);
    }

    #[test]
    fn test_keypress_q_quit_to_repl() {
        let mut app = App::new();
        let doc = LoadedDocument {
            tokens: vec![Token {
                text: "test".to_string(),
                punctuation: vec![],
                is_sentence_start: true,
            }],
            source: "test.pdf".to_string(),
        };
        app.apply_loaded_document(doc);

        // Initially Reading
        assert_eq!(app.mode, AppMode::Reading);

        // Press 'q' to quit to REPL
        let result = app.handle_keypress('q');
        assert!(result);
        assert_eq!(app.mode, AppMode::Command);
    }

    #[test]
    fn test_keypress_no_reading_state() {
        let mut app = App::new();

        // No document loaded - keypress should return false
        let result = app.handle_keypress('j');
        assert!(!result);
    }

    #[test]
    fn test_keypress_repl_mode_ignored() {
        let mut app = App::new();
        // Not in Reading mode - keypress should return false
        let result = app.handle_keypress('j');
        assert!(!result);
    }

    // Bead 2B-1-2: Tests for advance_reading method

    #[test]
    fn test_advance_reading_moves_to_next_word() {
        let mut app = App::new();
        app.start_reading("hello world test", 300);
        assert_eq!(app.reading_state.as_ref().unwrap().current_index, 0);

        let advanced = app.advance_reading();
        assert!(advanced);
        assert_eq!(app.reading_state.as_ref().unwrap().current_index, 1);
    }

    #[test]
    fn test_advance_reading_returns_false_at_end() {
        let mut app = App::new();
        app.start_reading("hello", 300); // Single word
        assert_eq!(app.reading_state.as_ref().unwrap().current_index, 0);

        let advanced = app.advance_reading();
        assert!(!advanced); // At end, should return false
        assert_eq!(app.reading_state.as_ref().unwrap().current_index, 0);
    }

    #[test]
    fn test_advance_reading_returns_false_with_no_state() {
        let mut app = App::new();
        // No reading state initialized
        let advanced = app.advance_reading();
        assert!(!advanced);
    }
}
