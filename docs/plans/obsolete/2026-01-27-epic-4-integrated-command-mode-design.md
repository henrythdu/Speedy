# Epic 4: Integrated Command Mode & REPL

## Overview

Transform Speedy from separate terminal/TUI modes to integrated TUI with embedded command panel (like opencode).

## Problem Statement

**Current State:**
```
User types "@file.pdf" in terminal REPL
         ↓
TUI launches (full screen)
         ↓
User reads, presses 'q'
         ↓
Back to terminal REPL (context switch)
         ↓
Must type another command to continue
```

**Issues:**
- Jarring context switch between terminal and TUI
- Breaks reading flow when switching back to terminal
- No way to issue commands while viewing reading context
- REPL and TUI feel like separate applications

**Solution:**
Integrated command panel within TUI that:
- Stays visible but dimmed during reading
- Becomes interactive when user presses 'q' 
- Allows typing commands without leaving TUI
- Provides document info and help in same view

## Detailed Design

### Epic Scope

**3 Beads (Focused Implementation)**

### Bead 4-1: Command Mode State & Input Buffer

**Changes to App:**
```rust
// src/app/app.rs
pub struct App {
    mode: AppMode,
    reading_state: Option<ReadingState>,
    command_input: String,           // NEW: Buffer for typed commands
}

pub enum AppMode {
    Repl,      // OLD: Exits TUI → now called "Command" mode
    Reading,
    Paused,
    Command,   // NEW: Typing commands within TUI
}
```

**Key Behavior:**
- `Command` mode means "typing in command panel" (not exit TUI)
- TUI never exits until `AppMode::Quit`
- `command_input` buffers characters typed in command panel

**Acceptance:**
- `AppMode::Command` variant compiles and exists
- `command_input: String` field present in `App` struct
- Mode transitions work: Command ↔ Reading/Paused

### Bead 4-2: Character Input & Command Execution

**New Methods:**
```rust
impl App {
    pub fn handle_command_char(&mut self, c: char) -> bool {
        // Only accept input in Command mode
        if !matches!(self.mode, AppMode::Command) {
            return false;
        }
        
        match c {
            '\n' | '\r' => { self.execute_command(); true },  // Enter
            '\x08' => { self.command_input.pop(); true },    // Backspace
            '\x03' => { self.mode = AppMode::Quit; true },    // Ctrl+C
            _ if c.is_ascii() && !c.is_control() => {
                self.command_input.push(c); true
            }
            _ => false,
        }
    }
    
    pub fn execute_command(&mut self) {
        let input = self.command_input.trim();
        self.command_input.clear();
        
        if input.is_empty() { return; }
        
        let event = parse_repl_input(input);
        self.handle_event(event);
        
        // If command loaded document, mode becomes Reading (auto-switch)
    }
}
```

**Key Behavior:**
- Character-by-character building of command
- Enter executes command
- Backspace deletes character
- Ctrl+C quits app from any mode
- Reuses existing `parse_repl_input()` logic

**Acceptance:**
- Can type `@file.pdf` in command panel
- Enter loads document and switches to Reading mode
- Backspace works correctly
- Ctrl+C quits entire application

### Bead 4-3: Render Command Panel

**New Function:**
```rust
// src/ui/render.rs
pub fn render_command_line(
    input: &str,
    mode: AppMode,
) -> Paragraph<'static> {
    match mode {
        AppMode::Command => {
            let help = "Speedy | @@ clipboard | :h help | :q quit";
            Paragraph::new(vec![
                Line::from(help),
                Line::from(format!("> {}", input)),
            ]).style(Style::default().fg(colors::text()))
        }
        AppMode::Reading | AppMode::Paused => {
            Paragraph::new(vec![
                Line::from(Span::styled(
                    "Press 'q' for commands",
                    Style::default().fg(colors::dimmed()),
                )),
            ]).style(Style::default().fg(colors::dimmed()))
        }
        _ => Paragraph::new(""),
    }
}
```

