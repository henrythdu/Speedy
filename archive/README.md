# Archive: Obsolete Code (Post Architecture Pivot)

This directory contains code that is no longer part of the active codebase.

## Rationale

The project pivoted from a **REPL-based architecture** to a **pure TUI-based architecture** with a dual-engine graphics system (Ratatui + Kitty Graphics Protocol). This was documented in the design review on 2026-01-28.

## New Architecture

- **Before**: REPL loop (`speedy> @file.pdf`) → Launch TUI → Return to REPL
- **After**: TUI-only startup with command deck at bottom of screen

## Contents

- `repl/` - Complete REPL module (rustyline-based input, parsing, commands)
  - `input.rs` - Rustyline wrapper for line input
  - `parser.rs` - String parsing for @ and : commands  
  - `command.rs` - Command enum and AppEvent conversion

## Why These Are Obsolete

1. **REPL pattern doesn't fit graphics engine** - The new design uses a viewport overlay where Ratatui renders layout and a graphics engine renders pixel-perfect words. A REPL loop would be disruptive.

2. **Commands moved to TUI command deck** - The @file.pdf and @@ clipboard commands will be handled in a bottom panel within the TUI itself.

3. **rustyline dependency** - Will be removed from Cargo.toml as we no longer need line editing.

## Migration Notes

The functionality from `repl/command.rs` and `repl/parser.rs` needs to be reimplemented as:
- TUI keybindings in `ui/terminal.rs`
- Command deck widgets in the new UI architecture
- File loading via TUI input mode

## Date Archived
2026-01-28
