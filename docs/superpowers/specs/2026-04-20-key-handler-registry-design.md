# Key Handler Registry Design

**Date:** 2026-04-20
**Status:** Approved
**Scope:** OCP-compliant key handling + dead code cleanup

## Problem

Key handling is hardcoded across `terminal.rs` (KeyCode dispatch) and `app_impl.rs` (char dispatch). Adding new key bindings requires modifying these files directly, violating the Open/Closed Principle. Meanwhile, a failed refactor left ~8 dead files and dead code in active files.

The command registry (`src/ui/commands/`) already demonstrates a working OCP pattern — this design extends it to key handling.

## Architecture

### Event Flow (After)

```
terminal.rs::run_event_loop()
  → read KeyEvent
  → global keys (Ctrl+C) handled directly
  → key_registry.dispatch(key_code, app.mode, &mut app)
  → registry finds matching KeyHandler, executes it
  → unhandled keys: ignored
```

### Core Types

```rust
// src/ui/key_handler.rs

pub enum KeyResult {
    Consumed,
    Ignored,
}

pub trait KeyHandler: Send + Sync {
    fn mode(&self) -> AppMode;
    fn keys(&self) -> Vec<KeyCode>;
    fn handle(&self, app: &mut App) -> Result<KeyResult>;
    fn help_text(&self) -> &str;
}

pub struct KeyHandlerRegistry {
    handlers: Vec<Box<dyn KeyHandler>>,
}

impl KeyHandlerRegistry {
    pub fn dispatch(&self, key: KeyCode, mode: AppMode, app: &mut App)
        -> Option<Result<KeyResult>>
    {
        self.handlers.iter()
            .find(|h| h.mode() == mode && h.keys().contains(&key))
            .map(|h| h.handle(app))
    }

    pub fn handlers_for_mode(&self, mode: AppMode) -> Vec<&dyn KeyHandler> {
        self.handlers.iter()
            .filter(|h| h.mode() == mode)
            .map(|h| h.as_ref())
            .collect()
    }
}
```

### Design Decisions

| Decision | Rationale |
|----------|-----------|
| `keys()` returns `Vec<KeyCode>` | One handler can bind multiple keys (j + Space for next word) |
| `mode()` returns single mode | Each handler belongs to one mode |
| No `key` param in `handle()` | Handler knows its keys; if behavior diverges, split into two handlers |
| `KeyResult` is simple enum | Only Consumed/Ignored — mode changes happen through App state |
| Global keys stay in terminal.rs | Ctrl+C and similar should never be overridden by registry |

### Concrete Handlers

**Reading mode** (replaces `app_impl.rs::handle_keypress()`):