**Layout:**
```
┌────────────────────────────────────────┐
│                                        │  Main area (reading)
│                                        │  - Dimmed in Command mode
│                                        │
├────────────────────────────────────────┤
│Speedy | @@ | :h | :q                │  Line 1: Help text
│> @file.pdf                            │  Line 2: Command input with cursor
└────────────────────────────────────────┘
```

**Key Behavior:**
- 2-line panel at bottom of screen
- Line 1: Help text showing available commands
- Line 2: `>` prompt + typed input + cursor position
- Dimmed when in Reading/Paused mode
- Fully interactive (bright) when in Command mode

**Acceptance:**
- Command panel renders at bottom with 2 lines
- Help text shows on line 1
- Typed input shows on line 2 with `>` prompt
- Dimming changes based on mode

## Implementation Order

**Cannot do in parallel with Epic 3** - Must be sequential:
1. Complete Epic 3 (visual fixes) first
2. Epic 4 depends on stable visual foundation

**Within Epic 4:**
1. Add Command mode enum and input buffer (simplest)
2. Implement character handling and command execution (medium complexity)
3. Create command panel renderer and integrate into layout (most complex)

## Testing Checklist

- [ ] Can start in Command mode with no document
- [ ] Type `@file.pdf` + Enter → loads and switches to Reading
- [ ] Read, press 'q' → switches to Command mode (no terminal exit)
- [ ] Command panel shows help text on line 1
- [ ] Can type another command (e.g., `@other.pdf`)
- [ ] Backspace works in command input
- [ ] Ctrl+C quits app from Command mode
- [ ] Ctrl+C quits from Reading/Paused mode
- [ ] All existing keybindings (j/k/space) still work
- [ ] Visual transition between modes is smooth

## Dependencies

**Depends on:** Epic 3 (visual fixes)
- Need stable OVP anchoring before changing modes
- Need proper dimming for command panel visibility

**No external dependencies:** Uses existing crossterm input handling

## Interface Changes

**Breaking Changes:**
- `main.rs`: No longer exits TUI on 'q', stays in Command mode
- `AppMode::Repl` renamed/clarified as `AppMode::Command` (semantic change)
- TUI runs persistently (only Quit exits)

**New Public Methods:**
- `App::handle_command_char()` - Character input handler
- `App::execute_command()` - Command execution
- `render_command_line()` - Panel renderer

## Risks

**Medium Risk** - Major architectural change

**Mitigations:**
- Keep command parsing simple (reuse existing logic)
- Test mode transitions thoroughly
- Verify no resource leaks from persistent TUI
- Ensure cleanup on Ctrl+C (disable_raw_mode, leave_alternate_screen)

## Backwards Compatibility

**Breaking for UX:**
- Users can no longer exit to terminal REPL (must use :q)
- This is intentional improvement (integrated experience)

**Code Compatibility:**
- All reading mode keybindings preserved
- File loading logic unchanged
- Event handling extended, not replaced

## PRD Alignment

- **PRD 7.1 (REPL Modes)**: Transforms from separate terminal to integrated panel
- **PRD 7.2 (Reading Controls)**: 'q' now switches to command mode (not exit)
- **PRD 2.3 (Discovery)**: Commands always visible via help text
- **PRD 4.2 (Gutter)**: Command panel provides spatial context

## Future Extensions (Not in Epic 4)

- Command history (up/down arrows)
- Tab completion for file paths
- Command autocomplete for :commands
- Syntax highlighting in command input
- Error messages in command panel
- Multi-line command support

---

## Summary

Epic 4 transforms Speedy's UX by embedding REPL functionality within TUI:

1. **Command Mode** - Type commands without leaving reading context
2. **Integrated Panel** - Always-visible help and command input
3. **Smooth Transitions** - No jarring terminal context switches

This is a **major UX improvement** that removes friction and makes Speedy feel like a cohesive application rather than two separate tools.