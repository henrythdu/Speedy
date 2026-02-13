# Config Picker Popup Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a `Ctrl+P` config picker popup for editing default WPM, theme, and ghost words with live preview.

**Architecture:** New `src/ui/config_popup/` module with state, render, and handler components. Reuses popup positioning from autocomplete. Adds `default_wpm` field to Config struct.

**Tech Stack:** Rust, Ratatui, existing Config system

---

## Task 1: Add `default_wpm` to Config (~15min)

**Files:**
- Modify: `src/config/mod.rs`
- Test: `src/config/mod.rs` (inline tests)

**Step 1: Write the failing test**

Add to `src/config/mod.rs` tests:

```rust
#[test]
fn test_default_wpm_field() {
    let config = Config::default();
    assert!(config.default_wpm >= 50);
    assert!(config.default_wpm <= 1000);
}

#[test]
fn test_default_wpm_clamped() {
    let mut config = Config::default();
    config.default_wpm = 25; // Below min
    config.validate();
    assert_eq!(config.default_wpm, 50);
    
    config.default_wpm = 2000; // Above max
    config.validate();
    assert_eq!(config.default_wpm, 1000);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_default_wpm --no-fail-fast`
Expected: FAIL - field does not exist

**Step 3: Add `default_wpm` field to Config struct**

In `src/config/mod.rs`, add to `Config` struct:

```rust
pub struct Config {
    pub theme: String,
    pub default_wpm: u32,  // NEW: Starting WPM for new sessions
    pub ghost_words: bool,
    pub timing: TimingConfig,
}
```

Update `Default` impl:

```rust
impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "tokyo-night".to_string(),
            default_wpm: 300,  // NEW
            ghost_words: false,
            timing: TimingConfig::default(),
        }
    }
}
```

Add validation in `Config::validate()`:

```rust
pub fn validate(&mut self) {
    self.default_wpm = self.default_wpm.clamp(50, 1000);
    self.timing.validate();
}
```

Update `Config::load()` to read `default_wpm`:

```rust
pub fn load() -> Self {
    // ... existing code ...
    let default_wpm = config_file
        .get::<u32>("default_wpm")
        .unwrap_or(300);
    
    let mut config = Self {
        theme,
        default_wpm,  // NEW
        ghost_words,
        timing,
    };
    config.validate();
    config
}
```

Update `Config::save()` to write `default_wpm`:

```rust
pub fn save(&self) -> Result<()> {
    let content = format!(
        r#"theme = "{}"
default_wpm = {}
ghost_words = {}

[timing]
wpm = {}
# ... rest of timing config ...
"#,
        self.theme, self.default_wpm, self.ghost_words, self.timing.wpm
    );
    // ... write to file ...
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test test_default_wpm --no-fail-fast`
Expected: PASS

**Step 5: Commit**

```bash
git add src/config/mod.rs
git commit -m "feat(config): add default_wpm field for config picker"
```

**Acceptance:** Config has default_wpm field, tests pass

---

## Task 2: Create Config Popup Module Structure (~30min)

**Files:**
- Create: `src/ui/config_popup/mod.rs`
- Create: `src/ui/config_popup/state.rs`
- Modify: `src/ui/mod.rs`

**Step 1: Write failing tests for state**

