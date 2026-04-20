# Key Handler Registry Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement OCP-compliant key handling via KeyHandler trait + registry, replacing hardcoded dispatch in terminal.rs and app_impl.rs

**Architecture:** Mirror the working CommandRegistry pattern. Each key binding is a struct implementing KeyHandler trait with mode(), keys(), handle(), help_text() methods. Registry dispatches by (mode, key) lookup.

**Tech Stack:** Rust, crossterm (KeyCode), ratatui

---

## File Structure

**New files:**
- `src/ui/key_handler.rs` - KeyHandler trait, KeyHandlerRegistry, KeyResult enum
- `src/ui/key_handlers.rs` - Concrete handler implementations (Reading, Command, Popup modes)

**Modified files:**
- `src/ui/mod.rs` - Add key_handler, key_handlers modules
- `src/ui/terminal.rs` - Replace hardcoded dispatch with registry
- `src/app/app_impl.rs` - Remove handle_keypress(), reading_state_mut()

---

## Task 1: Create KeyHandler Core Types

**Files:**
- Create: `src/ui/key_handler.rs`

**Prerequisites:**
- Read `src/ui/commands/mod.rs` to understand CommandHandler trait pattern
- Read `src/app/mode.rs` for AppMode enum

- [ ] **Step 1: Write KeyHandler trait and KeyResult enum**

```rust
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
    Ignored,
}

/// Trait for key handlers
///
/// Implement this trait to create new key bindings that can be registered
/// without modifying existing code.
pub trait KeyHandler: Send + Sync {
    /// Which mode this handler applies to
    fn mode(&self) -> AppMode;
    
    /// Which keys this handler responds to
    fn keys(&self) -> Vec<KeyCode>;
    
    /// Handle the key press
    ///
    /// # Arguments
    /// * `app` - Mutable reference to application state
    ///
    /// # Returns
    /// `KeyResult` indicating whether the key was consumed
    fn handle(&self, app: &mut App) -> Result<KeyResult>;
    
    /// Get help text for this key binding
    fn help_text(&self) -> &str {
        ""
    }
}

/// Registry for key handlers
///
/// Manages a collection of key handlers and dispatches key events
/// to the appropriate handler based on mode and key code.
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
    ///
    /// # Arguments
    /// * `handler` - The key handler to register
    pub fn register<H: KeyHandler + 'static>(&mut self, handler: H) {
        self.handlers.push(Box::new(handler));
    }
    
    /// Dispatch a key event to the appropriate handler
    ///
    /// Finds the first handler matching (mode, key) and executes it.
    ///
    /// # Arguments
    /// * `key` - The key code that was pressed
    /// * `mode` - Current application mode
    /// * `app` - Mutable reference to application state
    ///
    /// # Returns
    /// * `Some(Result)` if a handler was found and executed
    /// * `None` if no handler could handle the key
    pub fn dispatch(&self, key: KeyCode, mode: AppMode, app: &mut App) -> Option<Result<KeyResult>> {
        for handler in &self.handlers {
            if handler.mode() == mode && handler.keys().contains(&key) {
                return Some(handler.handle(app));
            }
        }
        None
    }
    
    /// Get all handlers for a specific mode
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
        
        // Should find handler for 'x' in Reading mode
        let mut app = App::default();
        let result = registry.dispatch(KeyCode::Char('x'), AppMode::Reading, &mut app);
        assert!(result.is_some());
        assert_eq!(result.unwrap().unwrap(), KeyResult::Consumed);
        
        // Should not find handler for 'y'
        let result = registry.dispatch(KeyCode::Char('y'), AppMode::Reading, &mut app);
        assert!(result.is_none());
        
        // Should not find handler in Command mode
        let result = registry.dispatch(KeyCode::Char('x'), AppMode::Command, &mut app);
        assert!(result.is_none());
    }
}
```

- [ ] **Step 2: Run tests to verify the trait and registry work**

```bash
cargo test ui::key_handler::tests --lib
```

Expected: Tests pass

- [ ] **Step 3: Commit**

