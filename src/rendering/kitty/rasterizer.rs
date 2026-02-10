//! Word rasterization for Kitty Graphics rendering
//!
//! Converts text words to RGBA image buffers with OVP anchor highlighting.
//! Uses anti-aliased font rendering via ab_glyph for high-quality text.

use crate::rendering::font::{calculate_char_width, calculate_string_width, FontMetrics};
use ab_glyph::{point, Font, FontRef, PxScale, ScaleFont};
use imageproc::image::{ImageBuffer, Rgba};

/// Theme colors from PRD Section 4.1
pub const TEXT_COLOR: Rgba<u8> = Rgba([169, 177, 214, 255]); // #A9B1D6 Light Blue
pub const ANCHOR_COLOR: Rgba<u8> = Rgba([247, 118, 142, 255]); // #F7768E Coral Red

/// Rasterize word to RGBA buffer with anti-aliased text rendering
///
/// Creates an image buffer sized to fit the word, fills it with transparent background,
/// and renders the text with anchor character highlighted in coral red per PRD Section 4.1.
/// Uses ab_glyph's outline_glyph for high-quality anti-aliased rendering.
///
/// # Arguments
/// * `word` - The word to rasterize
/// * `anchor_position` - Character index that should be highlighted (0-based)
/// * `font` - Font reference for rendering
/// * `font_size` - Font size in pixels
/// * `metrics` - Font metrics for sizing
///
/// # Returns
/// RGBA image buffer containing the rendered word, or None if font not available
pub fn rasterize_word(
    word: &str,
    anchor_position: usize,
    font: &FontRef,
    font_size: f32,
    metrics: &FontMetrics,
) -> Option<ImageBuffer<Rgba<u8>, Vec<u8>>> {
    // Calculate word dimensions
    let word_width = calculate_string_width(font, word, font_size);
    let word_height = metrics.height;

    // Round up to integer dimensions with padding for anti-aliasing
    let width = word_width.ceil() as u32 + 2; // +2 for anti-aliasing edge
    let height = word_height.ceil() as u32 + 2;

    if width <= 2 || height <= 2 {
        return None;
    }

    // Create RGBA buffer with transparent background
    // The reading area has theme background (#1A1B26), word is transparent overlay
    let mut image = ImageBuffer::from_pixel(width, height, Rgba([0, 0, 0, 0]));

    // Split word into: prefix (before anchor), anchor_char, suffix (after anchor)
    let chars: Vec<char> = word.chars().collect();
    let anchor_idx = anchor_position.min(chars.len().saturating_sub(1));

    let prefix: String = chars.iter().take(anchor_idx).collect();
    let anchor_char = chars.get(anchor_idx).copied().unwrap_or(' ');
    let suffix: String = chars.iter().skip(anchor_idx + 1).collect();

    // Calculate pixel widths for positioning
    let prefix_width = calculate_string_width(font, &prefix, font_size);
    let anchor_width = calculate_char_width(font, anchor_char, font_size);

    // Get scaled font for glyph layout
    let scale = PxScale::from(font_size);
    let scaled_font = font.as_scaled(scale);

    // Baseline position (1px padding for anti-aliasing)
    let baseline_y = 1.0 + scaled_font.ascent();
    let mut x_offset = 1.0f32; // 1px left padding

    // Render prefix (before anchor) in text color
    if !prefix.is_empty() {
        render_text_anti_aliased(
            &mut image, &prefix, x_offset, baseline_y, font, scale, TEXT_COLOR,
        );
        x_offset += prefix_width;
    }

    // Render anchor character in anchor color
    render_text_anti_aliased(
        &mut image,
        &anchor_char.to_string(),
        x_offset,
        baseline_y,
        font,
        scale,
        ANCHOR_COLOR,
    );
    x_offset += anchor_width;

    // Render suffix (after anchor) in text color
    if !suffix.is_empty() {
        render_text_anti_aliased(
            &mut image, &suffix, x_offset, baseline_y, font, scale, TEXT_COLOR,
        );
    }

    Some(image)
}