Add to a new test file or inline in `src/ui/config_popup/state.rs` (will create module first, then add tests):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cycle_wpm_bounds() {
        let mut state = ConfigPopupState::new();
        state.temp_default_wpm = 75;
        state.cycle_left();
        assert_eq!(state.temp_default_wpm, 50);
        state.cycle_left(); // Should stay at 50
        assert_eq!(state.temp_default_wpm, 50);
        
        state.temp_default_wpm = 975;
        state.cycle_right();
        assert_eq!(state.temp_default_wpm, 1000);
        state.cycle_right(); // Should stay at 1000
        assert_eq!(state.temp_default_wpm, 1000);
    }

    #[test]
    fn test_cycle_theme_wraps() {
        let mut state = ConfigPopupState::new();
        state.temp_theme_index = 0;
        state.cycle_left();
        assert_eq!(state.temp_theme_index, THEMES.len() - 1);
        
        state.temp_theme_index = THEMES.len() - 1;
        state.cycle_right();
        assert_eq!(state.temp_theme_index, 0);
    }

    #[test]
    fn test_row_navigation_bounds() {
        let mut state = ConfigPopupState::new();
        state.move_up(); // Should stay at 0
        assert_eq!(state.selected_row, 0);
        
        state.move_down();
        state.move_down();
        assert_eq!(state.selected_row, 2);
        
        state.move_down(); // Should stay at 2
        assert_eq!(state.selected_row, 2);
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test config_popup::state --no-fail-fast`
Expected: FAIL - module does not exist

**Step 3: Create module directory and mod.rs**

Create `src/ui/config_popup/mod.rs`:

```rust
//! Config picker popup module
//!
//! Provides a Ctrl+P popup for editing default WPM, theme, and ghost words.
//! Uses inline editors with arrow-key cycling and live theme preview.

mod state;

pub use state::ConfigPopupState;

/// Available theme names in order
pub const THEMES: &[&str] = &[
    "tokyo-night",
    "dracula",
    "gruvbox",
    "catppuccin",
    "nord",
    "light",
];

/// Get theme index by name, defaults to 0 if not found
pub fn theme_index(name: &str) -> usize {
    THEMES.iter().position(|&t| t == name).unwrap_or(0)
}
```

**Step 4: Create state.rs with ConfigPopupState**

Create `src/ui/config_popup/state.rs`:

```rust
use crate::config::Config;
use super::{THEMES, theme_index};

/// State for the config picker popup
pub struct ConfigPopupState {
    pub is_open: bool,
    pub selected_row: usize,
    pub temp_default_wpm: u32,
    pub temp_theme_index: usize,
    pub temp_ghost_words: bool,
    original_theme_index: usize,
}

impl ConfigPopupState {
    /// Create a new closed popup state
    pub fn new() -> Self {
        Self {
            is_open: false,
            selected_row: 0,
            temp_default_wpm: 300,
            temp_theme_index: 0,
            temp_ghost_words: false,
            original_theme_index: 0,
        }
    }

    /// Open popup with current config values
    pub fn open(&mut self, config: &Config) {
        self.is_open = true;
        self.selected_row = 0;
        self.temp_default_wpm = config.default_wpm;
        self.temp_theme_index = theme_index(&config.theme);
        self.temp_ghost_words = config.ghost_words;
        self.original_theme_index = self.temp_theme_index;
    }

    /// Close popup
    pub fn close(&mut self) {
        self.is_open = false;
    }

    /// Move selection up
    pub fn move_up(&mut self) {
        if self.selected_row > 0 {
            self.selected_row -= 1;
        }
    }

    /// Move selection down
    pub fn move_down(&mut self) {
        if self.selected_row < 2 {
            self.selected_row += 1;
        }
    }

    /// Cycle current row value left (decrease/prev)
    pub fn cycle_left(&mut self) {
        match self.selected_row {
            0 => self.temp_default_wpm = self.temp_default_wpm.saturating_sub(50).max(50),
            1 => self.temp_theme_index = if self.temp_theme_index == 0 { THEMES.len() - 1 } else { self.temp_theme_index - 1 },
            2 => self.temp_ghost_words = !self.temp_ghost_words,
            _ => {}
        }
    }

    /// Cycle current row value right (increase/next)
    pub fn cycle_right(&mut self) {
        match self.selected_row {
            0 => self.temp_default_wpm = (self.temp_default_wpm + 50).min(1000),
            1 => self.temp_theme_index = (self.temp_theme_index + 1) % THEMES.len(),
            2 => self.temp_ghost_words = !self.temp_ghost_words,
            _ => {}
        }
    }

    /// Apply changes to config and save
    pub fn apply_to_config(&self, config: &mut Config) -> std::io::Result<()> {
        config.default_wpm = self.temp_default_wpm;
        config.theme = THEMES[self.temp_theme_index].to_string();
        config.ghost_words = self.temp_ghost_words;
        config.save()
    }

    /// Revert theme to original (for Esc flow)
    pub fn original_theme(&self) -> &'static str {
        THEMES[self.original_theme_index]
    }

    /// Get current temp theme name
    pub fn current_theme(&self) -> &'static str {
        THEMES[self.temp_theme_index]
    }
}

