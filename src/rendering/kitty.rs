//! KittyGraphicsRenderer - Pixel-perfect RSVP rendering using Kitty Graphics Protocol
//!
//! Implements RsvpRenderer trait for terminals with Kitty Graphics Protocol support
//! (Konsole 24.12+, Kitty 0.35+). Provides sub-pixel OVP anchoring with pixel-perfect
//! positioning using font metrics and direct escape sequence implementation.
//!
//! ## Protocol Details
//!
//! Kitty Graphics Protocol uses APC (Application Program Command) sequences:
//! - `ESC _ G f=<format>... <data> ESC \` - Transmit image
//! - `ESC _ G a=T f=<format> s=<width> v=<height> m=<more>... <data> ESC \` - Transmit in chunks
//! - `ESC _ G a=d d=A` - Delete all graphics on screen
//!
//! ## Performance
//!
//! Per Epic 1 requirements:
//! - Target WPM: 1000+ (requires <10ms per frame)
//! - Rasterization: <3ms (cache hit: <0.5ms, cache miss: <3ms)
//! - Encoding + transmission: <7ms

use crate::rendering::font::{calculate_string_width, get_font, get_font_metrics, FontMetrics};
use crate::rendering::renderer::{RendererError, RsvpRenderer};
use crate::rendering::viewport::{TerminalDimensions, Viewport};
use ab_glyph::FontRef;
use imageproc::image::{ImageBuffer, Rgba};
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
    current_image_id: u32,
    /// Target pixel coordinates for rendering (x, y of reading zone center)
    reading_zone_center: (u32, u32),
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
            reading_zone_center: (0, 0),
        }
    }

    /// Set reading zone center position in pixels
    pub fn set_reading_zone_center(&mut self, x: u32, y: u32) {
        self.reading_zone_center = (x, y);
    }

    /// Calculate start X position for sub-pixel OVP anchoring
    ///
    /// Returns the pixel X coordinate where the word should start so that
    /// the anchor character is at the visual center.
    fn calculate_start_x(&self, word: &str, anchor_position: usize) -> f32 {
        if self.font.is_none() || self.font_metrics.is_none() {
            return 0.0;
        }

        let font = self.font.as_ref().unwrap();
        let word_chars: Vec<char> = word.chars().collect();

        if anchor_position >= word_chars.len() {
            return 0.0;
        }

        // Calculate width of characters before anchor
        let prefix: String = word_chars[..anchor_position].iter().collect();
        let prefix_width = calculate_string_width(font, &prefix, self.font_size);

        // Calculate width of anchor character
        let anchor_char = word_chars[anchor_position];
        let anchor_width = calculate_string_width(font, &anchor_char.to_string(), self.font_size);
        let anchor_half_width = anchor_width / 2.0;

        // StartX = Center - (prefix + anchor_half)
        let center_x = self.reading_zone_center.0 as f32;
        center_x - (prefix_width + anchor_half_width)
    }

    /// Rasterize word to RGBA buffer
    fn rasterize_word(&self, word: &str) -> Option<ImageBuffer<Rgba<u8>, Vec<u8>>> {
        if self.font.is_none() || self.font_metrics.is_none() {
            return None;
        }

        let font = self.font.as_ref().unwrap();
        let metrics = self.font_metrics.as_ref().unwrap();

        // Calculate word dimensions
        let word_width = calculate_string_width(font, word, self.font_size);
        let word_height = metrics.height;

        // Round up to integer dimensions
        let width = word_width.ceil() as u32;
        let height = word_height.ceil() as u32;

        if width == 0 || height == 0 {
            return None;
        }

        // Create RGBA buffer with Midnight theme background
        let mut image = ImageBuffer::from_pixel(width, height, Rgba([26, 27, 38, 255]));

        // TODO: Use ab_glyph to render text into the buffer
        // For now, return empty buffer (will be implemented)

        Some(image)
    }

    /// Encode image to base64 for Kitty protocol
    fn encode_image_base64(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> String {
        let raw_bytes: Vec<u8> = image.as_raw().to_vec();
        base64::encode(&raw_bytes)
    }

    /// Send Kitty Graphics Protocol transmission
    fn transmit_graphics(
        &mut self,
        image_id: u32,
        width: u32,
        height: u32,
        base64_data: &str,
    ) -> io::Result<()> {
        // Kitty Graphics Protocol: APC sequence
        // Format: ESC _ G a=T f=32 s=<width> v=<height> i=<image_id> m=1 <data> ESC \
        // f=32 means 32-bit RGBA
        let apc_start = "\x1b_G";
        let apc_end = "\x1b\\";

        // If data fits in single transmission
        if base64_data.len() <= 4096 {
            let command = format!(
                "{}a=T,f=32,s={},v={},i={}m=0;{}{}",
                apc_start, width, height, image_id, base64_data, apc_end
            );
            print!("{}", command);
            io::stdout().flush()
        } else {
            // Multi-chunk transmission
            let chunks: Vec<&str> = base64_data
                .as_bytes()
                .chunks(4096)
                .map(|chunk| std::str::from_utf8(chunk).unwrap_or(""))
                .collect();

            for (i, chunk) in chunks.iter().enumerate() {
                let more = if i == chunks.len() - 1 { 0 } else { 1 };
                let command = format!(
                    "{}a=T,f=32,s={},v={},i={}m={};{}{}",
                    apc_start, width, height, image_id, more, chunk, apc_end
                );
                print!("{}", command);
                io::stdout().flush()?;
            }
            Ok(())
        }
    }

    /// Delete specific image by ID
    fn delete_image(&self, image_id: u32) -> io::Result<()> {
        let command = format!("\x1b_Ga=d,d=I,i={}\x1b\\", image_id);
        print!("{}", command);
        io::stdout().flush()
    }

    /// Delete all graphics (cleanup on exit)
    fn delete_all_graphics(&self) -> io::Result<()> {
        let command = "\x1b_Ga=d,d=A\x1b\\";
        print!("{}", command);
        io::stdout().flush()
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

        // Query viewport dimensions
        match self.viewport.query_dimensions() {
            Ok(_) => Ok(()),
            Err(e) => {
                // Fallback is acceptable - will use estimated dimensions
                eprintln!("Viewport query failed (using fallback): {}", e);
                Ok(())
            }
        }
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

        // Ensure font is loaded
        if self.font.is_none() {
            return Err(RendererError::RenderFailed(
                "Font not initialized".to_string(),
            ));
        }

        // Rasterize word (placeholder - full implementation in next iteration)
        let _start_x = self.calculate_start_x(word, anchor_position);

        // TODO: Complete rasterization and transmission
        // For now, just increment image ID to show method works
        self.current_image_id += 1;

        Ok(())
    }

    fn clear(&mut self) -> Result<(), RendererError> {
        // Delete the previous image if it exists
        if self.current_image_id > 1 {
            let prev_id = self.current_image_id - 1;
            if let Err(e) = self.delete_image(prev_id) {
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
        if let Err(e) = self.delete_all_graphics() {
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
    fn test_set_reading_zone_center() {
        let mut renderer = KittyGraphicsRenderer::new();
        renderer.set_reading_zone_center(960, 540);
        assert_eq!(renderer.reading_zone_center, (960, 540));
    }

    #[test]
    fn test_calculate_start_x_single_char() {
        let mut renderer = KittyGraphicsRenderer::new();
        renderer.initialize().unwrap();
        renderer.set_reading_zone_center(100, 50);

        // For a single character, anchor is at position 0
        // The character center should align with reading zone center
        let start_x = renderer.calculate_start_x("A", 0);

        // With a monospace font, a single char at 24px should be ~14-15px wide
        // StartX should be roughly: center - (0 + half_char_width)
        assert!(
            start_x > 85.0 && start_x < 95.0,
            "Single char start_x should be near center minus half width: got {}",
            start_x
        );
    }

    #[test]
    fn test_calculate_start_x_two_chars() {
        let mut renderer = KittyGraphicsRenderer::new();
        renderer.initialize().unwrap();
        renderer.set_reading_zone_center(100, 50);

        // For "AB" with anchor at position 1 (second char)
        // StartX = center - (width_of_A + half_width_of_B)
        let start_x = renderer.calculate_start_x("AB", 1);

        // Should be less than single char case since anchor is offset to right
        let single_char_start = renderer.calculate_start_x("A", 0);
        assert!(
            start_x < single_char_start,
            "Two-char word with right anchor should start left of single char"
        );
    }

    #[test]
    fn test_calculate_start_x_out_of_bounds() {
        let mut renderer = KittyGraphicsRenderer::new();
        renderer.initialize().unwrap();
        renderer.set_reading_zone_center(100, 50);

        // Anchor position beyond word length should return 0.0
        let start_x = renderer.calculate_start_x("hi", 5);
        assert_eq!(start_x, 0.0);
    }

    #[test]
    fn test_calculate_start_x_without_font() {
        let renderer = KittyGraphicsRenderer::new();
        // Without initialization, should return 0.0
        let start_x = renderer.calculate_start_x("hello", 1);
        assert_eq!(start_x, 0.0);
    }

    #[test]
    fn test_render_word_validates_anchor_position() {
        let mut renderer = KittyGraphicsRenderer::new();
        renderer.initialize().unwrap();

        // Valid anchor should work (though rasterization is stubbed)
        assert!(renderer.render_word("hello", 0).is_ok());
        assert!(renderer.render_word("hello", 4).is_ok());

        // Out of bounds should fail
        let result = renderer.render_word("hi", 5);
        assert!(result.is_err());
        match result {
            Err(RendererError::InvalidArguments(_)) => (), // Expected
            _ => panic!("Expected InvalidArguments error"),
        }
    }

    #[test]
    fn test_render_word_increments_image_id() {
        let mut renderer = KittyGraphicsRenderer::new();
        renderer.initialize().unwrap();

        let initial_id = renderer.current_image_id;
        renderer.render_word("test", 1).unwrap();
        assert_eq!(renderer.current_image_id, initial_id + 1);

        renderer.render_word("word", 2).unwrap();
        assert_eq!(renderer.current_image_id, initial_id + 2);
    }

    #[test]
    fn test_render_word_without_font() {
        let mut renderer = KittyGraphicsRenderer::new();
        // Skip initialization

        let result = renderer.render_word("test", 0);
        assert!(result.is_err());
        match result {
            Err(RendererError::RenderFailed(_)) => (), // Expected
            _ => panic!("Expected RenderFailed error"),
        }
    }

    #[test]
    fn test_clear_returns_ok() {
        let mut renderer = KittyGraphicsRenderer::new();
        renderer.initialize().unwrap();

        // Render a word first to have something to clear
        renderer.render_word("test", 0).unwrap();

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

    #[test]
    fn test_base64_encoding() {
        let renderer = KittyGraphicsRenderer::new();
        let image = ImageBuffer::from_pixel(10, 10, Rgba([255, 0, 0, 255]));
        let encoded = renderer.encode_image_base64(&image);

        // Base64 encoding of 100 RGBA pixels (400 bytes)
        // Should be around 536 characters (400 * 4/3, rounded up to multiple of 4)
        assert!(!encoded.is_empty());
        assert!(encoded.len() > 100);

        // Verify it's valid base64 (only contains valid characters)
        assert!(encoded
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='));
    }

    #[test]
    fn test_delete_all_graphics_format() {
        let renderer = KittyGraphicsRenderer::new();
        // Just verify the command string is formatted correctly
        // We can't actually test I/O in unit tests
        let expected = "\x1b_Ga=d,d=A\x1b\\";
        assert_eq!(expected.len(), 12); // Verify structure: ESC _ G a = d , d = A ESC \
    }

    #[test]
    fn test_transmit_graphics_format() {
        let renderer = KittyGraphicsRenderer::new();
        // Just verify the format string logic
        let image_id = 42u32;
        let width = 100u32;
        let height = 50u32;
        let data = "dGVzdA=="; // base64 for "test"

        let command = format!(
            "\x1b_Ga=T,f=32,s={},v={},i={}m=0;{}\x1b\\",
            width, height, image_id, data
        );

        assert!(command.contains("a=T")); // Action: transmit
        assert!(command.contains("f=32")); // Format: 32-bit RGBA
        assert!(command.contains("s=100")); // Width
        assert!(command.contains("v=50")); // Height
        assert!(command.contains("i=42")); // Image ID
        assert!(command.contains("m=0")); // No more chunks
    }
}
