//! Concrete key handler implementations

use crate::app::mode::AppMode;
use crate::app::App;
use crate::ui::key_handler::{KeyHandler, KeyResult};
use anyhow::Result;
use crossterm::event::KeyCode;

// ============================================================================
// Command Mode Handlers
// ============================================================================

/// Handler for any character input in Command mode - buffers to command_buffer
pub struct CommandCharHandler;

impl KeyHandler for CommandCharHandler {
    fn mode(&self) -> AppMode {
        AppMode::Command
    }

    fn keys(&self) -> Vec<KeyCode> {
        // Match all printable characters
        // This is handled specially by checking if it's a Char without Ctrl modifier
        vec![]
    }

    fn handle(&self, _app: &mut App) -> Result<KeyResult> {
        // This handler is dispatched directly from the event loop for Char keys
        // The actual character is passed via a custom dispatch mechanism
        Ok(KeyResult::Ignored)
    }

    fn help_text(&self) -> &str {
        "Type command character"
    }
}

/// Handler for Backspace in Command mode
pub struct CommandBackspaceHandler;

impl KeyHandler for CommandBackspaceHandler {
    fn mode(&self) -> AppMode {
        AppMode::Command
    }

    fn keys(&self) -> Vec<KeyCode> {
        vec![KeyCode::Backspace]
    }

    fn handle(&self, app: &mut App) -> Result<KeyResult> {
        app.pop_command_char();
        Ok(KeyResult::Consumed)
    }

    fn help_text(&self) -> &str {
        "Delete last character (Backspace)"
    }
}

/// Handler for Enter in Command mode - executes the command
pub struct CommandEnterHandler;

impl KeyHandler for CommandEnterHandler {
    fn mode(&self) -> AppMode {
        AppMode::Command
    }

    fn keys(&self) -> Vec<KeyCode> {
        vec![KeyCode::Enter]
    }

    fn handle(&self, app: &mut App) -> Result<KeyResult> {
        let command = app.command_buffer().to_string();
        if !command.is_empty() {
            app.clear_command_buffer();
            // Execute the command
            use crate::ui::command_executor::{execute_command, CommandResult};
            match execute_command(app, &command)? {
                CommandResult::Continue => {}
                CommandResult::Exit(_) => {
                    // Exit is handled by the caller checking app.mode()
                }
            }
        }
        Ok(KeyResult::Consumed)
    }

    fn help_text(&self) -> &str {
        "Execute command (Enter)"
    }
}

/// Handler for Escape in Command mode - exits to Reading mode
pub struct CommandEscapeHandler;

impl KeyHandler for CommandEscapeHandler {
    fn mode(&self) -> AppMode {
        AppMode::Command
    }

    fn keys(&self) -> Vec<KeyCode> {
        vec![KeyCode::Esc]
    }

    fn handle(&self, app: &mut App) -> Result<KeyResult> {
        app.clear_command_buffer();
        app.set_mode(AppMode::Reading);
        Ok(KeyResult::Consumed)
    }

    fn help_text(&self) -> &str {
        "Cancel command (Esc)"
    }
}

/// Create registry with all command mode handlers
pub fn create_command_handlers(registry: &mut crate::ui::key_handler::KeyHandlerRegistry) {
    registry.register(CommandBackspaceHandler);
    registry.register(CommandEnterHandler);
    registry.register(CommandEscapeHandler);
    // Note: CommandCharHandler is handled specially since it needs the character
}

// ============================================================================
// Reading Mode Handlers
// ============================================================================

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
