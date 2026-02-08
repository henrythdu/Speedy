use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use lazy_static::lazy_static;

const JETBRAINS_MONO_BYTES: &[u8] = include_bytes!("../../assets/fonts/JetBrainsMono-Regular.otf");

lazy_static! {
    static ref EMBEDDED_FONT: Option<FontRef<'static>> =
        FontRef::try_from_slice(JETBRAINS_MONO_BYTES).ok();
}

pub fn get_font() -> Option<FontRef<'static>> {
    EMBEDDED_FONT.clone()
}

pub fn calculate_char_width(font: &FontRef, c: char, font_size: f32) -> f32 {
    let scale = PxScale::from(font_size);
    let scaled_font = font.as_scaled(scale);

    let glyph_id = font.glyph_id(c);
    scaled_font.h_advance(glyph_id)
}

pub fn calculate_string_width(font: &FontRef, text: &str, font_size: f32) -> f32 {
    text.chars()
        .map(|c| calculate_char_width(font, c, font_size))
        .sum()
}

pub fn get_font_metrics(font: &FontRef, font_size: f32) -> FontMetrics {
    let scale = PxScale::from(font_size);
    let metrics = font.as_scaled(scale);

    FontMetrics {
        height: metrics.height(),
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FontMetrics {
    pub height: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_loads_from_embedded_bytes() {
        let font = get_font();
        assert!(font.is_some(), "Font should load from embedded bytes");
    }

    #[test]
    fn test_font_provides_metrics() {
        let font = get_font().expect("Font should be available");
        let font_metrics = get_font_metrics(&font, 24.0);

        assert!(font_metrics.height > 0.0, "Font should have positive height");
    }

    #[test]
    fn test_calculate_character_width() {
        let font = get_font().expect("Font should be available");
        let width = calculate_char_width(&font, 'W', 24.0);

        assert!(width > 0.0, "Character width should be positive");
        // JetBrains Mono is a monospace font, so all characters have same width
        assert_eq!(
            width,
            calculate_char_width(&font, 'i', 24.0),
            "Monospace font: 'W' and 'i' should have same width"
        );
    }

    #[test]
    fn test_calculate_string_width() {
        let font = get_font().expect("Font should be available");
        let width = calculate_string_width(&font, "Hello", 24.0);

        assert!(width > 0.0, "String width should be positive");
    }
}
