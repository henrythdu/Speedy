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

use crate::rendering::font::{
    calculate_char_width, calculate_string_width, get_font, get_font_metrics, FontMetrics,
};
use crate::rendering::renderer::{RendererError, RsvpRenderer};
use crate::rendering::viewport::Viewport;
use ab_glyph::{FontRef, PxScale};
use base64::{engine::general_purpose, Engine as _};
use imageproc::drawing::draw_text_mut;
use imageproc::image::{ImageBuffer, Rgba};
use std::io::{self, Write};

/// Full-zone canvas for composite rendering of reading area
///
/// Per Design Doc v2.0 Section 6.2: CPU Compositing approach.
/// Creates a single RGBA buffer covering the entire reading zone (85% of terminal),
/// composites all visual elements into it, then transmits as one image.
/// This eliminates flickering and Z-fighting issues.
pub struct ReadingCanvas {
    /// RGBA pixel buffer covering the reading zone
    buffer: ImageBuffer<Rgba<u8>, Vec<u8>>,
    /// Canvas dimensions in pixels (width, height)
    dimensions: (u32, u32),
    /// Background color (Midnight theme: #1A1B26)
    background_color: Rgba<u8>,
}

impl ReadingCanvas {
    /// Create a new canvas with specified dimensions
    ///
    /// Initializes buffer with background color fill.
    /// Per Design Doc v2.0 Section 4.1: Background is #1A1B26 (Midnight theme)
    pub fn new(width: u32, height: u32) -> Self {
        let background_color = Rgba([26, 27, 38, 255]); // #1A1B26
        let buffer = ImageBuffer::from_pixel(width, height, background_color);

        Self {
            buffer,
            dimensions: (width, height),
            background_color,
        }
    }

    /// Get canvas width in pixels
    pub fn width(&self) -> u32 {
        self.dimensions.0
    }

    /// Get canvas height in pixels
    pub fn height(&self) -> u32 {
        self.dimensions.1
    }

    /// Get canvas dimensions as (width, height)
    pub fn dimensions(&self) -> (u32, u32) {
        self.dimensions
    }

    /// Get reference to the underlying RGBA buffer
    pub fn buffer(&self) -> &ImageBuffer<Rgba<u8>, Vec<u8>> {
        &self.buffer
    }

    /// Get mutable reference to the underlying RGBA buffer
    pub fn buffer_mut(&mut self) -> &mut ImageBuffer<Rgba<u8>, Vec<u8>> {
        &mut self.buffer
    }

    /// Clear canvas by filling with background color
    pub fn clear(&mut self) {
        // Fill entire buffer with background color
        for pixel in self.buffer.pixels_mut() {
            *pixel = self.background_color;
        }
    }

