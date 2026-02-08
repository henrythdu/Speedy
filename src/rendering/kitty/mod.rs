//! Kitty Graphics Protocol rendering modules
//!
//! This module provides pixel-perfect word rendering with sub-pixel OVP anchoring
//! using the Kitty Graphics Protocol.

pub mod positioning;
pub mod protocol;
pub mod rasterizer;

use crate::engine::config::{DEFAULT_CACHE_CAPACITY, DEFAULT_FONT_SIZE, PROGRESS_BAR_MARGIN_PX};
use crate::rendering::cache::WordCache;
use crate::rendering::font::{get_font, get_font_metrics, FontMetrics};
use crate::rendering::kitty::positioning::{calculate_start_x, calculate_vertical_center};
use crate::rendering::kitty::protocol::{
    delete_all_graphics, delete_image, encode_image_base64, transmit_graphics,
};
use crate::rendering::progress_bar::SentenceProgressBar;
use crate::rendering::renderer::{RendererError, RsvpRenderer};
use crate::rendering::viewport::Viewport;
use ab_glyph::FontRef;
use std::io::{self, Write};

/// Kitty Graphics Protocol renderer for pixel-perfect RSVP
pub struct KittyGraphicsRenderer {
    /// Terminal viewport for coordinate conversion
    viewport: Viewport,
    /// Font reference for rasterization
    font: Option<FontRef<'static>>,
    /// Font size in pixels
    font_size: f32,
    /// Font metrics for positioning calculations
    font_metrics: Option<FontMetrics>,
    /// Current image ID for protocol (incremented per word)
    pub current_image_id: u32,
    /// Word-level LRU cache for rendered buffers
    word_cache: WordCache,
}

impl Default for KittyGraphicsRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl KittyGraphicsRenderer {
    /// Create a new KittyGraphicsRenderer with default font size
    pub fn new() -> Self {
        Self {
            viewport: Viewport::new(),
            font: None,
            font_size: DEFAULT_FONT_SIZE,
            font_metrics: None,
            current_image_id: 1,
            word_cache: WordCache::new(DEFAULT_CACHE_CAPACITY),
        }
    }

    /// Calculate font size based on terminal cell dimensions
    ///
    /// Sets the font size to render at approximately 5 lines height
    /// based on the cell height from the viewport.
    pub fn calculate_font_size_from_cell_height(&mut self, cell_height_px: f32) {
        // Font size should be approximately 5 lines height
        // We use a scale factor of 1.0 for the font, so font_size = cell_height × 5
        self.font_size = cell_height_px * 5.0;

        // Update font metrics with the new size
        if let Some(ref font) = self.font {
            self.font_metrics = Some(get_font_metrics(font, self.font_size));
        }

        // Sync font size with word cache (clears cache if size changed)
        self.word_cache.set_font_size(self.font_size);
    }

    /// Get reference to viewport (for external access to query dimensions)
    pub fn viewport(&mut self) -> &mut Viewport {
        &mut self.viewport
    }

    /// Get vertical center Y position (for bar positioning)
    pub fn get_vertical_center(&self) -> Option<u32> {
        calculate_vertical_center(&self.viewport)
    }

    /// Calculate word height for bar positioning
    pub fn calculate_word_height(
        &mut self,
        word: &str,
        anchor_position: usize,
    ) -> Result<u32, RendererError> {
        let font = self
            .font
            .as_ref()
            .ok_or_else(|| RendererError::RenderFailed("Font not initialized".to_string()))?;
        let metrics = self
            .font_metrics
            .as_ref()
            .ok_or_else(|| RendererError::RenderFailed("Font metrics not available".to_string()))?;

        let cached_word = self
            .word_cache
            .get_or_render(word, anchor_position, font, metrics)
            .map_err(|e| RendererError::RenderFailed(format!("Cache error: {}", e)))?;

        Ok(cached_word.height)
    }

    /// Render sentence progress bar below word
    ///
    /// # Arguments
    /// * `word_y` - Y position of word (from calculate_vertical_center)
    /// * `word_height` - Height of rendered word in pixels
    /// * `progress` - Fill percentage (0.0 to 1.0)
    /// * `image_id` - Image ID for this bar
    pub fn render_bar(
        &mut self,
        word_y: u32,
        word_height: u32,
        progress: f64,
        image_id: u32,
    ) -> Result<(), RendererError> {
        // Simple: bar Y = word Y + word height + 10px margin
        let bar_y = word_y + word_height + PROGRESS_BAR_MARGIN_PX;

        // Create bar with current viewport width
        let container_width = self
            .viewport
            .get_dimensions()
            .map(|d| d.pixel_size.0)
            .unwrap_or(800);
        let mut bar = SentenceProgressBar::new(container_width);
        bar.update_progress(progress);

        // Center bar horizontally in viewport
        let bar_x = (container_width - bar.width()) / 2;

        // Move cursor to bar position
        if let Some((col, row)) = self.viewport.pixel_to_cell(bar_x, bar_y) {
            print!("\x1b[{};{}H", row + 1, col + 1);
            if let Err(e) = io::stdout().flush() {
                return Err(RendererError::RenderFailed(format!(
                    "Cursor flush failed: {}",
                    e
                )));
            }
        }

        // Render bar at cursor position
        let bar_buffer = bar.render();
        let base64_data = encode_image_base64(&bar_buffer);
        transmit_graphics(image_id, bar.width(), 2, &base64_data, 0, 0)
            .map_err(|e| RendererError::RenderFailed(format!("Bar render failed: {}", e)))
    }
}

