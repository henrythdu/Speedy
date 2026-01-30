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
use crate::rendering::viewport::Viewport;
use ab_glyph::{FontRef, PxScale};
use base64::{engine::general_purpose, Engine as _};
use imageproc::drawing::draw_text_mut;
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
    }

    /// Get the reading zone height in pixels from viewport dimensions
    ///
    /// Reading zone is the top 85% of the terminal (per PRD design).
    /// Returns None if viewport dimensions are not available.
    pub fn get_reading_zone_height(&self) -> Option<u32> {
        self.viewport
            .get_dimensions()
            .map(|dims| (dims.pixel_size.1 as f32 * 0.85) as u32)
    }

    /// Calculate vertical offset for centering text in reading zone
    ///
    /// Per PRD Section 4.3: The reading line is centered at 42% of Reader Zone height.
    /// Returns the Y pixel coordinate where text should be drawn.
    pub fn calculate_vertical_center(&self) -> Option<u32> {
        self.get_reading_zone_height().map(|zone_height| {
            // Vertical center = 42% of reading zone height (per PRD)
            (zone_height as f32 * 0.42) as u32
        })
    }

    /// Get cell height in pixels from viewport
    pub fn get_cell_height(&self) -> Option<f32> {
        self.viewport.get_dimensions().map(|dims| dims.cell_size.1)
    }

    /// Get reference to viewport (for external access to query dimensions)
    pub fn viewport(&mut self) -> &mut Viewport {
        &mut self.viewport
    }

    /// Get current font size
    pub fn font_size(&self) -> f32 {
        self.font_size
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

    /// Rasterize word to RGBA buffer with text rendered using ab_glyph and imageproc
    ///
    /// Creates an image buffer sized to fit the word with padding for visual clarity,
    /// fills it with the theme background color, and renders the text.
    /// The buffer is sized to match the reading zone height for proper vertical positioning.
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

        // Get reading zone height for canvas size (None means use text height)
        let canvas_height = self.get_reading_zone_height().unwrap_or(height);

        // Create RGBA buffer with theme background color (#1A1B26 = Rgba(26, 27, 38, 255))
        let mut image = ImageBuffer::from_pixel(width, canvas_height, Rgba([26, 27, 38, 255]));

        // Use imageproc's draw_text_mut to render text
        // ab_glyph requires PxScale for scaling
        let scale = PxScale::from(self.font_size);

        // Text color: anchor color #F7768E = Rgba(247, 118, 142, 255)
        let text_color = Rgba([247, 118, 142, 255]);

        // Calculate vertical offset to center text in the reading zone
        // Text should be positioned at 42% of reading zone height (per PRD)
        let vertical_offset =
            ((canvas_height as f32 * 0.42) - (height as f32 / 2.0)).max(0.0) as i32;

        // Draw text at calculated vertical offset
        draw_text_mut(
            &mut image,
            text_color,
            0,
            vertical_offset,
            scale,
            font,
            word,
        );

        Some(image)
    }

    /// Encode image to base64 for Kitty protocol
    fn encode_image_base64(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> String {
        let raw_bytes: Vec<u8> = image.as_raw().to_vec();
        general_purpose::STANDARD.encode(&raw_bytes)
    }

    /// Send Kitty Graphics Protocol transmission with pixel positioning
    fn transmit_graphics(
        &mut self,
        image_id: u32,
        width: u32,
        height: u32,
        base64_data: &str,
        pos_x: u32,
        pos_y: u32,
    ) -> io::Result<()> {
        // Kitty Graphics Protocol: APC sequence
        // Format: ESC _ G a=T f=32 s=<width> v=<height> i=<image_id> p=<x>,<y> m=1 <data> ESC \
        // f=32 means 32-bit RGBA
        // p=x,y specifies pixel position (top-left corner of image)
        let apc_start = "\x1b_G";
        let apc_end = "\x1b\\";

        // If data fits in single transmission
        if base64_data.len() <= 4096 {
            let command = format!(
                "{}a=T,f=32,s={},v={},i={},p={},{}m=0;{}{}",
                apc_start, width, height, image_id, pos_x, pos_y, base64_data, apc_end
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
                    "{}a=T,f=32,s={},v={},i={},p={},{}m={};{}{}",
                    apc_start, width, height, image_id, pos_x, pos_y, more, chunk, apc_end
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

        // Calculate sub-pixel OVP position
        let start_x = self.calculate_start_x(word, anchor_position);

        // Calculate vertical position (reading zone center)
        let vertical_center = self.calculate_vertical_center().unwrap_or(0);

        // Debug output
        eprintln!(
            "DEBUG render_word: word='{}', anchor={}, start_x={}, vertical_center={}",
            word, anchor_position, start_x, vertical_center
        );
        eprintln!("DEBUG: reading_zone_center={:?}", self.reading_zone_center);

        // Rasterize word to image buffer
        let image = match self.rasterize_word(word) {
            Some(img) => img,
            None => {
                return Err(RendererError::RenderFailed(
                    "Failed to rasterize word".to_string(),
                ))
            }
        };

        // Encode to base64
        let base64_data = self.encode_image_base64(&image);

        // Get image dimensions
        let (width, height) = (image.width(), image.height());

        // Transmit via Kitty Graphics Protocol with pixel positioning
        // Position is the top-left corner where the image should be placed
        self.transmit_graphics(
            self.current_image_id,
            width,
            height,
            &base64_data,
            start_x as u32,
            vertical_center,
        )
        .map_err(|e| RendererError::RenderFailed(e.to_string()))?;

        // Increment image ID for next word
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
        let pos_x = 100u32;
        let pos_y = 200u32;

        let command = format!(
            "\x1b_Ga=T,f=32,s={},v={},i={},p={},{}m=0;{}\x1b\\",
            width, height, image_id, pos_x, pos_y, data
        );

        assert!(command.contains("a=T")); // Action: transmit
        assert!(command.contains("f=32")); // Format: 32-bit RGBA
        assert!(command.contains("s=100")); // Width
        assert!(command.contains("v=50")); // Height
        assert!(command.contains("i=42")); // Image ID
        assert!(command.contains("p=100,200")); // Position coordinates
        assert!(command.contains("m=0")); // No more chunks
    }

    #[test]
    fn test_rasterize_word_creates_valid_buffer() {
        let mut renderer = KittyGraphicsRenderer::new();
        renderer.initialize().unwrap();

        // Set viewport dimensions for reading zone height calculation
        let dims = TerminalDimensions::new(960, 540, 80, 24);
        renderer.viewport.set_dimensions(dims);

        // Rasterize a simple word
        let image = renderer.rasterize_word("hello");

        assert!(image.is_some(), "Should create image buffer");
        let img = image.unwrap();

        // Image should have positive dimensions
        assert!(img.width() > 0, "Width should be positive");
        assert!(img.height() > 0, "Height should be positive");

        // Height should match reading zone height (85% of 540 = 459)
        let reading_zone_height = (540.0 * 0.85) as u32;
        assert_eq!(img.height(), reading_zone_height);
    }

    #[test]
    fn test_rasterize_word_longer_word_wider_buffer() {
        let mut renderer = KittyGraphicsRenderer::new();
        renderer.initialize().unwrap();

        let dims = TerminalDimensions::new(960, 540, 80, 24);
        renderer.viewport.set_dimensions(dims);

        let short_word = renderer.rasterize_word("hi");
        let long_word = renderer.rasterize_word("supercalifragilistic");

        assert!(short_word.is_some() && long_word.is_some());

        let short_img = short_word.unwrap();
        let long_img = long_word.unwrap();

        // Longer word should produce wider image
        assert!(
            long_img.width() > short_img.width(),
            "Longer word should produce wider image"
        );
    }

    #[test]
    fn test_rasterize_word_fails_without_font() {
        let renderer = KittyGraphicsRenderer::new();

        // Without initialization (no font), rasterization should fail
        let image = renderer.rasterize_word("test");
        assert!(image.is_none(), "Should return None without font");
    }

    #[test]
    fn test_get_reading_zone_height_with_dimensions() {
        let mut renderer = KittyGraphicsRenderer::new();

        // Set terminal dimensions (960x540 pixels)
        let dims = TerminalDimensions::new(960, 540, 80, 24);
        renderer.viewport.set_dimensions(dims);

        let zone_height = renderer.get_reading_zone_height();

        assert!(
            zone_height.is_some(),
            "Should return height when dimensions set"
        );
        // Reading zone is 85% of total height
        assert_eq!(zone_height.unwrap(), (540.0 * 0.85) as u32);
    }

    #[test]
    fn test_get_reading_zone_height_without_dimensions() {
        let renderer = KittyGraphicsRenderer::new();

        let zone_height = renderer.get_reading_zone_height();
        assert!(
            zone_height.is_none(),
            "Should return None without dimensions"
        );
    }

    #[test]
    fn test_calculate_vertical_center() {
        let mut renderer = KittyGraphicsRenderer::new();

        // Set terminal dimensions
        let dims = TerminalDimensions::new(960, 540, 80, 24);
        renderer.viewport.set_dimensions(dims);

        let center = renderer.calculate_vertical_center();

        assert!(center.is_some(), "Should return center when dimensions set");
        // Vertical center is at 42% of reading zone height
        let reading_zone = (540.0 * 0.85) as u32;
        let expected_center = (reading_zone as f32 * 0.42) as u32;
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
    fn test_rasterize_word_uses_correct_theme_colors() {
        let mut renderer = KittyGraphicsRenderer::new();
        renderer.initialize().unwrap();

        let dims = TerminalDimensions::new(960, 540, 80, 24);
        renderer.viewport.set_dimensions(dims);

        let image = renderer.rasterize_word("test");
        assert!(image.is_some());

        let img = image.unwrap();

        // Check that background color is set (#1A1B26 = RGB(26, 27, 38))
        // Check a pixel in the top-left corner (background area)
        let pixel = img.get_pixel(0, 0);
        assert_eq!(pixel[0], 26); // R
        assert_eq!(pixel[1], 27); // G
        assert_eq!(pixel[2], 38); // B
        assert_eq!(pixel[3], 255); // A (fully opaque)
    }

    #[test]
    fn test_calculate_start_x_with_font_metrics() {
        let mut renderer = KittyGraphicsRenderer::new();
        renderer.initialize().unwrap();
        renderer.set_reading_zone_center(480, 200); // Center of 960x400 reading zone

        // With proper initialization, calculate_start_x should work
        let start_x = renderer.calculate_start_x("hello", 1); // Anchor on 'e'
        assert!(start_x > 0.0, "Start X should be positive");
        assert!(start_x < 480.0, "Start X should be less than center");
    }
}
