//! Key handler registry for OCP-compliant key handling
//!
//! Mirrors the CommandRegistry pattern - add key bindings without modifying existing code.

use crate::app::mode::AppMode;
use crate::app::App;
use anyhow::Result;
use crossterm::event::KeyCode;

/// Result of handling a key event
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyResult {
    /// Key was consumed, stop processing
    Consumed,
    /// Key was ignored, continue to next handler
    /// (Reserved for future handler chain extensions)
    #[allow(dead_code)]
    Ignored,
}

/// Trait for key handlers
pub trait KeyHandler: Send + Sync {
    /// Which mode this handler applies to
    fn mode(&self) -> AppMode;

    /// Which keys this handler responds to
    fn keys(&self) -> Vec<KeyCode>;

    /// Handle the key press
    fn handle(&self, app: &mut App) -> Result<KeyResult>;

    /// Get help text for this key binding
    #[allow(dead_code)]
    fn help_text(&self) -> &str {
        ""
    }
}

/// Registry for key handlers
pub struct KeyHandlerRegistry {
    handlers: Vec<Box<dyn KeyHandler>>,
}

impl KeyHandlerRegistry {
    /// Create a new empty key handler registry
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }
    
    /// Register a key handler
    pub fn register<H: KeyHandler + 'static>(&mut self, handler: H) {
        self.handlers.push(Box::new(handler));
    }
    
    /// Dispatch a key event to the appropriate handler
    pub fn dispatch(&self, key: KeyCode, mode: AppMode, app: &mut App) -> Option<Result<KeyResult>> {
        for handler in &self.handlers {
            if handler.mode() == mode && handler.keys().contains(&key) {
                return Some(handler.handle(app));
            }
        }
        None
    }

    /// Get all handlers for a specific mode (used in tests)
    #[allow(dead_code)]
    pub fn handlers_for_mode(&self, mode: AppMode) -> Vec<&dyn KeyHandler> {
        self.handlers
            .iter()
            .filter(|h| h.mode() == mode)
            .map(|h| h.as_ref())
            .collect()
    }
}

impl Default for KeyHandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    struct TestHandler;
    impl KeyHandler for TestHandler {
        fn mode(&self) -> AppMode {
            AppMode::Reading
        }
        
        fn keys(&self) -> Vec<KeyCode> {
            vec![KeyCode::Char('x')]
        }
        
        fn handle(&self, _app: &mut App) -> Result<KeyResult> {
            Ok(KeyResult::Consumed)
        }
    }
    
    #[test]
    fn test_register_and_dispatch() {
        let mut registry = KeyHandlerRegistry::new();
        registry.register(TestHandler);
        
        let mut app = App::default();
        let result = registry.dispatch(KeyCode::Char('x'), AppMode::Reading, &mut app);
        assert!(result.is_some());
        let result = result.unwrap();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), KeyResult::Consumed);
        
        let result = registry.dispatch(KeyCode::Char('y'), AppMode::Reading, &mut app);
        assert!(result.is_none());
    }
}
