//! Concrete key handler implementations

use crate::app::mode::AppMode;
use crate::app::App;
use crate::ui::key_handler::{KeyHandler, KeyResult};
use anyhow::Result;
use crossterm::event::KeyCode;

// ============================================================================
// Reading Mode Handlers
// ============================================================================

/// Handler for 'j' and Space - move to next sentence
pub struct NextSentenceHandler;

impl KeyHandler for NextSentenceHandler {
    fn mode(&self) -> AppMode {
        AppMode::Reading
    }

    fn keys(&self) -> Vec<KeyCode> {
        vec![KeyCode::Char('j'), KeyCode::Char(' ')]
    }

    fn handle(&self, app: &mut App) -> Result<KeyResult> {
        if let Some(state) = app.reading_state_mut() {
            state.jump_to_next_sentence();
        }
        Ok(KeyResult::Consumed)
    }

    fn help_text(&self) -> &str {
        "Next sentence (j, Space)"
    }
}

/// Handler for 'k' - move to previous sentence
pub struct PrevSentenceHandler;

impl KeyHandler for PrevSentenceHandler {
    fn mode(&self) -> AppMode {
        AppMode::Reading
    }

    fn keys(&self) -> Vec<KeyCode> {
        vec![KeyCode::Char('k')]
    }

    fn handle(&self, app: &mut App) -> Result<KeyResult> {
        if let Some(state) = app.reading_state_mut() {
            state.jump_to_previous_sentence();
        }
        Ok(KeyResult::Consumed)
    }

    fn help_text(&self) -> &str {
        "Previous sentence (k)"
    }
}

/// Handler for ']' - increase speed
pub struct SpeedUpHandler;

impl KeyHandler for SpeedUpHandler {
    fn mode(&self) -> AppMode {
        AppMode::Reading
    }

    fn keys(&self) -> Vec<KeyCode> {
        vec![KeyCode::Char(']')]
    }

    fn handle(&self, app: &mut App) -> Result<KeyResult> {
        if let Some(state) = app.reading_state_mut() {
            state.adjust_wpm(50);
        }
        Ok(KeyResult::Consumed)
    }

    fn help_text(&self) -> &str {
        "Increase speed (])"
    }
}

/// Handler for '[' - decrease speed
pub struct SpeedDownHandler;

impl KeyHandler for SpeedDownHandler {
    fn mode(&self) -> AppMode {
        AppMode::Reading
    }

    fn keys(&self) -> Vec<KeyCode> {
        vec![KeyCode::Char('[')]
    }

    fn handle(&self, app: &mut App) -> Result<KeyResult> {
        if let Some(state) = app.reading_state_mut() {
            state.adjust_wpm(-50);
        }
        Ok(KeyResult::Consumed)
    }

    fn help_text(&self) -> &str {
        "Decrease speed ([)"
    }
}

/// Handler for 'p' - toggle pause
pub struct PauseToggleHandler;

impl KeyHandler for PauseToggleHandler {
    fn mode(&self) -> AppMode {
        AppMode::Reading
    }

    fn keys(&self) -> Vec<KeyCode> {
        vec![KeyCode::Char('p')]
    }

    fn handle(&self, app: &mut App) -> Result<KeyResult> {
        app.toggle_pause();
        Ok(KeyResult::Consumed)
    }

    fn help_text(&self) -> &str {
        "Toggle pause (p)"
    }
}

/// Create registry with all reading mode handlers
pub fn create_reading_handlers(registry: &mut crate::ui::key_handler::KeyHandlerRegistry) {
    registry.register(NextSentenceHandler);
    registry.register(PrevSentenceHandler);
    registry.register(SpeedUpHandler);
    registry.register(SpeedDownHandler);
    registry.register(PauseToggleHandler);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::key_handler::KeyHandlerRegistry;

    #[test]
    fn test_create_reading_handlers_registers_all() {
        let mut registry = KeyHandlerRegistry::new();
        create_reading_handlers(&mut registry);

        let handlers = registry.handlers_for_mode(AppMode::Reading);
        assert_eq!(handlers.len(), 5);
    }

    #[test]
    fn test_next_sentence_handler_keys() {
        let handler = NextSentenceHandler;
        let keys = handler.keys();
        assert!(keys.contains(&KeyCode::Char('j')));
        assert!(keys.contains(&KeyCode::Char(' ')));
    }

    #[test]
    fn test_prev_sentence_handler_keys() {
        let handler = PrevSentenceHandler;
        let keys = handler.keys();
        assert!(keys.contains(&KeyCode::Char('k')));
    }

    #[test]
    fn test_speed_up_handler_keys() {
        let handler = SpeedUpHandler;
        let keys = handler.keys();
        assert!(keys.contains(&KeyCode::Char(']')));
    }

    #[test]
    fn test_speed_down_handler_keys() {
        let handler = SpeedDownHandler;
        let keys = handler.keys();
        assert!(keys.contains(&KeyCode::Char('[')));
    }

    #[test]
    fn test_pause_toggle_handler_keys() {
        let handler = PauseToggleHandler;
        let keys = handler.keys();
        assert!(keys.contains(&KeyCode::Char('p')));
    }
}
