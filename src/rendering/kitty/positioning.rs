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