impl Default for ConfigPopupState {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 5: Run tests to verify they pass**

Run: `cargo test config_popup::state --no-fail-fast`
Expected: PASS

**Step 6: Export module from ui/mod.rs**

In `src/ui/mod.rs`, add:

```rust
pub mod config_popup;
```

**Step 7: Commit**

```bash
git add src/ui/config_popup/ src/ui/mod.rs
git commit -m "feat(ui): add config_popup module with state management"
```

**Acceptance:** config_popup module exists, state tests pass

---

## Task 3: Add ConfigPopupState to App (~10min)

**Files:**
- Modify: `src/app/mod.rs`

**Step 1: Add config_popup field to App**

In `src/app/mod.rs`:

```rust
use crate::ui::config_popup::ConfigPopupState;

pub struct App {
    // ... existing fields ...
    pub config_popup: ConfigPopupState,
}
```

**Step 2: Initialize in App::new()**

```rust
impl App {
    pub fn new(config: Config) -> Self {
        Self {
            // ... existing fields ...
            config_popup: ConfigPopupState::new(),
        }
    }
}
```

**Step 3: Build and verify**

Run: `cargo build`
Expected: Success

**Step 4: Commit**

```bash
git add src/app/mod.rs
git commit -m "feat(app): add config_popup state to App"
```

**Acceptance:** App compiles with config_popup field

---

## Task 4: Create Config Popup Renderer (~20min)

**Files:**
- Create: `src/ui/config_popup/render.rs`
- Modify: `src/ui/config_popup/mod.rs`

**Step 0: Write test for render_row styling**

Add to `src/ui/config_popup/render.rs` (after implementation):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;

    #[test]
    fn test_render_row_selected_style() {
        // Test that selected row uses accent background
        // This is a visual test - verify the style is correct
        let style = Style::default().fg(Color::Black).bg(colors::accent());
        assert!(style.bg.is_some());
    }

    #[test]
    fn test_calculate_popup_area() {
        let command_area = Rect::new(0, 20, 80, 3);
        let popup = calculate_popup_area(command_area);
        
        // Popup should be above command area
        assert!(popup.y < command_area.y);
        // Popup should be centered
        assert!(popup.x > 0);
        // Popup should have reasonable width
        assert!(popup.width >= 40);
    }
}
```

**Step 1: Create render.rs**

Create `src/ui/config_popup/render.rs`:

```rust
use super::{ConfigPopupState, THEMES};
use crate::ui::theme::colors;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

/// Render the config popup overlay
pub fn render_config_popup(
    frame: &mut Frame,
    state: &ConfigPopupState,
    command_area: Rect,
) {
    if !state.is_open {
        return;
    }

    let popup_area = calculate_popup_area(command_area);
    
    // Clear the popup area
    frame.render_widget(Clear, popup_area);
    
    // Render popup
    render_popup_content(frame, state, popup_area);
}

/// Calculate popup area above command section
fn calculate_popup_area(command_area: Rect) -> Rect {
    const POPUP_HEIGHT: u16 = 7;
    const POPUP_WIDTH_PERCENT: u16 = 80;
    
    let popup_width = (command_area.width as u16 * POPUP_WIDTH_PERCENT / 100)
        .max(40);
    let popup_x = command_area.x + (command_area.width - popup_width) / 2;
    let popup_y = command_area.y.saturating_sub(POPUP_HEIGHT);
    
    Rect::new(popup_x, popup_y, popup_width, POPUP_HEIGHT)
}

/// Render popup content
fn render_popup_content(frame: &mut Frame, state: &ConfigPopupState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Header
            Constraint::Length(1),  // Row 0: WPM
            Constraint::Length(1),  // Row 1: Theme
            Constraint::Length(1),  // Row 2: Ghost
            Constraint::Length(1),  // Spacer
            Constraint::Length(1),  // Footer
        ])
        .split(area);
    
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(colors::text()))
        .style(Style::default().bg(colors::surface()));
    
    // Header
    let header = Paragraph::new(Span::styled(
        " SETTINGS                                              ✕ Esc",
        Style::default().fg(colors::text()).add_modifier(Modifier::BOLD),
    ))
    .block(block.clone())
    .alignment(Alignment::Left);
    frame.render_widget(header, chunks[0]);
    
    // Rows
    render_row(frame, chunks[1], "Default WPM", &format!("{}", state.temp_default_wpm), state.selected_row == 0);
    render_row(frame, chunks[2], "Theme", THEMES[state.temp_theme_index], state.selected_row == 1);
    render_row(frame, chunks[3], "Ghost Words", if state.temp_ghost_words { "on" } else { "off" }, state.selected_row == 2);
    
    // Footer
    let footer = Paragraph::new(" ← → change  •  Enter save  •  Esc discard ")
        .alignment(Alignment::Center)
        .style(Style::default().fg(colors::text()).add_modifier(Modifier::DIM));
    frame.render_widget(footer, chunks[5]);
}