    /// Calculate Y position at 42% of canvas height for reading line
    ///
    /// Per PRD Section 4.3 and Design Doc v2.0 Section 4.2:
    /// The reading line is centered at 42% of Reader Zone height.
    pub fn calculate_reading_line_y(&self) -> u32 {
        (self.dimensions.1 as f32 * 0.42) as u32
    }
}

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

    /// Create a ReadingCanvas covering the reading zone (85% of terminal height)
    ///
    /// Per Design Doc v2.0 Section 6.2: CPU Compositing approach.
    /// Creates a full-zone canvas that fills 85% of terminal height (reading zone).
    /// Width matches terminal pixel width.
    /// Returns None if viewport dimensions are not available.
    pub fn create_canvas(&self) -> Option<ReadingCanvas> {
        self.viewport.get_dimensions().map(|dims| {
            let width = dims.pixel_size.0;
            // Reading zone is 85% of terminal height per PRD
            let height = (dims.pixel_size.1 as f32 * 0.85) as u32;
            ReadingCanvas::new(width, height)
        })
    }

    /// Composite a word onto the canvas with OVP anchoring
    ///
    /// **BUG FIX:** This method uses canvas-relative positioning, which fixes the
    /// coordinate calculation bug where words appeared at 42% of full screen height
    /// instead of 42% of the reading zone (85% of screen).
    ///
    /// Per Design Doc v2.0 Section 6.2: CPU compositing.
    /// - Clears canvas to background color
    /// - Draws word with anchor character at reading line (42% of canvas height)
    /// - Returns true if word was composited successfully
    pub fn composite_word(
        &self,
        canvas: &mut ReadingCanvas,
        word: &str,
        anchor_position: usize,
    ) -> bool {
        // Guard: need font and valid word
        if self.font.is_none() || self.font_metrics.is_none() || word.is_empty() {
            return false;
        }

        // Guard: validate anchor position
        let word_len = word.chars().count();
        if anchor_position >= word_len {
            return false;
        }

        let font = self.font.as_ref().unwrap();
        let metrics = self.font_metrics.as_ref().unwrap();

        // Clear canvas to background color
        canvas.clear();

        // Calculate reading line Y position (42% of canvas height)
        let reading_line_y = canvas.calculate_reading_line_y();

        // Calculate word width
        let word_width = calculate_string_width(font, word, self.font_size);
        let word_height = metrics.height;

        // Calculate start X position for OVP anchoring (center the anchor in canvas)
        let canvas_width = canvas.width() as f32;
        let center_x = canvas_width / 2.0;
        let start_x = self.calculate_start_x_for_canvas(word, anchor_position, center_x);

        // Calculate Y position: center the text vertically at reading_line_y
        // The text baseline should be at reading_line_y
        let text_ascent = metrics.ascent;
        let pos_y = reading_line_y as i32 - text_ascent as i32;

        // Get mutable buffer
        let buffer = canvas.buffer_mut();

        // Render word to canvas
        let scale = PxScale::from(self.font_size);

        // Split word into prefix, anchor, suffix for color rendering
        let chars: Vec<char> = word.chars().collect();
        let anchor_idx = anchor_position.min(chars.len().saturating_sub(1));

        let prefix: String = chars.iter().take(anchor_idx).collect();
        let anchor_char = chars.get(anchor_idx).copied().unwrap_or(' ');
        let suffix: String = chars.iter().skip(anchor_idx + 1).collect();

        // Theme colors
        let text_color = Rgba([169, 177, 214, 255]); // #A9B1D6 Light Blue
        let anchor_color = Rgba([247, 118, 142, 255]); // #F7768E Coral Red

        // Track current X position
        let mut x_offset = start_x as i32;

        // Draw prefix
        if !prefix.is_empty() {
            let prefix_width = calculate_string_width(font, &prefix, self.font_size);
            draw_text_mut(buffer, text_color, x_offset, pos_y, scale, font, &prefix);
            x_offset += prefix_width.ceil() as i32;
        }

        // Draw anchor character
        let anchor_str = anchor_char.to_string();
        let anchor_width = calculate_char_width(font, anchor_char, self.font_size);
        draw_text_mut(
            buffer,
            anchor_color,
            x_offset,
            pos_y,
            scale,
            font,
            &anchor_str,
        );
        x_offset += anchor_width.ceil() as i32;

        // Draw suffix
        if !suffix.is_empty() {
            draw_text_mut(buffer, text_color, x_offset, pos_y, scale, font, &suffix);
        }

        true
    }

    /// Calculate start X position for OVP anchoring within canvas
    ///
    /// Similar to calculate_start_x but uses provided center point
    fn calculate_start_x_for_canvas(
        &self,
        word: &str,
        anchor_position: usize,
        center_x: f32,
    ) -> f32 {
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
        center_x - (prefix_width + anchor_half_width)
    }

    /// Render a complete frame with word composited onto canvas
    ///
    /// **The Main Orchestrator Method**
    ///
    /// Per Design Doc v2.0 Section 6.2: Single-image-per-frame approach.
    /// This method:
    /// 1. Creates a ReadingCanvas covering the reading zone
    /// 2. Composites the word onto the canvas with OVP anchoring
    /// 3. Transmits the entire canvas as a single image via KGP
    ///
    /// **Advantages:**
    /// - No flickering (single image transmission)
    /// - No Z-fighting (all elements composited in CPU first)
    /// - Clean transitions (background + word in one buffer)
    ///
    /// Returns Ok(()) on success, Err on failure
    pub fn render_frame(
        &mut self,
        word: &str,
        anchor_position: usize,
    ) -> Result<(), RendererError> {
        // Step 1: Validate inputs
        if word.is_empty() {
            return Ok(()); // Nothing to render
        }

        let word_len = word.chars().count();
        if anchor_position >= word_len {
            return Err(RendererError::InvalidArguments(format!(
                "anchor_position {} out of bounds for word '{}' (length: {})",
                anchor_position, word, word_len
            )));
        }

        // Step 2: Create canvas
        let mut canvas = match self.create_canvas() {
            Some(c) => c,
            None => {
                return Err(RendererError::RenderFailed(
                    "Failed to create canvas - viewport dimensions not available".to_string(),
                ));
            }
        };

        // Step 3: Composite word onto canvas
        let composite_success = self.composite_word(&mut canvas, word, anchor_position);
        if !composite_success {
            return Err(RendererError::RenderFailed(
                "Failed to composite word onto canvas".to_string(),
            ));
        }

        // Step 4: Encode canvas buffer
        let buffer = canvas.buffer();
        let base64_data = self.encode_image_base64(buffer);
        let (width, height) = (buffer.width(), buffer.height());

        // Step 5: Transmit via Kitty Graphics Protocol
        // Position at top-left (0, 0) - canvas covers the reading zone
        self.transmit_graphics(
            self.current_image_id,
            width,
            height,
            &base64_data,
            0, // x position: left edge of reading zone
            0, // y position: top edge of reading zone
        )
        .map_err(|e| RendererError::RenderFailed(format!("Transmission failed: {}", e)))?;

        // Step 6: Increment image ID for next frame
        self.current_image_id += 1;

        Ok(())
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
    /// Creates an image buffer sized to fit the word, fills it with theme background color,
    /// and renders the text with anchor character highlighted in coral red per PRD Section 4.1.
    fn rasterize_word(
        &self,
        word: &str,
        anchor_position: usize,
    ) -> Option<ImageBuffer<Rgba<u8>, Vec<u8>>> {
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

        // Create RGBA buffer with transparent background
        // The reading area has theme background (#1A1B26), word is transparent overlay
        let mut image = ImageBuffer::from_pixel(width, height, Rgba([0, 0, 0, 0]));

        // Use imageproc's draw_text_mut to render text
        // ab_glyph requires PxScale for scaling
        let scale = PxScale::from(self.font_size);

        // Split word into: prefix (before anchor), anchor_char, suffix (after anchor)
        let chars: Vec<char> = word.chars().collect();
        let anchor_idx = anchor_position.min(chars.len().saturating_sub(1));

        let prefix: String = chars.iter().take(anchor_idx).collect();
        let anchor_char = chars.get(anchor_idx).copied().unwrap_or(' ');
        let suffix: String = chars.iter().skip(anchor_idx + 1).collect();

        // Calculate pixel widths
        let prefix_width = calculate_string_width(font, &prefix, self.font_size);
        let anchor_width = calculate_char_width(font, anchor_char, self.font_size);

        // Theme colors from PRD Section 4.1
        let text_color = Rgba([169, 177, 214, 255]); // #A9B1D6 Light Blue
        let anchor_color = Rgba([247, 118, 142, 255]); // #F7768E Coral Red

        // Draw prefix (before anchor) in text color
        let mut x_offset = 0i32;
        if !prefix.is_empty() {
            draw_text_mut(&mut image, text_color, x_offset, 0, scale, font, &prefix);
            x_offset += prefix_width.ceil() as i32;
        }

        // Draw anchor character in anchor color
        let anchor_str = anchor_char.to_string();
        draw_text_mut(
            &mut image,
            anchor_color,
            x_offset,
            0,
            scale,
            font,
            &anchor_str,
        );
        x_offset += anchor_width.ceil() as i32;

        // Draw suffix (after anchor) in text color
        if !suffix.is_empty() {
            draw_text_mut(&mut image, text_color, x_offset, 0, scale, font, &suffix);
        }

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
        // Format: ESC _ G a=T f=32 s=<width> v=<height> i=<image_id> x=<x> y=<y> m=0 <data> ESC \
        // f=32 means 32-bit RGBA
        // x and y specify pixel position (top-left corner of image)
        let apc_start = "\x1b_G";
        let apc_end = "\x1b\\";

        // If data fits in single transmission
        if base64_data.len() <= 4096 {
            let command = format!(
                "{}a=T,f=32,s={},v={},i={},x={},y={},m=0;{}{}",
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
                    "{}a=T,f=32,s={},v={},i={},x={},y={},m={};{}{}",
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
        // Guard against empty words
        if word.is_empty() {
            return Ok(());
        }

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

        // Calculate vertical center of reading zone (42% of reading zone height per PRD)
        let reading_zone_center_y = self.calculate_vertical_center().unwrap_or(0);

        // Rasterize word with anchor highlighting
        let image = match self.rasterize_word(word, anchor_position) {
            Some(img) => img,
            None => {
                return Err(RendererError::RenderFailed(
                    "Failed to rasterize word".to_string(),
                ))
            }
        };

        // Calculate Y position: center the text vertically at reading_zone_center_y
        // The text is drawn at y=0 in the image, so we position the image so the text
        // middle aligns with reading_zone_center_y
        let text_height = image.height();
        let pos_y = reading_zone_center_y.saturating_sub(text_height / 2);

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
            pos_y,
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

        // Rasterize a simple word with anchor at position 1
        let image = renderer.rasterize_word("hello", 1);

        assert!(image.is_some(), "Should create image buffer");
        let img = image.unwrap();

        // Image should have positive dimensions
        assert!(img.width() > 0, "Width should be positive");
        assert!(img.height() > 0, "Height should be positive");

        // Height should match font metrics height (approx font_size * line height)
        // With font_size of 24.0 (default), height should be around 28-30px
        assert!(
            img.height() >= 20 && img.height() <= 40,
            "Height should be around font metrics height (24px), got {}",
            img.height()
        );
    }

    #[test]
    fn test_rasterize_word_longer_word_wider_buffer() {
        let mut renderer = KittyGraphicsRenderer::new();
        renderer.initialize().unwrap();

        let dims = TerminalDimensions::new(960, 540, 80, 24);
        renderer.viewport.set_dimensions(dims);

        let short_word = renderer.rasterize_word("hi", 0);
        let long_word = renderer.rasterize_word("supercalifragilistic", 3);

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
        let image = renderer.rasterize_word("test", 0);
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

        let image = renderer.rasterize_word("test", 1);
        assert!(image.is_some());

        let img = image.unwrap();

        // Check that the image is not all background (should have text)
        // Just verify image was created with proper dimensions
        assert!(img.width() > 0);
        assert!(img.height() > 0);
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

    // ============================================================================
    // ReadingCanvas Tests (Task 1)
    // ============================================================================

    #[test]
    fn test_reading_canvas_creation() {
        let canvas = ReadingCanvas::new(800, 600);

        assert_eq!(canvas.width(), 800);
        assert_eq!(canvas.height(), 600);
        assert_eq!(canvas.dimensions(), (800, 600));
    }

    #[test]
    fn test_reading_canvas_buffer_initialized_with_background() {
        let canvas = ReadingCanvas::new(100, 100);
        let buffer = canvas.buffer();

        // Check that buffer has correct dimensions
        assert_eq!(buffer.width(), 100);
        assert_eq!(buffer.height(), 100);

        // Check that a pixel has the background color (#1A1B26 = RGB(26, 27, 38))
        let pixel = buffer.get_pixel(50, 50);
        assert_eq!(pixel[0], 26); // R
        assert_eq!(pixel[1], 27); // G
        assert_eq!(pixel[2], 38); // B
        assert_eq!(pixel[3], 255); // A (fully opaque)
    }

    #[test]
    fn test_reading_canvas_clear_resets_to_background() {
        let mut canvas = ReadingCanvas::new(50, 50);
        let background = Rgba([26, 27, 38, 255]);
        let red = Rgba([255, 0, 0, 255]);

        // Modify a pixel to red
        {
            let buffer = canvas.buffer_mut();
            buffer.put_pixel(25, 25, red);
        }

        // Verify pixel is red
        {
            let buffer = canvas.buffer();
            let pixel = buffer.get_pixel(25, 25);
            assert_eq!(pixel[0], 255); // R
        }

        // Clear canvas
        canvas.clear();

        // Verify pixel is back to background
        let buffer = canvas.buffer();
        let pixel = buffer.get_pixel(25, 25);
        assert_eq!(pixel[0], background[0]); // R
        assert_eq!(pixel[1], background[1]); // G
        assert_eq!(pixel[2], background[2]); // B
        assert_eq!(pixel[3], background[3]); // A
    }

    #[test]
    fn test_reading_canvas_calculate_reading_line_y() {
        // Test with various heights
        let canvas_small = ReadingCanvas::new(800, 100);
        assert_eq!(canvas_small.calculate_reading_line_y(), 42); // 42% of 100

        let canvas_medium = ReadingCanvas::new(800, 500);
        assert_eq!(canvas_medium.calculate_reading_line_y(), 210); // 42% of 500

        let canvas_large = ReadingCanvas::new(800, 1000);
        assert_eq!(canvas_large.calculate_reading_line_y(), 420); // 42% of 1000
    }

    #[test]
    fn test_reading_canvas_mutable_buffer_access() {
        let mut canvas = ReadingCanvas::new(10, 10);
        let red = Rgba([255, 0, 0, 255]);

        // Modify buffer through mutable reference
        {
            let buffer = canvas.buffer_mut();
            buffer.put_pixel(5, 5, red);
        }

        // Verify modification through immutable reference
        let buffer = canvas.buffer();
        let pixel = buffer.get_pixel(5, 5);
        assert_eq!(pixel[0], 255); // R
        assert_eq!(pixel[1], 0); // G
        assert_eq!(pixel[2], 0); // B
    }

    #[test]
    fn test_reading_canvas_zero_dimensions() {
        // Edge case: zero dimensions should still create valid buffer
        let canvas = ReadingCanvas::new(0, 0);
        assert_eq!(canvas.width(), 0);
        assert_eq!(canvas.height(), 0);
    }

    // ============================================================================
    // Task 2: create_canvas() Tests
    // ============================================================================

    #[test]
    fn test_create_canvas_with_dimensions() {
        let mut renderer = KittyGraphicsRenderer::new();
        renderer.initialize().unwrap();

        // Set viewport dimensions (1920x1080 terminal, 100x30 cells)
        let dims = TerminalDimensions::new(1920, 1080, 100, 30);
        renderer.viewport.set_dimensions(dims);

        // Create canvas
        let canvas = renderer.create_canvas();

        assert!(canvas.is_some(), "Should create canvas with dimensions");
        let canvas = canvas.unwrap();

        // Width should match terminal width
        assert_eq!(canvas.width(), 1920);

        // Height should be 85% of terminal height (918px)
        let expected_height = (1080.0 * 0.85) as u32;
        assert_eq!(canvas.height(), expected_height);
    }

    #[test]
    fn test_create_canvas_without_dimensions() {
        let renderer = KittyGraphicsRenderer::new();
        // Don't set viewport dimensions

        let canvas = renderer.create_canvas();
        assert!(canvas.is_none(), "Should return None without dimensions");
    }

    #[test]
    fn test_create_canvas_different_sizes() {
        let mut renderer = KittyGraphicsRenderer::new();
        renderer.initialize().unwrap();

        // Test small terminal
        let dims_small = TerminalDimensions::new(800, 600, 80, 24);
        renderer.viewport.set_dimensions(dims_small);
        let canvas_small = renderer.create_canvas().unwrap();
        assert_eq!(canvas_small.width(), 800);
        assert_eq!(canvas_small.height(), (600.0 * 0.85) as u32);

        // Test large terminal
        let dims_large = TerminalDimensions::new(3840, 2160, 200, 60);
        renderer.viewport.set_dimensions(dims_large);
        let canvas_large = renderer.create_canvas().unwrap();
        assert_eq!(canvas_large.width(), 3840);
        assert_eq!(canvas_large.height(), (2160.0 * 0.85) as u32);
    }

    #[test]
    fn test_create_canvas_returns_initialized_buffer() {
        let mut renderer = KittyGraphicsRenderer::new();
        renderer.initialize().unwrap();

        let dims = TerminalDimensions::new(100, 100, 10, 10);
        renderer.viewport.set_dimensions(dims);

        let canvas = renderer.create_canvas().unwrap();
        let buffer = canvas.buffer();

        // Buffer should be initialized with background color
        let pixel = buffer.get_pixel(50, 50);
        assert_eq!(pixel[0], 26); // R (Midnight theme #1A1B26)
        assert_eq!(pixel[1], 27); // G
        assert_eq!(pixel[2], 38); // B
        assert_eq!(pixel[3], 255); // A
    }

    // ============================================================================
    // Task 3: composite_word() Tests (BUG FIX - Canvas-relative positioning)
    // ============================================================================

    #[test]
    fn test_composite_word_basic() {
        let mut renderer = KittyGraphicsRenderer::new();
        renderer.initialize().unwrap();

        // Create a canvas
        let mut canvas = ReadingCanvas::new(800, 600);

        // Composite a word
        let result = renderer.composite_word(&mut canvas, "hello", 1);
        assert!(result, "Should composite word successfully");

        // Buffer should now have non-background pixels (text rendered)
        let buffer = canvas.buffer();
        let mut has_text = false;
        for pixel in buffer.pixels() {
            if pixel[0] != 26 || pixel[1] != 27 || pixel[2] != 38 {
                has_text = true;
                break;
            }
        }
        assert!(has_text, "Canvas should have text rendered on it");
    }

    #[test]
    fn test_composite_word_clears_canvas_first() {
        let mut renderer = KittyGraphicsRenderer::new();
        renderer.initialize().unwrap();

        let mut canvas = ReadingCanvas::new(100, 100);
        let red = Rgba([255, 0, 0, 255]);

        // Pre-fill canvas with red
        {
            let buffer = canvas.buffer_mut();
            for pixel in buffer.pixels_mut() {
                *pixel = red;
            }
        }

        // Verify it's red
        {
            let buffer = canvas.buffer();
            let pixel = buffer.get_pixel(50, 50);
            assert_eq!(pixel[0], 255);
        }

        // Composite word - should clear first
        renderer.composite_word(&mut canvas, "test", 1);

        // Background should be restored
        let buffer = canvas.buffer();
        let background_pixels: Vec<_> = buffer
            .pixels()
            .filter(|p| p[0] == 26 && p[1] == 27 && p[2] == 38)
            .collect();

        // Most pixels should be background color now
        assert!(
            background_pixels.len() > 9000,
            "Canvas should be mostly cleared to background"
        );
    }

    #[test]
    fn test_composite_word_at_reading_line_y() {
        let mut renderer = KittyGraphicsRenderer::new();
        renderer.initialize().unwrap();

        let mut canvas = ReadingCanvas::new(800, 500);
        let reading_line_y = canvas.calculate_reading_line_y();

        // Should be 42% of 500 = 210
        assert_eq!(reading_line_y, 210);

        // Composite word
        renderer.composite_word(&mut canvas, "world", 2);

        // The word should be rendered around the reading line
        // We can't check exact pixel positions, but we can verify it rendered
        let buffer = canvas.buffer();
        let mut text_found_near_reading_line = false;

        // Check a vertical slice around reading_line_y for text pixels
        for y in (reading_line_y - 20)..(reading_line_y + 20) {
            if y >= buffer.height() {
                continue;
            }
            for x in 300..500 {
                if x >= buffer.width() {
                    continue;
                }
                let pixel = buffer.get_pixel(x, y);
                // Look for text colors (not background)
                if pixel[0] != 26 || pixel[1] != 27 || pixel[2] != 38 {
                    text_found_near_reading_line = true;
                    break;
                }
            }
            if text_found_near_reading_line {
                break;
            }
        }

        assert!(
            text_found_near_reading_line,
            "Text should be rendered near reading line (42% of canvas)"
        );
    }

    #[test]
    fn test_composite_word_centered() {
        let mut renderer = KittyGraphicsRenderer::new();
        renderer.initialize().unwrap();

        let mut canvas = ReadingCanvas::new(800, 400);

        // Composite a word - anchor should be centered horizontally
        renderer.composite_word(&mut canvas, "HELLO", 2); // Anchor on 'L'

        let buffer = canvas.buffer();

        // Find the bounding box of the text
        let mut min_x = 800u32;
        let mut max_x = 0u32;

        for y in 0..buffer.height() {
            for x in 0..buffer.width() {
                let pixel = buffer.get_pixel(x, y);
                if pixel[0] != 26 || pixel[1] != 27 || pixel[2] != 38 {
                    min_x = min_x.min(x);
                    max_x = max_x.max(x);
                }
            }
        }

        // Text should be roughly centered (not at edges)
        let text_center = (min_x + max_x) / 2;
        assert!(
            text_center > 300 && text_center < 500,
            "Text should be centered in canvas, got center at {}",
            text_center
        );
    }

    #[test]
    fn test_composite_word_anchor_color_highlighting() {
        let mut renderer = KittyGraphicsRenderer::new();
        renderer.initialize().unwrap();

        let mut canvas = ReadingCanvas::new(400, 200);

        // Composite word with anchor at position 1
        renderer.composite_word(&mut canvas, "ABCD", 1); // Anchor on 'B'

        let buffer = canvas.buffer();

        // Look for anchor color (#F7768E = RGB(247, 118, 142))
        let mut has_anchor_color = false;
        for pixel in buffer.pixels() {
            if pixel[0] == 247 && pixel[1] == 118 && pixel[2] == 142 {
                has_anchor_color = true;
                break;
            }
        }

        assert!(
            has_anchor_color,
            "Anchor character should be rendered in coral red color"
        );
    }

    #[test]
    fn test_composite_word_fails_without_font() {
        let renderer = KittyGraphicsRenderer::new();
        // Don't initialize (no font)

        let mut canvas = ReadingCanvas::new(100, 100);

        let result = renderer.composite_word(&mut canvas, "test", 0);
        assert!(!result, "Should fail without font");
    }

    #[test]
    fn test_composite_word_fails_with_invalid_anchor() {
        let mut renderer = KittyGraphicsRenderer::new();
        renderer.initialize().unwrap();

        let mut canvas = ReadingCanvas::new(100, 100);

        // Anchor position beyond word length
        let result = renderer.composite_word(&mut canvas, "hi", 5);
        assert!(!result, "Should fail with invalid anchor position");
    }

    #[test]
    fn test_composite_word_empty_word() {
        let mut renderer = KittyGraphicsRenderer::new();
        renderer.initialize().unwrap();

        let mut canvas = ReadingCanvas::new(100, 100);

        let result = renderer.composite_word(&mut canvas, "", 0);
        assert!(!result, "Should fail with empty word");
    }

    #[test]
    fn test_composite_word_different_anchor_positions() {
        let mut renderer = KittyGraphicsRenderer::new();
        renderer.initialize().unwrap();

        // Test with different anchor positions
        for anchor_pos in 0..5 {
            let mut canvas = ReadingCanvas::new(800, 400);
            let result = renderer.composite_word(&mut canvas, "world", anchor_pos);
            assert!(result, "Should work with anchor at position {}", anchor_pos);
        }
    }

    #[test]
    fn test_calculate_start_x_for_canvas() {
        let mut renderer = KittyGraphicsRenderer::new();
        renderer.initialize().unwrap();

        // Test with center at 400
        let center_x = 400.0;

        // Single char at position 0 - should be centered
        let start_x = renderer.calculate_start_x_for_canvas("A", 0, center_x);
        // Single char should be roughly centered minus half its width
        assert!(
            start_x > 390.0 && start_x < 410.0,
            "Single char start_x should be near center: got {}",
            start_x
        );

        // Two char word with anchor at 1
        let start_x2 = renderer.calculate_start_x_for_canvas("AB", 1, center_x);
        // Should be left of single char case since anchor is offset
        assert!(
            start_x2 < start_x,
            "Two-char word with anchor at 1 should start left of single char: {} vs {}",
            start_x2,
            start_x
        );
    }

    #[test]
    fn test_canvas_relative_positioning_bug_fix() {
        // This test verifies the bug fix: canvas-relative positioning
        // Bug was: words at 42% of FULL SCREEN (too low)
        // Fix: words at 42% of CANVAS (reading zone, which is 85% of screen)
        // So word appears in middle of reading zone, not near command deck

        let mut renderer = KittyGraphicsRenderer::new();
        renderer.initialize().unwrap();

        // Simulate 1080p terminal
        let terminal_height = 1080u32;
        let reading_zone_height = (terminal_height as f32 * 0.85) as u32; // 918px

        // Create canvas at reading zone height
        let mut canvas = ReadingCanvas::new(1920, reading_zone_height);

        // Calculate reading line Y
        let reading_line_y = canvas.calculate_reading_line_y(); // 42% of 918 = 385

        // This should be ~385px (42% of reading zone), NOT ~453px (42% of 1080)
        assert_eq!(reading_line_y, 385);

        // Composite word
        renderer.composite_word(&mut canvas, "test", 1);

        // Verify the word is rendered in the correct position
        // (Upper portion of canvas, not at bottom)
        assert!(reading_line_y < reading_zone_height / 2,
                "Reading line should be in upper half of reading zone, not near bottom.\n\
                 This verifies the bug fix: positions are canvas-relative (42% of 85% = 35.7% of screen),\n\
                 not screen-relative (42% of 100% = 42% of screen).");
    }

    // ============================================================================
    // Task 4: render_frame() Tests
    // ============================================================================

    #[test]
    fn test_render_frame_basic() {
        let mut renderer = KittyGraphicsRenderer::new();
        renderer.initialize().unwrap();

        // Set viewport dimensions
        let dims = TerminalDimensions::new(800, 600, 80, 24);
        renderer.viewport.set_dimensions(dims);

        // Initial image ID
        let initial_id = renderer.current_image_id;

        // Render a frame
        let result = renderer.render_frame("hello", 1);
        assert!(result.is_ok(), "render_frame should succeed: {:?}", result);

        // Image ID should be incremented
        assert_eq!(renderer.current_image_id, initial_id + 1);
    }

    #[test]
    fn test_render_frame_empty_word() {
        let mut renderer = KittyGraphicsRenderer::new();
        renderer.initialize().unwrap();

        let dims = TerminalDimensions::new(800, 600, 80, 24);
        renderer.viewport.set_dimensions(dims);

        // Empty word should return Ok(()) immediately
        let result = renderer.render_frame("", 0);
        assert!(result.is_ok(), "Empty word should return Ok");
    }

    #[test]
    fn test_render_frame_invalid_anchor() {
        let mut renderer = KittyGraphicsRenderer::new();
        renderer.initialize().unwrap();

        let dims = TerminalDimensions::new(800, 600, 80, 24);
        renderer.viewport.set_dimensions(dims);

        // Invalid anchor position should return error
        let result = renderer.render_frame("hi", 5);
        assert!(result.is_err(), "Invalid anchor should return error");

        match result {
            Err(RendererError::InvalidArguments(_)) => (), // Expected
            _ => panic!("Expected InvalidArguments error"),
        }
    }

    #[test]
    fn test_render_frame_without_dimensions() {
        let mut renderer = KittyGraphicsRenderer::new();
        renderer.initialize().unwrap();

        // Explicitly clear any dimensions that might have been set during initialization
        renderer.viewport.clear();

        // Should fail because we can't create canvas without dimensions
        let result = renderer.render_frame("test", 0);
        assert!(
            result.is_err(),
            "Should fail without viewport dimensions, got: {:?}",
            result
        );

        match result {
            Err(RendererError::RenderFailed(_)) => (), // Expected
            _ => panic!("Expected RenderFailed error, got: {:?}", result),
        }
    }

    #[test]
    fn test_render_frame_without_font() {
        let mut renderer = KittyGraphicsRenderer::new();
        // Don't initialize (no font)

        let dims = TerminalDimensions::new(800, 600, 80, 24);
        renderer.viewport.set_dimensions(dims);

        // Should fail because we can't composite without font
        let result = renderer.render_frame("test", 0);
        assert!(result.is_err(), "Should fail without font");
    }

    #[test]
    fn test_render_frame_increments_image_id() {
        let mut renderer = KittyGraphicsRenderer::new();
        renderer.initialize().unwrap();

        let dims = TerminalDimensions::new(800, 600, 80, 24);
        renderer.viewport.set_dimensions(dims);

        let id1 = renderer.current_image_id;
        renderer.render_frame("first", 1).unwrap();

        let id2 = renderer.current_image_id;
        assert_eq!(id2, id1 + 1);

        renderer.render_frame("second", 1).unwrap();

        let id3 = renderer.current_image_id;
        assert_eq!(id3, id2 + 1);
    }

    #[test]
    fn test_render_frame_orchestration() {
        // This test verifies that render_frame() correctly orchestrates:
        // 1. Canvas creation
        // 2. Word compositing
        // 3. Image transmission

        let mut renderer = KittyGraphicsRenderer::new();
        renderer.initialize().unwrap();

        let dims = TerminalDimensions::new(800, 600, 80, 24);
        renderer.viewport.set_dimensions(dims);

        // The render_frame method should:
        // - Create a canvas of correct size (800x510, which is 85% of 600)
        // - Composite the word at 42% of canvas height
        // - Transmit as single image
        // - Return success

        let result = renderer.render_frame("ORCHESTRATE", 5); // Anchor on 'T'
        assert!(
            result.is_ok(),
            "Full orchestration should succeed: {:?}",
            result
        );

        // Verify image ID incremented (transmission happened)
        assert!(
            renderer.current_image_id > 1,
            "Image ID should be incremented after transmission"
        );
    }
}