/// Render text with anti-aliasing using ab_glyph's outline_glyph
///
/// This function renders text character by character, using ab_glyph's coverage-based
/// rasterization for smooth edges. Each pixel's alpha value is determined by the
/// glyph's coverage at that point (0.0 = empty, 1.0 = full coverage).
fn render_text_anti_aliased(
    image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    text: &str,
    start_x: f32,
    baseline_y: f32,
    font: &FontRef,
    scale: PxScale,
    color: Rgba<u8>,
) {
    let scaled_font = font.as_scaled(scale);
    let mut x_position = start_x;

    for ch in text.chars() {
        let mut glyph = scaled_font.scaled_glyph(ch);

        // Position the glyph at the baseline
        glyph.position = point(x_position, baseline_y);

        // Outline and render the glyph
        if let Some(outlined) = font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();

            // Draw each pixel with coverage-based anti-aliasing
            outlined.draw(|x, y, coverage| {
                let px_x = (bounds.min.x as u32 + x) as i32;
                let px_y = (bounds.min.y as u32 + y) as i32;

                // Check bounds
                if px_x >= 0
                    && px_x < image.width() as i32
                    && px_y >= 0
                    && px_y < image.height() as i32
                {
                    let pixel = image.get_pixel_mut(px_x as u32, px_y as u32);

                    // Alpha blending with coverage
                    let alpha = (coverage * color[3] as f32) as u8;

                    if pixel[3] == 0 {
                        // Pixel is transparent, just set it
                        *pixel = Rgba([color[0], color[1], color[2], alpha]);
                    } else {
                        // Alpha blending for overlapping glyphs
                        let existing_alpha = pixel[3] as f32 / 255.0;
                        let new_alpha = alpha as f32 / 255.0;
                        let out_alpha = new_alpha + existing_alpha * (1.0 - new_alpha);

                        if out_alpha > 0.0 {
                            for i in 0..3 {
                                pixel[i] = ((color[i] as f32 * new_alpha
                                    + pixel[i] as f32 * existing_alpha * (1.0 - new_alpha))
                                    / out_alpha) as u8;
                            }
                            pixel[3] = (out_alpha * 255.0) as u8;
                        }
                    }
                }
            });
        }

        // Advance x position
        x_position += scaled_font.h_advance(font.glyph_id(ch));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rendering::font::{get_font, get_font_metrics};

    fn setup_renderer() -> (FontRef<'static>, FontMetrics) {
        let font = get_font().expect("Font should be available");
        let metrics = get_font_metrics(&font, 24.0);
        (font, metrics)
    }

    #[test]
    fn test_rasterize_word_creates_valid_buffer() {
        let (font, metrics) = setup_renderer();

        // Rasterize a simple word with anchor at position 1
        let image = rasterize_word("hello", 1, &font, 24.0, &metrics);

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
        let (font, metrics) = setup_renderer();

        let short_word = rasterize_word("hi", 0, &font, 24.0, &metrics);
        let long_word = rasterize_word("supercalifragilistic", 3, &font, 24.0, &metrics);

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
    fn test_rasterize_word_with_special_characters() {
        let (font, metrics) = setup_renderer();

        // Test words that appear in the user's text
        let test_words = vec![
            ("Pastas", true, "normal word"),
            ("categories:", true, "word with colon"),
            ("(Italian:", true, "word with paren and colon"),
            ("secca)", true, "word with paren"),
            ("fresca).", true, "word with paren and period"),
        ];

        for (word, should_succeed, description) in test_words {
            let result = rasterize_word(word, 0, &font, 24.0, &metrics);
            assert!(
                result.is_some() == should_succeed,
                "Failed to render '{}': {:?} ({})",
                word,
                result,
                description
            );
        }
    }
}
