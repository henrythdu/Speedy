//! Reader UI component that displays words using CellRenderer
//!
//! This is a new component that wraps CellRenderer for TUI fallback mode.
//! It keeps the existing simple reader.rs function unchanged.

use crate::engine::cell_renderer::CellRenderer;
use crate::engine::renderer::RsvpRenderer;
use ratatui::{
    layout::{Alignment, Rect},
    prelude::Widget,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
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
    /// Uses Ratatui's layout system for proper Unicode character handling
    /// and applies different styles for the prefix, anchor, and suffix.
    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        // Update terminal size in renderer
        self.renderer.update_terminal_size(area.width, area.height);

        if let Some((word, anchor_position)) = self.renderer.get_current_word_state() {
            use unicode_segmentation::UnicodeSegmentation;

            // Split the word into graphemes for proper Unicode handling
            let graphemes: Vec<&str> = word.graphemes(true).collect();

            if anchor_position >= graphemes.len() {
                // Fallback: render the whole word if anchor is out of bounds
                let paragraph = Paragraph::new(word).alignment(Alignment::Center);
                paragraph.render(area, frame.buffer_mut());
                return;
            }

            // Split word into three parts: prefix, anchor, suffix
            let prefix = graphemes[..anchor_position].join("");
            let anchor_char = graphemes[anchor_position];
            let suffix = graphemes[anchor_position + 1..].join("");

            // Theme colors from PRD Section 4.1 (Midnight theme)
            let text_style = Style::default()
                .fg(Color::Rgb(169, 177, 214)) // #A9B1D6 Light Blue
                .add_modifier(Modifier::BOLD);

            let anchor_style = Style::default()
                .fg(Color::Rgb(247, 118, 142)) // #F7768E Coral Red (anchor color)
                .add_modifier(Modifier::BOLD);

            // Create a Line with different styles for each segment
            let line = Line::from(vec![
                Span::styled(prefix, text_style),
                Span::styled(anchor_char, anchor_style),
                Span::styled(suffix, text_style),
            ]);

            // Use Ratatui's Paragraph for proper centering and rendering
            let paragraph = Paragraph::new(line).alignment(Alignment::Center);
            paragraph.render(area, frame.buffer_mut());
        }
    }

    /// Display a word with OVP anchoring
    pub fn display_word(
        &mut self,
        word: &str,
        anchor_position: usize,
    ) -> Result<(), crate::engine::renderer::RendererError> {
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
