//! Event Handler Module - Command Pattern Implementation
//!
//! Reduces cyclomatic complexity of the event loop by extracting mode-specific
//! handlers into separate structs implementing the EventHandler trait.
//!
//! Architecture:
//! - EventHandler trait: Common interface for all handlers
//! - CommandModeHandler: Handles input collection and command execution
//! - ReadingModeHandler: Handles RSVP reading navigation keys
//! - AutocompleteHandler: Handles file autocomplete navigation and selection
//! - EventDispatcher: Routes events to the appropriate handler based on mode

use crate::app::mode::AppMode;
use crate::app::App;
use crate::ui::autocomplete::state::AutocompleteState;
use crate::ui::UIError;
use crossterm::event::{Event, KeyCode, KeyEvent};
use std::cell::RefCell;
use std::rc::Rc;

/// Trait for mode-specific event handlers
///
/// Each handler implements this trait to process events in its specific mode.
/// Handlers return Ok(()) on success or UIError on failure.
///
/// # Example
/// ```
/// let handler = CommandModeHandler::new();
/// handler.handle(event, &mut app)?;
/// ```
pub trait EventHandler {
    /// Handle an event in the appropriate mode context
    ///
    /// # Arguments
    /// * `event` - The crossterm event to handle
    /// * `app` - Mutable reference to the application state
    ///
    /// # Returns
    /// * `Ok(())` - Event was handled successfully
    /// * `Err(UIError)` - An error occurred during handling
    fn handle(&self, event: Event, app: &mut App) -> Result<(), UIError>;
}

/// Context shared between handlers and the main event loop
///
/// Contains mutable state that handlers need to modify but that doesn't
/// belong in the App struct (UI-specific state like command_buffer).
#[derive(Debug)]
pub struct HandlerContext {
    /// Current command input buffer
    pub command_buffer: String,
    /// Whether the cursor should be visible (for blinking)
    pub cursor_visible: bool,
    /// Time of last keypress (for cursor blink pause)
    pub last_keypress: std::time::Instant,
}

impl HandlerContext {
    /// Create a new handler context with default values
    pub fn new() -> Self {
        Self {
            command_buffer: String::new(),
            cursor_visible: true,
            last_keypress: std::time::Instant::now(),
        }
    }

    /// Reset the context (called when entering command mode)
    pub fn reset(&mut self) {
        self.command_buffer.clear();
        self.cursor_visible = true;
        self.last_keypress = std::time::Instant::now();
    }

    /// Record a keypress to pause cursor blinking
    pub fn record_keypress(&mut self) {
        self.last_keypress = std::time::Instant::now();
        self.cursor_visible = true;
    }
}

impl Default for HandlerContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Handler for Command mode events
///
/// Handles:
/// - Character input for command building
/// - Enter key for command execution
/// - Backspace for character deletion
/// - Esc to exit to Reading mode (if reading state exists)
#[derive(Debug, Clone)]
pub struct CommandModeHandler {
    /// Shared context for command handling
    context: Rc<RefCell<HandlerContext>>,
}

impl CommandModeHandler {
    /// Create a new command mode handler with the given context
    pub fn new(context: Rc<RefCell<HandlerContext>>) -> Self {
        Self { context }
    }

    /// Process a key event in command mode
    pub fn handle_key_event(&self, key: KeyEvent, app: &mut App) -> Result<CommandAction, UIError> {
        use crossterm::event::KeyModifiers;

        let mut context = self.context.borrow_mut();

        // Handle Ctrl+C to quit
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            app.set_mode(AppMode::Quit);
            return Ok(CommandAction::Quit);
        }

        match key.code {
            KeyCode::Char(c) => {
                context.command_buffer.push(c);
                context.record_keypress();
                Ok(CommandAction::Continue)
            }
            KeyCode::Enter => {
                if !context.command_buffer.is_empty() {
                    let command = context.command_buffer.clone();
                    context.command_buffer.clear();
                    return Ok(CommandAction::ExecuteCommand(command));
                }
                Ok(CommandAction::Continue)
            }
            KeyCode::Backspace => {
                context.command_buffer.pop();
                Ok(CommandAction::Continue)
            }
            KeyCode::Esc => {
                // Exit to Reading mode if we have reading state
                if app.reading_state.is_some() {
                    app.set_mode(AppMode::Reading);
                }
                Ok(CommandAction::Continue)
            }
            _ => Ok(CommandAction::Continue),
        }
    }
}

impl EventHandler for CommandModeHandler {
    fn handle(&self, event: Event, app: &mut App) -> Result<(), UIError> {
        if let Event::Key(key) = event {
            self.handle_key_event(key, app)?;
        }
        Ok(())
    }
}

/// Actions that can result from command mode handling
#[derive(Debug)]
pub enum CommandAction {
    /// Continue processing events normally
    Continue,
    /// Execute the given command string
    ExecuteCommand(String),
    /// Quit the application
    Quit,
}

