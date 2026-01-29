//! CellRenderer - TUI fallback renderer using pure Ratatui widgets
//!
//! This renderer implements RsvpRenderer trait for terminals that
//! don't support Kitty Graphics Protocol. OVP anchoring snaps to
//! nearest character cell (not sub-pixel accurate like graphics mode).
//!
//! **IMPORTANT:** In TUI mode, the terminal controls fonts, not the application.
//! This module does NOT use font.rs metrics. It operates purely on
//! cell-based positioning using unicode-width crate for proper
//! character width calculations (handling emoji, CJK, etc.).

use super::renderer::{RendererError, RsvpRenderer};

/// TUI fallback renderer using character cells
pub struct CellRenderer {
    /// Terminal size in cells (columns, rows)
    terminal_size: (u16, u16),
    /// Current word state: (word, anchor_position)
    current_word_state: Option<(String, usize)>,
}

impl CellRenderer {
    /// Create a new CellRenderer instance
    pub fn new() -> Self {
        Self {
            terminal_size: (80, 24),
            current_word_state: None,
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
        self.current_word_state
            .as_ref()
            .map(|(word, _)| word.as_str())
    }

    /// Get the current word and anchor position
    pub fn get_current_word_state(&self) -> Option<(&str, usize)> {
        self.current_word_state
            .as_ref()
            .map(|(word, anchor)| (word.as_str(), *anchor))
    }

    /// Calculate the starting column for OVP anchoring
    ///
    /// Given a word and an anchor position (0-based), calculate the column
    /// where the word should start so that the anchor character is centered.
    ///
    /// # Arguments
    /// * `word` - The word to position
    /// * `anchor_position` - Character index that should be at center (OVP)
    ///
    /// # Returns
    /// The column index where the word should start
    pub fn calculate_start_column(
        &self,
        word: &str,
        anchor_position: usize,
    ) -> Result<u16, RendererError> {
        let word_len = word.chars().count();
        if anchor_position >= word_len {
            return Err(RendererError::InvalidArguments(format!(
                "anchor_position {} out of bounds for word '{}' (length: {})",
                anchor_position, word, word_len
            )));
        }

        let (terminal_width, _) = self.terminal_size;
        let center_col = terminal_width / 2;

        // Calculate the display width of characters before the anchor position
        // For proper Unicode support, we use unicode-width to get actual display width
        let prefix: String = word.chars().take(anchor_position).collect();
        let prefix_width = unicode_width::UnicodeWidthStr::width(prefix.as_str()) as u16;

        // Start column = center - prefix_width
        // This ensures the anchor character is visually centered
        let start_col = center_col.saturating_sub(prefix_width);
        Ok(start_col)
    }
}

impl Default for CellRenderer {
    fn default() -> Self {
        Self::new()
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

        // Store both the word and the anchor position for rendering
        self.current_word_state = Some((word.to_string(), anchor_position));

        // In TUI mode, actual rendering happens in ReaderComponent::render()
        // which uses Ratatui's layout system for proper Unicode handling
        Ok(())
    }

    fn clear(&mut self) -> Result<(), RendererError> {
        self.current_word_state = None;
        Ok(())
    }

    fn supports_subpixel_ovp(&self) -> bool {
        false // TUI mode only supports cell-level positioning
    }

