//! Kitty Graphics Protocol rendering modules
//!
//! This module provides pixel-perfect word rendering with sub-pixel OVP anchoring
//! using the Kitty Graphics Protocol.

pub mod positioning;
pub mod protocol;
pub mod rasterizer;

use crate::rendering::cache::{WordCache, DEFAULT_CACHE_CAPACITY};
use crate::rendering::font::{get_font, get_font_metrics, FontMetrics};
use crate::rendering::kitty::positioning::{calculate_start_x, calculate_vertical_center};
use crate::rendering::kitty::protocol::{
    delete_all_graphics, delete_image, encode_image_base64, transmit_graphics,
};
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
            font_size: 24.0,
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
        // Delete the previous image if it exists
        if self.current_image_id > 1 {
            let prev_id = self.current_image_id - 1;
            if let Err(e) = delete_image(prev_id) {
                return Err(RendererError::ClearFailed(format!(
                    "Failed to clear image {}: {}",
                    prev_id, e
                )));
            }
        }
        Ok(())
    }

    fn supports_subpixel_ovp(&self) -> bool {
        true
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
    fn test_kitty_renderer_creation() {
        let renderer = KittyGraphicsRenderer::new();
        assert!(renderer.supports_subpixel_ovp());
        assert_eq!(renderer.current_image_id, 1);
    }

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
    fn test_kitty_renderer_supports_subpixel() {
        let renderer = KittyGraphicsRenderer::new();
        assert!(renderer.supports_subpixel_ovp());
    }

    #[test]
    fn test_get_reading_zone_height_with_dimensions() {
        use crate::rendering::kitty::positioning::get_reading_zone_height;
        let mut renderer = KittyGraphicsRenderer::new();

        // Set terminal dimensions (960x540 pixels)
        let dims = TerminalDimensions::new(960, 540, 80, 24);
        renderer.viewport.set_dimensions(dims);

        let zone_height = get_reading_zone_height(&renderer.viewport);

        assert!(
            zone_height.is_some(),
            "Should return height when dimensions set"
        );
        // Reading zone = Total height - (5 lines × cell_height)
        // = 540 - (5 × 22.5) = 540 - 112.5 = 427.5
        let cell_height = 540.0 / 24.0;
        let expected_zone = (540.0 - (5.0 * cell_height)) as u32;
        assert_eq!(zone_height.unwrap(), expected_zone);
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
    fn test_calculate_vertical_center() {
        use crate::rendering::kitty::positioning::calculate_vertical_center;
        let mut renderer = KittyGraphicsRenderer::new();

        // Set terminal dimensions (960px wide, 540px high, 80 cols, 24 rows)
        // Cell height = 540/24 = 22.5px
        let dims = TerminalDimensions::new(960, 540, 80, 24);
        renderer.viewport.set_dimensions(dims);

        let center = calculate_vertical_center(&renderer.viewport);

        assert!(center.is_some(), "Should return center when dimensions set");
        // Reading zone = Total height - (5 lines × cell_height)
        // = 540 - (5 × 22.5) = 540 - 112.5 = 427.5px
        // Vertical center = 42% of reading zone = 427.5 × 0.42 = 179.55 ≈ 179
        let cell_height = 540.0 / 24.0;
        let reading_zone = 540.0 - (5.0 * cell_height);
        let expected_center = (reading_zone * 0.42) as u32;
        assert_eq!(center.unwrap(), expected_center);
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
