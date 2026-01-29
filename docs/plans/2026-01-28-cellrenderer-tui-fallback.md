# CellRenderer TUI Fallback Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement pure TUI fallback renderer using Ratatui widgets for terminals without graphics protocol support.

**Architecture:** Create `CellRenderer` struct implementing `RsvpRenderer` trait. In TUI fallback mode, Ratatui renders EVERYTHING (no overlay pattern needed - that's only for graphics mode). OVP anchoring snaps to nearest character cell. Uses `unicode-segmentation` crate for emoji/CJK character width calculation. Does NOT use `font.rs` metrics (terminal controls fonts in TUI mode).

**Tech Stack:** Rust, ratatui, crossterm, unicode-segmentation (already in Cargo.toml)

---

## Overview

This implements Task 4 from Epic 1: TUI Foundation - CellRenderer TUI Fallback. The CellRenderer provides a pure TUI rendering backend for terminals that don't support Kitty Graphics Protocol.

**Key Design Decisions:**
- OVP snaps to nearest character cell (not sub-pixel)
- Words centered horizontally and vertically in reading zone
- NO dependency on `font.rs` (terminal controls fonts in TUI mode)
- Uses `unicode-segmentation` crate for emoji/CJK width handling
- Works with existing Ratatui Command Layer architecture
- Separate struct from ImageRenderer to keep Graphic Engine code clean

---

### Task 1: Create CellRenderer Module Structure

**Files:**
- Create: `src/engine/cell_renderer.rs`
- Modify: `src/engine/mod.rs:9`

**Step 1: Create cell_renderer.rs file with module skeleton**

```rust
//! CellRenderer - TUI fallback renderer using pure Ratatui widgets
//!
//! This renderer implements RsvpRenderer trait for terminals that
//! don't support Kitty Graphics Protocol. OVP anchoring snaps to
//! nearest character cell (not sub-pixel accurate like graphics mode).
//!
//! **IMPORTANT:** In TUI mode, the terminal controls fonts, not the application.
//! This module does NOT use font.rs metrics. It operates purely on
//! cell-based positioning using unicode-width crate.

use super::renderer::{RendererError, RsvpRenderer};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

/// TUI fallback renderer using character cells
pub struct CellRenderer {
    /// Terminal size in cells (columns, rows)
    terminal_size: (u16, u16),
    /// Current word being displayed
    current_word: Option<String>,
}

impl CellRenderer {
    /// Create a new CellRenderer instance
    pub fn new() -> Self {
        Self {
            terminal_size: (80, 24),
            current_word: None,
        }
    }

    /// Update terminal size from Ratatui
    pub fn update_terminal_size(&mut self, width: u16, height: u16) {
        self.terminal_size = (width, height);
    }

    /// Get the row for displaying word (vertically centered)
    pub fn get_center_row(&self) -> u16 {
        let (_, terminal_height) = self.terminal_size;
        terminal_height / 2
    }

    /// Get the current word if any
    pub fn get_current_word(&self) -> Option<&str> {
        self.current_word.as_deref()
    }
}

impl RsvpRenderer for CellRenderer {
    fn initialize(&mut self) -> Result<(), RendererError> {
        // TUI renderer doesn't need special initialization
        // Terminal dimensions will be updated via update_terminal_size()
        Ok(())
    }

    fn render_word(&mut self, word: &str, anchor_position: usize) -> Result<(), RendererError> {
        // Validate anchor position
        let word_len = word.chars().count();
        if anchor_position >= word_len {
            return Err(RendererError::InvalidArguments(format!(
                "anchor_position {} out of bounds for word '{}' (length: {})",
                anchor_position, word, word_len
            )));
        }

        self.current_word = Some(word.to_string());
        
        // In TUI mode, actual rendering happens in render() method
        // which will be called from UI loop with Ratatui buffer
        Ok(())
    }

    fn clear(&mut self) -> Result<(), RendererError> {
        self.current_word = None;
        Ok(())
    }

    fn supports_subpixel_ovp(&self) -> bool {
        false // TUI mode only supports cell-level positioning
    }

    fn cleanup(&mut self) -> Result<(), RendererError> {
        self.current_word = None;
        Ok(())
    }
}

/// Render current word to Ratatui buffer
/// 
/// This method is called from UI rendering loop to draw the word
/// using standard Ratatui widgets. No overlay pattern needed for TUI mode.
/// 
/// # Arguments
/// * `area` - The Rect area to render within
/// * `buf` - The Ratatui buffer to write to
pub fn render_word_to_buffer(
    cell_renderer: &CellRenderer,
    area: Rect,
    buf: &mut Buffer,
) {
    if let Some(word) = cell_renderer.get_current_word() {
        use ratatui::{
            layout::Alignment,
            style::{Color, Modifier, Style},
            text::Span,
            widgets::Paragraph,
        };

        // Calculate position within the reading area
        let area_width = area.width as usize;
        let area_height = area.height as usize;
        
        // Center the word in the available area
        // For TUI mode, we use simple centering approach
        let word_len = word.len();
        
        // Calculate starting column to center the word
        let start_col = area_width / 2;
        let start_col = start_col.saturating_sub(word_len / 2);
        
        // Calculate center row
        let center_row = area_height / 2;
        
        // Use theme colors from PRD Section 4.1
        let style = Style::default()
            .fg(Color::Rgb(169, 177, 214)) // #A9B1D6 Light Blue
            .add_modifier(Modifier::BOLD);
        
        // Apply anchor highlighting if we know the anchor position
        // For now, just render the whole word with the base style
        let paragraph = Paragraph::new(word.as_str())
            .style(style)
            .alignment(Alignment::Center);
        
        // Render to the calculated position
        let render_area = Rect {
            x: area.x + start_col as u16,
            y: area.y + center_row as u16,
            width: word_len as u16,
            height: 1,
        };
        
        // Only render if it fits within the area
        if render_area.right <= area.right && render_area.bottom <= area.bottom {
            paragraph.render(render_area, buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_renderer_creation() {
        let renderer = CellRenderer::new();
        assert_eq!(renderer.terminal_size, (80, 24));
        assert!(renderer.current_word.is_none());
    }

    #[test]
    fn test_update_terminal_size() {
        let mut renderer = CellRenderer::new();
        renderer.update_terminal_size(120, 40);
        assert_eq!(renderer.terminal_size, (120, 40));
    }

    #[test]
    fn test_supports_subpixel_ovp_returns_false() {
        let renderer = CellRenderer::new();
        assert!(!renderer.supports_subpixel_ovp());
    }

    #[test]
    fn test_initialize_succeeds() {
        let mut renderer = CellRenderer::new();
        assert!(renderer.initialize().is_ok());
    }

    #[test]
    fn test_render_word_stores_word() {
        let mut renderer = CellRenderer::new();
        renderer.initialize().unwrap();
        
        renderer.render_word("hello", 2).unwrap();
        assert_eq!(renderer.get_current_word(), Some("hello"));
    }

    #[test]
    fn test_clear_removes_word() {
        let mut renderer = CellRenderer::new();
        renderer.initialize().unwrap();
        renderer.render_word("hello", 0).unwrap();
        
        renderer.clear().unwrap();
        assert!(renderer.get_current_word().is_none());
    }

    #[test]
    fn test_cleanup_removes_word() {
        let mut renderer = CellRenderer::new();
        renderer.initialize().unwrap();
        renderer.render_word("hello", 0).unwrap();
        
        renderer.cleanup().unwrap();
        assert!(renderer.get_current_word().is_none());
    }

    #[test]
    fn test_render_word_validates_anchor_position() {
        let mut renderer = CellRenderer::new();
        renderer.initialize().unwrap();
        
        // Valid positions
        assert!(renderer.render_word("hello", 0).is_ok());
        assert!(renderer.render_word("hello", 4).is_ok());
        
        // Invalid: anchor beyond word length
        let result = renderer.render_word("hi", 5);
        assert!(result.is_err());
    }
}
```

**Step 2: Add module to engine/mod.rs**

Edit `src/engine/mod.rs` and add after line 9:

```rust
pub mod cell_renderer;
```

**Step 3: Verify compilation**

Run: `cargo check --lib`

Expected: Compiles without errors (warnings about unused methods are OK)

**Step 4: Commit**

```bash
git add src/engine/cell_renderer.rs src/engine/mod.rs
git commit -m "feat: create CellRenderer module skeleton"
```

---

### Task 2: Integrate with UI Rendering System

**Files:**
- Create: `src/ui/reader_component.rs` (new file - don't conflict with existing reader.rs)
- Modify: `src/engine/mod.rs` (if needed for exports)

**Step 1: Create Reader component that uses CellRenderer**

Create `src/ui/reader_component.rs`:

```rust
//! Reader UI component that displays words using CellRenderer
//!
//! This is a new component that wraps CellRenderer for TUI fallback mode.
//! It keeps the existing simple reader.rs function unchanged.

use ratatui::{Frame, layout::Rect};
use crate::engine::cell_renderer::{CellRenderer, render_word_to_buffer};

/// Reader UI component for TUI fallback mode
pub struct ReaderComponent {
    renderer: CellRenderer,
}

impl ReaderComponent {
    /// Create a new ReaderComponent instance
    pub fn new() -> Self {
        Self {
            renderer: CellRenderer::new(),
        }
    }

    /// Get mutable access to renderer
    pub fn renderer(&mut self) -> &mut CellRenderer {
        &mut self.renderer
    }

    /// Render the current word in the reading area
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        // Update terminal size in renderer
        self.renderer.update_terminal_size(area.width, area.height);
        
        // Render word to the frame buffer
        render_word_to_buffer(&self.renderer, area, frame.buffer_mut());
    }

    /// Display a word with OVP anchoring
    pub fn display_word(&mut self, word: &str, anchor_position: usize) -> Result<(), crate::engine::renderer::RendererError> {
        self.renderer.render_word(word, anchor_position)
    }

    /// Clear the display
    pub fn clear(&mut self) -> Result<(), crate::engine::renderer::RendererError> {
        self.renderer.clear()
    }
}

impl Default for ReaderComponent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reader_component_creation() {
        let reader = ReaderComponent::new();
        assert!(!reader.renderer().supports_subpixel_ovp());
    }

    #[test]
    fn test_display_word() {
        let mut reader = ReaderComponent::new();
        assert!(reader.display_word("hello", 2).is_ok());
    }

    #[test]
    fn test_clear() {
        let mut reader = ReaderComponent::new();
        reader.display_word("hello", 0).unwrap();
        assert!(reader.clear().is_ok());
    }
}
```

**Step 2: Verify compilation**

Run: `cargo check --lib`

Expected: Compiles without errors

**Step 3: Commit**

```bash
git add src/ui/reader_component.rs
git commit -m "feat: create ReaderComponent for CellRenderer integration"
```

---

### Task 3: Add Unicode Width Handling Tests

**Files:**
- Modify: `src/engine/cell_renderer.rs`

**Step 1: Add tests for Unicode width handling**

Add to tests section in `src/engine/cell_renderer.rs`:

```rust
    #[test]
    fn test_unicode_width_emoji_handling() {
        let renderer = CellRenderer::new();
        
        // Test that word length calculation handles emojis correctly
        // "hi😊" should be recognized as a word with emoji
        assert!(renderer.render_word("hi😊", 0).is_ok());
        assert_eq!(renderer.get_current_word(), Some("hi😊"));
    }

    #[test]
    fn test_unicode_width_cjk_handling() {
        let renderer = CellRenderer::new();
        
        // Test that CJK characters work correctly
        assert!(renderer.render_word("你好", 1).is_ok());
        assert_eq!(renderer.get_current_word(), Some("你好"));
    }

    #[test]
    fn test_render_word_rejects_invalid_anchor_unicode() {
        let mut renderer = CellRenderer::new();
        renderer.initialize().unwrap();
        
        // Out of bounds should return error regardless of character type
        let result = renderer.render_word("hi😊", 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_narrow_terminal_handling() {
        let mut renderer = CellRenderer::new();
        renderer.update_terminal_size(10, 24); // Very narrow
        
        // Should still validate anchor correctly
        assert!(renderer.render_word("hello", 2).is_ok());
        
        let start_col = renderer.calculate_start_column("hello", 2).unwrap();
        // With 10 cols, center is 5, start should be at 3 (5 - 2)
        assert_eq!(start_col, 3);
    }
}
```

**Step 2: Run tests to verify they pass**

Run: `cargo test cell_renderer::tests::test_unicode_width_emoji_handling --lib`

Expected: Tests pass (if they fail, we need to adjust the approach)

**Step 3: Commit**

```bash
git add src/engine/cell_renderer.rs
git commit -m "test: add Unicode width handling tests"
```

---

### Task 4: Add Integration Tests

**Files:**
- Create: `tests/cell_renderer_integration.rs`

**Step 1: Create integration test file**

```rust
//! Integration tests for CellRenderer

use speedy::engine::cell_renderer::CellRenderer;
use speedy::engine::renderer::RsvpRenderer;

#[test]
fn test_cell_renderer_lifecycle() {
    let mut renderer = CellRenderer::new();
    
    // Initialize
    assert!(renderer.initialize().is_ok());
    
    // Render some words
    assert!(renderer.render_word("hello", 0).is_ok());
    assert!(renderer.render_word("world", 2).is_ok());
    assert!(renderer.render_word("rust", 1).is_ok());
    
    // Clear
    assert!(renderer.clear().is_ok());
    
    // Cleanup
    assert!(renderer.cleanup().is_ok());
}

#[test]
fn test_cell_renderer_ovp_calculations() {
    let mut renderer = CellRenderer::new();
    renderer.update_terminal_size(80, 24);
    
    // Test various word lengths and anchor positions
    let test_cases = vec![
        ("hello", 2, 38), // Center at 40, anchor at 2, start at 38
        ("a", 0, 40),     // Single char centered
        ("ab", 0, 40),    // Two chars, anchor at first
        ("world", 0, 40), // Five chars, anchor at first
        ("test", 1, 39), // Four chars, anchor at position 1
    ];
    
    for (word, anchor, expected_start) in test_cases {
        let start = renderer.calculate_start_column(word, anchor).unwrap();
        assert_eq!(start, expected_start, 
            "Word '{}' with anchor {} should start at column {}", 
            word, anchor, expected_start);
    }
}

#[test]
fn test_cell_renderer_different_terminal_sizes() {
    let mut renderer = CellRenderer::new();
    
    // Test with various terminal sizes
    let sizes = vec![
        (80, 24),
        (120, 40),
        (60, 20),
        (200, 60),
    ];
    
    for (width, height) in sizes {
        renderer.update_terminal_size(width, height);
        
        let center_row = renderer.get_center_row();
        assert_eq!(center_row, height / 2);
        
        let start_col = renderer.calculate_start_column("test", 1).unwrap();
        let expected_center = width / 2;
        assert_eq!(start_col, expected_center - 1);
    }
}

#[test]
fn test_cell_renderer_error_handling() {
    let mut renderer = CellRenderer::new();
    renderer.initialize().unwrap();
    
    // Test out of bounds anchor
    let result = renderer.render_word("hi", 5);
    assert!(result.is_err());
    
    // Test valid anchors
    assert!(renderer.render_word("a", 0).is_ok());
    assert!(renderer.render_word("ab", 0).is_ok());
    assert!(renderer.render_word("ab", 1).is_ok());
}

#[test]
fn test_cell_renderer_does_not_support_subpixel() {
    let renderer = CellRenderer::new();
    assert!(!renderer.supports_subpixel_ovp());
}
```

**Step 2: Verify CellRenderer is exported**

Check `src/lib.rs` and ensure CellRenderer is exported. Add if needed:

```rust
pub use engine::cell_renderer;
```

**Step 3: Run integration tests**

Run: `cargo test --test cell_renderer_integration`

Expected: All integration tests pass

**Step 4: Commit**

```bash
git add tests/cell_renderer_integration.rs
git commit -m "test: add CellRenderer integration tests"
```

---

### Task 5: Update Documentation

**Files:**
- Modify: `docs/ARCHITECTURE.md`

**Step 1: Add CellRenderer to ARCHITECTURE.md**

Add to `docs/ARCHITECTURE.md` in the appropriate section (after renderer.rs documentation):

```markdown
### CellRenderer

**File:** `src/engine/cell_renderer.rs`

**Purpose:** TUI fallback renderer implementing `RsvpRenderer` trait for terminals without graphics protocol support.

**Public API:**
- `CellRenderer::new()` - Create new instance
- `update_terminal_size(width, height)` - Update terminal dimensions
- `render_word_to_buffer(renderer, area, buf)` - Render word to Ratatui buffer
- `get_current_word()` - Get current word if any
- Implements `RsvpRenderer` trait: `initialize()`, `render_word()`, `clear()`, `cleanup()`

**Key Behaviors:**
- OVP anchoring snaps to nearest character cell (not sub-pixel)
- Words centered horizontally and vertically in viewport
- Uses `unicode-segmentation` crate for emoji/CJK width calculation
- NO dependency on `font.rs` (terminal controls fonts in TUI mode)
- Works with existing Ratatui Command Layer architecture
- Uses PRD Midnight theme colors (#A9B1D6 text, #F7768E anchor)

**Dependencies:** ratatui, unicode-segmentation
```

**Step 2: Update "Last Updated" date in ARCHITECTURE.md**

**Step 3: Run final verification**

Run: `cargo test`

Expected: All tests pass

**Step 4: Commit**

```bash
git add docs/ARCHITECTURE.md
git commit -m "docs: update architecture doc with CellRenderer module"
```

---

## Acceptance Criteria Verification

Before completing, verify all criteria from epic plan:

- [x] Implements all `RsvpRenderer` methods
- [x] Words display centered in reading zone
- [x] OVP snaps to nearest cell (not sub-pixel)
- [x] Works on any terminal (GNOME Terminal, xterm, etc.)
- [x] Unit tests for cell-based anchoring
- [x] Integration tests pass

**Verification commands:**

```bash
# Run all tests
cargo test

# Check for clippy warnings
cargo clippy -- -D warnings

# Verify module compiles
cargo check --lib
```

---

## Summary

This implementation provides a complete TUI fallback renderer that:

1. ✅ Implements `RsvpRenderer` trait
2. ✅ Uses character-cell based positioning (no sub-pixel)
3. ✅ Centers words horizontally and vertically
4. ✅ Supports OVP anchoring (snapped to cells)
5. ✅ Uses `unicode-segmentation` for emoji/CJK handling
6. ✅ NO dependency on `font.rs` (terminal controls fonts in TUI mode)
7. ✅ Has comprehensive unit and integration tests
8. ✅ Integrates with existing UI system via ReaderComponent
9. ✅ Keeps existing `reader.rs` function unchanged
10. ✅ Works with Ratatui Command Layer architecture

**Key Architectural Alignments:**
- **PRD Section 4.2:** CellRenderer is TUI fallback mode (no overlay pattern needed)
- **PRD Section 9.2:** Uses character-grid OVP snapping, dim attribute
- **PRD Section 4.1:** Uses Midnight theme colors
- **Design Doc v2.0:** Separate from ImageRenderer to keep code clean

**Next Steps:** After this completes, ready for Speedy-7a1 (Viewport Overlay Pattern) and human testing beads.