```bash
git add src/ui/key_handler.rs
git commit -m "feat: Add KeyHandler trait and KeyHandlerRegistry

- KeyHandler trait with mode(), keys(), handle(), help_text()
- KeyHandlerRegistry with register() and dispatch()
- KeyResult enum (Consumed/Ignored)
- Mirrors CommandRegistry pattern for OCP compliance"
```

---

## Task 2: Create Reading Mode Key Handlers

**Files:**
- Create: `src/ui/key_handlers.rs`
- Read: `src/app/app_impl.rs` handle_keypress() method (lines ~150-220)

**Prerequisites:**
- Understand app.next_word(), app.prev_word(), adjust_wpm() methods

- [ ] **Step 1: Write Reading mode handlers**

```rust
//! Concrete key handler implementations
//!
//! Handlers are organized by mode:
//! - Reading mode: j/k/[/]/Space for navigation and speed
//! - Command mode: Enter/Backspace/Esc for command input
//! - Popup mode: Enter/Esc for popup interactions

use crate::app::mode::AppMode;
use crate::app::App;
use crate::ui::key_handler::{KeyHandler, KeyResult};
use anyhow::Result;
use crossterm::event::KeyCode;

// ============================================================================
// Reading Mode Handlers
// ============================================================================

/// Handler for 'j' and Space - move to next word
pub struct NextWordHandler;

impl KeyHandler for NextWordHandler {
    fn mode(&self) -> AppMode {
        AppMode::Reading
    }
    
    fn keys(&self) -> Vec<KeyCode> {
        vec![KeyCode::Char('j'), KeyCode::Char(' ')]
    }
    
    fn handle(&self, app: &mut App) -> Result<KeyResult> {
        app.next_word()?;
        Ok(KeyResult::Consumed)
    }
    
    fn help_text(&self) -> &str {
        "Next word (j, Space)"
    }
}

/// Handler for 'k' - move to previous word
pub struct PrevWordHandler;

impl KeyHandler for PrevWordHandler {
    fn mode(&self) -> AppMode {
        AppMode::Reading
    }
    
    fn keys(&self) -> Vec<KeyCode> {
        vec![KeyCode::Char('k')]
    }
    
    fn handle(&self, app: &mut App) -> Result<KeyResult> {
        app.prev_word()?;
        Ok(KeyResult::Consumed)
    }
    
    fn help_text(&self) -> &str {
        "Previous word (k)"
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
        // Increase WPM by 10
        if let Some(state) = app.reading_state.as_mut() {
            state.adjust_wpm(10)?;
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
        // Decrease WPM by 10
        if let Some(state) = app.reading_state.as_mut() {
            state.adjust_wpm(-10)?;
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
pub fn create_reading_handlers(registry: &mut KeyHandlerRegistry) {
    registry.register(NextWordHandler);
    registry.register(PrevWordHandler);
    registry.register(SpeedUpHandler);
    registry.register(SpeedDownHandler);
    registry.register(PauseToggleHandler);
}
```

- [ ] **Step 2: Add module declaration to ui/mod.rs**

```rust
// In src/ui/mod.rs, add:
pub mod key_handler;
pub mod key_handlers;

pub use terminal::TuiManager;
```

- [ ] **Step 3: Verify compilation**

```bash
cargo check
```

Expected: No errors

- [ ] **Step 4: Commit**

```bash
git add src/ui/key_handlers.rs src/ui/mod.rs
git commit -m "feat: Add Reading mode key handlers

- NextWordHandler (j, Space)
- PrevWordHandler (k)
- SpeedUpHandler (])
- SpeedDownHandler ([)
- PauseToggleHandler (p)
- create_reading_handlers() helper function"
```

---

## Task 3: Integrate KeyHandlerRegistry into Terminal

**Files:**
- Modify: `src/ui/terminal.rs`
- Read: Current handle_key_event() method (lines ~100-150)

**Prerequisites:**
- Understand current hardcoded dispatch in handle_key_event()

- [ ] **Step 1: Add KeyHandlerRegistry to TuiManager**