/// Render a single config row
fn render_row(frame: &mut Frame, area: Rect, label: &str, value: &str, selected: bool) {
    let style = if selected {
        Style::default().fg(Color::Black).bg(colors::accent())
    } else {
        Style::default().fg(colors::text()).bg(colors::surface())
    };
    
    let text = format!(" {:<14} ◀  [ {:^12} ]  ▶", label, value);
    let paragraph = Paragraph::new(text).style(style);
    frame.render_widget(paragraph, area);
}
```

**Step 2: Export render from mod.rs**

In `src/ui/config_popup/mod.rs`, add:

```rust
mod render;

pub use render::render_config_popup;
```

**Step 3: Build and verify**

Run: `cargo build`
Expected: Success

**Step 4: Commit**

```bash
git add src/ui/config_popup/
git commit -m "feat(ui): add config popup renderer"
```

**Acceptance:** Popup renders correctly, selected row highlighted

---

## Task 5: Integrate Popup Rendering in Terminal (~10min)

**Files:**
- Modify: `src/ui/terminal.rs`

**Step 1: Import render_config_popup**

At top of `src/ui/terminal.rs`:

```rust
use crate::ui::config_popup::render_config_popup;
```

**Step 2: Render popup after main UI**

In the main render function, after rendering command area:

```rust
// Render config popup overlay (if open)
render_config_popup(frame, &app.config_popup, command_area);
```

**Step 3: Build and verify**

Run: `cargo build`
Expected: Success

**Step 4: Commit**

```bash
git add src/ui/terminal.rs
git commit -m "feat(ui): integrate config popup rendering"
```

**Acceptance:** Popup appears above command area

---

## Task 6: Add Key Event Handling (~25min)

**Files:**
- Create: `src/ui/config_popup/handler.rs`
- Modify: `src/ui/config_popup/mod.rs`
- Modify: `src/ui/terminal.rs` or `src/ui/event_handler.rs`

**Step 0: Write tests for handle_popup_key key routing**

Add to `src/ui/config_popup/handler.rs` (after implementation):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    fn create_test_app() -> App {
        App::new(Config::default())
    }

    #[test]
    fn test_ctrl_p_opens_popup() {
        let mut app = create_test_app();
        assert!(!app.config_popup.is_open);
        
        let consumed = handle_popup_key(KeyCode::Char('p'), KeyModifiers::CONTROL, &mut app);
        
        assert!(consumed);
        assert!(app.config_popup.is_open);
    }

    #[test]
    fn test_esc_closes_popup() {
        let mut app = create_test_app();
        app.config_popup.open(&app.config);
        
        let consumed = handle_popup_key(KeyCode::Esc, KeyModifiers::empty(), &mut app);
        
        assert!(consumed);
        assert!(!app.config_popup.is_open);
    }

    #[test]
    fn test_up_down_navigation() {
        let mut app = create_test_app();
        app.config_popup.open(&app.config);
        
        handle_popup_key(KeyCode::Down, KeyModifiers::empty(), &mut app);
        assert_eq!(app.config_popup.selected_row, 1);
        
        handle_popup_key(KeyCode::Up, KeyModifiers::empty(), &mut app);
        assert_eq!(app.config_popup.selected_row, 0);
    }

    #[test]
    fn test_left_right_cycles_values() {
        let mut app = create_test_app();
        app.config_popup.open(&app.config);
        
        let initial_wpm = app.config_popup.temp_default_wpm;
        handle_popup_key(KeyCode::Right, KeyModifiers::empty(), &mut app);
        assert_eq!(app.config_popup.temp_default_wpm, initial_wpm + 50);
        
        handle_popup_key(KeyCode::Left, KeyModifiers::empty(), &mut app);
        assert_eq!(app.config_popup.temp_default_wpm, initial_wpm);
    }
}
```

