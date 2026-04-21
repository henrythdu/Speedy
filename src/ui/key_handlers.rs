//! Concrete key handler implementations

use crate::app::mode::AppMode;
use crate::app::App;
use crate::ui::key_handler::{KeyHandler, KeyResult};
use anyhow::Result;
use crossterm::event::KeyCode;

// ============================================================================
// Command Mode Handlers
// ============================================================================

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

// ============================================================================
// Popup Mode Handlers
// ============================================================================

/// Handler for Enter in Popup mode - confirms changes and closes popup
pub struct PopupConfirmHandler;

impl KeyHandler for PopupConfirmHandler {
    fn mode(&self) -> AppMode {
        AppMode::Popup
    }

    fn keys(&self) -> Vec<KeyCode> {
        vec![KeyCode::Enter]
    }

    fn handle(&self, app: &mut App) -> Result<KeyResult> {
        // Save popup changes to config
        app.config_popup.apply_to_config(&mut app.config);
        app.config_popup.close();
        // Save config to disk
        if let Err(e) = app.save_config() {
            app.set_error(format!("Failed to save config: {}", e));
        }
        // Return to previous mode (Reading or Paused)
        app.set_mode(AppMode::Reading);
        Ok(KeyResult::Consumed)
    }

    fn help_text(&self) -> &str {
        "Confirm and save (Enter)"
    }
}

/// Handler for Esc in Popup mode - dismisses popup without saving
pub struct PopupDismissHandler;

impl KeyHandler for PopupDismissHandler {
    fn mode(&self) -> AppMode {
        AppMode::Popup
    }

    fn keys(&self) -> Vec<KeyCode> {
        vec![KeyCode::Esc]
    }

    fn handle(&self, app: &mut App) -> Result<KeyResult> {
        // Revert theme to original
        app.config.theme = app.config_popup.original_theme().to_string();
        // Revert ghost_words to original
        app.config.ghost_words = app.config_popup.original_ghost_words();
        app.config_popup.close();
        // Return to previous mode (Reading or Paused)
        app.set_mode(AppMode::Reading);
        Ok(KeyResult::Consumed)
    }

    fn help_text(&self) -> &str {
        "Cancel and close (Esc)"
    }
}

/// Handler for Down arrow or 'j' in Popup mode - navigate to next option
pub struct PopupNavigateDownHandler;

impl KeyHandler for PopupNavigateDownHandler {
    fn mode(&self) -> AppMode {
        AppMode::Popup
    }

    fn keys(&self) -> Vec<KeyCode> {
        vec![KeyCode::Down, KeyCode::Char('j')]
    }

    fn handle(&self, app: &mut App) -> Result<KeyResult> {
        app.config_popup.move_down();
        Ok(KeyResult::Consumed)
    }

    fn help_text(&self) -> &str {
        "Next option (j, Down)"
    }
}

/// Handler for Up arrow or 'k' in Popup mode - navigate to previous option
pub struct PopupNavigateUpHandler;

impl KeyHandler for PopupNavigateUpHandler {
    fn mode(&self) -> AppMode {
        AppMode::Popup
    }

    fn keys(&self) -> Vec<KeyCode> {
        vec![KeyCode::Up, KeyCode::Char('k')]
    }

    fn handle(&self, app: &mut App) -> Result<KeyResult> {
        app.config_popup.move_up();
        Ok(KeyResult::Consumed)
    }

    fn help_text(&self) -> &str {
        "Previous option (k, Up)"
    }
}

/// Handler for Left arrow or 'h' in Popup mode - cycle value left/decrease
pub struct PopupCycleLeftHandler;

impl KeyHandler for PopupCycleLeftHandler {
    fn mode(&self) -> AppMode {
        AppMode::Popup
    }

    fn keys(&self) -> Vec<KeyCode> {
        vec![KeyCode::Left, KeyCode::Char('h')]
    }

    fn handle(&self, app: &mut App) -> Result<KeyResult> {
        app.config_popup.cycle_left();
        // Live preview: apply theme immediately if on theme row
        if app.config_popup.selected_row == 1 {
            app.config.theme = app.config_popup.current_theme().to_string();
        }
        // Live preview: apply ghost words immediately if on ghost words row
        if app.config_popup.selected_row == 2 {
            app.config.ghost_words = app.config_popup.temp_ghost_words;
        }
        Ok(KeyResult::Consumed)
    }

    fn help_text(&self) -> &str {
        "Cycle left/decrease (h, Left)"
    }
}

/// Handler for Right arrow or 'l' in Popup mode - cycle value right/increase
pub struct PopupCycleRightHandler;

impl KeyHandler for PopupCycleRightHandler {
    fn mode(&self) -> AppMode {
        AppMode::Popup
    }