```rust
// In src/ui/terminal.rs, add to imports:
use crate::ui::key_handler::KeyHandlerRegistry;
use crate::ui::key_handlers::create_reading_handlers;

// Add to TuiManager struct:
pub struct TuiManager {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    command_buffer: String,
    input_handler: InputHandler,
    autocomplete: AutocompleteManager,
    frame_renderer: FrameRenderer,
    key_registry: KeyHandlerRegistry,  // NEW
}

// In TuiManager::new(), initialize the registry:
let mut key_registry = KeyHandlerRegistry::new();
create_reading_handlers(&mut key_registry);

// Return TuiManager with key_registry field
```

- [ ] **Step 2: Replace handle_key_event() with registry dispatch**

```rust
// Replace the current handle_key_event() method:
fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
    use crossterm::event::KeyModifiers;
    
    let key = key_event.code;
    
    // Global keys - always handled directly
    match key {
        KeyCode::Char('c') if key_event.modifiers == KeyModifiers::CONTROL => {
            self.app.set_running(false);
            return Ok(());
        }
        _ => {}
    }
    
    // Popup mode - handle popup-specific keys
    if self.app.config_popup.is_open() {
        return self.handle_popup_key(key);
    }
    
    // Dispatch through key registry for current mode
    match self.key_registry.dispatch(key, self.app.mode, &mut self.app) {
        Some(Ok(KeyResult::Consumed)) => {
            // Key was handled
            return Ok(());
        }
        Some(Ok(KeyResult::Ignored)) => {
            // Handler ignored it, continue
        }
        Some(Err(e)) => {
            return Err(e);
        }
        None => {
            // No handler found, continue to other logic
        }
    }
    
    // Handle mode transitions and command mode
    match self.app.mode {
        AppMode::Reading => {
            if let KeyCode::Char(':') = key {
                self.app.mode = AppMode::Command;
                self.command_buffer.clear();
            }
        }
        AppMode::Command => {
            self.handle_command_mode_key(key_event)?;
        }
        _ => {}
    }
    
    Ok(())
}
```

- [ ] **Step 3: Verify compilation and run tests**

```bash
cargo check
cargo test
```

Expected: No errors, all tests pass

- [ ] **Step 4: Commit**

```bash
git add src/ui/terminal.rs
git commit -m "feat: Integrate KeyHandlerRegistry into TuiManager

- Add key_registry field to TuiManager
- Initialize with create_reading_handlers()
- Replace hardcoded Reading mode dispatch with registry
- Keep global keys (Ctrl+C) and mode transitions as direct matches"
```

---

## Task 4: Remove Old handle_keypress() from App

**Files:**
- Modify: `src/app/app_impl.rs`
- Remove: handle_keypress() method (lines ~150-220)
- Remove: reading_state_mut() method (lines ~120-125)

- [ ] **Step 1: Remove handle_keypress() method**

```rust
// DELETE this entire method from app_impl.rs:
pub fn handle_keypress(&mut self, c: char) -> Result<()> {
    // ... existing implementation handling j/k/[/]/Space ...
}
```

- [ ] **Step 2: Remove reading_state_mut() method**

```rust
// DELETE this method (already removed in earlier cleanup, verify it's gone):
pub fn reading_state_mut(&mut self) -> Option<&mut ReadingState> {
    self.reading_state.as_mut()
}
```

- [ ] **Step 3: Verify all key functionality still works**

```bash
cargo test
cargo build --release
```

Expected: All tests pass, clean build

- [ ] **Step 4: Commit**

```bash
git add src/app/app_impl.rs
git commit -m "refactor: Remove handle_keypress() from App

Key handling now fully handled by KeyHandlerRegistry.
App no longer has direct key handling logic - follows SRP."
```

---

## Task 5: Add Command Mode Handlers

**Files:**
- Modify: `src/ui/key_handlers.rs`

**Prerequisites:**
- Understand how command_buffer works in terminal.rs
- Understand how to call command_executor::execute_command()

- [ ] **Step 1: Add Command mode handlers**

