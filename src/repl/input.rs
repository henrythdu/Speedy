use crate::app::AppEvent;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

/// Simple REPL input using rustyline
///
/// Provides:
/// - Command history (persistent across sessions)
/// - Arrow key navigation (left/right, up/down)
/// - Basic editing (backspace)
pub struct ReplInput {
    editor: DefaultEditor,
}

impl ReplInput {
    pub fn new() -> Result<Self, ReadlineError> {
        let editor = DefaultEditor::new()?;
        Ok(Self { editor })
    }

    /// Read a line of input with "speedy> " prompt
    ///
    /// This is a blocking call that returns when user presses Enter
    pub fn readline(&mut self) -> Result<String, ReadlineError> {
        self.editor.readline("speedy> ")
    }

    /// Add command to rustyline history
    pub fn add_history_entry(&mut self, line: &str) -> Result<(), ReadlineError> {
        self.editor.add_history_entry(line).map(|_| ())
    }

    /// Get command for App core
    ///
    /// Parses a input line and converts to an AppEvent
    pub fn to_app_event(&mut self, line: &str) -> crate::app::AppEvent {
        use crate::repl::command::command_to_app_event;
        use crate::repl::parser::parse_repl_input;

        let command = parse_repl_input(line);
        command_to_app_event(command)
    }
}
