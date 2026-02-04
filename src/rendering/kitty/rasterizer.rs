//! Word rasterization for Kitty Graphics rendering
//!
//! Converts text words to RGBA image buffers with OVP anchor highlighting.

use crate::rendering::font::{calculate_char_width, calculate_string_width, FontMetrics};
use ab_glyph::{FontRef, PxScale};
use imageproc::drawing::draw_text_mut;
use imageproc::image::{ImageBuffer, Rgba};

/// Theme colors from PRD Section 4.1
pub const TEXT_COLOR: Rgba<u8> = Rgba([169, 177, 214, 255]); // #A9B1D6 Light Blue
pub const ANCHOR_COLOR: Rgba<u8> = Rgba([247, 118, 142, 255]); // #F7768E Coral Red

/// Rasterize word to RGBA buffer with text rendered using ab_glyph and imageproc
///
/// Creates an image buffer sized to fit the word, fills it with transparent background,
/// and renders the text with anchor character highlighted in coral red per PRD Section 4.1.
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
    let scale = PxScale::from(font_size);

    // Split word into: prefix (before anchor), anchor_char, suffix (after anchor)
    let chars: Vec<char> = word.chars().collect();
    let anchor_idx = anchor_position.min(chars.len().saturating_sub(1));

    let prefix: String = chars.iter().take(anchor_idx).collect();
    let anchor_char = chars.get(anchor_idx).copied().unwrap_or(' ');
    let suffix: String = chars.iter().skip(anchor_idx + 1).collect();

    // Calculate pixel widths
    let prefix_width = calculate_string_width(font, &prefix, font_size);
    let anchor_width = calculate_char_width(font, anchor_char, font_size);

    // Draw prefix (before anchor) in text color
    let mut x_offset = 0i32;
    if !prefix.is_empty() {
        draw_text_mut(&mut image, TEXT_COLOR, x_offset, 0, scale, font, &prefix);
        x_offset += prefix_width.ceil() as i32;
    }

    // Draw anchor character in anchor color
    let anchor_str = anchor_char.to_string();
    draw_text_mut(
        &mut image,
        ANCHOR_COLOR,
        x_offset,
        0,
        scale,
        font,
        &anchor_str,
    );
    x_offset += anchor_width.ceil() as i32;

    // Draw suffix (after anchor) in text color
    if !suffix.is_empty() {
        draw_text_mut(&mut image, TEXT_COLOR, x_offset, 0, scale, font, &suffix);
    }

    Some(image)
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