    fn keys(&self) -> Vec<KeyCode> {
        vec![KeyCode::Right, KeyCode::Char('l')]
    }

    fn handle(&self, app: &mut App) -> Result<KeyResult> {
        app.config_popup.cycle_right();
        // Live preview: apply theme immediately if on theme row
        if app.config_popup.selected_row == 1 {
            app.config.theme = app.config_popup.current_theme().to_string();
        }
        // Live preview: apply ghost words immediately if on ghost words row
        if app.config_popup.selected_row == 2 {
            app.config.ghost_words = app.config_popup.temp_ghost_words;
        }
        Ok(KeyResult::Consumed)
    }

    fn help_text(&self) -> &str {
        "Cycle right/increase (l, Right)"
    }
}

/// Create registry with all popup mode handlers
pub fn create_popup_handlers(registry: &mut crate::ui::key_handler::KeyHandlerRegistry) {
    registry.register(PopupConfirmHandler);
    registry.register(PopupDismissHandler);
    registry.register(PopupNavigateDownHandler);
    registry.register(PopupNavigateUpHandler);
    registry.register(PopupCycleLeftHandler);
    registry.register(PopupCycleRightHandler);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::key_handler::KeyHandlerRegistry;

    #[test]
    fn test_create_reading_handlers_registers_all() {
        let mut registry = KeyHandlerRegistry::new();
        create_reading_handlers(&mut registry);

        // Verify handlers are registered by checking they can be dispatched
        let mut app = App::default();
        app.start_reading("test", 300);

        // Test j key (next sentence)
        assert!(registry.dispatch(KeyCode::Char('j'), AppMode::Reading, &mut app).is_some());
        // Test k key (previous sentence)
        assert!(registry.dispatch(KeyCode::Char('k'), AppMode::Reading, &mut app).is_some());
        // Test p key (pause toggle)
        assert!(registry.dispatch(KeyCode::Char('p'), AppMode::Reading, &mut app).is_some());
        // Test [ key (speed down)
        assert!(registry.dispatch(KeyCode::Char('['), AppMode::Reading, &mut app).is_some());
        // Test ] key (speed up)
        assert!(registry.dispatch(KeyCode::Char(']'), AppMode::Reading, &mut app).is_some());
    }

    #[test]
    fn test_create_popup_handlers_registers_all() {
        let mut registry = KeyHandlerRegistry::new();
        create_popup_handlers(&mut registry);

        // Verify handlers are registered by checking they can be dispatched
        let mut app = App::default();

        // Test Enter key (confirm)
        assert!(registry.dispatch(KeyCode::Enter, AppMode::Popup, &mut app).is_some());
        // Test Esc key (dismiss)
        assert!(registry.dispatch(KeyCode::Esc, AppMode::Popup, &mut app).is_some());
        // Test j key (navigate down)
        assert!(registry.dispatch(KeyCode::Char('j'), AppMode::Popup, &mut app).is_some());
        // Test k key (navigate up)
        assert!(registry.dispatch(KeyCode::Char('k'), AppMode::Popup, &mut app).is_some());
        // Test h key (cycle left)
        assert!(registry.dispatch(KeyCode::Char('h'), AppMode::Popup, &mut app).is_some());
        // Test l key (cycle right)
        assert!(registry.dispatch(KeyCode::Char('l'), AppMode::Popup, &mut app).is_some());
    }

    #[test]
    fn test_popup_confirm_handler_keys() {
        let handler = PopupConfirmHandler;
        let keys = handler.keys();
        assert!(keys.contains(&KeyCode::Enter));
    }

    #[test]
    fn test_popup_dismiss_handler_keys() {
        let handler = PopupDismissHandler;
        let keys = handler.keys();
        assert!(keys.contains(&KeyCode::Esc));
    }

    #[test]
    fn test_popup_navigate_down_handler_keys() {
        let handler = PopupNavigateDownHandler;
        let keys = handler.keys();
        assert!(keys.contains(&KeyCode::Down));
        assert!(keys.contains(&KeyCode::Char('j')));
    }

    #[test]
    fn test_popup_navigate_up_handler_keys() {
        let handler = PopupNavigateUpHandler;
        let keys = handler.keys();
        assert!(keys.contains(&KeyCode::Up));
        assert!(keys.contains(&KeyCode::Char('k')));
    }

    #[test]
    fn test_popup_cycle_left_handler_keys() {
        let handler = PopupCycleLeftHandler;
        let keys = handler.keys();
        assert!(keys.contains(&KeyCode::Left));
        assert!(keys.contains(&KeyCode::Char('h')));
    }

    #[test]
    fn test_popup_cycle_right_handler_keys() {
        let handler = PopupCycleRightHandler;
        let keys = handler.keys();
        assert!(keys.contains(&KeyCode::Right));
        assert!(keys.contains(&KeyCode::Char('l')));
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
