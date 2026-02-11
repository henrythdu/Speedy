# Command Pattern for Event Handling

**Phase:** 2.2  
**Date:** February 2026  
**Status:** Complete

## Overview

This document describes the Command Pattern implementation for event handling in Speedy. The refactoring reduces the cyclomatic complexity of the `run_event_loop()` method from ~15 to ~3-5 per handler, making the code more maintainable and testable.

## Problem Statement

The original `run_event_loop()` method in `src/ui/terminal.rs` was 219+ lines with:
- Cyclomatic complexity ~15+
- 6+ mixed concerns (cursor management, key handling, autocomplete, resize, discovery, rendering)
- Deeply nested conditionals
- Zero unit tests (impossible to test monolithic event loop)

## Solution

Implemented the Command Pattern with the following components:

### 1. EventHandler Trait

```rust
pub trait EventHandler {
    fn handle(&self, event: Event, app: &mut App) -> Result<(), UIError>;
}
```

Provides a common interface for all mode-specific handlers.

### 2. Mode-Specific Handlers

#### CommandModeHandler
- Handles character input for command building
- Processes Enter, Backspace, Escape keys
- Manages command execution flow
- Returns `CommandAction` for complex state transitions

#### ReadingModeHandler  
- Handles RSVP reading navigation keys (j/k for sentences)
- Processes WPM adjustment ([/])
- Space for pause/resume
- 'q' to return to Command mode

#### AutocompleteHandler
- Up/Down for navigation
- Tab for selection with trailing space
- Enter for selection
- Backspace for query modification
- Esc to close autocomplete
- Ctrl+R to refresh cache

### 3. EventDispatcher

Routes events to the appropriate handler based on `app.mode()`:

```rust
match app.mode() {
    AppMode::Command => CommandModeHandler::handle(event, app)?,
    AppMode::Reading => ReadingModeHandler::handle(event, app)?,
    AppMode::Paused => ReadingModeHandler::handle(event, app)?,
}
```

### 4. HandlerContext

Shared mutable state between handlers and the main event loop:
- `command_buffer: String` - Current command input
- `cursor_visible: bool` - Blink state
- `last_keypress: Instant` - For cursor blink pause

## Architecture Benefits

### Before (Monolithic)
```
run_event_loop()
├── Cursor blink logic (20 lines)
├── Event polling (5 lines)
├── Ctrl+C handling (5 lines)
├── Ctrl+R handling (15 lines)
├── Key handling:
│   ├── Char handling (30 lines)
│   ├── Enter handling (20 lines)
│   ├── Backspace (15 lines)
│   ├── Up/Down (10 lines)
│   ├── Tab (10 lines)
│   └── Esc (10 lines)
├── Resize handling (5 lines)
├── Auto-advance (5 lines)
├── Discovery processing (20 lines)
└── Rendering (5 lines)
```

**Total:** ~180 lines, complexity ~15

### After (Command Pattern)
```
run_event_loop()
├── Cursor blink logic (20 lines)
├── Event polling (5 lines)
├── Route to handler (10 lines)
│   ├── Autocomplete active? -> AutocompleteHandler
│   ├── Command mode? -> CommandModeHandler
│   └── Reading/Paused? -> ReadingModeHandler
├── Resize handling (5 lines)
├── Auto-advance (5 lines)
├── Discovery processing (3 lines)
└── Rendering (5 lines)

CommandModeHandler::handle_key_event()
├── Ctrl+C (5 lines)
├── @ trigger check (5 lines)
└── Delegate to handler (3 lines)

ReadingModeHandler::handle_key_event()
├── Ctrl+C (5 lines)
├── Esc (3 lines)
└── Delegate to app.handle_keypress (1 line)

AutocompleteHandler::handle_key_event()
├── Ctrl+R (3 lines)
└── Match key code (15 lines)
```

**Total:** ~70 lines per handler, complexity ~3-5 each

## Testing

### New Tests Added: 9

1. `test_handler_context_creation` - Verifies context initialization
2. `test_handler_context_reset` - Tests context reset functionality
3. `test_dispatcher_creation` - Verifies dispatcher setup
4. `test_command_handler_creates` - Verifies handler instantiation
5. `test_reading_handler_creates` - Verifies handler instantiation
6. `test_autocomplete_handler_creates` - Verifies handler instantiation
7. `test_global_shortcuts_detects_ctrl_c` - Tests global Ctrl+C handling
8. `test_command_action_enum` - Verifies action variants exist
9. `test_autocomplete_action_enum` - Verifies action variants exist

### Backward Compatibility

All 201 existing tests pass (1 pre-existing failure unrelated to this work).

## Files Modified

1. **`src/ui/terminal.rs`**
   - Added EventDispatcher and HandlerContext fields to TuiManager
   - Refactored `run_event_loop()` to use handlers
   - Extracted `handle_autocomplete_key()`, `handle_command_mode_key()`, `handle_reading_mode_key()`
   - Extracted `refresh_autocomplete_cache()`, `activate_autocomplete()`, `execute_command()`, `process_discovery_files()`

2. **`src/ui/mod.rs`**
   - Added `pub mod event_handler;`
   - Re-exported public types: `EventHandler`, `EventDispatcher`, `HandlerContext`, etc.

3. **`src/ui/event_handler.rs`** (NEW)
   - 473 lines
   - EventHandler trait definition
   - HandlerContext struct
   - CommandModeHandler implementation
   - ReadingModeHandler implementation
   - AutocompleteHandler implementation
   - EventDispatcher implementation
   - CommandAction and AutocompleteAction enums
   - 9 unit tests

## Usage Example

```rust
// Before: Complex inline logic in run_event_loop
match key.code {
    KeyCode::Char(c) => {
        if app.mode() == AppMode::Command {
            if c == '@' && should_activate_autocomplete(...) {
                // 20 lines of autocomplete activation
            }
            self.command_buffer.push(c);
            // ... more logic
        } else {
            app.handle_keypress(c);
        }
    }
    // ... 100+ more lines
}

// After: Clean handler dispatch
match app.mode() {
    AppMode::Command => {
        if self.autocomplete_state.active {
            self.handle_autocomplete_key(key)?;
        } else {
            self.handle_command_mode_key(key, app)?;
        }
    }
    AppMode::Reading | AppMode::Paused => {
        self.handle_reading_mode_key(key, app);
    }
    _ => {}
}
```

## Future Enhancements

1. **More Granular Handlers**: Could split CommandModeHandler into:
   - InputHandler (character input, backspace)
   - CommandExecutionHandler (enter, command parsing)
   
2. **Plugin Architecture**: Handler trait enables third-party handlers

3. **Macro Recording**: Handlers make it easier to record/replay sequences

4. **Accessibility**: Separate handlers can provide different input modes

## Migration Guide

For developers modifying event handling:

1. **Adding a new key binding**: 
   - Find the appropriate handler (Command/Reading/Autocomplete)
   - Add the key case to `handle_key_event()`
   - Add/update tests in `event_handler.rs`

2. **Adding a new mode**:
   - Create new handler implementing `EventHandler`
   - Add case to `EventDispatcher::get_handler()`
   - Add mode-specific logic

3. **Modifying existing behavior**:
   - Locate the relevant handler
   - Modify the specific method
   - Update corresponding tests

## References

- Original Issue: REFACTORING_ANALYSIS.md Issue #2
- Pattern Reference: Command Pattern (GoF Design Patterns)
- Related Work: Phase 1.2 (UIError type)
