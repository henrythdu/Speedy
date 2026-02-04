//! Command execution for TUI
//!
//! Handles execution of parsed commands, separating command logic from event loop.

use crate::app::mode::AppMode;
use crate::app::App;
use crate::ui::command::{parse_command, tokens_to_text, Command};
use std::io;

/// Result of executing a command
#[derive(Debug, Clone, PartialEq)]
pub enum CommandResult {
    /// Continue running, no mode change
    Continue,
    /// Exit the event loop with specified mode
    Exit(AppMode),
}

/// Execute a command string
///
/// Parses the command and executes it against the app state.
/// Returns CommandResult indicating whether to continue or exit.
pub fn execute_command(app: &mut App, command_str: &str) -> io::Result<CommandResult> {
    match parse_command(command_str) {
        Command::LoadFile(path) => {
            let result = if path.to_lowercase().ends_with(".epub") {
                crate::input::epub::load(&path)
            } else {
                crate::input::pdf::load(&path)
            };

            match result {
                Ok(doc) => {
                    let text = tokens_to_text(&doc);
                    app.start_reading(&text, 300);
                }
                Err(e) => {
                    app.set_error(format!("Failed to load file: {}", e));
                }
            }
            Ok(CommandResult::Continue)
        }
        Command::LoadClipboard => {
            use crate::input::clipboard;
            match clipboard::load() {
                Ok(doc) => {
                    let text = tokens_to_text(&doc);
                    app.start_reading(&text, 300);
                }
                Err(e) => {
                    app.set_error(format!("Failed to load clipboard: {}", e));
                }
            }
            Ok(CommandResult::Continue)
        }
        Command::Quit => {
            app.set_mode(AppMode::Quit);
            Ok(CommandResult::Exit(AppMode::Quit))
        }
        Command::Help => {
            // Show help - for now just stay in command mode
            Ok(CommandResult::Continue)
        }
        Command::Unknown(_) => {
            app.set_error(format!("Unknown command: {}", command_str));
            Ok(CommandResult::Continue)
        }
    }
}

#[cfg(test)]
mod tests {
    // Tests would require mock App - verified through integration tests
}