```rust
// Add to src/ui/key_handlers.rs:

use crate::ui::command_executor;

// ============================================================================
// Command Mode Handlers
// ============================================================================

/// Handler for character input in Command mode - append to buffer
pub struct CommandCharHandler;

impl KeyHandler for CommandCharHandler {
    fn mode(&self) -> AppMode {
        AppMode::Command
    }
    
    fn keys(&self) -> Vec<KeyCode> {
        // Match all printable characters
        (32..=126u8)
            .map(|b| KeyCode::Char(b as char))
            .collect()
    }
    
    fn handle(&self, app: &mut App) -> Result<KeyResult> {
        // Note: This handler needs access to command_buffer
        // This requires refactoring - see note below
        Ok(KeyResult::Ignored)
    }
}

/// Handler for Enter in Command mode - execute command
pub struct CommandEnterHandler;

impl KeyHandler for CommandEnterHandler {
    fn mode(&self) -> AppMode {
        AppMode::Command
    }
    
    fn keys(&self) -> Vec<KeyCode> {
        vec![KeyCode::Enter]
    }
    
    fn handle(&self, app: &mut App) -> Result<KeyResult> {
        // Execute command and return to Reading mode
        // Note: Needs access to command_buffer
        app.mode = AppMode::Reading;
        Ok(KeyResult::Consumed)
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
    
    fn handle(&self, _app: &mut App) -> Result<KeyResult> {
        // Note: Needs access to command_buffer
        Ok(KeyResult::Consumed)
    }
}

/// Handler for Escape in Command mode - cancel
pub struct CommandEscapeHandler;

impl KeyHandler for CommandEscapeHandler {
    fn mode(&self) -> AppMode {
        AppMode::Command
    }
    
    fn keys(&self) -> Vec<KeyCode> {
        vec![KeyCode::Esc]
    }
    
    fn handle(&self, app: &mut App) -> Result<KeyResult> {
        app.mode = AppMode::Reading;
        app.clear_error();
        Ok(KeyResult::Consumed)
    }
    
    fn help_text(&self) -> &str {
        "Cancel command (Esc)"
    }
}

/// Add command mode handlers to registry
pub fn create_command_handlers(registry: &mut KeyHandlerRegistry) {
    registry.register(CommandEscapeHandler);
    // Note: Other handlers need command_buffer access - see Task 6
}
```

**Note:** Command mode handlers need access to `command_buffer` which lives in `TuiManager`, not `App`. This requires either:
1. Moving command_buffer to App
2. Adding a context parameter to KeyHandler::handle()
3. Keeping command mode handling in terminal.rs

For now, we keep command mode handling in terminal.rs - Reading mode handlers demonstrate the OCP pattern.

- [ ] **Step 2: Update terminal.rs to register command handlers**

```rust
// In terminal.rs, add to imports:
use crate::ui::key_handlers::create_command_handlers;

// In TuiManager::new(), after create_reading_handlers:
create_command_handlers(&mut key_registry);
```

- [ ] **Step 3: Verify and commit**

```bash
cargo check
git add src/ui/key_handlers.rs src/ui/terminal.rs
git commit -m "feat: Add Command mode key handlers (partial)

- CommandEscapeHandler implemented and working
- Other command handlers need command_buffer access (tracked separately)
- Reading mode handlers are fully functional"
```

---

## Task 6: Add Popup Mode Handlers

**Files:**
- Modify: `src/ui/key_handlers.rs`

- [ ] **Step 1: Add Popup mode handlers**

```rust
// Add to src/ui/key_handlers.rs:

// ============================================================================
// Popup Mode Handlers
// ============================================================================

/// Handler for Enter in Popup mode - confirm action
pub struct PopupConfirmHandler;

impl KeyHandler for PopupConfirmHandler {
    fn mode(&self) -> AppMode {
        AppMode::ConfigPopup
    }
    
    fn keys(&self) -> Vec<KeyCode> {
        vec![KeyCode::Enter]
    }
    
    fn handle(&self, app: &mut App) -> Result<KeyResult> {
        app.handle_popup_key('y')?;
        Ok(KeyResult::Consumed)
    }
    
    fn help_text(&self) -> &str {
        "Confirm (Enter)"
    }
}

/// Handler for Escape in Popup mode - dismiss
pub struct PopupDismissHandler;

impl KeyHandler for PopupDismissHandler {
    fn mode(&self) -> AppMode {
        AppMode::ConfigPopup
    }
    
    fn keys(&self) -> Vec<KeyCode> {
        vec![KeyCode::Esc, KeyCode::Char('n')]
    }
    
    fn handle(&self, app: &mut App) -> Result<KeyResult> {
        app.handle_popup_key('n')?;
        Ok(KeyResult::Consumed)
    }
    
    fn help_text(&self) -> &str {
        "Dismiss (Esc, n)"
    }
}

/// Add popup mode handlers to registry
pub fn create_popup_handlers(registry: &mut KeyHandlerRegistry) {
    registry.register(PopupConfirmHandler);
    registry.register(PopupDismissHandler);
}
```

