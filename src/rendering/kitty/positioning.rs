//! Positioning calculations for Kitty Graphics rendering
//!
//! Handles OVP (Optimal Viewing Position) anchoring and reading zone positioning.

use crate::rendering::font::calculate_string_width;
use crate::rendering::viewport::Viewport;
use ab_glyph::FontRef;

/// Calculate the X coordinate for positioning a word with OVP anchoring
///
/// # Arguments
/// * `word` - The word to position
/// * `anchor_position` - Character index that should be at the OVP (0-based)
/// * `font` - Font reference for width calculations
/// * `font_size` - Font size in pixels
/// * `viewport` - Viewport for center position calculation
///
/// # Returns
/// The X pixel coordinate where the word should start to center the anchor
pub fn calculate_start_x(
    word: &str,
    anchor_position: usize,
    font: &FontRef,
    font_size: f32,
    viewport: &Viewport,
) -> f32 {
    let word_chars: Vec<char> = word.chars().collect();

    if anchor_position >= word_chars.len() {
        return 0.0;
    }

    // Calculate width of characters before anchor
    let prefix: String = word_chars[..anchor_position].iter().collect();
    let prefix_width = calculate_string_width(font, &prefix, font_size);

    // Calculate width of anchor character
    let anchor_char = word_chars[anchor_position];
    let anchor_width = calculate_string_width(font, &anchor_char.to_string(), font_size);
    let anchor_half_width = anchor_width / 2.0;

    // StartX = Center - (prefix + anchor_half)
    // Calculate center_x dynamically from viewport dimensions (fixes resize bug)
    let center_x = viewport
        .get_dimensions()
        .map(|dims| dims.pixel_size.0 / 2)
        .unwrap_or(0) as f32;
    let result = center_x - (prefix_width + anchor_half_width);

    // Ensure result is non-negative (clamp to 0)
    result.max(0.0)
}

/// Calculate reading zone height in pixels
///
/// Reading zone is total height minus fixed 5-line command deck.
/// Returns None if viewport dimensions are not available.
pub fn get_reading_zone_height(viewport: &Viewport) -> Option<u32> {
    viewport.get_dimensions().map(|dims| {
        let command_zone_height = dims.cell_size.1 * 5.0; // Fixed 5 lines
        (dims.pixel_size.1 as f32 - command_zone_height) as u32
    })
}

/// Calculate vertical offset for centering text in reading zone
///
/// Per PRD Section 4.3: The reading line is centered at 42% of Reader Zone height.
/// Returns the Y pixel coordinate where text should be drawn.
pub fn calculate_vertical_center(viewport: &Viewport) -> Option<u32> {
    get_reading_zone_height(viewport).map(|zone_height| {
        // Vertical center = 42% of reading zone height (per PRD)
        (zone_height as f32 * 0.42) as u32
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rendering::font::get_font;
    use crate::rendering::viewport::TerminalDimensions;

    fn create_test_viewport() -> Viewport {
        let mut viewport = Viewport::new();
        let dims = TerminalDimensions::new(960, 540, 80, 24);
        viewport.set_dimensions(dims);
        viewport
    }

    #[test]
    fn test_calculate_start_x_single_char() {
        let font = get_font().expect("Font should be available");
        let viewport = create_test_viewport();

        // For a single character, anchor is at position 0
        // The character center should align with viewport center (480)
        let start_x = calculate_start_x("A", 0, &font, 24.0, &viewport);

        // With a monospace font, a single char should be ~14-15px wide
        // StartX should be roughly: center - (0 + half_char_width) = ~472-473
        assert!(
            start_x > 465.0 && start_x < 475.0,
            "Single char start_x should be near center minus half width: got {}",
            start_x
        );
    }

    #[test]
    fn test_calculate_start_x_two_chars() {
        let font = get_font().expect("Font should be available");
        let viewport = create_test_viewport();

        // For "AB" with anchor at position 1 (second char)
        // StartX = center - (width_of_A + half_width_of_B)
        let start_x = calculate_start_x("AB", 1, &font, 24.0, &viewport);

        // Should be less than single char case since anchor is offset to right
        let single_char_start = calculate_start_x("A", 0, &font, 24.0, &viewport);
        assert!(
            start_x < single_char_start,
            "Two-char word with right anchor should start left of single char"
        );
    }

    #[test]
    fn test_calculate_start_x_out_of_bounds() {
        let font = get_font().expect("Font should be available");
        let viewport = create_test_viewport();

        // Anchor position beyond word length should return 0.0
        let start_x = calculate_start_x("hi", 5, &font, 24.0, &viewport);
        assert_eq!(start_x, 0.0);
    }

    #[test]
    fn test_get_reading_zone_height() {
        let viewport = create_test_viewport();

        let zone_height = get_reading_zone_height(&viewport);

        assert!(
            zone_height.is_some(),
            "Should return height when dimensions set"
        );
        // Reading zone = Total height - (5 lines × cell_height)
        // = 540 - (5 × 6.75) = 540 - 33.75 = 506.25
        let cell_height = 540.0 / 24.0;
        let expected_zone = (540.0 - (5.0 * cell_height)) as u32;
        assert_eq!(zone_height.unwrap(), expected_zone);
    }

    #[test]
    fn test_get_reading_zone_height_without_dimensions() {
        let viewport = Viewport::new();

        let zone_height = get_reading_zone_height(&viewport);
        assert!(
            zone_height.is_none(),
            "Should return None without dimensions"
        );
    }

    #[test]
    fn test_calculate_vertical_center() {
        let viewport = create_test_viewport();

        let center = calculate_vertical_center(&viewport);

        assert!(center.is_some(), "Should return center when dimensions set");
        // Reading zone = Total height - (5 lines × cell_height)
        // = 540 - (5 × 6.75) = 506.25px
        // Vertical center = 42% of reading zone = 506.25 × 0.42 = 212.625 ≈ 212
        let cell_height = 540.0 / 24.0;
        let reading_zone = 540.0 - (5.0 * cell_height);
        let expected_center = (reading_zone * 0.42) as u32;
        assert_eq!(center.unwrap(), expected_center);
    }
}
