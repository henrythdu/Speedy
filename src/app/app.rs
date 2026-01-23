use super::mode::AppMode;
use super::state::ReadingState;
use crate::engine::timing::tokenize_text;

#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub enum AppEvent {
    LoadFile(String),
    LoadClipboard,
    Quit,
    Help,
    None,
}

#[allow(dead_code)]
pub struct RenderState {
    pub mode: AppMode,
    pub current_word: Option<String>,
}

#[allow(dead_code)]
pub struct App {
    pub mode: AppMode,
    pub reading_state: Option<ReadingState>,
}

#[allow(dead_code)]
impl App {
    pub fn new() -> Self {
        Self {
            mode: AppMode::default(),
            reading_state: None,
        }
    }

    #[allow(dead_code)]
    pub fn start_reading(&mut self, text: &str, wpm: u32) {
        let tokens = tokenize_text(text);
        self.reading_state = Some(ReadingState::new(tokens, wpm));
        self.mode = AppMode::Reading;
    }

    #[allow(dead_code)]
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
            AppEvent::LoadFile(_path) => {
                // TODO: Implement file loading logic (Task 2A-4)
            }
            AppEvent::LoadClipboard => {
                // TODO: Implement clipboard loading logic (Task 2A-4)
            }
            AppEvent::None => {
                // No action required
            }
        }
    }

    #[allow(dead_code)]
    pub fn get_render_state(&self) -> RenderState {
        let current_word = self
            .reading_state
            .as_ref()
            .and_then(|state| state.current_token())
            .map(|token| token.text.clone());

        RenderState {
            mode: self.mode.clone(),
            current_word,
        }
    }
}
