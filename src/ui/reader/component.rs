//! Reader UI component that displays words using CellRenderer
//!
//! This is a new component that wraps CellRenderer for TUI fallback mode.
//! It keeps the existing simple reader.rs function unchanged.

use crate::rendering::cell::CellRenderer;
use crate::rendering::renderer::RsvpRenderer;
use crate::ui::theme;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    Frame,
};

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
    ///
    /// Positions the word so the anchor character is at the visual center (OVP).
    /// Uses calculate_start_column() from CellRenderer for accurate positioning.
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        // Update terminal size in renderer
        self.renderer.update_terminal_size(area.width, area.height);

        if let Some((word, anchor_position)) = self.renderer.get_current_word_state() {
            use unicode_segmentation::UnicodeSegmentation;

            // Split the word into graphemes for proper Unicode handling
            let graphemes = word.graphemes(true).collect::<Vec<&str>>();

            // Calculate the starting position using OVP anchoring
            let start_col = match self.renderer.calculate_start_column(word, anchor_position) {
                Ok(col) => col,
                Err(_) => {
                    // Fallback: render the whole word centered if calculation fails
                    let line = Line::from(word);
                    let center_row = area.y + area.height / 2;
                    frame
                        .buffer_mut()
                        .set_line(area.x, center_row, &line, area.width);
                    return;
                }
            };

            let center_row = area.y + self.renderer.get_center_row();

            // Split word into three parts: prefix, anchor, suffix
            let prefix = graphemes[..anchor_position].join("");
            let anchor_char = graphemes[anchor_position];
            let suffix = graphemes[anchor_position + 1..].join("");

            // Use theme colors instead of hard-coded values
            let text_style = Style::default()
                .fg(theme::colors::text())
                .add_modifier(Modifier::BOLD);

            let anchor_style = Style::default()
                .fg(theme::colors::anchor())
                .add_modifier(Modifier::BOLD);

            // Create a Line with different styles for each segment
            let line = Line::from(vec![
                Span::styled(prefix, text_style),
                Span::styled(anchor_char, anchor_style),
                Span::styled(suffix, text_style),
            ]);

            // Position the word with anchor at center using set_line
            // This places the anchor character at the calculated OVP position
            frame
                .buffer_mut()
                .set_line(area.x + start_col, center_row, &line, line.width() as u16);
        }
    }

    /// Display a word with OVP anchoring
    pub fn display_word(
        &mut self,
        word: &str,
        anchor_position: usize,
    ) -> Result<(), crate::rendering::renderer::RendererError> {
        self.renderer.render_word(word, anchor_position)
    }

    /// Clear the display
    pub fn clear(&mut self) -> Result<(), crate::rendering::renderer::RendererError> {
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
        let mut reader = ReaderComponent::new();
        assert!(!reader.renderer().supports_subpixel_ovp());
    }

    #[test]
    fn test_display_word_stores_anchor() {
        let mut reader = ReaderComponent::new();
        reader.display_word("hello", 2).unwrap();

        // Verify word and anchor are stored correctly
        assert_eq!(reader.renderer().get_current_word(), Some("hello"));
        assert_eq!(
            reader.renderer().get_current_word_state(),
            Some(("hello", 2))
        );
    }

    #[test]
    fn test_clear_removes_word_and_anchor() {
        let mut reader = ReaderComponent::new();
        reader.display_word("hello", 1).unwrap();
        reader.clear().unwrap();

        assert!(reader.renderer().get_current_word().is_none());
        assert!(reader.renderer().get_current_word_state().is_none());
    }

    #[test]
    fn test_unicode_word_with_anchor() {
        let mut reader = ReaderComponent::new();

        // CJK characters
        assert!(reader.display_word("你好", 1).is_ok());
        assert_eq!(
            reader.renderer().get_current_word_state(),
            Some(("你好", 1))
        );

        reader.clear().unwrap();

        // Emoji
        assert!(reader.display_word("hi😊", 2).is_ok());
        assert_eq!(
            reader.renderer().get_current_word_state(),
            Some(("hi😊", 2))
        );
    }
}