/// Handler for Reading and Paused mode events
///
/// Handles:
/// - Navigation keys (j/k for sentence navigation)
/// - WPM adjustment ([/])
/// - Space for pause/resume
/// - q to return to Command mode
/// - Character keys passed to app's handle_keypress
#[derive(Debug, Clone, Copy)]
pub struct ReadingModeHandler;

impl ReadingModeHandler {
    /// Create a new reading mode handler
    pub fn new() -> Self {
        Self
    }

    /// Process a key event in reading or paused mode
    pub fn handle_key_event(&self, key: KeyEvent, app: &mut App) -> Result<(), UIError> {
        use crossterm::event::KeyModifiers;

        // Handle Ctrl+C to quit
        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            app.set_mode(AppMode::Quit);
            return Ok(());
        }

        match key.code {
            KeyCode::Char(c) => {
                app.handle_keypress(c);
                Ok(())
            }
            KeyCode::Esc => {
                app.set_mode(AppMode::Command);
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

impl EventHandler for ReadingModeHandler {
    fn handle(&self, event: Event, app: &mut App) -> Result<(), UIError> {
        if let Event::Key(key) = event {
            self.handle_key_event(key, app)?;
        }
        Ok(())
    }
}

/// Handler for autocomplete-specific events
///
/// Handles:
/// - Up/Down for navigation
/// - Tab for selection with trailing space
/// - Enter for selection
/// - Backspace for query modification
/// - Esc to close autocomplete
#[derive(Debug, Clone)]
pub struct AutocompleteHandler {
    /// Shared context for command handling
    context: Rc<RefCell<HandlerContext>>,
    /// Shared autocomplete state
    autocomplete_state: Rc<RefCell<AutocompleteState>>,
}

impl AutocompleteHandler {
    /// Create a new autocomplete handler with the given context and state
    pub fn new(
        context: Rc<RefCell<HandlerContext>>,
        autocomplete_state: Rc<RefCell<AutocompleteState>>,
    ) -> Self {
        Self {
            context,
            autocomplete_state,
        }
    }

    /// Process a key event when autocomplete is active
    ///
    /// Returns the action indicating how the event was handled
    pub fn handle_key_event(
        &self,
        key: KeyEvent,
        _app: &mut App,
    ) -> Result<AutocompleteAction, UIError> {
        use crossterm::event::KeyModifiers;

        let mut context = self.context.borrow_mut();
        let mut autocomplete_state = self.autocomplete_state.borrow_mut();

        // Handle Ctrl+R to refresh cache
        if key.code == KeyCode::Char('r') && key.modifiers.contains(KeyModifiers::CONTROL) {
            return Ok(AutocompleteAction::RefreshCache);
        }

        match key.code {
            KeyCode::Up => {
                autocomplete_state.select_previous();
                Ok(AutocompleteAction::Continue)
            }
            KeyCode::Down => {
                autocomplete_state.select_next();
                Ok(AutocompleteAction::Continue)
            }
            KeyCode::Tab => {
                autocomplete_state.apply_selection(&mut context.command_buffer);
                context.command_buffer.push(' ');
                Ok(AutocompleteAction::Continue)
            }
            KeyCode::Enter => {
                autocomplete_state.apply_selection(&mut context.command_buffer);
                autocomplete_state.deactivate();
                Ok(AutocompleteAction::Continue)
            }
            KeyCode::Backspace => {
                autocomplete_state.backspace();
                if autocomplete_state.query.is_empty() {
                    autocomplete_state.deactivate();
                }
                context.command_buffer.pop();
                Ok(AutocompleteAction::Continue)
            }
            KeyCode::Esc => {
                autocomplete_state.deactivate();
                Ok(AutocompleteAction::Continue)
            }
            KeyCode::Char(c) => {
                // Pass through to command buffer but also update autocomplete
                context.command_buffer.push(c);
                context.record_keypress();
                autocomplete_state.handle_input(c);
                Ok(AutocompleteAction::Continue)
            }
            _ => Ok(AutocompleteAction::PassThrough),
        }
    }
}

impl EventHandler for AutocompleteHandler {
    fn handle(&self, event: Event, app: &mut App) -> Result<(), UIError> {
        if let Event::Key(key) = event {
            self.handle_key_event(key, app)?;
        }
        Ok(())
    }
}

/// Actions that can result from autocomplete handling
#[derive(Debug)]
pub enum AutocompleteAction {
    /// Continue processing events normally (event was consumed by autocomplete)
    Continue,
    /// Event should be passed through to normal command handling
    PassThrough,
    /// Refresh the autocomplete cache
    RefreshCache,
    /// Execute the given command string
    ExecuteCommand(String),
    /// Return the specified mode
    ReturnMode(crate::app::mode::AppMode),
}

/// Dispatches events to the appropriate handler based on application mode
///
/// This is the main entry point for event handling. It examines the current
/// mode and routes events to the correct handler.
#[derive(Debug)]
pub struct EventDispatcher {
    command_handler: CommandModeHandler,
    reading_handler: ReadingModeHandler,
    autocomplete_handler: AutocompleteHandler,
}

impl EventDispatcher {
    /// Create a new event dispatcher with all handlers initialized
    pub fn new(
        context: Rc<RefCell<HandlerContext>>,
        autocomplete_state: Rc<RefCell<AutocompleteState>>,
    ) -> Self {
        Self {
            command_handler: CommandModeHandler::new(context.clone()),
            reading_handler: ReadingModeHandler::new(),
            autocomplete_handler: AutocompleteHandler::new(context, autocomplete_state),
        }
    }

    /// Get the appropriate handler for the current mode
    ///
    /// Returns the handler that should process events in the given mode.
    /// Note: Autocomplete is handled specially since it's a sub-mode of Command.
    pub fn get_handler(&self, mode: AppMode) -> Option<&dyn EventHandler> {
        match mode {
            AppMode::Command => Some(&self.command_handler as &dyn EventHandler),
            AppMode::Reading | AppMode::Paused => Some(&self.reading_handler as &dyn EventHandler),
            _ => None,
        }
    }

    /// Handle Ctrl+C globally (works in any mode)
    ///
    /// Returns true if the event was Ctrl+C and was handled
    pub fn handle_global_shortcuts(&self, key: KeyEvent, app: &mut App) -> bool {
        use crossterm::event::{KeyCode, KeyModifiers};

        if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
            app.set_mode(AppMode::Quit);
            return true;
        }
        false
    }

    /// Get the command handler for direct access
    pub fn command_handler(&self) -> &CommandModeHandler {
        &self.command_handler
    }

    /// Get the reading handler for direct access
    pub fn reading_handler(&self) -> &ReadingModeHandler {
        &self.reading_handler
    }

    /// Get the autocomplete handler for direct access
    pub fn autocomplete_handler(&self) -> &AutocompleteHandler {
        &self.autocomplete_handler
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_handler_context_creation() {
        let context = HandlerContext::new();
        assert!(context.command_buffer.is_empty());
        assert!(context.cursor_visible);
    }

    #[test]
    fn test_handler_context_reset() {
        let mut context = HandlerContext::new();
        context.command_buffer.push_str("test");
        context.reset();
        assert!(context.command_buffer.is_empty());
        assert!(context.cursor_visible);
    }

    #[test]
    fn test_dispatcher_creation() {
        let context = Rc::new(RefCell::new(HandlerContext::new()));
        let autocomplete_state = Rc::new(RefCell::new(AutocompleteState::new()));
        let dispatcher = EventDispatcher::new(context, autocomplete_state);
        assert!(matches!(dispatcher.get_handler(AppMode::Command), Some(_)));
        assert!(matches!(dispatcher.get_handler(AppMode::Reading), Some(_)));
        assert!(matches!(dispatcher.get_handler(AppMode::Paused), Some(_)));
        assert!(matches!(dispatcher.get_handler(AppMode::Quit), None));
    }

    #[test]
    fn test_command_handler_creates() {
        let context = Rc::new(RefCell::new(HandlerContext::new()));
        let _handler = CommandModeHandler::new(context);
    }

    #[test]
    fn test_reading_handler_creates() {
        let _handler = ReadingModeHandler::new();
    }

    #[test]
    fn test_autocomplete_handler_creates() {
        let context = Rc::new(RefCell::new(HandlerContext::new()));
        let autocomplete_state = Rc::new(RefCell::new(AutocompleteState::new()));
        let _handler = AutocompleteHandler::new(context, autocomplete_state);
    }

    #[test]
    fn test_global_shortcuts_detects_ctrl_c() {
        let context = Rc::new(RefCell::new(HandlerContext::new()));
        let autocomplete_state = Rc::new(RefCell::new(AutocompleteState::new()));
        let dispatcher = EventDispatcher::new(context, autocomplete_state);
        let mut app = App::new();

        let key = KeyEvent::from(KeyCode::Char('c'));
        let handled = dispatcher.handle_global_shortcuts(key, &mut app);
        assert!(!handled); // Without Ctrl modifier

        // Test with Ctrl+C (using modifiers)
        let key = KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };
        let handled = dispatcher.handle_global_shortcuts(key, &mut app);
        assert!(handled);
        assert_eq!(app.mode(), AppMode::Quit);
    }

    #[test]
    fn test_command_action_enum() {
        let _action = CommandAction::Continue;
        let _action = CommandAction::ExecuteCommand("test".to_string());
        let _action = CommandAction::Quit;
    }

    #[test]
    fn test_autocomplete_action_enum() {
        let _action = AutocompleteAction::Continue;
        let _action = AutocompleteAction::PassThrough;
        let _action = AutocompleteAction::RefreshCache;
        let _action = AutocompleteAction::ExecuteCommand("test".to_string());
        let _action = AutocompleteAction::ReturnMode(crate::app::mode::AppMode::Command);
    }
}