| Handler | Keys | Action |
|---------|------|--------|
| `NextWordHandler` | j, Space | `app.next_word()` |
| `PrevWordHandler` | k | `app.prev_word()` |
| `SpeedUpHandler` | ] | speed increase |
| `SpeedDownHandler` | [ | speed decrease |
| `HelpToggleHandler` | F1 | toggle help |

**Command mode** (replaces char matching in terminal.rs):

| Handler | Keys | Action |
|---------|------|--------|
| `CommandCharHandler` | any Char | buffer character to command input |
| `ExecuteCommandHandler` | Enter | execute command via command_executor |
| `CommandBackspaceHandler` | Backspace | delete last char from command buffer |
| `CommandEscapeHandler` | Esc | exit command mode |
| `CommandTabHandler` | Tab | autocomplete |

**Popup mode:**

| Handler | Keys | Action |
|---------|------|--------|
| `PopupConfirmHandler` | Enter | confirm popup action |
| `PopupDismissHandler` | Esc, 'n' | dismiss popup |
| `PopupNavigateHandler` | j/k or Up/Down | navigate popup options |

**Mode transitions** (currently hardcoded in terminal.rs):

| Handler | Keys | Mode | Action |
|---------|------|------|--------|
| `EnterCommandModeHandler` | : | Reading | switch to command mode |
| `EnterCommandModeFromPopupHandler` | : | Popup | switch to command mode |

### Integration

`terminal.rs` changes:

```rust
fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
    // Global: always exit on Ctrl+C
    if key_event.code == KeyCode::Char('c')
        && key_event.modifiers.contains(KeyModifiers::CONTROL)
    {
        self.app.set_running(false);
        return Ok(());
    }

    // Dispatch through registry
    match self.key_registry.dispatch(key_event.code, self.app.mode, &mut self.app) {
        Some(Ok(KeyResult::Consumed)) => {},
        Some(Ok(KeyResult::Ignored)) => {},
        Some(Err(e)) => return Err(e),
        None => {}, // key not handled
    }
    Ok(())
}
```

### File Changes Summary

**New files:**
- `src/ui/key_handler.rs` — KeyHandler trait, KeyHandlerRegistry, KeyResult
- `src/ui/key_handlers.rs` — All concrete handler implementations

**Modified files:**
- `src/ui/mod.rs` — Add key_handler, key_handlers modules
- `src/ui/terminal.rs` — Replace hardcoded dispatch with registry
- `src/app/app_impl.rs` — Remove `handle_keypress()`, `reading_state_mut()`

## Dead Code Cleanup

### Files to Delete

| File | Lines | Reason |
|------|-------|--------|
| `src/handlers/mod.rs` | ~5 | Never declared in main.rs |
| `src/ui/event_handler.rs` | ~485 | Dead architecture experiment |
| `src/ui/command_handler.rs` | ~60 | Superseded by `commands/handler.rs` |
| `src/ui/command_registry.rs` | ~80 | Superseded by `commands/mod.rs` |
| `src/ui/file_loader/mod.rs` | ~30 | Never imported |
| `src/ui/file_loader/loaders.rs` | ~100 | Part of dead module |
| `src/input/file_loader.rs` | ~50 | Only used by dead file_loader |
| `src/input/loader.rs` | ~30 | Only used by dead file_loader |

**Total removed: ~840 lines of dead code**

### Dead Code in Active Files

| File | What | Action |
|------|------|--------|
| `src/ui/command.rs` | Entire file (Command enum, parse_command, tokens_to_text) | Delete file + remove `pub mod command;` from mod.rs |
| `src/ui/commands/handler.rs` | Duplicate CommandHandler trait with different signature | Delete file |
| `src/app/app_impl.rs` | `reading_state_mut()` method | Remove method |
| `src/ui/commands/mod.rs` | `can_handle()`, `help_text()` on CommandRegistry | Remove unused methods |
| `src/ui/commands/handlers.rs` | Duplicated `tokens_to_text` in LoadFileHandler + LoadClipboardHandler | Extract to shared utility |

### DRY Fix

`tokens_to_text` is duplicated in two handlers. Extract to `src/ui/commands/utils.rs`:

```rust
pub fn tokens_to_text(tokens: &[&str]) -> String {
    tokens.join(" ")
}
```

Both `LoadFileHandler` and `LoadClipboardHandler` use this utility.

## Implementation Order

1. Create `key_handler.rs` with trait + registry
2. Create `key_handlers.rs` with Reading mode handlers
3. Wire registry into `terminal.rs` (Reading mode first)
4. Add Command mode handlers, wire in
5. Add Popup mode handlers, wire in
6. Remove `app_impl.rs::handle_keypress()`
7. Delete all dead files
8. Remove dead code from active files
9. Extract `tokens_to_text` utility
10. Run `cargo test`, `cargo clippy`, verify build

## Acceptance Criteria

- [ ] All key bindings work identically to before (no behavior changes)
- [ ] Adding a new key binding requires zero changes to terminal.rs or existing handlers
- [ ] All dead files deleted, no compiler warnings for dead code
- [ ] `cargo test` passes
- [ ] `cargo clippy` passes with no warnings