    fn cleanup(&mut self) -> Result<(), RendererError> {
        self.current_word_state = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_renderer_creation() {
        let renderer = CellRenderer::new();
        assert_eq!(renderer.terminal_size, (80, 24));
        assert!(renderer.current_word_state.is_none());
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
    fn test_render_word_stores_word_and_anchor() {
        let mut renderer = CellRenderer::new();
        renderer.initialize().unwrap();

        renderer.render_word("hello", 2).unwrap();
        assert_eq!(renderer.get_current_word(), Some("hello"));
        assert_eq!(renderer.get_current_word_state(), Some(("hello", 2)));
    }

    #[test]
    fn test_clear_removes_word() {
        let mut renderer = CellRenderer::new();
        renderer.initialize().unwrap();
        renderer.render_word("hello", 0).unwrap();

        renderer.clear().unwrap();
        assert!(renderer.get_current_word().is_none());
        assert!(renderer.get_current_word_state().is_none());
    }

    #[test]
    fn test_cleanup_removes_word() {
        let mut renderer = CellRenderer::new();
        renderer.initialize().unwrap();
        renderer.render_word("hello", 0).unwrap();

        renderer.cleanup().unwrap();
        assert!(renderer.get_current_word().is_none());
        assert!(renderer.get_current_word_state().is_none());
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

    #[test]
    fn test_calculate_start_column_basic() {
        let renderer = CellRenderer::new();
        // Terminal width is 80, center is 40
        // For "hello" with anchor at 2 (0-indexed), prefix is "he" (width 2), start at 40 - 2 = 38
        assert_eq!(renderer.calculate_start_column("hello", 2).unwrap(), 38);
    }

    #[test]
    fn test_calculate_start_column_single_char() {
        let renderer = CellRenderer::new();
        // Single char at center (40), prefix is "" (width 0)
        assert_eq!(renderer.calculate_start_column("a", 0).unwrap(), 40);
    }

    #[test]
    fn test_calculate_start_column_out_of_bounds() {
        let renderer = CellRenderer::new();
        let result = renderer.calculate_start_column("hi", 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_start_column_with_cjk() {
        let renderer = CellRenderer::new();
        // "你好" - each CJK char is 2 cells wide
        // Anchor at 0: prefix is "" (width 0), start at 40
        assert_eq!(renderer.calculate_start_column("你好", 0).unwrap(), 40);

        // Anchor at 1: prefix is "你" (width 2), start at 40 - 2 = 38
        assert_eq!(renderer.calculate_start_column("你好", 1).unwrap(), 38);
    }

    #[test]
    fn test_calculate_start_column_with_emoji() {
        let renderer = CellRenderer::new();
        // "hi😊" - emoji is typically 2 cells wide
        // Anchor at 2 (the emoji position): prefix is "hi" (width 2), start at 40 - 2 = 38
        assert_eq!(renderer.calculate_start_column("hi😊", 2).unwrap(), 38);
    }

    #[test]
    fn test_unicode_width_emoji_handling() {
        let mut renderer = CellRenderer::new();
        renderer.initialize().unwrap();

        // Test that word length calculation handles emojis correctly
        // "hi😊" should be recognized as a word with emoji
        assert!(renderer.render_word("hi😊", 0).is_ok());
        assert_eq!(renderer.get_current_word(), Some("hi😊"));
        assert_eq!(renderer.get_current_word_state(), Some(("hi😊", 0)));

        // Emoji at position 2 should work
        assert!(renderer.render_word("hi😊", 2).is_ok());
        assert_eq!(renderer.get_current_word_state(), Some(("hi😊", 2)));
    }

    #[test]
    fn test_unicode_width_cjk_handling() {
        let mut renderer = CellRenderer::new();
        renderer.initialize().unwrap();

        // Test that CJK characters work correctly
        assert!(renderer.render_word("你好", 1).is_ok());
        assert_eq!(renderer.get_current_word(), Some("你好"));
        assert_eq!(renderer.get_current_word_state(), Some(("你好", 1)));
    }

    #[test]
    fn test_render_word_rejects_invalid_anchor_unicode() {
        let mut renderer = CellRenderer::new();
        renderer.initialize().unwrap();

        // Out of bounds should return error regardless of character type
        // "hi😊" has 3 chars (h, i, 😊)
        let result = renderer.render_word("hi😊", 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_narrow_terminal_handling() {
        let mut renderer = CellRenderer::new();
        renderer.update_terminal_size(10, 24); // Very narrow

        // Should still validate anchor correctly
        assert!(renderer.render_word("hello", 2).is_ok());

        // With 10 cols, center is 5, prefix "he" is width 2, start at 5 - 2 = 3
        let start_col = renderer.calculate_start_column("hello", 2).unwrap();
        assert_eq!(start_col, 3);
    }

    #[test]
    fn test_wide_terminal_handling() {
        let mut renderer = CellRenderer::new();
        renderer.update_terminal_size(200, 60); // Very wide

        let start_col = renderer.calculate_start_column("test", 1).unwrap();
        // Center at 100, prefix "t" is width 1, start at 100 - 1 = 99
        assert_eq!(start_col, 99);
    }
}
