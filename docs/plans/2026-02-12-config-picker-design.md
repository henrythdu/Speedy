# Design Doc: Config Picker Popup

**Version:** 1.0
**Date:** 2026-02-12
**Status:** Ready for implementation

---

## 1. Overview

A popup config picker triggered by `Ctrl+P` that allows users to quickly view and modify persistent settings without leaving the reading experience. Inspired by OpenCode's `Ctrl+P` command palette.

**Key Features:**
- Inline editors with arrow-key cycling
- Live theme preview
- Instant save with `Enter`
- Discard with `Esc`

---

## 2. UI Structure

```
┌─────────────────────────────────────────┐
│                                         │
│                                         │
│          [current word visible]         │  ← Reader zone dimmed
│                                         │
│                                         │
├─────────────────────────────────────────┤
│ ┌─────────────────────────────────────┐ │
│ │ SETTINGS                       ✕ Esc│ │  ← Header
│ ├─────────────────────────────────────┤ │
│ │ Default WPM    ◀  [ 300 ]  ▶       │ │  ← Row 0
│ │ Theme          ◀  [ tokyo-night ] ▶ │ │  ← Row 1
│ │ Ghost Words    ◀  [ off ]  ▶       │ │  ← Row 2
│ ├─────────────────────────────────────┤ │
│ │ ← → change  •  Enter save  •  Esc  │ │  ← Footer
│ └─────────────────────────────────────┘ │
│ ▌command▌                               │
└─────────────────────────────────────────┘
```

**UI Legend:** '▌command▌' represents the command input area at bottom of screen with accent-colored left border.

**Dimensions:**
- Width: 80% of terminal, centered
- Height: 7 lines (header + 3 rows + footer)
- Position: Above command section (reuse `calculate_popup_area` from autocomplete)

---

## 3. Interaction Model

### 3.1 Key Bindings

| Key | Action |
|-----|--------|
| `Ctrl+P` | Toggle popup open/close |
| `↑` / `↓` | Move selection between rows |
| `←` / `→` | Cycle value on selected row |
| `Enter` | Save changes, close popup |
| `Esc` | Discard changes, close popup |

### 3.2 Value Cycling

| Config | `←` | `→` | Range/Values |
|--------|-----|-----|--------------|
| Default WPM | -50 | +50 | 50-1000 |
| Theme | prev theme | next theme | tokyo-night, dracula, gruvbox, catppuccin-mocha, nord, light (wrap) |
| Ghost Words | toggle | toggle | on ↔ off |

**Feature Descriptions:**
- **Ghost Words:** When enabled, shows the previous and next words faintly at 15% opacity to provide context without distraction.

*Footnote: WPM clamps at boundaries - no effect when at 50 or 1000*

---

## 4. State Management

```rust
pub struct ConfigPopupState {
    pub is_open: bool,
    pub selected_row: usize,        // 0=WPM, 1=Theme, 2=Ghost
    pub temp_default_wpm: u32,      // Working copy
    pub temp_theme_index: usize,    // Working copy (index into THEMES array)
    pub temp_ghost_words: bool,     // Working copy
    original_theme_index: usize,    // For revert on Esc
}

const THEMES: &[&str] = &[
    "tokyo-night",
    "dracula", 
    "gruvbox",
    "catppuccin-mocha",
    "nord",
    "light",
];
```

---

## 5. Data Flow

### 5.1 Config File

**Location:** `~/.config/speedy/config.toml`

**Updated Structure:**
```toml
theme = "tokyo-night"
default_wpm = 300           # NEW FIELD - starting WPM for new sessions
ghost_words = false

[timing]
wpm = 300                   # Runtime WPM (adjustable via [ ])
# ... other timing settings
```

**Note:** `default_wpm` is the initial WPM for new reading sessions. `timing.wpm` is the current runtime WPM, adjustable via `[ ]` keys during reading.

### 5.2 Save Flow (Enter)

1. Update `Config` struct with `temp_*` values
2. Write to `config.toml` (preserve existing structure)
3. Apply theme immediately to running app
4. Close popup

**Error Handling:** If config write fails (disk full, permissions), show error in popup footer and keep popup open for user to retry.

### 5.3 Discard Flow (Esc)

1. Revert theme to `original_theme_index`
2. Close popup without writing to file

---

## 6. Live Preview

- Theme changes apply **immediately** to the reader zone background
- User sees the effect before committing
- Ghost words toggle shows current state (actual toggle applies on save)
- WPM changes are preview only (no visual effect until saved)

---

## 7. Architecture

### 7.1 New Module

```
src/ui/config_popup/
├── mod.rs          # Public interface, THEMES constant
├── state.rs        # ConfigPopupState, navigation/cycling logic
├── render.rs       # Popup rendering (ratatui widgets)
└── handler.rs      # Key event handling
```

### 7.2 Integration Points

| File | Change |
|------|--------|
| `src/config/mod.rs` | Add `default_wpm: u32` field |
| `src/app/mod.rs` | Add `config_popup: ConfigPopupState` to App |
| `src/ui/terminal.rs` | Handle `Ctrl+P`, render popup overlay after main UI |
| `src/ui/event_handler.rs` | Route popup keys when popup is open |

### 7.3 Key Methods

```rust
impl ConfigPopupState {
    pub fn open(config: &Config) -> Self;
    pub fn close(&mut self);
    pub fn move_up(&mut self);
    pub fn move_down(&mut self);
    pub fn cycle_left(&mut self);
    pub fn cycle_right(&mut self);
    pub fn apply_to_config(self, config: &mut Config) -> Result<()>;
    pub fn revert_theme(&self, config: &mut Config);
    pub fn current_theme(&self) -> &str;
}
```

---

## 8. Implementation Checklist

### Phase 1: Core State & Config
- [ ] Add `default_wpm: u32` to `Config` struct
- [ ] Create `src/ui/config_popup/mod.rs` with `THEMES` constant
- [ ] Create `src/ui/config_popup/state.rs` with `ConfigPopupState`
- [ ] Add `config_popup: ConfigPopupState` to `App`

### Phase 2: Rendering
- [ ] Create `src/ui/config_popup/render.rs`
- [ ] Render popup with Clear overlay
- [ ] Render header, rows, footer
- [ ] Style selected row with accent color

### Phase 3: Input Handling
- [ ] Create `src/ui/config_popup/handler.rs`
- [ ] Handle `Ctrl+P` to toggle popup
- [ ] Handle `↑↓←→` navigation and cycling
- [ ] Handle `Enter` to save
- [ ] Handle `Esc` to discard

### Phase 4: Persistence
- [ ] Implement `apply_to_config()` to write to config.toml
- [ ] Implement `revert_theme()` for Esc flow
- [ ] Ensure config file preserves comments

### Phase 5: Testing
- [ ] Unit tests for ConfigPopupState cycling logic
- [ ] Unit tests for value bounds (WPM 50-1000)
- [ ] Manual test: live theme preview
- [ ] Manual test: save/discard flows

---

## 9. Future Enhancements

- Add more config options (e.g., timing multipliers for advanced users)
- Search/filter for theme selection if list grows
- Config validation feedback in popup
