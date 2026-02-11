//! Command execution for TUI
//!
//! Handles execution of parsed commands, separating command logic from event loop.

use crate::app::mode::AppMode;
use crate::app::App;
use crate::engine::config::DEFAULT_WPM;
use crate::ui::command::{parse_command, tokens_to_text, Command};
use anyhow::{Context, Result};

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
pub fn execute_command(app: &mut App, command_str: &str) -> Result<CommandResult> {
    match parse_command(command_str) {
        Command::LoadFile(path) => {
            let doc = if path.to_lowercase().ends_with(".epub") {
                crate::input::epub::load(&path)
            } else {
                crate::input::pdf::load(&path)
            }
            .with_context(|| format!("Failed to load file: {}", path))?;

            let text = tokens_to_text(&doc);
            app.start_reading(&text, DEFAULT_WPM);
            Ok(CommandResult::Continue)
        }
        Command::LoadClipboard => {
            use crate::input::clipboard;
            let doc = clipboard::load().with_context(|| "Failed to load clipboard")?;

            let text = tokens_to_text(&doc);
            app.start_reading(&text, DEFAULT_WPM);
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