- [ ] **Step 2: Update terminal.rs to register popup handlers**

```rust
// In terminal.rs, add to imports:
use crate::ui::key_handlers::create_popup_handlers;

// In TuiManager::new(), after create_command_handlers:
create_popup_handlers(&mut key_registry);
```

- [ ] **Step 3: Update handle_popup_key logic**

The popup handling in terminal.rs can now be simplified since the registry handles Enter and Esc:

```rust
// In terminal.rs, modify handle_popup_key or remove if using registry exclusively
```

- [ ] **Step 4: Verify and commit**

```bash
cargo test
git add src/ui/key_handlers.rs src/ui/terminal.rs
git commit -m "feat: Add Popup mode key handlers

- PopupConfirmHandler (Enter)
- PopupDismissHandler (Esc, n)
- Integrated into TuiManager key_registry"
```

---

## Task 7: Final Integration and Testing

**Files:**
- All modified files

- [ ] **Step 1: Run full test suite**

```bash
cargo test
cargo clippy -- -D warnings
cargo build --release
```

Expected: All tests pass, no clippy warnings, clean release build

- [ ] **Step 2: Manual verification**

Test key bindings work as expected:
- `j` or `Space` → next word
- `k` → previous word
- `]` → speed up
- `[` → speed down
- `p` → pause
- `:` → command mode
- `Esc` in command mode → back to reading

- [ ] **Step 3: Final commit**

```bash
git log --oneline -10
git status
git commit --allow-empty -m "feat: Complete KeyHandlerRegistry implementation

Implements OCP-compliant key handling:
- KeyHandler trait for extensible key bindings
- KeyHandlerRegistry for mode-aware dispatch
- Reading mode: j/k/[/]/Space/p handlers
- Command mode: Esc handler (partial - others need command_buffer)
- Popup mode: Enter/Esc/n handlers
- Removed handle_keypress() from App (SRP compliance)

Adding new key bindings now requires:
1. Create struct implementing KeyHandler
2. Register in appropriate create_*_handlers() function
3. No changes to terminal.rs or existing handlers needed"
```

- [ ] **Step 4: Push to remote**

```bash
git push origin master
```

---

## Architecture Verification

After completion, verify these SOLID improvements:

| Principle | Before | After |
|-----------|--------|-------|
| **OCP** | Hardcoded match in terminal.rs + app_impl.rs | Registry dispatch - add handlers without modifying existing code |
| **SRP** | App had state + key handling | App has state only, handlers have key logic |
| **DIP** | Terminal directly called app.handle_keypress() | Terminal depends on KeyHandlerRegistry trait |

---

## Testing Checklist

- [ ] Unit tests for KeyHandlerRegistry dispatch
- [ ] Unit tests for each handler type
- [ ] Integration test: key press → handler → app state change
- [ ] Manual test: All key bindings work in TUI
- [ ] No compiler warnings
- [ ] Clippy clean
- [ ] Documentation updated (if needed)

---

## Notes for Implementers

1. **Command mode limitation**: Full command mode handling requires command_buffer in App or context parameter. Current implementation keeps command_buffer in TuiManager and handles some keys directly.

2. **Mode transitions**: Mode changes (e.g., ':' to enter Command mode) are still handled in terminal.rs as they're not key bindings per se.

3. **Global keys**: Ctrl+C for exit remains a direct match in terminal.rs - this is intentional as it should never be overridden.

4. **Extending**: To add a new key binding:
   ```rust
   pub struct MyHandler;
   impl KeyHandler for MyHandler { ... }
   // In create_reading_handlers():
   registry.register(MyHandler);
   ```