**Step 1: Create handler.rs**

Create `src/ui/config_popup/handler.rs`:

```rust
use super::ConfigPopupState;
use crate::app::App;
use crossterm::event::{KeyCode, KeyModifiers};

/// Handle key events when config popup is open
/// Returns true if the event was consumed
pub fn handle_popup_key(key_code: KeyCode, modifiers: KeyModifiers, app: &mut App) -> bool {
    if !app.config_popup.is_open {
        // Check for Ctrl+P to open
        if key_code == KeyCode::Char('p') && modifiers.contains(KeyModifiers::CONTROL) {
            app.config_popup.open(&app.config);
            return true;
        }
        return false;
    }
    
    // Popup is open, handle keys
    match key_code {
        KeyCode::Esc => {
            // Discard changes, revert theme
            let original_theme = app.config_popup.original_theme().to_string();
            app.config.theme = original_theme;
            app.config_popup.close();
        }
        KeyCode::Enter => {
            // Save changes
            let _ = app.config_popup.apply_to_config(&mut app.config);
            app.config_popup.close();
        }
        KeyCode::Up => {
            app.config_popup.move_up();
        }
        KeyCode::Down => {
            app.config_popup.move_down();
        }
        KeyCode::Left => {
            app.config_popup.cycle_left();
            // Live theme preview
            app.config.theme = app.config_popup.current_theme().to_string();
        }
        KeyCode::Right => {
            app.config_popup.cycle_right();
            // Live theme preview
            app.config.theme = app.config_popup.current_theme().to_string();
        }
        KeyCode::Char('p') if modifiers.contains(KeyModifiers::CONTROL) => {
            // Toggle close
            let original_theme = app.config_popup.original_theme().to_string();
            app.config.theme = original_theme;
            app.config_popup.close();
        }
        _ => return false,
    }
    true
}
```

**Step 2: Export handler from mod.rs**

In `src/ui/config_popup/mod.rs`, add:

```rust
mod handler;

pub use handler::handle_popup_key;
```

**Step 3: Integrate in main event loop**

In `src/ui/terminal.rs` main event loop, add before other key handling:

```rust
use crate::ui::config_popup::handle_popup_key;

// In event loop:
if handle_popup_key(key_code, modifiers, &mut app) {
    continue; // Event consumed by popup
}
```

**Step 4: Build and verify**

Run: `cargo build`
Expected: Success

**Step 5: Commit**

```bash
git add src/ui/config_popup/ src/ui/terminal.rs
git commit -m "feat(ui): add config popup key handling with live preview"
```

**Acceptance:** All keybindings work, theme previews live

---

## Task 7: Full Integration Test (~15min)

**Step 1: Run all tests**

Run: `cargo test`
Expected: All PASS

**Step 2: Build release**

Run: `cargo build --release`
Expected: Success

**Step 3: Manual testing checklist**

- [ ] Start app, press `Ctrl+P` → Popup opens above command area
- [ ] Press `↓` to select Theme row
- [ ] Press `→` to cycle themes → Background changes live
- [ ] Press `Esc` → Popup closes, theme reverts
- [ ] Open popup again, change WPM to 500
- [ ] Press `Enter` → Popup closes
- [ ] Check `~/.config/speedy/config.toml` → `default_wpm = 500` written
- [ ] Restart app → WPM starts at 500

**Step 4: Final commit**

```bash
git add -A
git commit -m "feat: add Ctrl+P config picker popup with live theme preview"
```

**Acceptance:** Manual checklist complete

---

## Summary

| Task | Description | Time | Key Files |
|------|-------------|------|-----------|
| 1 | Add `default_wpm` to Config | ~15min | `src/config/mod.rs` |
| 2 | Create config_popup module | ~30min | `src/ui/config_popup/mod.rs`, `state.rs` |
| 3 | Add popup state to App | ~10min | `src/app/mod.rs` |
| 4 | Create popup renderer | ~20min | `src/ui/config_popup/render.rs` |
| 5 | Integrate popup rendering | ~10min | `src/ui/terminal.rs` |
| 6 | Add key handling | ~25min | `src/ui/config_popup/handler.rs` |
| 7 | Integration test | ~15min | All files |

**Total: ~2 hours**