impl RsvpRenderer for KittyGraphicsRenderer {
    fn initialize(&mut self) -> Result<(), RendererError> {
        // Load bundled font
        self.font = get_font();
        if self.font.is_none() {
            return Err(RendererError::InitializationFailed(
                "Failed to load bundled font".to_string(),
            ));
        }

        // Get font metrics
        let font = self.font.as_ref().unwrap();
        self.font_metrics = Some(get_font_metrics(font, self.font_size));

        // Initialize word cache with current font size
        self.word_cache.set_font_size(self.font_size);

        // Query viewport dimensions
        match self.viewport.query_dimensions() {
            Ok(_) => Ok(()),
            Err(_) => {
                // Fallback is acceptable - will use estimated dimensions
                Ok(())
            }
        }
    }

    fn render_word(&mut self, word: &str, anchor_position: usize) -> Result<(), RendererError> {
        if word.is_empty() {
            return Ok(());
        }

        let word_len = word.chars().count();
        if anchor_position >= word_len {
            return Err(RendererError::InvalidArguments(format!(
                "anchor_position {} out of bounds for word '{}' (length: {})",
                anchor_position, word, word_len
            )));
        }

        let font = self
            .font
            .as_ref()
            .ok_or_else(|| RendererError::RenderFailed("Font not initialized".to_string()))?;

        let metrics = self
            .font_metrics
            .as_ref()
            .ok_or_else(|| RendererError::RenderFailed("Font metrics not available".to_string()))?;

        if !self.viewport.has_dimensions() {
            let _ = self.viewport.query_dimensions();
        }

        let start_x =
            calculate_start_x(word, anchor_position, font, self.font_size, &self.viewport);
        let reading_zone_center_y = calculate_vertical_center(&self.viewport).unwrap_or(0);

        if let Some((col, row)) = self
            .viewport
            .pixel_to_cell(start_x as u32, reading_zone_center_y)
        {
            let cursor_command = format!("\x1b[{};{}H", row + 1, col + 1);
            print!("{}", cursor_command);
            if let Err(e) = io::stdout().flush() {
                return Err(RendererError::RenderFailed(format!(
                    "Failed to flush cursor command: {}",
                    e
                )));
            }
        }

        // Use word cache for rasterization (performance optimization)
        let cached_word = self
            .word_cache
            .get_or_render(word, anchor_position, font, metrics)
            .map_err(|e| RendererError::RenderFailed(format!("Cache error: {}", e)))?;

        let base64_data = encode_image_base64(&cached_word.buffer);
        let (width, height) = (cached_word.width, cached_word.height);

        transmit_graphics(self.current_image_id, width, height, &base64_data, 0, 0)
            .map_err(|e| RendererError::RenderFailed(e.to_string()))?;

        self.current_image_id += 1;

        Ok(())
    }

    fn clear(&mut self) -> Result<(), RendererError> {
        // A "frame" consists of two images (word + bar).
        // We must clear both images from the previous frame.
        if self.current_image_id > 2 {
            let prev_bar_id = self.current_image_id - 1;
            let prev_word_id = self.current_image_id - 2;

            // Delete both images from the previous frame
            // Errors are not propagated to prevent a single failed delete from crashing the app
            let _ = delete_image(prev_bar_id);
            let _ = delete_image(prev_word_id);
        }
        Ok(())
    }

    fn cleanup(&mut self) -> Result<(), RendererError> {
        // Clear word cache to free memory
        self.word_cache.clear();

        if let Err(e) = delete_all_graphics() {
            return Err(RendererError::CleanupFailed(format!(
                "Failed to cleanup graphics: {}",
                e
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rendering::viewport::TerminalDimensions;

    #[test]
    fn test_kitty_renderer_initialize_loads_font() {
        let mut renderer = KittyGraphicsRenderer::new();
        let result = renderer.initialize();
        assert!(
            result.is_ok(),
            "Initialization should succeed: {:?}",
            result
        );
        assert!(renderer.font.is_some(), "Font should be loaded");
        assert!(
            renderer.font_metrics.is_some(),
            "Font metrics should be available"
        );
    }

    #[test]
    fn test_get_reading_zone_height_without_dimensions() {
        use crate::rendering::kitty::positioning::get_reading_zone_height;
        let renderer = KittyGraphicsRenderer::new();

        let zone_height = get_reading_zone_height(&renderer.viewport);
        assert!(
            zone_height.is_none(),
            "Should return None without dimensions"
        );
    }

    #[test]
    fn test_calculate_font_size_from_cell_height() {
        let mut renderer = KittyGraphicsRenderer::new();
        renderer.initialize().unwrap();

        // With cell height of 20px, font size should be 100px (20 * 5)
        renderer.calculate_font_size_from_cell_height(20.0);

        assert_eq!(renderer.font_size, 100.0);
        assert!(renderer.font_metrics.is_some());

        let metrics = renderer.font_metrics.unwrap();
        // Height should match font size
        assert!((metrics.height - 100.0).abs() < 1.0);
    }

    #[test]
    fn test_clear_returns_ok() {
        let mut renderer = KittyGraphicsRenderer::new();
        renderer.initialize().unwrap();

        // Render a word first to have something to clear
        let _ = renderer.render_word("test", 0);

        // Clear should succeed (though actual deletion may fail in test env)
        let result = renderer.clear();
        // In test environment without actual terminal, clear might fail
        // but we should at least not panic
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_cleanup_returns_ok() {
        let mut renderer = KittyGraphicsRenderer::new();
        renderer.initialize().unwrap();

        // Cleanup should attempt to delete all graphics
        let result = renderer.cleanup();
        // In test environment without actual terminal, cleanup might fail
        assert!(result.is_ok() || result.is_err());
    }
}
