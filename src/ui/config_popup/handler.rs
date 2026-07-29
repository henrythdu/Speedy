//! Key event handling for config popup
//!
//! Provides test helpers for popup handling. The actual popup key handling
//! is now done via the key handler registry in key_handlers.rs.

#[cfg(test)]
use super::state::ConfigPopupState;
#[cfg(test)]
use crate::config::Config;
#[cfg(test)]
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Result of handling a popup key event
#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupAction {
    /// No action taken, let other handlers process this key
    None,
    /// Popup state changed, needs redraw
    Handled,
    /// User pressed Enter to save and close
    SaveAndClose,
    /// User pressed Esc to cancel and close
    CancelAndClose,
}

/// Handle key events for the config popup (test helper)
///
/// # Arguments
/// * `key` - The key event to process
/// * `popup` - Mutable reference to popup state
/// * `config` - Mutable reference to app config (for live theme preview)
///
/// # Returns
/// * `PopupAction::None` - Key not handled by popup, let other handlers process
/// * `PopupAction::Handled` - Key handled, popup state changed
/// * `PopupAction::SaveAndClose` - Save changes and close popup
/// * `PopupAction::CancelAndClose` - Revert changes and close popup
///
/// # Behavior
/// - When popup is closed: Ctrl+P opens it
/// - When popup is open:
///   - Esc: Revert theme to original, close popup
///   - Enter: Save via apply_to_config(), close popup
///   - Up/Down: Navigate between rows
///   - Left/Right: Cycle values and apply live theme preview
///   - Ctrl+P: Toggle close
#[cfg(test)]
pub fn handle_popup_key(
    key: KeyEvent,
    popup: &mut ConfigPopupState,
    config: &mut Config,
) -> PopupAction {
    // Ctrl+P handling (both open and close)
    if key.code == KeyCode::Char('p') && key.modifiers.contains(KeyModifiers::CONTROL) {
        if popup.is_open {
            // Toggle close when already open
            popup.close();
            return PopupAction::Handled;
        } else {
            // Open popup with current config values
            popup.open(config);
            return PopupAction::Handled;
        }
    }

    // If popup is closed, don't handle any other keys
    if !popup.is_open {
        return PopupAction::None;
    }

    // Handle keys when popup is open
    match key.code {
        KeyCode::Esc => {
            // Revert theme to original before closing
            config.theme = popup.original_theme().to_string();
            // Revert ghost_words to original before closing
            config.ghost_words = popup.original_ghost_words();
            popup.close();
            PopupAction::CancelAndClose
        }
        KeyCode::Enter => {
            // Save current values to config
            popup.apply_to_config(config);
            popup.close();
            PopupAction::SaveAndClose
        }
        KeyCode::Up => {
            popup.move_up();
            PopupAction::Handled
        }
        KeyCode::Down => {
            popup.move_down();
            PopupAction::Handled
        }
        KeyCode::Left => {
            popup.cycle_left();
            // Live preview: apply theme immediately if on theme row
            if popup.selected_row == 1 {
                config.theme = popup.current_theme().to_string();
            }
            // Live preview: apply ghost words immediately if on ghost words row
            if popup.selected_row == 2 {
                config.ghost_words = popup.temp_ghost_words;
            }
            PopupAction::Handled
        }
        KeyCode::Right => {
            popup.cycle_right();
            // Live preview: apply theme immediately if on theme row
            if popup.selected_row == 1 {
                config.theme = popup.current_theme().to_string();
            }
            // Live preview: apply ghost words immediately if on ghost words row
            if popup.selected_row == 2 {
                config.ghost_words = popup.temp_ghost_words;
            }
            PopupAction::Handled
        }
        _ => PopupAction::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> Config {
        Config {
            theme: "tokyo-night".to_string(),
            default_wpm: 300,
            timing: Default::default(),
            ghost_words: false,
        }
    }

    fn ctrl_p() -> KeyEvent {
        KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL)
    }

    fn esc() -> KeyEvent {
        KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)
    }

    fn enter() -> KeyEvent {
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)
    }

    fn up() -> KeyEvent {
        KeyEvent::new(KeyCode::Up, KeyModifiers::NONE)
    }

    fn down() -> KeyEvent {
        KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)
    }

    fn left() -> KeyEvent {
        KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)
    }

    fn right() -> KeyEvent {
        KeyEvent::new(KeyCode::Right, KeyModifiers::NONE)
    }

    #[test]
    fn test_ctrl_p_opens_closed_popup() {
        let mut popup = ConfigPopupState::new();
        let mut config = create_test_config();

        assert!(!popup.is_open);

        let result = handle_popup_key(ctrl_p(), &mut popup, &mut config);

        assert!(popup.is_open);
        assert_eq!(result, PopupAction::Handled);
    }

    #[test]
    fn test_ctrl_p_closes_open_popup() {
        let mut popup = ConfigPopupState::new();
        let mut config = create_test_config();
        popup.open(&config);

        assert!(popup.is_open);

        let result = handle_popup_key(ctrl_p(), &mut popup, &mut config);

        assert!(!popup.is_open);
        assert_eq!(result, PopupAction::Handled);
    }

    #[test]
    fn test_esc_closes_and_reverts_theme() {
        let mut popup = ConfigPopupState::new();
        let mut config = create_test_config();
        popup.open(&config);

        // Cycle theme to something different
        popup.selected_row = 1;
        popup.cycle_right(); // Now on "dracula"

        // Verify temp theme changed
        assert_eq!(popup.current_theme(), "dracula");

        // But config should still be original
        assert_eq!(config.theme, "tokyo-night");

        let result = handle_popup_key(esc(), &mut popup, &mut config);

        assert!(!popup.is_open);
        assert_eq!(result, PopupAction::CancelAndClose);
        // Theme should be reverted to original
        assert_eq!(config.theme, "tokyo-night");
    }

    #[test]
    fn test_enter_saves_and_closes() {
        let mut popup = ConfigPopupState::new();
        let mut config = create_test_config();
        popup.open(&config);

        // Change some values
        popup.selected_row = 1;
        popup.cycle_right(); // Theme -> "dracula"
        popup.selected_row = 0;
        popup.cycle_right(); // WPM -> 350

        let result = handle_popup_key(enter(), &mut popup, &mut config);

        assert!(!popup.is_open);
        assert_eq!(result, PopupAction::SaveAndClose);
        // Config should have new values
        assert_eq!(config.theme, "dracula");
        assert_eq!(config.default_wpm, 350);
    }

    #[test]
    fn test_up_down_navigate_rows() {
        let mut popup = ConfigPopupState::new();
        let mut config = create_test_config();
        popup.open(&config);

        assert_eq!(popup.selected_row, 0);

        handle_popup_key(down(), &mut popup, &mut config);
        assert_eq!(popup.selected_row, 1);

        handle_popup_key(down(), &mut popup, &mut config);
        assert_eq!(popup.selected_row, 2);

        handle_popup_key(down(), &mut popup, &mut config);
        assert_eq!(popup.selected_row, 2); // Max row

        handle_popup_key(up(), &mut popup, &mut config);
        assert_eq!(popup.selected_row, 1);

        handle_popup_key(up(), &mut popup, &mut config);
        assert_eq!(popup.selected_row, 0);

        handle_popup_key(up(), &mut popup, &mut config);
        assert_eq!(popup.selected_row, 0); // Min row
    }

    #[test]
    fn test_left_right_cycle_values() {
        let mut popup = ConfigPopupState::new();
        let mut config = create_test_config();
        popup.open(&config);

        // Test WPM cycling (row 0)
        popup.selected_row = 0;
        assert_eq!(popup.temp_default_wpm, 300);

        handle_popup_key(right(), &mut popup, &mut config);
        assert_eq!(popup.temp_default_wpm, 350);

        handle_popup_key(left(), &mut popup, &mut config);
        assert_eq!(popup.temp_default_wpm, 300);

        // Test theme cycling (row 1)
        popup.selected_row = 1;
        assert_eq!(popup.temp_theme_index, 0);

        handle_popup_key(right(), &mut popup, &mut config);
        assert_eq!(popup.temp_theme_index, 1);

        handle_popup_key(left(), &mut popup, &mut config);
        assert_eq!(popup.temp_theme_index, 0);

        // Test ghost words cycling (row 2)
        popup.selected_row = 2;
        assert!(!popup.temp_ghost_words);

        handle_popup_key(right(), &mut popup, &mut config);
        assert!(popup.temp_ghost_words);

        handle_popup_key(left(), &mut popup, &mut config);
        assert!(!popup.temp_ghost_words);
    }

    #[test]
    fn test_live_theme_preview() {
        let mut popup = ConfigPopupState::new();
        let mut config = create_test_config();
        popup.open(&config);

        // Select theme row
        popup.selected_row = 1;
        assert_eq!(config.theme, "tokyo-night");

        // Right arrow should immediately update config.theme
        handle_popup_key(right(), &mut popup, &mut config);
        assert_eq!(popup.temp_theme_index, 1);
        assert_eq!(config.theme, "dracula"); // Live preview applied!

        // Another right
        handle_popup_key(right(), &mut popup, &mut config);
        assert_eq!(popup.temp_theme_index, 2);
        assert_eq!(config.theme, "gruvbox"); // Live preview applied!

        // Left should also apply
        handle_popup_key(left(), &mut popup, &mut config);
        assert_eq!(popup.temp_theme_index, 1);
        assert_eq!(config.theme, "dracula"); // Live preview applied!
    }

    #[test]
    fn test_other_keys_return_none_when_popup_closed() {
        let mut popup = ConfigPopupState::new();
        let mut config = create_test_config();

        // Popup is closed by default
        assert!(!popup.is_open);

        // These should all return None when popup is closed
        assert_eq!(
            handle_popup_key(esc(), &mut popup, &mut config),
            PopupAction::None
        );
        assert_eq!(
            handle_popup_key(enter(), &mut popup, &mut config),
            PopupAction::None
        );
        assert_eq!(
            handle_popup_key(up(), &mut popup, &mut config),
            PopupAction::None
        );
        assert_eq!(
            handle_popup_key(down(), &mut popup, &mut config),
            PopupAction::None
        );
        assert_eq!(
            handle_popup_key(left(), &mut popup, &mut config),
            PopupAction::None
        );
        assert_eq!(
            handle_popup_key(right(), &mut popup, &mut config),
            PopupAction::None
        );
    }

    #[test]
    fn test_unknown_key_returns_none_when_popup_open() {
        let mut popup = ConfigPopupState::new();
        let mut config = create_test_config();
        popup.open(&config);

        let unknown_key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        assert_eq!(
            handle_popup_key(unknown_key, &mut popup, &mut config),
            PopupAction::None
        );
    }

    #[test]
    fn test_esc_reverts_live_preview_to_original() {
        let mut popup = ConfigPopupState::new();
        let mut config = create_test_config();
        popup.open(&config);

        // Use live preview to change theme
        popup.selected_row = 1;
        handle_popup_key(right(), &mut popup, &mut config);
        handle_popup_key(right(), &mut popup, &mut config);

        // Config.theme was updated for live preview
        assert_eq!(config.theme, "gruvbox");

        // But original is still tracked
        assert_eq!(popup.original_theme(), "tokyo-night");

        // Esc should revert to original
        handle_popup_key(esc(), &mut popup, &mut config);
        assert_eq!(config.theme, "tokyo-night");
    }

    #[test]
    fn test_esc_reverts_ghost_words_to_original() {
        let mut popup = ConfigPopupState::new();
        let mut config = create_test_config();
        popup.open(&config);

        // Original ghost_words is false
        assert!(!config.ghost_words);
        assert!(!popup.original_ghost_words());

        // Use live preview to change ghost_words
        popup.selected_row = 2;
        handle_popup_key(right(), &mut popup, &mut config);

        // Config.ghost_words was updated for live preview
        assert!(config.ghost_words);

        // Esc should revert to original
        handle_popup_key(esc(), &mut popup, &mut config);
        assert!(!config.ghost_words);
    }

    #[test]
    fn test_esc_reverts_ghost_words_when_originally_true() {
        let mut popup = ConfigPopupState::new();
        let mut config = Config {
            theme: "tokyo-night".to_string(),
            default_wpm: 300,
            timing: Default::default(),
            ghost_words: true, // Start with ghost_words enabled
        };
        popup.open(&config);

        // Original ghost_words is true
        assert!(popup.original_ghost_words());

        // Use live preview to change ghost_words
        popup.selected_row = 2;
        handle_popup_key(right(), &mut popup, &mut config);

        // Config.ghost_words was updated for live preview (toggled to false)
        assert!(!config.ghost_words);

        // Esc should revert to original (true)
        handle_popup_key(esc(), &mut popup, &mut config);
        assert!(config.ghost_words);
    }
}
